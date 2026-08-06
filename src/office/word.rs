use crate::office::OfficeDocument;
use crate::error;

pub trait WordDocument: OfficeDocument {
    /// Returns the XML content of the document.
    fn document_xml(&self) -> error::Result<&[u8]>;

    /// Returns the XML content of the styles.
    fn styles_xml(&self) -> error::Result<Option<&[u8]>>;

    /// Replaces all occurrences of `search` with `replacement` in the document.
    fn replace_text(&mut self, search: &str, replacement: &str) -> error::Result<usize>;
}