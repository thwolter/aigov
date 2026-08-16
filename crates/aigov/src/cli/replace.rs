use crate::cli::{DestinationArgs, FileCommand};
use aigov::error;
use aigov::formats::Document;
use aigov::office::OfficeDocument;
use aigov::office::replacement::{CaseMatching, ReplaceOptions};
use clap::Args;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct ReplaceArgs {
    #[arg(short, long, value_name = "TEXT", help = "Text to search for")]
    search: String,

    #[arg(short, long, value_name = "TEXT", help = "Text to replace with")]
    replace: String,

    #[arg(short, long, help = "Ignore case when searching")]
    ignore_case: bool,

    #[command(flatten)]
    destination: DestinationArgs,
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

pub(crate) fn run(command: &FileCommand<ReplaceArgs>) -> error::Result<()> {
    let args = &command.args;

    let mut document = Document::from_file(&command.input)?;
    let options = args.options();

    let count = document.replace_text(&args.search, &args.replace, options)?;
    if count == 0 {
        super::print_warning(format!("Search term '{}' not found", args.search));
        return Ok(());
    }

    document.save(command.args.destination.output_or(&command.input))?;
    super::print_success(format!("Replaced {count} occurrences"));

    Ok(())
}
