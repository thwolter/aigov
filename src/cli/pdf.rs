use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct PdfArgs {
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}