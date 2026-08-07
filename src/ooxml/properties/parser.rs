use crate::error::{OfficeError, Result};
use crate::office::OfficeMetadata;
use quick_xml::{
    Reader, Writer,
    events::{BytesEnd, BytesStart, BytesText, Event},
};

pub trait MetadataProperty: Copy {
    /// Returns the property for the given local name.
    fn from_local_name(name: &[u8]) -> Option<Self>;

    /// Returns the local name of the property.
    fn local_name(self) -> &'static [u8];

    /// Sets the property value.
    fn set(self, metadata: &mut OfficeMetadata, value: &str);

    /// Parses the property value from the XML.
    fn parse(xml: &[u8], metadata: &mut OfficeMetadata) -> Result<()> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        let mut property = None;
        let mut value = String::new();

        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    if let Some(next_property) =
                        Self::from_local_name(element.local_name().as_ref())
                    {
                        property = Some(next_property);
                        value.clear();
                    }
                }
                Event::Text(text) if property.is_some() => {
                    let text = text.xml10_content().map_err(invalid_property_text)?;
                    let text = quick_xml::escape::unescape(&text).map_err(invalid_property_text)?;
                    value.push_str(&text);
                }
                Event::GeneralRef(reference) if property.is_some() => {
                    let reference = reference.xml10_content().map_err(invalid_property_text)?;
                    let reference = format!("&{reference};");
                    let reference =
                        quick_xml::escape::unescape(&reference).map_err(invalid_property_text)?;
                    value.push_str(&reference);
                }
                Event::CData(text) if property.is_some() => {
                    value.push_str(&text.xml10_content().map_err(invalid_property_text)?);
                }
                Event::End(element) => {
                    if let Some(current_property) = property
                        && current_property.local_name() == element.local_name().as_ref()
                    {
                        current_property.set(metadata, &value);
                        property = None;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buffer.clear();
        }

        Ok(())
    }
}

fn invalid_property_text(error: impl std::fmt::Display) -> OfficeError {
    OfficeError::InvalidDocument(format!("Invalid metadata property text: {error}"))
}

/// Writes a text element to the XML writer.
pub(super) fn write_text_element(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: &str,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;

    Ok(())
}

pub(super) fn write_timestamp_element(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: &str,
) -> Result<()> {
    let mut element = BytesStart::new(name);
    element.push_attribute(("xsi:type", "dcterms:W3CDTF"));
    writer.write_event(Event::Start(element))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;

    Ok(())
}
