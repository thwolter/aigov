pub mod metadata;
pub mod pdf;
pub mod policy;
pub mod replace;
pub mod unzip;

use std::io;
use std::io::IsTerminal;
use std::path::PathBuf;

use aigov::error;
use clap::{Args, CommandFactory, Parser, Subcommand};

/// Doc comment
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Unzip a office document and store its contents.
    Unzip(FileCommand<unzip::UnzipArgs>),

    /// Manage the document's metadata.
    Metadata(FileCommand<Box<metadata::MetadataArgs>>),

    /// Convert an Office document to PDF using LibreOffice.
    ///
    /// ⚠ Warning: LibreOffice must be installed and the `soffice` command
    /// must be available on `PATH`.
    Pdf(FileCommand<pdf::PdfArgs>),

    /// Replace text in an DOCX document.
    Replace(FileCommand<replace::ReplaceArgs>),

    /// Manage the document's policy.
    Policy(FileCommand<policy::PolicyArgs>),
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct FileCommand<T: Args> {
    #[arg(value_name = "FILEPATH")]
    pub filepath: PathBuf,

    #[command(flatten)]
    pub args: T,
}

pub fn run() -> error::Result<()> {
    let cli_args = Cli::parse();

    let Some(command) = cli_args.command.as_ref() else {
        Cli::command().print_help()?;
        std::process::exit(1);
    };

    match command {
        Commands::Unzip(command) => unzip::run(command),
        Commands::Metadata(command) => metadata::run(command),
        Commands::Pdf(command) => pdf::run(command),
        Commands::Replace(command) => replace::run(command),
        Commands::Policy(command) => policy::run(command),
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
                "metadata",
                "report.docx",
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
            Cli::try_parse_from(["aigov", "metadata", "report.docx", "set", "--pages", "3"])
                .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "aigov",
                "metadata",
                "report.docx",
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
