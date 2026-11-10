# mcping

Lightweight Minecraft (Java Edition) server status CLI — player counts, player
names and latency **without joining the server**, plus watch modes that notify
you when a server comes online or a player joins.

```sh
$ mcping status alwto.net
1.21 + | ᴀʟᴡᴛᴏ.ɴᴇᴛ | 26.3 UPDATE SPEARS
ᴀʟᴡᴛᴏ ᴘʀᴀᴄᴛɪᴄᴇ | ᴅᴜᴇʟs✦ғғᴀ✦ᴘᴀʀᴛʏ✦ᴇᴠᴇɴᴛs
alwto.net:25565
version   Velocity 1.7.2-26.3 (protocol 767)
players   332/2555
ping      35ms
```

## Install

Building from source requires Rust 1.85+ (edition 2024):

```sh
git clone https://github.com/tanishworks/mcping
cd mcping
cargo build --release
```

The release build produces three binaries:

| Binary   | Purpose                            |
| -------- | ---------------------------------- |
| `mcping` | the CLI                            |
| `mctui`  | full-screen live dashboard         |
| `mcmock` | mock server for testing (dev tool) |

## CLI

```sh
mcping status <addr[:port]>    # MOTD, version, players, names, latency
mcping ping <addr[:port]>      # round-trip latency
mcping watch server <addr>     # notify when the server goes online/offline
mcping watch player <addr> [user] [--leave]
```

`watch player` notifies on joins (`--leave` adds leaves); omit the user to
watch any player. Watch flags: `--interval <secs>` (default 10) and
`--no-notify` (terminal only).

### Addresses

Addresses accept `host`, `host:port` and `[ipv6]:port`. The default port is
25565.

## Watch modes

Watch events print to the terminal (timestamped, colored, with a bell) and
fire a desktop notification — `osascript` on macOS, `notify-send` on Linux.
Ctrl-C stops watching.

- **`watch server`** — polls until the server comes online or drops; latency
  stays accurate because every poll is a real ping/pong round-trip.
- **`watch player`** — alerts the moment a player joins (or leaves with
  `--leave`); watch the whole server or a single user.

## TUI

```sh
mctui <addr[:port]> [--interval 5]
```

Full-screen live dashboard: player gauge, ping-history chart, MOTD and a
scrollable player list. `q` quits, `j/k` or the arrow keys scroll.

## Notes & caveats

- Latency is a real ping/pong round-trip against the server.
- The server is only considered offline after 2 consecutive failed polls, so
  a single hiccup can't flap alerts; the first poll is a silent baseline and
  recovery never floods fake "joined" events.
- Player names rely on the status `sample` field. Vanilla servers cap it at
  ~12 random entries and some servers hide it entirely — player watching
  works best on servers that publish full lists.
- `mcmock` simulates servers for testing: `--script "5:+Hero,10:-Steve"` for
  join/leave timelines, `--up-at` / `--down-at` for offline transitions.
