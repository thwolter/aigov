pub mod metadata;
pub mod pdf;
pub mod replace;
pub mod unzip;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

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

    /// Set/show metadata for a office document.
    Metadata(Box<metadata::MetadataArgs>),

    /// Convert an Office document to PDF using LibreOffice.
    ///
    /// ⚠ Warning: LibreOffice must be installed and the `soffice` command
    /// must be available on `PATH`.
    Pdf(pdf::PdfArgs),

    /// Replace text in an DOCX document.
    Replace(replace::ReplaceArgs),
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
