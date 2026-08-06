mod cli;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Inspect { filepath } => {
            println!("Test command with file: {}", filepath);
        }
    }
    println!();
}
