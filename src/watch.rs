//! Watch mode: poll a server on an interval and emit events on state
//! transitions.
//!
//! # Transition semantics (deliberate choices)
//!
//! 1. The first poll is a silent baseline: no events fire for the state we
//!    start in, only for *changes* after it.
//! 2. The server is only considered offline after consecutive failed polls,
//!    so a single network hiccup cannot flap online/offline events.
//! 3. Player join/leave events describe the observed transition between two
//!    consecutive polls that both had a visible player sample.
//! 4. When the server goes offline — or a server stops publishing the player
//!    sample — the player baseline is reset to "unknown". When the sample
//!    reappears, it becomes the new baseline silently. This avoids the flood
//!    of bogus "X joined" events that would otherwise fire on recovery.
//! 5. Player names are compared case-insensitively (Minecraft names are
//!    case-insensitively unique) but events carry the original casing.

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::anyhow;

use crate::addr::ServerAddr;
use crate::notify;
use crate::protocol;
use crate::style;

/// Failed polls in a row required before declaring the server offline.
const OFFLINE_AFTER_FAILURES: u32 = 2;

/// One poll result fed into the state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum PollOutcome {
    Online {
        /// Player names from the sample; `None` when the server hides it.
        players: Option<Vec<String>>,
        online: u32,
        max: u32,
        latency_ms: Option<u128>,
    },
    Offline {
        reason: String,
    },
}

/// State changes detected between polls.
#[derive(Debug, Clone, PartialEq)]
pub enum WatchEvent {
    ServerOnline {
        online: u32,
        max: u32,
        latency_ms: Option<u128>,
    },
    ServerOffline {
        reason: String,
    },
    PlayerJoined {
        name: String,
    },
    PlayerLeft {
        name: String,
    },
}

/// Which events matter to the current watch mode.
#[derive(Debug, Clone, Default)]
pub struct WatchFilter {
    /// Only player events for this user (case-insensitive); `None` = any player.
    pub user: Option<String>,
    /// Report player leaves as well as joins.
    pub alert_leave: bool,
    /// Report server online/offline events.
    pub alert_server: bool,
}

impl WatchFilter {
    pub fn is_relevant(&self, event: &WatchEvent) -> bool {
        match event {
            WatchEvent::ServerOnline { .. } | WatchEvent::ServerOffline { .. } => self.alert_server,
            WatchEvent::PlayerJoined { name } => self.matches(name),
            WatchEvent::PlayerLeft { name } => self.alert_leave && self.matches(name),
        }
    }

    fn matches(&self, name: &str) -> bool {
        match &self.user {
            Some(user) => name.eq_ignore_ascii_case(user),
            None => true,
        }
    }
}

/// lowercase name → original casing, so diffs are case-insensitive while
/// events still display the server-provided spelling.
type PlayerMap = HashMap<String, String>;

#[derive(Debug, Default)]
pub struct WatchState {
    /// `None` until the first poll establishes a baseline.
    online: Option<bool>,
    players: Option<PlayerMap>,
    offline_streak: u32,
}

impl WatchState {
    pub fn server_is_online(&self) -> Option<bool> {
        self.online
    }
}

/// Fold one poll into the state, returning any detected events.
pub fn step(state: &mut WatchState, poll: PollOutcome) -> Vec<WatchEvent> {
    let mut events = Vec::new();

    match poll {
        PollOutcome::Online {
            players,
            online,
            max,
            latency_ms,
        } => {
            let recovered = state.online == Some(false);
            state.online = Some(true);
            state.offline_streak = 0;

            // Baseline poll and recovery from offline both rebaseline the
            // player list silently below.
            if recovered {
                events.push(WatchEvent::ServerOnline {
                    online,
                    max,
                    latency_ms,
                });
            }

            let current: Option<PlayerMap> = players.map(|names| {
                names
                    .into_iter()
                    .map(|name| (name.to_lowercase(), name))
                    .collect()
            });

            if let (Some(prev), Some(cur)) = (&state.players, &current) {
                let mut joined: Vec<&String> =
                    cur.keys().filter(|k| !prev.contains_key(*k)).collect();
                let mut left: Vec<&String> =
                    prev.keys().filter(|k| !cur.contains_key(*k)).collect();
                joined.sort();
                left.sort();

                for key in joined {
                    events.push(WatchEvent::PlayerJoined {
                        name: cur[key].clone(),
                    });
                }

                for key in left {
                    events.push(WatchEvent::PlayerLeft {
                        name: prev[key].clone(),
                    });
                }
            }

            // Rebaseline on: first poll, recovery from offline, or a poll
            // where the sample was hidden. Never flood "joined" events for
            // players we did not observe joining.
            state.players = current;
        }
        PollOutcome::Offline { reason } => {
            if state.online.is_none() {
                // Baseline poll: starting offline is not an event.
                state.online = Some(false);
            } else if state.online == Some(true) {
                state.offline_streak += 1;

                if state.offline_streak >= OFFLINE_AFTER_FAILURES {
                    events.push(WatchEvent::ServerOffline { reason });
                    state.online = Some(false);
                    // Everyone's presence becomes unknown; recovery will
                    // rebaseline silently instead of reporting mass joins.
                    state.players = None;
                }
            }
            // Already offline: nothing to emit.
        }
    }

    events
}

pub struct WatchConfig {
    pub addr: ServerAddr,
    pub interval: Duration,
    pub filter: WatchFilter,
    pub desktop_notifications: bool,
}

/// Run the watch loop until Ctrl-C. Terminal lines are printed for every
/// detected event; desktop notifications and the bell fire only for events
/// relevant to the configured filter.
pub fn run(config: &WatchConfig) -> anyhow::Result<()> {
    let (ctrlc_tx, ctrlc_rx) = mpsc::channel::<()>();

    ctrlc::set_handler(move || {
        let _ = ctrlc_tx.send(());
    })
    .map_err(|e| anyhow!("failed to install Ctrl-C handler: {e}"))?;

    let mut state = WatchState::default();
    let mut next_tick = Instant::now();
    let mut first_poll = true;

    loop {
        let outcome = poll(&config.addr);

        // First consecutive failure while we still think the server is up:
        // hint why nothing "official" happens yet.
        if matches!(outcome, PollOutcome::Offline { .. })
            && state.online == Some(true)
            && state.offline_streak == 0
        {
            println!(
                "[{}] {}",
                timestamp(),
                style::dim(&format!(
                    "server unreachable, marking offline after {OFFLINE_AFTER_FAILURES} consecutive failures"
                ))
            );
        }

        for event in step(&mut state, outcome.clone()) {
            let relevant = config.filter.is_relevant(&event);
            let server_event = matches!(
                event,
                WatchEvent::ServerOnline { .. } | WatchEvent::ServerOffline { .. }
            );

            // Player events only render when they matter to the filter;
            // server transitions stay visible (dimmed) in player modes.
            if relevant || (server_event && !config.filter.alert_server) {
                render_event(&config.addr, &event, config.filter.alert_server);
            }

            if relevant {
                if config.desktop_notifications {
                    notify::desktop("mcping", &desktop_body(&config.addr, &event));
                }
                notify::bell();
            }
        }

        if first_poll {
            first_poll = false;
            print_baseline(&config.addr, &state, &outcome, config.interval);
        }

        next_tick += config.interval;
        let now = Instant::now();

        // If a poll took longer than the interval, don't burst catch-up polls.
        if next_tick < now {
            next_tick = now + config.interval;
        }

        // Sleep until the next tick, waking immediately on Ctrl-C.
        let timeout = next_tick.saturating_duration_since(Instant::now());

        match ctrlc_rx.recv_timeout(timeout) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!("[{}] {}", timestamp(), style::dim("stopped watching"));
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

fn poll(addr: &ServerAddr) -> PollOutcome {
    match protocol::query(addr) {
        Ok(result) => PollOutcome::Online {
            players: result.status.player_names(),
            online: result.status.players.online,
            max: result.status.players.max,
            latency_ms: result.latency.map(|d| d.as_millis()),
        },
        Err(e) => PollOutcome::Offline {
            reason: e.to_string(),
        },
    }
}

fn print_baseline(
    addr: &ServerAddr,
    state: &WatchState,
    outcome: &PollOutcome,
    interval: Duration,
) {
    let head = format!(
        "watching {} every {}s — Ctrl-C to stop",
        style::bold(&addr.to_string()),
        interval.as_secs()
    );

    let detail = match (state.server_is_online(), outcome) {
        (
            Some(true),
            PollOutcome::Online {
                online,
                max,
                latency_ms,
                ..
            },
        ) => format!(
            "currently online ({}/{} players, ping {})",
            online,
            max,
            latency_text(*latency_ms)
        ),
        _ => "currently offline".to_string(),
    };

    println!("[{}] {} {}", timestamp(), head, style::dim(&detail));
}

fn render_event(addr: &ServerAddr, event: &WatchEvent, server_mode: bool) {
    let line = match event {
        WatchEvent::ServerOnline {
            online,
            max,
            latency_ms,
        } => {
            if server_mode {
                style::green(&format!(
                    "▲ {addr} is online — {}/{} players, ping {}",
                    online,
                    max,
                    latency_text(*latency_ms)
                ))
            } else {
                style::dim("▲ server is back online")
            }
        }
        WatchEvent::ServerOffline { reason } => {
            if server_mode {
                style::red(&format!("▼ {addr} is offline ({reason})"))
            } else {
                style::dim("▼ server went offline")
            }
        }
        WatchEvent::PlayerJoined { name } => style::green(&format!("▲ {name} joined")),
        WatchEvent::PlayerLeft { name } => style::yellow(&format!("▼ {name} left")),
    };

    println!("[{}] {}", timestamp(), line);
}

fn desktop_body(addr: &ServerAddr, event: &WatchEvent) -> String {
    match event {
        WatchEvent::ServerOnline {
            online,
            max,
            latency_ms,
        } => {
            format!(
                "{addr} is online — {online}/{max} players, ping {}",
                latency_text(*latency_ms)
            )
        }
        WatchEvent::ServerOffline { reason } => format!("{addr} is offline — {reason}"),
        WatchEvent::PlayerJoined { name } => format!("{name} joined {addr}"),
        WatchEvent::PlayerLeft { name } => format!("{name} left {addr}"),
    }
}

fn latency_text(latency_ms: Option<u128>) -> String {
    match latency_ms {
        Some(ms) => format!("{ms}ms"),
        None => "n/a".to_string(),
    }
}

fn timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn online(players: &[&str]) -> PollOutcome {
        PollOutcome::Online {
            players: Some(players.iter().map(|s| s.to_string()).collect()),
            online: players.len() as u32,
            max: 100,
            latency_ms: Some(35),
        }
    }

    fn online_hidden_sample() -> PollOutcome {
        PollOutcome::Online {
            players: None,
            online: 42,
            max: 100,
            latency_ms: Some(35),
        }
    }

    fn offline() -> PollOutcome {
        PollOutcome::Offline {
            reason: "connection refused".to_string(),
        }
    }

    fn names(events: &[WatchEvent]) -> Vec<(String, String)> {
        events
            .iter()
            .filter_map(|e| match e {
                WatchEvent::PlayerJoined { name } => Some(("join".to_string(), name.clone())),
                WatchEvent::PlayerLeft { name } => Some(("left".to_string(), name.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn first_poll_online_is_silent_baseline() {
        let mut state = WatchState::default();
        let events = step(&mut state, online(&["Steve", "Alex"]));

        assert!(events.is_empty());
        assert_eq!(state.server_is_online(), Some(true));
    }

    #[test]
    fn first_poll_offline_is_silent_then_online_alerts() {
        let mut state = WatchState::default();

        assert!(step(&mut state, offline()).is_empty());
        assert_eq!(state.server_is_online(), Some(false));

        let events = step(&mut state, online(&[]));
        assert_eq!(
            events,
            vec![WatchEvent::ServerOnline {
                online: 0,
                max: 100,
                latency_ms: Some(35)
            }]
        );
    }

    #[test]
    fn single_failed_poll_does_not_flap_offline() {
        let mut state = WatchState::default();
        step(&mut state, online(&[]));

        // One failure: below the threshold, no events.
        assert!(step(&mut state, offline()).is_empty());
        assert_eq!(state.server_is_online(), Some(true));

        // Recovered before the threshold was hit: still no events.
        assert!(step(&mut state, online(&[])).is_empty());
    }

    #[test]
    fn two_consecutive_failures_mark_offline_then_online() {
        let mut state = WatchState::default();
        step(&mut state, online(&[]));

        assert!(step(&mut state, offline()).is_empty());

        let events = step(&mut state, offline());
        assert_eq!(
            events,
            vec![WatchEvent::ServerOffline {
                reason: "connection refused".to_string()
            }]
        );
        assert_eq!(state.server_is_online(), Some(false));

        let events = step(&mut state, online(&[]));
        assert!(matches!(
            events.as_slice(),
            [WatchEvent::ServerOnline { .. }]
        ));
    }

    #[test]
    fn detects_join_and_leave_with_original_casing() {
        let mut state = WatchState::default();
        step(&mut state, online(&["Steve", "Alex"]));

        let events = step(&mut state, online(&["Alex", "Hero"]));
        assert_eq!(
            names(&events),
            vec![
                ("join".to_string(), "Hero".to_string()),
                ("left".to_string(), "Steve".to_string())
            ]
        );
    }

    #[test]
    fn detects_join_and_leave_in_the_same_poll() {
        let mut state = WatchState::default();
        step(&mut state, online(&["Steve"]));

        let events = step(&mut state, online(&["Hero"]));
        assert_eq!(
            names(&events),
            vec![
                ("join".to_string(), "Hero".to_string()),
                ("left".to_string(), "Steve".to_string())
            ]
        );
    }

    #[test]
    fn casing_only_change_is_not_an_event() {
        let mut state = WatchState::default();
        step(&mut state, online(&["Hero"]));

        let events = step(&mut state, online(&["HERO"]));
        assert!(events.is_empty());
    }

    #[test]
    fn offline_resets_baseline_no_join_flood_on_recovery() {
        let mut state = WatchState::default();
        step(&mut state, online(&["Steve", "Alex"]));

        step(&mut state, offline());
        let events = step(&mut state, offline()); // marked offline now
        assert!(matches!(
            events.as_slice(),
            [WatchEvent::ServerOffline { .. }]
        ));

        // Comes back with the same players: only the online event, no joins.
        let events = step(&mut state, online(&["Steve", "Alex"]));
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WatchEvent::ServerOnline { .. }));
    }

    #[test]
    fn hidden_sample_does_not_report_mass_leaves_or_joins() {
        let mut state = WatchState::default();
        step(&mut state, online(&["Steve", "Alex"]));

        // Sample hidden: nobody "left", we just can't see.
        assert!(step(&mut state, online_hidden_sample()).is_empty());

        // Sample back with a different set: rebaseline, no flood.
        let events = step(&mut state, online(&["Hero", "jeb_"]));
        assert!(events.is_empty());

        // From the new baseline normal diffing resumes.
        let events = step(&mut state, online(&["Hero", "jeb_", "Notch"]));
        assert_eq!(
            names(&events),
            vec![("join".to_string(), "Notch".to_string())]
        );
    }

    #[test]
    fn duplicate_sample_entries_do_not_confuse_the_diff() {
        let mut state = WatchState::default();
        step(&mut state, online(&["Steve"]));

        let events = step(&mut state, online(&["Steve", "Steve", "Alex"]));
        assert_eq!(
            names(&events),
            vec![("join".to_string(), "Alex".to_string())]
        );
    }

    #[test]
    fn player_filter_is_case_insensitive() {
        let filter = WatchFilter {
            user: Some("Hero".to_string()),
            ..Default::default()
        };

        assert!(filter.is_relevant(&WatchEvent::PlayerJoined {
            name: "hero".to_string()
        }));
        assert!(!filter.is_relevant(&WatchEvent::PlayerJoined {
            name: "Alex".to_string()
        }));
        assert!(!filter.is_relevant(&WatchEvent::PlayerLeft {
            name: "hero".to_string()
        })); // leave not enabled
    }

    #[test]
    fn leave_alerts_are_toggleable() {
        let join_only = WatchFilter {
            user: Some("Hero".to_string()),
            ..Default::default()
        };
        let with_leaves = WatchFilter {
            user: Some("Hero".to_string()),
            alert_leave: true,
            ..Default::default()
        };

        let leave = WatchEvent::PlayerLeft {
            name: "Hero".to_string(),
        };
        assert!(!join_only.is_relevant(&leave));
        assert!(with_leaves.is_relevant(&leave));
    }

    #[test]
    fn server_filter_gates_server_events() {
        let server = WatchFilter {
            alert_server: true,
            ..Default::default()
        };
        let player = WatchFilter {
            user: None,
            alert_leave: true,
            ..Default::default()
        };

        let online = WatchEvent::ServerOnline {
            online: 1,
            max: 2,
            latency_ms: None,
        };
        assert!(server.is_relevant(&online));
        assert!(!player.is_relevant(&online));
        assert!(player.is_relevant(&WatchEvent::PlayerJoined {
            name: "anyone".to_string()
        }));
    }

    #[test]
    fn offline_reason_is_carried_to_the_event() {
        let mut state = WatchState::default();
        step(&mut state, online(&[]));

        step(
            &mut state,
            PollOutcome::Offline {
                reason: "timed out".to_string(),
            },
        );
        let events = step(
            &mut state,
            PollOutcome::Offline {
                reason: "timed out".to_string(),
            },
        );
        assert_eq!(
            events,
            vec![WatchEvent::ServerOffline {
                reason: "timed out".to_string()
            }]
        );
    }
}
