use super::metadata::OfficeMetadata;
use crate::document::ValidationIssue;
use crate::error::Result;
pub(crate) use crate::package::{OoxmlPackage, PartName};
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

pub trait OoxmlDocument: OfficeDocument {
    fn from_package(package: OoxmlPackage) -> Self
    where
        Self: Sized;

    fn from_file(path: &Path) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::from_package(OoxmlPackage::from_file(path)?))
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::from_package(OoxmlPackage::from_bytes(bytes)?))
    }

    fn package(&self) -> &OoxmlPackage;

    fn mut_package(&mut self) -> &mut OoxmlPackage;

    fn parts(&self) -> Vec<PartName> {
        self.package().parts()
    }

    fn read_part(&self, part: &PartName) -> Result<&[u8]> {
        self.package().read_part(part)
    }

    fn write_part(&mut self, part: PartName, content: Vec<u8>) -> Result<()> {
        self.mut_package().write_part(part, content);
        Ok(())
    }

    fn remove_part(&mut self, part: &PartName) -> Result<()> {
        self.mut_package().remove_part(part);
        Ok(())
    }

    fn contains_part(&self, part: &PartName) -> bool {
        self.package().contains_part(part)
    }
}

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

    fn source_path(&self) -> Option<&Path>;

    fn metadata(&self) -> Result<OfficeMetadata>;

    fn set_metadata(&mut self, metadata: &OfficeMetadata) -> Result<()>;

    fn validate(&self) -> Result<Vec<ValidationIssue>>;

    fn save(&self, destination: &Path) -> Result<()>;
}
