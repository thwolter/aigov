pub mod commands;
pub mod error;
mod formats;
mod office;
mod ooxml;

pub mod cli;

pub use error::{OfficeError, Result};
