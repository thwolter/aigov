pub mod metadata;
pub use metadata::MetadataArgs;

use clap::{Parser, Subcommand};

/// Doc comment
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub filepath: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Inspect,
    Metadata(metadata::MetadataArgs),
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
            ])
                .is_ok()
        );
        assert!(
            Cli::try_parse_from(["aigov", "report.docx", "metadata", "set", "--pages", "3", ])
                .is_err()
        );
    }
}
