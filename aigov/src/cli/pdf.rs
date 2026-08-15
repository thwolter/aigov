use crate::cli::FileCommand;
use aigov::error;
use aigov::error::OfficeError;
use clap::Args;
use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args)]
pub struct PdfArgs {
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub fn run(command: &FileCommand<PdfArgs>) -> error::Result<()> {
    let output_dir = command
        .args
        .output
        .as_deref()
        .unwrap_or(command.input.parent().unwrap());
    let status = Command::new("soffice")
        .args([
            "--headless",
            "--convert-to",
            "pdf",
            "--outdir",
            output_dir.to_str().unwrap(),
            command.input.to_str().unwrap(),
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
        let command = FileCommand {
            input: input.to_path_buf(),
            args: pdf_args,
        };
        run(&command).unwrap();
        assert!(dir.path().join("test").exists());

        let bytes = fs::read(dir.path().join("test/minimal.pdf")).unwrap();
        assert!(bytes.starts_with(b"%PDF-"))
    }
}
