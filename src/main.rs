use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};

use mcping::addr::ServerAddr;
use mcping::watch::{WatchConfig, WatchFilter};
use mcping::{protocol, style};

#[derive(Parser, Debug)]
#[command(
    name = "mcping",
    version,
    about = "Minecraft server status CLI — player count, names and latency without joining"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Show player count, online player names, version and latency
    Status {
        /// Server address, e.g. alwto.net or localhost:25565
        addr: ServerAddr,
    },
    /// Measure the round-trip latency to a server
    Ping {
        /// Server address, e.g. alwto.net or localhost:25565
        addr: ServerAddr,
    },
    /// Watch a server and notify when things change
    Watch {
        #[command(subcommand)]
        mode: WatchMode,

        /// Seconds between polls
        #[arg(short, long, default_value_t = 10, global = true)]
        interval: u64,

        /// Disable desktop notifications (terminal output only)
        #[arg(long, global = true)]
        no_notify: bool,
    },
}

#[derive(Subcommand, Debug)]
enum WatchMode {
    /// Notify when the server comes online or goes offline
    Server {
        /// Server address, e.g. alwto.net or localhost:25565
        addr: ServerAddr,
    },
    /// Notify when a player joins, and with --leave, when they leave
    Player {
        /// Server address, e.g. alwto.net or localhost:25565
        addr: ServerAddr,
        /// Only watch this player (case-insensitive); omit for any player
        user: Option<String>,
        /// Also notify when a watched player leaves
        #[arg(long)]
        leave: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Status { addr } => cmd_status(&addr),
        Command::Ping { addr } => cmd_ping(&addr),
        Command::Watch { mode, interval, no_notify } => cmd_watch(mode, interval, !no_notify),
    }
}

fn cmd_status(addr: &ServerAddr) -> ExitCode {
    match protocol::query(addr) {
        Ok(result) => {
            let status = &result.status;

            let motd = status.motd();

            if !motd.is_empty() {
                println!("{}", style::cyan(&motd));
            }

            println!("{}", style::bold(&addr.to_string()));

            if let Some(version) = &status.version {
                println!(
                    "{} {} {}",
                    style::dim("version"),
                    version.name,
                    style::dim(&format!("(protocol {})", version.protocol))
                );
            }

            let players = &status.players;
            println!(
                "{} {}/{}",
                style::dim("players"),
                style::green(&players.online.to_string()),
                players.max
            );

            match result.latency {
                Some(latency) => {
                    println!(
                        "{} {}",
                        style::dim("ping"),
                        style::latency_colored(latency.as_millis())
                    );
                }
                None => println!("{} {}", style::dim("ping"), style::dim("n/a")),
            }

            match status.player_names() {
                Some(names) if names.is_empty() => {
                    println!("{}", style::dim("no players online"))
                }
                Some(names) => {
                    println!("{}", style::dim("online:"));
                    for name in names {
                        println!("  • {name}");
                    }
                }
                None => println!("{}", style::dim("player list hidden by server")),
            }

            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{} {error:#}", style::red("error:"));
            ExitCode::FAILURE
        }
    }
}

fn cmd_ping(addr: &ServerAddr) -> ExitCode {
    match protocol::query(addr) {
        Ok(result) => {
            match result.latency {
                Some(latency) => println!(
                    "{} {}",
                    style::bold(&addr.to_string()),
                    style::latency_colored(latency.as_millis())
                ),
                None => println!("{} {}", style::bold(&addr.to_string()), style::dim("n/a")),
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{} {error:#}", style::red("error:"));
            ExitCode::FAILURE
        }
    }
}

fn cmd_watch(mode: WatchMode, interval: u64, desktop_notifications: bool) -> ExitCode {
    if interval == 0 {
        eprintln!("{} interval must be at least 1 second", style::red("error:"));
        return ExitCode::FAILURE;
    }

    let (addr, filter) = match mode {
        WatchMode::Server { addr } => (
            addr,
            WatchFilter { alert_server: true, ..WatchFilter::default() },
        ),
        WatchMode::Player { addr, user, leave } => {
            let user = match user {
                Some(user) if user.trim().is_empty() => {
                    eprintln!("{} watched player name cannot be empty", style::red("error:"));
                    return ExitCode::FAILURE;
                }
                user => user,
            };

            (
                addr,
                WatchFilter { user, alert_leave: leave, alert_server: false },
            )
        }
    };

    let config = WatchConfig {
        addr,
        interval: Duration::from_secs(interval),
        filter,
        desktop_notifications,
    };

    match mcping::watch::run(&config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{} {error:#}", style::red("error:"));
            ExitCode::FAILURE
        }
    }
}
