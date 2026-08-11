pub mod metadata;
pub mod pdf;
pub mod policy;
pub mod replace;
pub mod unzip;

use std::io;
use std::io::IsTerminal;
use std::path::PathBuf;

use crate::cli;
use crate::cli::metadata::MetadataCommand;
use aigov::error;
use clap::{CommandFactory, Parser, Subcommand};

/// Doc comment
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(
        value_name = "FILEPATH",
        help = "Input document; supported file types depend on the selected command"
    )]
    pub filepath: PathBuf,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Unzip a office document and store its contents.
    Unzip(unzip::UnzipArgs),

    /// Manage the document's metadata.
    Metadata(Box<metadata::MetadataArgs>),

    /// Convert an Office document to PDF using LibreOffice.
    ///
    /// ⚠ Warning: LibreOffice must be installed and the `soffice` command
    /// must be available on `PATH`.
    Pdf(pdf::PdfArgs),

    /// Replace text in an DOCX document.
    Replace(replace::ReplaceArgs),

    /// Manage the document's policy.
    Policy(policy::PolicyArgs),
}

pub fn run() -> error::Result<()> {
    let args = Cli::parse();
    let result: error::Result<()> = match &args.command {
        None => print_help(),

        Some(Commands::Unzip(unzip_args)) => cli::unzip::unzip_document(&args.filepath, unzip_args),

        Some(Commands::Metadata(metadata_args)) => match &metadata_args.command {
            None => cli::metadata::show_metadata(&args.filepath, metadata_args),
            Some(MetadataCommand::Set(set_args)) if set_args.is_empty() => print_set_help(),
            Some(MetadataCommand::Set(set_args)) => {
                cli::metadata::set_metadata(&args.filepath, set_args)
            }
        },
        Some(Commands::Pdf(pdf_args)) => cli::pdf::convert_to_pdf(&args.filepath, pdf_args),

        Some(Commands::Replace(replace_args)) => {
            cli::replace::replace(&args.filepath, replace_args)
        }

        Some(Commands::Policy(policy_args)) => {
            cli::policy::handle_policy(&args.filepath, policy_args)
        }
    };
    result
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

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::Cli;

    #[test]
    fn metadata_set_accepts_editable_fields_only() {
        assert!(
            Cli::try_parse_from([
                "aigov",
                "report.docx",
                "metadata",
                "set",
                "--category",
                "Strategy",
                "--content-status",
                "Draft",
                "--language",
                "en-US",
                "--custom",
                "Client=Acme",
                "--custom",
                "Classification=Internal",
            ])
            .is_ok()
        );
        assert!(
            Cli::try_parse_from(["aigov", "report.docx", "metadata", "set", "--pages", "3",])
                .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "aigov",
                "report.docx",
                "metadata",
                "set",
                "--custom",
                "missing"
            ])
            .is_err()
        );
    }
}

fn print_success(message: impl std::fmt::Display) {
    if io::stdout().is_terminal() {
        println!("\x1b[32m✓ {message}\x1b[0m");
    } else {
        println!("✓ {message}");
    }
}

fn print_warning(message: impl std::fmt::Display) {
    if io::stdout().is_terminal() {
        println!("\x1b[33m⚠ {message}\x1b[0m");
    } else {
        println!("⚠ {message}");
    }
}
