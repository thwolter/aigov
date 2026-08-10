use crate::office::policy::AiPolicy;
use crate::{error, ooxml};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

const WORD_BODY: &[u8] = b"w:body";
const WORD_SECTION_PROPERTIES: &[u8] = b"w:sectPr";
const POLICY_PREFIX: &str = "aigov:policy:";

pub(super) fn read_policy_from_document_xml(
    reader: &mut Reader<&[u8]>,
) -> error::Result<Option<AiPolicy>> {
    let mut buffer = Vec::new();
    let mut text = None;

    loop {
        match reader.read_event_into(&mut buffer)?.into_owned() {
            Event::Start(start) if start.name().as_ref() == b"w:t" => {
                text = Some(String::new());
            }
            Event::Text(value) if text.is_some() => {
                let value = value
                    .xml10_content()
                    .map_err(|error| error::OfficeError::InvalidAiPolicy(error.to_string()))?;
                let value = quick_xml::escape::unescape(&value)
                    .map_err(|error| error::OfficeError::InvalidAiPolicy(error.to_string()))?;

                text.as_mut().expect("text exists").push_str(&value);
            }
            Event::GeneralRef(reference) if text.is_some() => {
                let reference = reference
                    .xml10_content()
                    .map_err(|error| error::OfficeError::InvalidAiPolicy(error.to_string()))?;
                let reference = format!("&{reference};");
                let reference = quick_xml::escape::unescape(&reference)
                    .map_err(|error| error::OfficeError::InvalidAiPolicy(error.to_string()))?;

                text.as_mut().expect("text exists").push_str(&reference);
            }
            Event::End(end) if end.name().as_ref() == b"w:t" => {
                if let Some(payload) = text
                    .take()
                    .and_then(|text| text.strip_prefix(POLICY_PREFIX).map(str::to_owned))
                {
                    return serde_json::from_str(&payload)
                        .map(Some)
                        .map_err(|error| error::OfficeError::InvalidAiPolicy(error.to_string()));
                }
            }
            Event::Eof => return Ok(None),
            _ => {}
        }

        buffer.clear();
    }
}

pub(super) fn write_hidden_policy_paragraph(
    policy: &AiPolicy,
    reader: &mut Reader<&[u8]>,
    writer: &mut Writer<Vec<u8>>,
) -> error::Result<bool> {
    let mut buffer = Vec::new();
    let mut body_depth: Option<usize> = None;
    let mut injected = false;

    loop {
        let event = reader.read_event_into(&mut buffer)?.into_owned();

        match event {
            Event::Start(start) => {
                let is_body = start.name().as_ref() == WORD_BODY;
                let is_body_section_properties =
                    body_depth == Some(1) && start.name().as_ref() == WORD_SECTION_PROPERTIES;

                if is_body_section_properties && !injected {
                    write_hidden_policy_prompt(writer, policy)?;
                    injected = true;
                }

                writer.write_event(Event::Start(start))?;

                if is_body {
                    body_depth = Some(1);
                } else if let Some(depth) = &mut body_depth {
                    *depth += 1;
                }
            }
            Event::Empty(empty) => {
                if body_depth == Some(1)
                    && empty.name().as_ref() == WORD_SECTION_PROPERTIES
                    && !injected
                {
                    write_hidden_policy_prompt(writer, policy)?;
                    injected = true;
                }

                writer.write_event(Event::Empty(empty))?;
            }
            Event::End(end) => {
                let is_body = end.name().as_ref() == WORD_BODY;

                if is_body && !injected {
                    write_hidden_policy_prompt(writer, policy)?;
                    injected = true;
                }

                writer.write_event(Event::End(end))?;

                if is_body {
                    body_depth = None;
                } else if let Some(depth) = &mut body_depth {
                    *depth -= 1;
                }
            }
            Event::Eof => break,
            event => writer.write_event(event)?,
        }

        buffer.clear();
    }
    Ok(injected)
}

fn write_hidden_policy_prompt(
    writer: &mut Writer<Vec<u8>>,
    policy: &AiPolicy,
) -> error::Result<()> {
    writer.write_event(Event::Start(BytesStart::new("w:p")))?;
    writer.write_event(Event::Start(BytesStart::new("w:r")))?;
    writer.write_event(Event::Start(BytesStart::new("w:rPr")))?;
    writer.write_event(Event::Empty(BytesStart::new("w:vanish")))?;
    writer.write_event(Event::End(BytesEnd::new("w:rPr")))?;

    let prompt = policy.to_prompt();
    ooxml::write_text_element(writer, "w:t", &prompt)?;

    writer.write_event(Event::End(BytesEnd::new("w:r")))?;
    writer.write_event(Event::End(BytesEnd::new("w:p")))?;
    Ok(())
}
