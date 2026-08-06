use clap::{Parser, Subcommand};

/// Doc comment
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Inspect {
        filepath: String,
    }
}