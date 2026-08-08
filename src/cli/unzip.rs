use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct UnzipArgs {
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}
