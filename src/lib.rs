pub mod document;
pub mod error;
mod office;
pub mod ooxml;
pub mod package;

pub use error::{OfficeError, Result};
pub use office::{metadata, metadata_patch};
