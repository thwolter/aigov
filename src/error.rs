use crate::office::PartName;

#[derive(Debug, thiserror::Error)]
pub enum OfficeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid ZIP package: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML processing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Package part not found: {0}")]
    PartNotFound(PartName),

    #[error("Unsupported Office file type: {0}")]
    UnsupportedFileType(String),

    #[error("Invalid Office office: {0}")]
    InvalidDocument(String),
}

pub type Result<T> = std::result::Result<T, OfficeError>;
