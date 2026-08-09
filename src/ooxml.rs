mod content_types;
mod package;
mod properties;
mod relationships;
mod xml;

pub use package::OoxmlPackage;
pub use properties::{read_metadata, write_metadata};
