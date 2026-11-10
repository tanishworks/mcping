//! Minecraft Java Edition status protocol: handshake → status request →
//! response → ping/pong round-trip latency.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, anyhow};

use crate::addr::ServerAddr;
use crate::status::{ServerStatus, parse_status};

/// Protocol version announced in the handshake. Servers respond to status
/// pings regardless of this value; 767 = 1.21.
const PROTOCOL_VERSION: i32 = 767;

const READ_WRITE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// The pong is a formality for most servers; don't let it stall the query.
const PONG_TIMEOUT: Duration = Duration::from_secs(2);
/// Maximum status packet length (protocol limit: 2^21 - 1).
const MAX_PACKET_LENGTH: i32 = 2_097_151;

/// Encode a Minecraft VarInt.
pub fn write_varint(mut value: i32, output: &mut Vec<u8>) {
    loop {
        if value & !0x7F == 0 {
            output.push(value as u8);
            return;
        }

        output.push(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

/// Decode a Minecraft VarInt. Generic over the reader so tests and the mock
/// server can reuse it.
pub fn read_varint(stream: &mut impl Read) -> io::Result<i32> {
    let mut result = 0u32;

    for shift in (0..35).step_by(7) {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte)?;

        result |= ((byte[0] & 0x7F) as u32) << shift;

        if byte[0] & 0x80 == 0 {
            return Ok(result as i32);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "VarInt too large",
    ))
}

/// Encode a Minecraft string (VarInt length prefix + UTF-8 bytes).
pub fn write_string(value: &str, output: &mut Vec<u8>) {
    write_varint(value.len() as i32, output);
    output.extend_from_slice(value.as_bytes());
}

#[derive(Debug)]
pub struct QueryResult {
    pub status: ServerStatus,
    /// Ping/pong round-trip latency. Falls back to the status round-trip if
    /// the server never answered the ping packet.
    pub latency: Option<Duration>,
}

/// Query a server for its status and measure latency.
pub fn query(addr: &ServerAddr) -> anyhow::Result<QueryResult> {
    let mut stream = connect(addr)?;

    stream
        .set_read_timeout(Some(READ_WRITE_TIMEOUT))
        .context("failed to configure socket")?;
    stream
        .set_write_timeout(Some(READ_WRITE_TIMEOUT))
        .context("failed to configure socket")?;

    send_handshake(&mut stream, addr)?;
    send_status_request(&mut stream)?;

    let status_rtt = Instant::now();
    let status = read_status_response(&mut stream)?;
    let status_latency = status_rtt.elapsed();

    // Vanilla clients measure latency with a ping/pong exchange right after
    // the status response. If the server doesn't play along, fall back to the
    // status round-trip so the user still gets a number.
    let latency = measure_pong_rtt(&mut stream).or(Some(status_latency));

    Ok(QueryResult { status, latency })
}

fn connect(addr: &ServerAddr) -> anyhow::Result<TcpStream> {
    use std::net::ToSocketAddrs;

    let socket_addrs: Vec<_> = (addr.host.as_str(), addr.port)
        .to_socket_addrs()
        .map_err(|e| anyhow!("could not resolve {}: {e}", addr.host))?
        .collect();

    if socket_addrs.is_empty() {
        return Err(anyhow!("could not resolve {}", addr.host));
    }

    // Try every resolved address (IPv4 and IPv6) with a bounded connect
    // timeout so filtered ports fail fast instead of hanging on the OS
    // default (~75s).
    let mut last_error = None;

    for socket_addr in socket_addrs {
        match TcpStream::connect_timeout(&socket_addr, CONNECT_TIMEOUT) {
            Ok(stream) => return Ok(stream),
            Err(e) => last_error = Some((socket_addr, e)),
        }
    }

    let (socket_addr, e) = last_error.expect("at least one connect attempt");
    let reason = match e.kind() {
        io::ErrorKind::ConnectionRefused => {
            "connection refused (is the server online?)".to_string()
        }
        io::ErrorKind::TimedOut => "connection timed out".to_string(),
        _ => e.to_string(),
    };

    Err(anyhow!(
        "could not connect to {addr} ({socket_addr}): {reason}"
    ))
}

fn send_handshake(stream: &mut TcpStream, addr: &ServerAddr) -> anyhow::Result<()> {
    let mut handshake = Vec::new();

    write_varint(0, &mut handshake); // Packet ID: Handshake
    write_varint(PROTOCOL_VERSION, &mut handshake);
    write_string(&addr.host, &mut handshake);
    handshake.extend_from_slice(&addr.port.to_be_bytes());
    write_varint(1, &mut handshake); // Next state: Status

    // Prefix the packet with its length.
    let mut packet = Vec::new();
    write_varint(handshake.len() as i32, &mut packet);
    packet.extend_from_slice(&handshake);

    stream
        .write_all(&packet)
        .context("failed to send handshake")
}

fn send_status_request(stream: &mut TcpStream) -> anyhow::Result<()> {
    // Length = 1, Packet ID = 0x00 (Status Request).
    stream
        .write_all(&[0x01, 0x00])
        .context("failed to send status request")
}

fn read_status_response(stream: &mut TcpStream) -> anyhow::Result<ServerStatus> {
    let packet_length = read_varint(stream).context("no response from server")?;

    if packet_length <= 0 || packet_length > MAX_PACKET_LENGTH {
        return Err(anyhow!("invalid status packet length: {packet_length}"));
    }

    let packet_id = read_varint(stream)?;

    if packet_id != 0 {
        return Err(anyhow!("unexpected packet id {packet_id} (expected 0)"));
    }

    let json_length = read_varint(stream)?;

    if json_length < 0 || json_length > packet_length {
        return Err(anyhow!("invalid JSON length: {json_length}"));
    }

    let mut json = vec![0u8; json_length as usize];
    stream
        .read_exact(&mut json)
        .context("truncated status response")?;

    let json = String::from_utf8(json).context("status response is not valid UTF-8")?;

    parse_status(&json).context("could not parse status response")
}

fn measure_pong_rtt(stream: &mut TcpStream) -> Option<Duration> {
    stream.set_read_timeout(Some(PONG_TIMEOUT)).ok()?;
    stream.set_write_timeout(Some(PONG_TIMEOUT)).ok()?;

    let payload = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    let mut ping = Vec::new();
    write_varint(9, &mut ping); // Length: 1 (id) + 8 (payload)
    write_varint(1, &mut ping); // Packet ID: Ping Start
    ping.extend_from_slice(&payload.to_be_bytes());

    let start = Instant::now();
    stream.write_all(&ping).ok()?;

    let packet_length = read_varint(stream).ok()?;
    let packet_id = read_varint(stream).ok()?;

    if packet_length != 9 || packet_id != 1 {
        return None;
    }

    let mut echo = [0u8; 8];
    stream.read_exact(&mut echo).ok()?;

    if u64::from_be_bytes(echo) != payload {
        return None;
    }

    Some(start.elapsed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_round_trip() {
        let values = [0, 1, 127, 128, 255, 2097151, i32::MAX, -1];

        for value in values {
            let mut buf = Vec::new();
            write_varint(value, &mut buf);
            let decoded = read_varint(&mut buf.as_slice()).unwrap();
            assert_eq!(decoded, value, "round trip failed for {value}");
        }
    }

    #[test]
    fn varint_is_minimal_length() {
        let mut buf = Vec::new();
        write_varint(0, &mut buf);
        assert_eq!(buf, vec![0x00]);

        buf.clear();
        write_varint(1, &mut buf);
        assert_eq!(buf, vec![0x01]);

        buf.clear();
        write_varint(127, &mut buf);
        assert_eq!(buf, vec![0x7f]);

        buf.clear();
        write_varint(128, &mut buf);
        assert_eq!(buf, vec![0x80, 0x01]);
    }

    #[test]
    fn varint_rejects_oversized() {
        let oversized = [0x80; 5]; // five continuation bytes, no terminator
        assert!(read_varint(&mut &oversized[..]).is_err());
    }

    #[test]
    fn string_length_is_byte_length() {
        let mut buf = Vec::new();
        write_string("héllo", &mut buf); // 'é' is 2 bytes in UTF-8
        assert_eq!(buf[0], 6); // VarInt length prefix, not char count
        assert_eq!(&buf[1..], "héllo".as_bytes());
    }
}
