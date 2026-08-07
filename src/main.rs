use aigov::cli::{metadata::MetadataCommand, Cli, Commands};
use aigov::commands;
use aigov::Result;
use clap::Parser;

fn main() {
    let args = Cli::parse();
    let result: Result<()> = match &args.command {
        Commands::Inspect => {
            println!("inspect");
            Ok(())
        }
        Commands::Metadata(metadata_args) => {
            match &metadata_args.command {
                None => commands::show_metadata(&args.filepath, metadata_args),
                Some(MetadataCommand::Set(set_args)) => {
                    commands::set_metadata(&args.filepath, set_args)
                }
            }
        }
    };

    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
