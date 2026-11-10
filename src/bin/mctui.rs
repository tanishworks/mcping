//! mctui — a live terminal dashboard for a Minecraft server.
//!
//! Polls the server in a background thread and renders player count, ping
//! history, MOTD and the online player list. Press q, Esc or Ctrl-C to quit.

use std::collections::VecDeque;
use std::io::stdout;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, Chart, Dataset, Gauge, GraphType, Paragraph};

use mcping::addr::ServerAddr;
use mcping::protocol;
use mcping::status::ServerStatus;

/// How many polls the ping chart keeps.
const HISTORY_LEN: usize = 120;

#[derive(Parser, Debug)]
#[command(
    name = "mctui",
    version,
    about = "Live Minecraft server dashboard — player count, ping and names at a glance"
)]
struct Args {
    /// Server address, e.g. alwto.net or localhost:25565
    addr: ServerAddr,

    /// Seconds between polls
    #[arg(short, long, default_value_t = 5)]
    interval: u64,
}

enum PollUpdate {
    Online {
        status: ServerStatus,
        latency: Option<Duration>,
    },
    Offline {
        reason: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let interval = Duration::from_secs(args.interval.max(1));

    let (tx, rx) = mpsc::channel::<PollUpdate>();
    let stop = Arc::new(AtomicBool::new(false));

    {
        let addr = args.addr.clone();
        let stop = Arc::clone(&stop);

        thread::spawn(move || {
            loop {
                if stop.load(Ordering::Relaxed) {
                    return;
                }

                let update = match protocol::query(&addr) {
                    Ok(result) => PollUpdate::Online {
                        status: result.status,
                        latency: result.latency,
                    },
                    Err(e) => PollUpdate::Offline {
                        reason: e.to_string(),
                    },
                };

                if tx.send(update).is_err() {
                    return; // UI thread is gone
                }

                let deadline = Instant::now() + interval;

                while Instant::now() < deadline {
                    if stop.load(Ordering::Relaxed) {
                        return;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });
    }

    let _guard = TerminalGuard::new();
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(args.addr, interval);

    loop {
        terminal.draw(|frame| app.draw(frame))?;

        while let Ok(update) = rx.try_recv() {
            app.apply(update);
        }

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Down | KeyCode::Char('j') => app.scroll = app.scroll.saturating_add(1),
                KeyCode::Up | KeyCode::Char('k') => app.scroll = app.scroll.saturating_sub(1),
                _ => {}
            }
        }
    }

    stop.store(true, Ordering::Relaxed);
    Ok(())
}

/// Restores the terminal even on panic (Drop runs during unwind).
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

struct App {
    addr: ServerAddr,
    interval: Duration,
    status: Option<ServerStatus>,
    player_names: Option<Vec<String>>,
    latency: Option<Duration>,
    last_poll: Option<String>,
    last_ok: Option<String>,
    offline_reason: Option<String>,
    history: VecDeque<Option<u128>>,
    scroll: u16,
}

impl App {
    fn new(addr: ServerAddr, interval: Duration) -> Self {
        Self {
            addr,
            interval,
            status: None,
            player_names: None,
            latency: None,
            last_poll: None,
            last_ok: None,
            offline_reason: None,
            history: VecDeque::new(),
            scroll: 0,
        }
    }

    fn apply(&mut self, update: PollUpdate) {
        let now = Local::now().format("%H:%M:%S").to_string();

        match update {
            PollUpdate::Online { status, latency } => {
                self.player_names = status.player_names();
                self.latency = latency;
                self.offline_reason = None;
                self.last_ok = Some(now.clone());
                self.history.push_back(latency.map(|d| d.as_millis()));
                self.status = Some(status);
            }
            PollUpdate::Offline { reason } => {
                self.offline_reason = Some(reason);
                self.latency = None;
                self.history.push_back(None);
            }
        }

        while self.history.len() > HISTORY_LEN {
            self.history.pop_front();
        }

        self.last_poll = Some(now);
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let outer = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(6),
                Constraint::Length(12),
                Constraint::Length(1),
            ])
            .split(frame.area());

        self.draw_header(frame, outer[0]);

        let mid = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
            .split(outer[1]);

        self.draw_stats(frame, mid[0]);
        self.draw_players(frame, mid[1]);
        self.draw_chart(frame, outer[2]);
        self.draw_footer(frame, outer[3]);
    }

    fn draw_header(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let mut lines = Vec::new();

        let motd = self.status.as_ref().map(|s| s.motd()).unwrap_or_default();

        for line in motd.lines().take(2) {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::new().cyan(),
            )));
        }

        if let Some(version) = self
            .status
            .as_ref()
            .and_then(|status| status.version.as_ref())
        {
            lines.push(Line::from(Span::styled(
                format!("{} (protocol {})", version.name, version.protocol),
                Style::new().dim(),
            )));
        }

        let block = Block::bordered().title(format!(" {} ", self.addr));
        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn draw_stats(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let stats = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(1)])
            .split(area);

        let players = self.status.as_ref().map(|s| &s.players);
        let online = players.as_ref().map_or(0, |p| p.online);
        let max = players.as_ref().map_or(1, |p| p.max).max(1);
        let offline = self.offline_reason.is_some();

        let fill = online.saturating_mul(100) / max;

        let gauge = Gauge::default()
            .block(Block::bordered().title(" players "))
            .gauge_style(Style::new().fg(if offline { Color::Red } else { Color::Green }))
            .percent(fill.min(100) as u16)
            .label(Span::from(format!("{online}/{max}")));

        frame.render_widget(gauge, stats[0]);

        let mut lines = Vec::new();

        if let Some(reason) = &self.offline_reason {
            lines.push(Line::from(Span::styled(
                "OFFLINE".to_string(),
                Style::new().red().bold().add_modifier(Modifier::SLOW_BLINK),
            )));
            lines.push(Line::from(Span::styled(
                truncate(reason, stats[1].width.saturating_sub(2) as usize),
                Style::new().red(),
            )));
        } else if let Some(latency) = self.latency {
            let ms = latency.as_millis();
            lines.push(Line::from(Span::styled(
                format!("{ms}ms"),
                Style::new().fg(ping_color(ms)).bold(),
            )));
            lines.push(Line::from(Span::styled(
                "round-trip latency",
                Style::new().dim(),
            )));
        } else {
            lines.push(Line::from(Span::styled("polling…", Style::new().dim())));
        }

        frame.render_widget(
            Paragraph::new(lines).block(Block::bordered().title(" ping ")),
            stats[1],
        );
    }

    fn draw_players(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let visible = area.height.saturating_sub(2) as usize;
        let mut lines = Vec::new();
        let mut title = " players ".to_string();

        match &self.player_names {
            None => {
                title = " players (list hidden) ".to_string();
                lines.push(Line::from(Span::styled(
                    "this server does not publish a player list",
                    Style::new().dim(),
                )));
            }
            Some(names) if names.is_empty() => {
                lines.push(Line::from(Span::styled(
                    "nobody is online",
                    Style::new().dim(),
                )));
            }
            Some(names) => {
                let online = self
                    .status
                    .as_ref()
                    .map_or(names.len(), |s| s.players.online as usize);

                title = format!(" online ({}/{}) ", names.len(), online);

                for name in names {
                    lines.push(Line::from(Span::from(format!("• {name}"))));
                }
            }
        }

        // Clamp scroll to the list length so the view never runs past the end.
        let content_len = lines.len();
        let max_scroll = content_len.saturating_sub(visible) as u16;
        self.scroll = self.scroll.min(max_scroll);

        let block = Block::bordered().title(title);
        frame.render_widget(
            Paragraph::new(lines).block(block).scroll((self.scroll, 0)),
            area,
        );
    }

    fn draw_chart(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let points: Vec<(f64, f64)> = self
            .history
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| entry.map(|ms| (i as f64, ms as f64)))
            .collect();

        let y_max = points.iter().map(|(_, y)| *y).fold(1.0f64, f64::max) * 1.2;

        let dataset = Dataset::default()
            .name("ping")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().cyan())
            .data(&points);

        let chart = Chart::new(vec![dataset])
            .block(Block::bordered().title(" ping history "))
            .x_axis(Axis::default().bounds([0.0, HISTORY_LEN as f64]))
            .y_axis(Axis::default().bounds([0.0, y_max]));

        frame.render_widget(chart, area);
    }

    fn draw_footer(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let last_poll = self.last_poll.as_deref().unwrap_or("never");
        let last_ok = self.last_ok.as_deref().unwrap_or("never");

        let line = Line::from(Span::styled(
            format!(
                " q quit · j/k scroll · polls every {}s · last poll {last_poll} · last online {last_ok} ",
                self.interval.as_secs()
            ),
            Style::new().dim(),
        ));

        frame.render_widget(Paragraph::new(line), area);
    }
}

fn ping_color(ms: u128) -> Color {
    match ms {
        0..=80 => Color::Green,
        81..=250 => Color::Yellow,
        _ => Color::Red,
    }
}

fn truncate(text: &str, max_chars: usize) -> String {
    if max_chars == 0 || text.chars().count() <= max_chars {
        return text.to_string();
    }

    let cut: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{cut}…")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    fn sample_status() -> ServerStatus {
        mcping::status::parse_status(
            r#"{
                "players": {"online": 42, "max": 100, "sample": [
                    {"name": "Steve", "id": "a"},
                    {"name": "Alex",  "id": "b"}
                ]},
                "version": {"name": "mcmock 1.21", "protocol": 767},
                "description": {"text": "test server"}
            }"#,
        )
        .unwrap()
    }

    fn draw_to_string(app: &mut App) -> String {
        let backend = TestBackend::new(110, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| app.draw(frame)).unwrap();

        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn renders_address_players_and_ping() {
        let mut app = App::new("127.0.0.1:25599".parse().unwrap(), Duration::from_secs(5));
        app.apply(PollUpdate::Online {
            status: sample_status(),
            latency: Some(Duration::from_millis(35)),
        });

        let screen = draw_to_string(&mut app);

        assert!(screen.contains("127.0.0.1:25599"));
        assert!(screen.contains("test server"));
        assert!(screen.contains("mcmock 1.21 (protocol 767)"));
        assert!(screen.contains("42/100"));
        assert!(screen.contains("35ms"));
        assert!(screen.contains("Steve"));
        assert!(screen.contains("Alex"));
    }

    #[test]
    fn renders_offline_state() {
        let mut app = App::new("127.0.0.1:25599".parse().unwrap(), Duration::from_secs(5));
        app.apply(PollUpdate::Offline {
            reason: "connection refused".to_string(),
        });

        let screen = draw_to_string(&mut app);

        assert!(screen.contains("OFFLINE"));
        assert!(screen.contains("connection refused"));
    }

    #[test]
    fn renders_hidden_player_list() {
        let json = r#"{
            "players": {"online": 7, "max": 100},
            "description": {"text": "no sample here"}
        }"#;

        let mut app = App::new("127.0.0.1:25599".parse().unwrap(), Duration::from_secs(5));
        app.apply(PollUpdate::Online {
            status: mcping::status::parse_status(json).unwrap(),
            latency: None,
        });

        let screen = draw_to_string(&mut app);

        assert!(screen.contains("list hidden"));
        assert!(screen.contains("does not publish a player list"));
    }

    #[test]
    fn clamps_scroll_to_content() {
        let mut app = App::new("127.0.0.1:25599".parse().unwrap(), Duration::from_secs(5));
        app.apply(PollUpdate::Online {
            status: sample_status(),
            latency: Some(Duration::from_millis(35)),
        });

        // Far beyond the two players rendered.
        app.scroll = 5000;

        let screen = draw_to_string(&mut app);

        // Render must not panic; scroll is clamped inside draw.
        assert!(screen.contains("Steve"));
    }
}
