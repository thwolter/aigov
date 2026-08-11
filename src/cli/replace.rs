use crate::cli::Cli;
use aigov::error;
use aigov::formats::Document;
use aigov::office::OfficeDocument;
use aigov::office::replacement::{CaseMatching, ReplaceOptions};
use clap::{Args, CommandFactory};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct ReplaceArgs {
    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "Write the updated document to PATH",
        group = "destination"
    )]
    output: Option<PathBuf>,

    #[arg(short, long, value_name = "TEXT", help = "Text to search for")]
    search: Option<String>,

    #[arg(short, long, value_name = "TEXT", help = "Text to replace with")]
    replace: Option<String>,

    #[arg(short, long, help = "Ignore case when searching")]
    ignore_case: bool,
}

impl ReplaceArgs {
    fn options(&self) -> ReplaceOptions {
        ReplaceOptions {
            case_matching: if self.ignore_case {
                CaseMatching::UnicodeInsensitive
            } else {
                CaseMatching::Sensitive
            },
        }
    }
}

pub fn run(filepath: &Path, args: &ReplaceArgs) -> error::Result<()> {
    let (Some(search), Some(replacement)) = (args.search.as_deref(), args.replace.as_deref())
    else {
        Cli::command()
            .find_subcommand_mut("replace")
            .expect("replace subcommand defined")
            .print_help()?;
        return Ok(());
    };

    let mut document = Document::from_file(filepath)?;
    let output = args.output.as_deref().unwrap_or(filepath);
    let options = args.options();

    let count = document.replace_text(search, replacement, options)?;
    if count == 0 {
        super::print_warning(format!("Search term '{}' not found", search));
        return Ok(());
    }

    document.save(output)?;
    super::print_success(format!("Replaced {count} occurrences"));

    Ok(())
}
