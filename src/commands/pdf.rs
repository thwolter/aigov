use crate::cli::pdf::PdfArgs;
use aigov::error::{OfficeError, Result};
use std::io;
use std::path::Path;
use std::process::Command;

pub fn convert_to_pdf(filepath: &Path, args: &PdfArgs) -> Result<()> {
    let output_dir = args.output.as_deref().unwrap_or(filepath.parent().unwrap());
    let status = Command::new("soffice")
        .args([
            "--headless",
            "--convert-to",
            "pdf",
            "--outdir",
            output_dir.to_str().unwrap(),
            filepath.to_str().unwrap(),
        ])
        .status()
        .map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => OfficeError::ExternalToolNotFound {
                tool: "LibreOffice",
                command: "soffice",
            },
            _ => OfficeError::Io(error),
        })?;

    if !status.success() {
        return Err(OfficeError::Conversion(status.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_convert_to_pdf() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("minimal.txt");
        fs::write(&input, b"Hello, world!").unwrap();
        assert!(input.exists());

        let pdf_args = PdfArgs {
            output: Some(dir.path().join("test")),
        };
        convert_to_pdf(&input, &pdf_args).unwrap();
        assert!(dir.path().join("test").exists());

        let bytes = fs::read(dir.path().join("test/minimal.pdf")).unwrap();
        assert!(bytes.starts_with(b"%PDF-"))
    }
}
