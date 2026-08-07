use clap::Parser;
use aigov::commands;
use aigov::cli::{Cli, Commands};


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
