use crate::cli::unzip::UnzipArgs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io};
use zip::ZipArchive;

/// Unzip a office and store its contents in a subfolder
pub fn unzip_document(filepath: &str, args: &UnzipArgs) -> crate::Result<()> {
    let file = File::open(filepath)?;
    let mut archive = ZipArchive::new(file)?;
    let document_path = Path::new(filepath);
    let default_output_directory = document_path.with_extension("");
    let output_directory = args
        .output
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or(default_output_directory);

    fs::create_dir_all(&output_directory)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(enclosed_name) = entry.enclosed_name() else {
            continue;
        };
        let output_path = output_directory.join(enclosed_name);

        if entry.is_dir() {
            fs::create_dir_all(&output_path)?;
        } else {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut output_file = File::create(&output_path)?;
            io::copy(&mut entry, &mut output_file)?;
        }
    }

    super::print_success(filepath, "Package unzipped");

    Ok(())
}
