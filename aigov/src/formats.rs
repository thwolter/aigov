mod docx;

use crate::office::{OfficeDocument, OoxmlDocument};
use crate::{error, office};
use file_format::FileFormat;
use std::path::Path;

use crate::office::policy;
pub use docx::DocxDocument;
use ooxml;

/// Opens Office documents through the format-specific [`office::OfficeDocument`]
/// implementation.
///
/// The factory detects the input format and returns its concrete document
/// variant. DOCX is currently the only supported format.
///
/// # Examples
///
/// This example validates a document before publishing it, rejecting errors
/// while accepting warnings.
///
/// ```
/// use aigov::{
///     document::Severity,
///     error::{OfficeError, Result},
///     formats::Document,
/// };
/// use aigov::office::OfficeDocument;
/// use std::path::Path;
///
/// fn publish(input: &Path, output: &Path) -> Result<()> {
///     let document = match Document::from_file(input) {
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
pub enum Document {
    Docx(DocxDocument),
}

impl Document {
    /// Detects the file format at `path` and opens it as an office document.
    ///
    /// Returns [`error::OfficeError::Io`] if the file cannot be read,
    /// [`error::OfficeError::UnsupportedFileType`] if its detected format is not
    /// supported, or the error produced while opening a detected DOCX package.
    pub fn from_file(path: impl AsRef<Path>) -> error::Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path)?;
        let fmt = FileFormat::from_bytes(&bytes);

        match fmt {
            FileFormat::OfficeOpenXmlDocument => Ok(Self::Docx(DocxDocument::from_bytes(&bytes)?)),
            // further file formats to be implemented
            _ => Err(error::OfficeError::UnsupportedFileType(
                path.display().to_string(),
            )),
        }
    }

    pub fn as_ai_policy_document(&mut self) -> Option<&mut dyn policy::AiPolicyDocument> {
        match self {
            Self::Docx(document) => Some(document),
        }
    }
}

impl OfficeDocument for Document {
    fn replace_text(
        &mut self,
        search: &str,
        replacement: &str,
        options: office::replacement::ReplaceOptions,
    ) -> error::Result<usize> {
        match self {
            Self::Docx(document) => document.replace_text(search, replacement, options),
        }
    }

    fn source_path(&self) -> Option<&Path> {
        match self {
            Self::Docx(document) => document.source_path(),
        }
    }

    fn metadata(&self) -> error::Result<ooxml::Metadata> {
        match self {
            Self::Docx(document) => document.metadata(),
        }
    }

    fn set_metadata(&mut self, metadata: &ooxml::Metadata) -> error::Result<()> {
        match self {
            Self::Docx(document) => document.set_metadata(metadata),
        }
    }

    fn validate(&self) -> error::Result<Vec<crate::document::ValidationIssue>> {
        match self {
            Self::Docx(document) => document.validate(),
        }
    }

    fn save(&self, destination: &Path) -> error::Result<()> {
        match self {
            Self::Docx(document) => document.save(destination),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::formats::docx::tests::minimal_docx;
    use crate::office::OfficeDocument;
    use std::env::temp_dir;

    #[test]
    fn load_from_file() {
        let path = temp_dir().as_path().join("test.docx");
        let _ = minimal_docx().save(path.as_path()).unwrap();
        assert!(Document::from_file(path).is_ok())
    }

    #[test]
    fn through_file_not_found_error() {
        let path = temp_dir().as_path().join("file-does-not-exist.docx");
        let error = match Document::from_file(&path).err() {
            Some(error) => error,
            None => panic!("Expected an error"),
        };
        assert!(
            matches!(error, error::OfficeError::Io(ref error) if error.kind() == std::io::ErrorKind::NotFound)
        );
    }

    #[test]
    fn through_unsupported_file_error() {
        let path = temp_dir().as_path().join("test.xyz");
        std::fs::write(&path, b"not an Office document").unwrap();
        assert!(Document::from_file(path).is_err())
    }
}
