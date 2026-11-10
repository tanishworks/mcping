use std::process::ExitCode;

use clap::{Parser, Subcommand};

use mcping::addr::ServerAddr;
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
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Status { addr } => cmd_status(&addr),
        Command::Ping { addr } => cmd_ping(&addr),
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
