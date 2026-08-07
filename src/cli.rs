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
