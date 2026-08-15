//! Error types returned by the library's package, metadata, and document APIs.

use ooxml::PartName;
use ooxml::error::OoxmlError;

/// Errors that can occur while reading, modifying, or writing an Office package.
///
/// Most APIs expose these errors through the crate-level [`Result`] alias. The
/// variants preserve the underlying I/O, ZIP, XML, and JSON errors where
/// applicable, so callers can inspect or display the original cause.
///
/// # Examples
///
/// ```
/// use aigov::error::{OfficeError, Result};
/// use std::path::Path;
///
/// fn open_document(path: &Path) -> Result<()> {
///     match aigov::package::OoxmlPackage::from_file(path) {
///         Ok(_) => Ok(()),
///         Err(OfficeError::Io(error)) => {
///             eprintln!("could not open document: {error}");
///             Err(OfficeError::Io(error))
///         }
///         Err(error) => Err(error),
///     }
/// }
/// # let _ = open_document;
/// ```
#[derive(Debug, thiserror::Error)]
pub enum OfficeError {
    /// An operating-system or filesystem error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The input is not a valid ZIP package.
    #[error("Invalid ZIP package: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// An XML part could not be parsed or serialized.
    #[error("XML processing error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// A requested package part does not exist.
    #[error("Package part not found: {0}")]
    PartNotFound(PartName),

    /// The document uses an Office file type unsupported by the operation.
    #[error("Unsupported Office file type: {0}")]
    UnsupportedFileType(String),

    /// The package contains invalid or inconsistent Office data.
    #[error("Invalid Office office: {0}")]
    InvalidDocument(String),

    /// Metadata could not be serialized or deserialized as JSON.
    #[error("Failed to serialize metadata: {0}")]
    SerializeMetadata(#[from] serde_json::Error),

    /// A document conversion operation failed.
    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error(
        "required tool {tool} was not found (`{command}`); install it and ensure it is available on PATH"
    )]
    ExternalToolNotFound {
        tool: &'static str,
        command: &'static str,
    },

    #[error("Document processing error: {0}")]
    Undoc(#[from] undoc::Error),

    #[error("AI policy error: {0}")]
    InvalidAiPolicy(String),

    #[error("Ooxml error: {0}")]
    Ooxml(#[from] OoxmlError),
}

/// The result type used by the library's public APIs.
pub type Result<T> = std::result::Result<T, OfficeError>;
