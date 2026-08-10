use crate::cli::replace::ReplaceArgs;
use aigov::error::Result;
use aigov::formats::Document;
use aigov::office::OfficeDocument;
use std::path::Path;

pub fn replace(filepath: &Path, args: &ReplaceArgs) -> Result<()> {
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
