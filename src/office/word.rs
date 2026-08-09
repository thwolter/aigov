use crate::error;
use crate::office::OfficeDocument;

// Common operations on Word-processing documents.
///
/// This trait is format-independent. DOCX, DOCM, and legacy DOC
/// implementations can provide their own backend.
pub trait WordDocument: OfficeDocument {
    /// Replaces all occurrences of `search` with `replacement`.
    ///
    /// Returns the number of replacements made.
    fn replace_text(
        &mut self,
        search: &str,
        replacement: &str,
        ignore_case: bool,
    ) -> error::Result<usize>;
}
