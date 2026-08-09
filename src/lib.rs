pub mod document;
pub mod error;
pub mod formats;
pub mod office;
pub mod ooxml;
pub mod package;

pub use error::{OfficeError, Result};
pub use formats::factory::OfficeDocumentFactory;
pub use office::{metadata, metadata_patch};
