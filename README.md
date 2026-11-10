# mcping

Lightweight Minecraft (Java Edition) server status CLI — player counts, player
names and latency **without joining the server**, plus watch modes that notify
you when a server comes online or a player joins.

```
$ mcping status alwto.net
1.21 + | ᴀʟᴡᴛᴏ.ɴᴇᴛ | 26.3 UPDATE SPEARS
ᴀʟᴡᴛᴏ ᴘʀᴀᴄᴛɪᴄᴇ | ᴅᴜᴇʟs✦ғғᴀ✦ᴘᴀʀᴛʏ✦ᴇᴠᴇɴᴛs
alwto.net:25565
version Velocity 1.7.2-26.3 (protocol 767)
players 332/2555
ping 35ms
online:
  • D1xP
  • gamersuyeb
  ...
```

## Build

Requires Rust 1.85+ (edition 2024).

```sh
cargo build --release
```

Produces:

| Binary | Purpose |
|---|---|
| `mcping` | the CLI |
| `mctui` | full-screen live dashboard |
| `mcmock` | mock server for testing (dev tool) |

## CLI

```sh
mcping status <addr[:port]>    # MOTD, version, players, names, latency
mcping ping <addr[:port]>      # round-trip latency
mcping watch server <addr>     # notify when the server goes online/offline
mcping watch player <addr> [user] [--leave]
                               # notify on joins (--leave adds leaves);
                               # omit user to watch any player
```

Watch flags: `--interval <secs>` (default 10), `--no-notify` (terminal only).

Events print to the terminal (timestamped, colored, with a bell) **and** fire a
desktop notification. Ctrl-C stops watching.

## TUI

```sh
mctui <addr[:port]> [--interval 5]
```

Live dashboard: player gauge, ping-history chart, MOTD and scrollable player
list. `q` quits, `j/k` or arrows scroll.

## Notes

- Addresses accept `host`, `host:port` and `[ipv6]:port`. Default port 25565.
- Latency is a real ping/pong round-trip against the server.
- The server is only considered offline after 2 consecutive failed polls, so a
  single hiccup can't flap alerts; the first poll is a silent baseline, and
  recovery never floods fake "joined" events.
- Player names rely on the status `sample` field. Vanilla servers cap it at ~12
  random entries and some servers hide it entirely — player watching works best
  on servers that publish full lists.
- Desktop notifications use `osascript` on macOS and `notify-send` on Linux.
- `mcmock` simulates servers for testing: `--script "5:+Hero,10:-Steve"` for
  join/leave timelines, `--up-at` / `--down-at` for offline transitions.
