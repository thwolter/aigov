use aigov::cli::{Cli, Commands};
use aigov::commands;
use clap::Parser;

fn main() {
    let args = Cli::parse();
    match &args.command {
        Commands::Inspect => {
            println!("inspect");
        }
        Commands::Metadata(metadata_args) => {
            commands::show_metadata(&args.filepath, metadata_args);
        }
    }
    println!();
}
