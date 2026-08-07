use aigov::Result;
use aigov::cli::{Cli, Commands, metadata::MetadataCommand};
use aigov::commands;
use clap::{CommandFactory, Parser};

fn main() {
    let args = Cli::parse();
    let result: Result<()> = match &args.command {
        Commands::Inspect => {
            println!("inspect");
            Ok(())
        }
        Commands::Metadata(metadata_args) => match &metadata_args.command {
            None => commands::show_metadata(&args.filepath, metadata_args),
            Some(MetadataCommand::Set(set_args)) if set_args.is_empty() => print_set_help(),
            Some(MetadataCommand::Set(set_args)) => {
                commands::set_metadata(&args.filepath, set_args)
            }
        },
    };

    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn print_set_help() -> Result<()> {
    let mut command = Cli::command();
    let metadata = command
        .find_subcommand_mut("metadata")
        .expect("metadata subcommand is defined");
    let set = metadata
        .find_subcommand_mut("set")
        .expect("metadata set subcommand is defined");
    set.print_help()?;
    println!();
    Ok(())
}
