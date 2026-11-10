use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::status::{parse_status, ServerStatus};

// Minecraft Java protocol version
const PROTOCOL_VERSION: i32 = 767;
const SERVER_PORT: u16 = 25565;

// Encode a Minecraft VarInt
fn write_varint(mut value: i32, output: &mut Vec<u8>) {
    loop {
        if value & !0x7F == 0 {
            output.push(value as u8);
            return;
        }

        output.push(((value & 0x7F) | 0x80) as u8);
        value = ((value as u32) >> 7) as i32;
    }
}

// Decode a Minecraft VarInt
fn read_varint(stream: &mut TcpStream) -> io::Result<i32> {
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

// Encode a Minecraft string
fn write_string(value: &str, output: &mut Vec<u8>) {
    write_varint(value.len() as i32, output);
    output.extend_from_slice(value.as_bytes());
}

pub fn ping(host: &str) -> Result<ServerStatus, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect((host, SERVER_PORT))?;

    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;

    // Build handshake packet
    let mut handshake = Vec::new();

    write_varint(0, &mut handshake); // Packet ID
    write_varint(PROTOCOL_VERSION, &mut handshake);
    write_string(host, &mut handshake);

    handshake.extend_from_slice(&SERVER_PORT.to_be_bytes());

    write_varint(1, &mut handshake); // Next state: Status

    // Prefix packet with its length
    let mut packet = Vec::new();

    write_varint(handshake.len() as i32, &mut packet);
    packet.extend_from_slice(&handshake);

    stream.write_all(&packet)?;

    // Send Status Request packet
    stream.write_all(&[0x01, 0x00])?;

    // Read Status Response
    let packet_length = read_varint(&mut stream)?;

    if packet_length <= 0 || packet_length > 1_048_576 {
        return Err("Invalid status packet length".into());
    }

    let packet_id = read_varint(&mut stream)?;

    if packet_id != 0 {
        return Err("Unexpected Minecraft packet ID".into());
    }

    let json_length = read_varint(&mut stream)?;

    if json_length < 0 || json_length as i32 > packet_length {
        return Err("Invalid JSON length".into());
    }

    let mut json = vec![0u8; json_length as usize];

    stream.read_exact(&mut json)?;

    let json = String::from_utf8(json)?;

    Ok(parse_status(&json)?)
}

