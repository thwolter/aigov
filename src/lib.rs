//! Read, validate, and modify OOXML Office documents.
//!
//! The crate currently opens DOCX files through [`formats::Document`]. It
//! exposes metadata, OOXML package access, and document capabilities through
//! traits such as [`office::OfficeDocument`].
//!
//! # Example
//!
//! Open a DOCX file, reject structural validation errors, and save a copy:
//!
//! ```no_run
//! use aigov::{
//!     document::Severity,
//!     formats::Document,
//!     office::OfficeDocument,
//! };
//! use std::path::Path;
//!
//! let document = Document::from_file(Path::new("report.docx"))?;
//!
//! if document
//!     .validate()?
//!     .iter()
//!     .any(|issue| issue.severity == Severity::Error)
//! {
//!     return Err("refusing to publish an invalid document".into());
//! }
//!
//! document.save(Path::new("published-report.docx"))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

/// Document validation types.
pub mod document;
/// Public error types and the crate result alias.
pub mod error;
/// Concrete document formats and format dispatch.
pub mod formats;
/// Format-neutral document capabilities and metadata types.
pub mod office;
/// OOXML metadata codecs.
pub mod ooxml;
/// OOXML package access and persistence.
pub mod package;
