use crate::OfficeError;
use crate::cli::pdf::PdfArgs;
use crate::error::Result;
use std::path::Path;
use std::process::Command;

pub fn convert_to_pdf(filepath: &Path, args: &PdfArgs) -> Result<()> {
    let output_dir = args.output.as_deref().unwrap_or(filepath.parent().unwrap());
    let status = Command::new("soffice")
        .args([
            "--headless",
            "--convert-to", "pdf",
            "--outdir",
            output_dir.to_str().unwrap(),
            filepath.to_str().unwrap(),
        ])
        .status()?;

    if !status.success() {
        return Err(OfficeError::Conversion(status.to_string()));
    }

    Ok(())
}