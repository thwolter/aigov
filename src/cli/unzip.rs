use clap::Args;

#[derive(Args)]
pub struct UnzipArgs {
    #[arg(short, long)]
    pub output: Option<String>,
}