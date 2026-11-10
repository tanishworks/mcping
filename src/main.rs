use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "mcping",
    version,
    about = "A lightweight Minecraft server status CLI"
)]
struct Args {
    /// Show online/max player count
    #[arg(short, long)]
    number: bool,

    /// Show available online player names
    #[arg(short, long)]
    player: bool,

    /// Minecraft server address
    #[arg(short, long, default_value = "localhost")]
    addr: String,
}

fn main() {
    let args = Args::parse();

    if !args.number && !args.player {
        println!("Choose --number or --player");
        return;
    }

    println!("Address: {}", args.addr);

    if args.number {
        println!("Player count requested");
    }

    if args.player {
        println!("Player names requested");
    }
}
