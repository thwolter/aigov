use crate::error::{OfficeError, Result};
use crate::formats::docx::DocxDocument;
use crate::office::OfficeDocument;
use file_format::FileFormat;
use std::path::Path;

/// Opens Office documents through the format-specific [`OfficeDocument`]
/// implementation.
///
/// The factory detects the input format and hides the concrete document type
/// behind a trait object. DOCX is currently the only supported format.
///
/// # Examples
///
/// This example validates a document before publishing it, rejecting errors
/// while accepting warnings.
///
/// ```
/// use aigov::document::Severity;
/// use aigov::{OfficeDocumentFactory, OfficeError, Result};
/// use aigov::office::OfficeDocument;
/// use std::path::Path;
///
/// fn publish(input: &Path, output: &Path) -> Result<()> {
///     let document = match OfficeDocumentFactory.open(input) {
///         Ok(document) => document,
///         Err(OfficeError::UnsupportedFileType(path)) => {
///             eprintln!("cannot publish unsupported Office file: {path}");
///             return Ok(());
///         }
///         Err(error) => return Err(error),
///     };
///
///     let issues = document.validate()?;
///     if issues.iter().any(|issue| issue.severity == Severity::Error) {
///         return Err(OfficeError::InvalidDocument(
///             "document failed validation and was not published".into(),
///         ));
///     }
///     document.save(output)
/// }
/// # let _ = publish;
/// ```
pub struct OfficeDocumentFactory;

impl OfficeDocumentFactory {
    /// Detects the file format at `path` and opens it as an office document.
    ///
    /// Returns [`OfficeError::Io`] if the file cannot be read,
    /// [`OfficeError::UnsupportedFileType`] if its detected format is not
    /// supported, or the error produced while opening a detected DOCX package.
    pub fn open(&self, path: impl AsRef<Path>) -> Result<Box<dyn OfficeDocument>> {
        let path = path.as_ref();
        let fmt = FileFormat::from_file(path)?;

        match fmt {
            FileFormat::OfficeOpenXmlDocument => {
                Ok(Box::new(<DocxDocument as OfficeDocument>::open(path)?))
            }
            _ => Err(OfficeError::UnsupportedFileType(path.display().to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::formats::docx::tests::minimal_docx;
    use std::env::temp_dir;

    #[test]
    fn through_unsupported_file_error() {
        let path = temp_dir().as_path().join("test.docx");
        let _ = minimal_docx().save(path.as_path()).unwrap();
        assert!(OfficeDocumentFactory.open(path).is_err())
    }
}
