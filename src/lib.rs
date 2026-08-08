pub mod error;
mod formats;
mod office;
pub mod ooxml;

pub use error::{OfficeError, Result};
pub use office::metadata;
