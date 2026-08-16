mod convert;
pub mod metadata;
pub mod pdf;
pub mod policy;
pub mod replace;
pub mod unzip;

use std::io;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use aigov::error;
use clap::{Args, Parser, Subcommand};

use indicatif::{ProgressBar, ProgressStyle};

/// Doc comment
#[derive(Parser)]
#[command(version, about, long_about = None, subcommand_required = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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

    /// Convert a document to Markdown
    #[command(visible_alias = "md")]
    Markdown(FileCommand<convert::MarkdownArgs>),

    /// Extract resources (images, media) from a document
    Extract(FileCommand<convert::ExtractArgs>),
}

#[derive(Args, Clone)]
#[group(required = true, multiple = false)]
struct DestinationArgs {
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Write the updated document to FILE"
    )]
    output: Option<PathBuf>,

    #[arg(long, help = "Update the input document")]
    in_place: bool,
}

impl DestinationArgs {
    /// Returns the output path or the input path if no output path is specified
    pub fn output_or<'a>(&'a self, input: &'a Path) -> &'a Path {
        self.output.as_deref().unwrap_or(input)
    }
}

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub(crate) struct FileCommand<T: Args> {
    #[arg(value_name = "FILE", help = "Path to the document")]
    input: PathBuf,

    #[command(flatten)]
    args: T,
}

pub(crate) fn run() -> error::Result<()> {
    let cli_args = Cli::parse();

    match cli_args.command.as_ref().unwrap() {
        Commands::Unzip(command) => unzip::run(command),
        Commands::Metadata(command) => metadata::run(command),
        Commands::Pdf(command) => pdf::run(command),
        Commands::Replace(command) => replace::run(command),
        Commands::Policy(command) => policy::run(command),
        Commands::Markdown(command) => convert::run_markdown(command),
        Commands::Extract(command) => convert::run_extract(command),
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
                "--in-place"
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

pub(crate) fn print_error(message: impl std::fmt::Display) {
    if io::stderr().is_terminal() {
        eprintln!("\x1b[31m✗ {message}\x1b[0m");
    } else {
        eprintln!("✗ {message}");
    }
}

fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

fn output_path(input: &Path, output: Option<&Path>, extension: &str) -> PathBuf {
    output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| input.with_extension(extension))
}
