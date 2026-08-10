use aigov::office::document::{CaseMatching, ReplaceOptions};
use clap::{ArgGroup, Args};
use std::path::PathBuf;

#[derive(Args)]
#[command(
    group(
        ArgGroup::new("destination")
            .required(true)
            .multiple(false)
    )
)]
pub struct ReplaceArgs {
    #[arg(
        long,
        value_name = "PATH",
        help = "Write the updated document to PATH",
        group = "destination"
    )]
    pub output: Option<PathBuf>,

    #[arg(short, long, help = "Replace the input file", group = "destination")]
    pub overwrite: bool,

    #[arg(short, long, value_name = "TEXT", help = "Text to search for")]
    pub search: String,

    #[arg(short, long, value_name = "TEXT", help = "Text to replace with")]
    pub replace: String,

    #[arg(short, long, help = "Ignore case when searching")]
    pub ignore_case: bool,
}

impl ReplaceArgs {
    pub fn options(&self) -> ReplaceOptions {
        ReplaceOptions {
            case_matching: if self.ignore_case {
                CaseMatching::UnicodeInsensitive
            } else {
                CaseMatching::Sensitive
            },
        }
    }
}
