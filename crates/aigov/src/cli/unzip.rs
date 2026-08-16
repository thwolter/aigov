use crate::cli::FileCommand;
use aigov::error;
use clap::Args;
use std::fs::File;
use std::path::PathBuf;
use zip::ZipArchive;

#[derive(Args)]
pub struct UnzipArgs {
    /// Output directory; if not provided, file will be extracted to a subfolder with the same name as the file
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,
}

/// Unzip a office and store its contents in a subfolder
pub fn run(command: &FileCommand<UnzipArgs>) -> error::Result<()> {
    let file = File::open(&command.input)?;
    let mut archive = ZipArchive::new(file)?;
    let output_directory = command
        .args
        .output
        .clone()
        .unwrap_or_else(|| command.input.with_extension(""));

    archive.extract(&output_directory)?;

    let filepath = command.input.to_string_lossy();
    super::print_success(format!("Package unzipped: {filepath}"));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::unzip::run;
    use std::{
        fs,
        io::{Cursor, Write},
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn temporary_path() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);

        std::env::temp_dir().join(format!(
            "aigov-unzip-{timestamp}-{}-{id}.docx",
            std::process::id()
        ))
    }

    #[test]
    fn extracts_nested_files_to_default_directory() {
        let source = temporary_path();
        let output = source.with_extension("");
        let mut bytes = Cursor::new(Vec::new());
        let mut archive = ZipWriter::new(&mut bytes);
        archive
            .start_file("word/document.xml", SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"document").unwrap();
        archive.finish().unwrap();
        fs::write(&source, bytes.into_inner()).unwrap();

        let command = FileCommand {
            input: source,
            args: UnzipArgs { output: None },
        };
        run(&command).unwrap();

        assert_eq!(
            fs::read(output.join("word/document.xml")).unwrap(),
            b"document"
        );

        fs::remove_dir_all(output).unwrap();
        fs::remove_file(&command.input).unwrap();
    }
}
