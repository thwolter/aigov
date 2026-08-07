use clap::{Args, Parser, Subcommand};

/// Doc comment
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub filepath: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct MetadataArgs {
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Inspect,
    Metadata(MetadataArgs),
}
