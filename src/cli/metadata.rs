use clap::{Args, Subcommand};

#[derive(Args)]
pub struct MetadataArgs {
    #[arg(short, long)]
    pub pretty: bool,

    #[command(subcommand)]
    pub command: Option<MetadataCommand>,
}

#[derive(Subcommand)]
pub enum MetadataCommand {
    #[command(override_usage = "aigov <FILEPATH> metadata set [OPTIONS]")]
    Set(SetArgs),
}

#[derive(Args)]
pub struct SetArgs {
    #[arg(long, short)]
    pub title: Option<String>,

    #[arg(long, short)]
    pub description: Option<String>,

    #[arg(long, short)]
    pub author: Option<String>,

    #[arg(long, short)]
    pub keywords: Option<String>,

    #[arg(long, short)]
    pub creator: Option<String>,

    #[arg(long, short)]
    pub subject: Option<String>,
}
