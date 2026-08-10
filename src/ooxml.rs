mod properties;
mod xml;

pub use properties::{read_metadata, write_metadata};
pub use xml::{parse_text_elements, write_text_element};
