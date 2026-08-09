pub mod metadata;
pub mod pdf;
pub mod unzip;

pub use metadata::MetadataArgs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Doc comment
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub filepath: PathBuf,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Unzip a office document and store its contents.
    Unzip(unzip::UnzipArgs),

    /// Set/show metadata for a office document.
    Metadata(Box<metadata::MetadataArgs>),

    /// Convert a office document to PDF.
    Pdf(pdf::PdfArgs),
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
