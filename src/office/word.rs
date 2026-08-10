use crate::office::OfficeDocument;

// Common operations on Word-processing documents.
///
/// This trait is format-independent. DOCX, DOCM, and legacy DOC
/// implementations can provide their own backend.
pub trait WordDocument: OfficeDocument {}
