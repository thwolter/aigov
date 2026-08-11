use aigov::error;
use aigov::formats::Document;
use aigov::office::OfficeDocument;
use aigov::office::replacement::{CaseMatching, ReplaceOptions};
use clap::{ArgGroup, Args};
use std::path::{Path, PathBuf};

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

pub fn replace(filepath: &Path, args: &ReplaceArgs) -> error::Result<()> {
    let mut document = Document::from_file(filepath)?;
    let output = args.output.as_deref().unwrap_or(filepath);
    let options = args.options();

    let count = document.replace_text(&args.search, &args.replace, options)?;
    if count == 0 {
        super::print_warning(format!("Search term '{}' not found", args.search));
        return Ok(());
    }

    document.save(output)?;
    super::print_success(format!("Replaced {count} occurrences"));

    Ok(())
}
