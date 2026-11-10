//! A mock Minecraft Java status server for exercising `mcping` without a
//! real server. Speaks the handshake/status/ping protocol and can simulate
//! players joining/leaving on a timeline, starting offline, and crashing.
//!
//! Examples:
//!   mcmock --port 25590 --players "Steve,Alex" --script "5:+Hero,10:-Steve"
//!   mcmock --port 25591 --up-at 5     # starts offline, listens after 5s
//!   mcmock --port 25592 --down-at 5   # listens, then refuses after 5s

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;
use mcping::protocol::{read_varint, write_varint, write_string};
use serde_json::json;

const PROTOCOL_VERSION: i32 = 767;
const MAX_PACKET_LEN: i32 = 32 * 1024;

#[derive(Parser, Debug)]
#[command(
    name = "mcmock",
    about = "Mock Minecraft status server for testing mcping",
    disable_version_flag = true
)]
struct Args {
    /// Port to listen on
    #[arg(long, default_value_t = 25565)]
    port: u16,

    /// Initial player list, comma separated
    #[arg(long, default_value = "")]
    players: String,

    /// Timed events relative to startup, e.g. "10:+Hero,20:-Steve"
    #[arg(long, default_value = "")]
    script: String,

    /// Max players reported in the status response
    #[arg(long, default_value_t = 100)]
    max: u32,

    /// Seconds after startup to stop listening (simulate crash); 0 = never
    #[arg(long, default_value_t = 0)]
    down_at: u64,

    /// Seconds to wait before starting to listen (start offline)
    #[arg(long, default_value_t = 0)]
    up_at: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let start = Instant::now();

    let players: Vec<String> = args
        .players
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    let script = parse_script(&args.script)?;
    let players = Arc::new(Mutex::new(players));

    if args.up_at > 0 {
        println!("mcmock: simulating offline, listening in {}s", args.up_at);
        thread::sleep(Duration::from_secs(args.up_at));
    }

    let listener = TcpListener::bind(("127.0.0.1", args.port))?;
    println!("mcmock: listening on 127.0.0.1:{} (players: {:?})", args.port, players.lock().unwrap());

    // Apply scripted join/leave events on their own thread.
    if !script.is_empty() {
        let players = Arc::clone(&players);

        thread::spawn(move || {
            for (at_secs, event) in script {
                let deadline = start + Duration::from_secs(at_secs);
                let now = Instant::now();

                if deadline > now {
                    thread::sleep(deadline - now);
                }

                match event {
                    ScriptEvent::Join(name) => {
                        players.lock().unwrap().push(name.clone());
                        println!("mcmock: +{name}");
                    }
                    ScriptEvent::Leave(name) => {
                        players.lock().unwrap().retain(|p| *p != name);
                        println!("mcmock: -{name}");
                    }
                }
            }
        });
    }

    listener.set_nonblocking(true)?;

    loop {
        if args.down_at > 0 && start.elapsed().as_secs() >= args.down_at {
            println!("mcmock: simulating crash, shutting down");
            return Ok(());
        }

        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false)?;
                let players = Arc::clone(&players);

                thread::spawn(move || {
                    let _ = handle(stream, players, args.max);
                });
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(e.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ScriptEvent {
    Join(String),
    Leave(String),
}

/// Parse "10:+Hero,20:-Steve" into `[(10, Join(Hero)), (20, Leave(Steve))]`.
fn parse_script(script: &str) -> anyhow::Result<Vec<(u64, ScriptEvent)>> {
    let mut events = Vec::new();

    for entry in script.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let (secs, rest) = entry
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("invalid script entry '{entry}', expected 'secs:+Name'"))?;

        let secs = secs.parse::<u64>().map_err(|_| anyhow::anyhow!("invalid seconds in '{entry}'"))?;
        let (sign, name) = rest
            .split_at(rest.char_indices().nth(1).map(|(i, _)| i).unwrap_or(rest.len()));

        match sign {
            "+" if !name.is_empty() => events.push((secs, ScriptEvent::Join(name.to_string()))),
            "-" if !name.is_empty() => events.push((secs, ScriptEvent::Leave(name.to_string()))),
            _ => return Err(anyhow::anyhow!("invalid event in '{entry}', expected +Name or -Name")),
        }
    }

    Ok(events)
}

fn handle(
    mut stream: TcpStream,
    players: Arc<Mutex<Vec<String>>>,
    max: u32,
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    // Handshake: id 0, protocol varint, host string, port u16, next state varint.
    let handshake = read_packet(&mut stream)?;
    let mut cursor = handshake.as_slice();

    if read_varint(&mut cursor)? != 0 {
        return Ok(());
    }

    let _protocol = read_varint(&mut cursor)?;

    let host_len = read_varint(&mut cursor)? as usize;
    let mut host = vec![0u8; host_len];
    cursor.read_exact(&mut host)?;
    let _port = cursor.read_exact(&mut [0u8; 2])?;

    if read_varint(&mut cursor)? != 1 {
        return Ok(()); // not a status ping, ignore
    }

    // Status Request: id 0.
    let request = read_packet(&mut stream)?;

    if read_varint(&mut request.as_slice())? != 0 {
        return Ok(());
    }

    // Status Response with the current player list.
    let names = players.lock().unwrap().clone();
    let sample: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(i, name)| json!({ "name": name, "id": fake_uuid(name, i) }))
        .collect();

    let body = json!({
        "version": { "name": "mcmock 1.21", "protocol": PROTOCOL_VERSION },
        "players": { "online": names.len() as u32, "max": max, "sample": sample },
        "description": { "text": "mcping mock server" }
    })
    .to_string();

    let mut response = Vec::new();
    write_varint(0, &mut response);
    write_string(&body, &mut response);

    let mut framed = Vec::new();
    write_varint(response.len() as i32, &mut framed);
    framed.extend_from_slice(&response);
    stream.write_all(&framed)?;

    // Ping Start → Pong echo. Best effort: the client may just close.
    if let Ok(ping) = read_packet(&mut stream) {
        let mut cursor = ping.as_slice();

        if read_varint(&mut cursor)? == 1 {
            let mut payload = [0u8; 8];

            if cursor.read_exact(&mut payload).is_ok() {
                let mut pong = Vec::new();
                write_varint(9, &mut pong);
                write_varint(1, &mut pong);
                pong.extend_from_slice(&payload);
                stream.write_all(&pong)?;
            }
        }
    }

    Ok(())
}

fn read_packet(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let length = read_varint(stream)?;

    if length <= 0 || length > MAX_PACKET_LEN {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "bad packet length"));
    }

    let mut buf = vec![0u8; length as usize];
    stream.read_exact(&mut buf)?;
    Ok(buf)
}

/// Deterministic pseudo-UUID so player ids are stable across polls.
fn fake_uuid(name: &str, index: usize) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64 ^ (index as u64);

    for byte in name.bytes() {
        hash = (hash ^ byte as u64).wrapping_mul(0x100_0000_01b3);
    }

    format!("{:08x}-0000-4000-8000-{:012x}", (hash >> 32) as u32, hash & 0xfff_ffff_ffff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_script_timeline() {
        let script = parse_script("10:+Hero,20:-Steve, 30:+jeb_ ").unwrap();
        assert_eq!(
            script,
            vec![
                (10, ScriptEvent::Join("Hero".into())),
                (20, ScriptEvent::Leave("Steve".into())),
                (30, ScriptEvent::Join("jeb_".into())),
            ]
        );
    }

    #[test]
    fn rejects_bad_script() {
        assert!(parse_script("10:*Hero").is_err());
        assert!(parse_script("ten:+Hero").is_err());
        assert!(parse_script("10:+").is_err());
    }

    #[test]
    fn fake_uuid_is_stable() {
        assert_eq!(fake_uuid("Hero", 0), fake_uuid("Hero", 0));
        assert_ne!(fake_uuid("Hero", 0), fake_uuid("Hero", 1));
    }
}
