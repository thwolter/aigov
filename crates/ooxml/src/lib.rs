pub mod error;
mod opc;
mod properties;
pub mod xml;

pub use opc::{OoxmlPackage, PartName};
pub use properties::metadata::Metadata;
pub use properties::metadata_patch::MetadataPatch;
pub use properties::{read_metadata, write_metadata};
pub use xml::write_text_element;
