import socket
import json
import struct
import sys


def varint(n):
    out = b""
    while True:
        b = n & 0x7F
        n >>= 7
        out += bytes([b | (0x80 if n else 0)])
        if not n:
            return out


def read_varint(sock):
    result = shift = 0
    while True:
        b = sock.recv(1)[0]
        result |= (b & 0x7F) << shift
        if not b & 0x80:
            return result
        shift += 7


def ping(host, port=25565):
    with socket.create_connection((host, port), timeout=5) as s:
        hostname = host.encode()

        handshake = (
            b"\x00"
            + varint(767)
            + varint(len(hostname))
            + hostname
            + struct.pack(">H", port)
            + b"\x01"
        )

        packet = varint(len(handshake)) + handshake
        s.sendall(packet + b"\x01\x00")

        read_varint(s)
        read_varint(s)
        length = read_varint(s)

        data = b""
        while len(data) < length:
            data += s.recv(length - len(data))

        status = json.loads(data)
        players = status["players"]

        print(f"{players['online']}/{players['max']} online")

        for player in players.get("sample", []):
            print(" -", player["name"])


if __name__ == "__main__":
    ping(sys.argv[1])
