use super::metadata::OfficeMetadata;
use crate::document::ValidationIssue;
use crate::error::Result;
use crate::office::replacement::ReplaceOptions;
pub(crate) use crate::package::{OoxmlPackage, PartName};
use std::path::Path;

/// An [`OfficeDocument`] backed by an OOXML package.
///
/// This trait provides package-level construction and access for concrete
/// OOXML formats. Most applications should open a document through
/// [`crate::formats::Document`]; use this trait when a concrete type is needed.
///
/// # Examples
///
/// ```no_run
/// use aigov::{
///     formats::DocxDocument,
///     office::{OfficeDocument, OoxmlDocument},
/// };
/// use std::path::Path;
///
/// let document = <DocxDocument as OoxmlDocument>::from_file(Path::new("report.docx"))?;
/// assert_eq!(document.source_path(), Some(Path::new("report.docx")));
/// # Ok::<(), aigov::error::OfficeError>(())
/// ```
pub trait OoxmlDocument: OfficeDocument {
    /// Creates a document from an already-open OOXML package.
    fn from_package(package: OoxmlPackage) -> Self
    where
        Self: Sized;

    /// Opens an OOXML package from `path` and constructs the document.
    ///
    /// The resulting document retains the source path.
    fn from_file(path: &Path) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::from_package(OoxmlPackage::from_file(path)?))
    }

    /// Creates a document from OOXML package bytes.
    ///
    /// Documents created this way have no source path.
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::from_package(OoxmlPackage::from_bytes(bytes)?))
    }

    /// Returns the underlying OOXML package.
    fn package(&self) -> &OoxmlPackage;

    /// Returns mutable access to the underlying OOXML package.
    fn mut_package(&mut self) -> &mut OoxmlPackage;

    /// Returns all package part names.
    fn parts(&self) -> Vec<PartName> {
        self.package().parts()
    }

    /// Returns the bytes stored in `part`.
    fn read_part(&self, part: &PartName) -> Result<&[u8]> {
        self.package().read_part(part)
    }

    /// Writes or replaces `part` in the in-memory package.
    ///
    /// Call [`OfficeDocument::save`] to persist the change.
    fn write_part(&mut self, part: PartName, content: Vec<u8>) -> Result<()> {
        self.mut_package().write_part(part, content);
        Ok(())
    }

    /// Removes `part` from the in-memory package.
    fn remove_part(&mut self, part: &PartName) -> Result<()> {
        self.mut_package().remove_part(part);
        Ok(())
    }

    /// Returns whether the package contains `part`.
    fn contains_part(&self, part: &PartName) -> bool {
        self.package().contains_part(part)
    }
}

/// Format-neutral operations on a mutable Office document.
///
/// Implementations expose semantic text replacement, metadata, structural
/// validation, and persistence. Use [`crate::formats::Document`] when the
/// document format is not known at compile time.
pub trait OfficeDocument {
    /// Replaces all occurrences of `search` with `replacement`.
    ///
    /// Returns the number of replacements made.
    fn replace_text(
        &mut self,
        search: &str,
        replacement: &str,
        options: ReplaceOptions,
    ) -> Result<usize>;

    /// Returns the source path when the document was opened from a file.
    ///
    /// Documents created from bytes have no source path.
    fn source_path(&self) -> Option<&Path>;

    /// Reads the document's core, extended, and custom metadata.
    fn metadata(&self) -> Result<OfficeMetadata>;

    /// Replaces the document's metadata in memory.
    ///
    /// Call [`Self::save`] to persist the updated document.
    fn set_metadata(&mut self, metadata: &OfficeMetadata) -> Result<()>;

    /// Checks the document for structural issues.
    ///
    /// Callers decide whether warnings are acceptable; errors should normally
    /// block publishing.
    fn validate(&self) -> Result<Vec<ValidationIssue>>;

    /// Saves the in-memory document to `destination`.
    ///
    /// An existing destination file is replaced.
    fn save(&self, destination: &Path) -> Result<()>;
}
