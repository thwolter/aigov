mod cli;
mod commands;

use crate::commands::{metadata, pdf, replace, unzip};
use aigov::error;
use clap::{CommandFactory, Parser};
use cli::{Cli, Commands, metadata::MetadataCommand};

fn main() {
    let args = Cli::parse();
    let result: error::Result<()> = match &args.command {
        None => print_help(),
        Some(Commands::Unzip(unzip_args)) => unzip::unzip_document(&args.filepath, unzip_args),
        Some(Commands::Metadata(metadata_args)) => match &metadata_args.command {
            None => metadata::show_metadata(&args.filepath, metadata_args),
            Some(MetadataCommand::Set(set_args)) if set_args.is_empty() => print_set_help(),
            Some(MetadataCommand::Set(set_args)) => {
                metadata::set_metadata(&args.filepath, set_args)
            }
        },
        Some(Commands::Pdf(pdf_args)) => pdf::convert_to_pdf(&args.filepath, pdf_args),
        Some(Commands::Replace(replace_args)) => replace::replace(&args.filepath, replace_args),
    };

    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn print_help() -> error::Result<()> {
    let mut command = Cli::command();
    command.print_help()?;
    println!();
    Ok(())
}

fn print_set_help() -> error::Result<()> {
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
