pub mod document;
pub mod error;
pub mod formats;
pub mod office;
pub mod ooxml;
pub mod package;

pub use error::{OfficeError, Result};
pub use formats::Document;
pub use office::{metadata, metadata_patch};
