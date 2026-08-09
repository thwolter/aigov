use crate::document::ValidationIssue;
use crate::error::Result;
use crate::package::PartName;
use std::path::Path;

use super::metadata::OfficeMetadata;

pub trait OfficeDocument {
    fn open(path: &Path) -> Result<Self>
    where
        Self: Sized;

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
