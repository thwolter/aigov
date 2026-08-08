pub mod content_types;
pub mod package;
pub mod properties;
pub mod relationships;
pub mod xml;

pub use package::OoxmlPackage;
pub use properties::{read_metadata, write_metadata};
