use crate::error::{OfficeError, Result};
use crate::formats::docx::DocxDocument;
use crate::office::OfficeDocument;
use file_format::FileFormat;
use std::path::Path;

pub struct Document;

impl Document {
    pub fn open(path: impl AsRef<Path>) -> Result<Box<dyn OfficeDocument>> {
        let path = path.as_ref();
        let fmt = FileFormat::from_file(path)?;

        match fmt.short_name() {
            Some("DOCX") => Ok(Box::new(<DocxDocument as OfficeDocument>::open(path)?)),
            _ => Err(OfficeError::UnsupportedFileType(path.display().to_string())),
        }
    }
}
