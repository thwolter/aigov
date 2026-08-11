pub mod metadata;
pub mod pdf;
pub mod policy;
pub mod replace;
pub mod unzip;

use std::io;
use std::io::IsTerminal;
use std::path::PathBuf;

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
    filepath: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
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
    let cli_args = Cli::parse();

    let filepath = cli_args.filepath;
    if !filepath.exists() {
        print_error("Filepath does not exist");
        std::process::exit(1);
    };

    let Some(command) = cli_args.command.as_ref() else {
        Cli::command().print_help()?;
        std::process::exit(1);
    };

    match command {
        Commands::Unzip(args) => unzip::run(&filepath, args),
        Commands::Metadata(args) => metadata::run(&filepath, args),
        Commands::Pdf(args) => pdf::run(&filepath, args),
        Commands::Replace(args) => replace::run(&filepath, args),
        Commands::Policy(args) => policy::run(&filepath, args),
    }
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

pub(crate) fn print_success(message: impl std::fmt::Display) {
    if io::stdout().is_terminal() {
        println!("\x1b[32m✓ {message}\x1b[0m");
    } else {
        println!("✓ {message}");
    }
}

pub(crate) fn print_warning(message: impl std::fmt::Display) {
    if io::stdout().is_terminal() {
        println!("\x1b[33m⚠ {message}\x1b[0m");
    } else {
        println!("⚠ {message}");
    }
}

pub(crate) fn print_error(message: impl std::fmt::Display) {
    if io::stderr().is_terminal() {
        eprintln!("\x1b[31m✗ {message}\x1b[0m");
    } else {
        eprintln!("✗ {message}");
    }
}
