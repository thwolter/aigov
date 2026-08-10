use crate::error;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

#[derive(Debug, PartialEq, Eq)]
pub(in crate::ooxml) struct TextElement {
    pub name: Vec<u8>,
    pub value: String,
}

/// Constructs one parsed text element.
fn parse_text_element(name: &[u8], value: String) -> TextElement {
    TextElement {
        name: name.to_vec(),
        value,
    }
}

/// Parses the text elements contained by an XML document.
///
/// The root element is ignored. Child elements are returned with their
/// namespace prefix removed from the name.
pub(in crate::ooxml) fn parse_text_elements(xml: &[u8]) -> error::Result<Vec<TextElement>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut elements = Vec::new();

    let mut depth = 0usize;
    let mut current: Option<(Vec<u8>, String, usize)> = None;

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                depth += 1;

                // The root element is at depth 1. Parse its children.
                if depth > 1 && current.is_none() {
                    current = Some((element.local_name().as_ref().to_vec(), String::new(), depth));
                }
            }

            Event::Empty(element) if depth >= 1 => {
                elements.push(parse_text_element(
                    element.local_name().as_ref(),
                    String::new(),
                ));
            }

            Event::Text(text) if current.is_some() => {
                let text = text.xml10_content().map_err(invalid_property_text)?;

                let text = quick_xml::escape::unescape(&text).map_err(invalid_property_text)?;

                if let Some((_, value, _)) = current.as_mut() {
                    value.push_str(&text);
                }
            }

            Event::GeneralRef(reference) if current.is_some() => {
                let reference = reference.xml10_content().map_err(invalid_property_text)?;

                let reference = format!("&{reference};");
                let reference =
                    quick_xml::escape::unescape(&reference).map_err(invalid_property_text)?;

                if let Some((_, value, _)) = current.as_mut() {
                    value.push_str(&reference);
                }
            }

            Event::CData(text) if current.is_some() => {
                let text = text.xml10_content().map_err(invalid_property_text)?;

                if let Some((_, value, _)) = current.as_mut() {
                    value.push_str(&text);
                }
            }

            Event::End(_) => {
                let finished = current
                    .as_ref()
                    .is_some_and(|(_, _, element_depth)| *element_depth == depth);

                if finished {
                    let (name, value, _) = current.take().expect("element exists");

                    elements.push(parse_text_element(&name, value));
                }

                depth = depth.saturating_sub(1);
            }

            Event::Eof => break,

            _ => {}
        }

        buffer.clear();
    }

    Ok(elements)
}

fn invalid_property_text(error: impl std::fmt::Display) -> error::OfficeError {
    error::OfficeError::InvalidDocument(format!("Invalid metadata property text: {error}"))
}

/// Writes a text element to the XML writer.
pub(in crate::ooxml) fn write_text_element(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: &str,
) -> error::Result<()> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;

    Ok(())
}
