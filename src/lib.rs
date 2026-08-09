pub mod error;
pub mod document;
mod office;
pub mod package;
pub mod ooxml;

pub use error::{OfficeError, Result};
pub use office::{metadata, metadata_patch};
