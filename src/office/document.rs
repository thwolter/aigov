use super::metadata::OfficeMetadata;
use crate::document::ValidationIssue;
use crate::error::Result;
use crate::package::PartName;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default)]
pub struct ReplaceOptions {
    pub case_matching: CaseMatching,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CaseMatching {
    #[default]
    Sensitive,
    UnicodeInsensitive,
}

pub trait OfficeDocument {
    fn from_file(path: &Path) -> Result<Self>
    where
        Self: Sized;

    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;

    /// Replaces all occurrences of `search` with `replacement`.
    ///
    /// Returns the number of replacements made.
    fn replace_text(
        &mut self,
        search: &str,
        replacement: &str,
        options: ReplaceOptions,
    ) -> Result<usize>;

    fn source_path(&self) -> Option<&Path>;

    fn parts(&self) -> Vec<PartName>;

    fn read_part(&self, part: &PartName) -> Result<&[u8]>;

    fn write_part(&mut self, part: PartName, content: Vec<u8>) -> Result<()>;

    fn remove_part(&mut self, part: &PartName) -> Result<()>;

    fn contains_part(&self, part: &PartName) -> bool;

    fn metadata(&self) -> Result<OfficeMetadata>;

    fn set_metadata(&mut self, metadata: &OfficeMetadata) -> Result<()>;

    fn validate(&self) -> Result<Vec<ValidationIssue>>;

    fn save(&self, destination: &Path) -> Result<()>;
}
