use crate::office::policy::AiPolicy;
use crate::{error, ooxml};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

const WORD_BODY: &[u8] = b"w:body";
const WORD_PARAGRAPH: &[u8] = b"w:p";
const WORD_SECTION_PROPERTIES: &[u8] = b"w:sectPr";
const WORD_TEXT: &[u8] = b"w:t";
const POLICY_PREFIX: &str = "aigov:policy:";

pub(super) fn has_policy_marker(xml: &[u8]) -> error::Result<bool> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut paragraph = None;

    loop {
        let event = reader.read_event_into(&mut buffer)?.into_owned();

        match event {
            Event::Eof => break,
            Event::Start(start) if start.name().as_ref() == WORD_PARAGRAPH => {
                paragraph = Some(vec![Event::Start(start)]);
            }
            Event::End(end) if paragraph.is_some() && end.name().as_ref() == WORD_PARAGRAPH => {
                paragraph
                    .as_mut()
                    .expect("paragraph exists")
                    .push(Event::End(end));
                if paragraph_contains_policy(&paragraph.take().expect("paragraph exists"))? {
                    return Ok(true);
                }
            }
            event => {
                if let Some(events) = &mut paragraph {
                    events.push(event);
                }
            }
        }

        buffer.clear();
    }

    if paragraph.is_some() {
        return Err(error::OfficeError::InvalidAiPolicy(
            "unterminated Word paragraph".into(),
        ));
    }

    Ok(false)
}

pub(super) fn rewrite_hidden_policy(
    xml: &[u8],
    replacement: Option<&AiPolicy>,
) -> error::Result<(bool, Vec<u8>)> {
    let mut reader = Reader::from_reader(xml);
    let mut writer = Writer::new(Vec::new());
    let mut buffer = Vec::new();
    let mut paragraph = None;
    let mut rewritten = false;

    loop {
        let event = reader.read_event_into(&mut buffer)?.into_owned();

        match event {
            Event::Eof => break,
            Event::Start(start) if start.name().as_ref() == WORD_PARAGRAPH => {
                paragraph = Some(vec![Event::Start(start)]);
            }
            Event::End(end) if paragraph.is_some() && end.name().as_ref() == WORD_PARAGRAPH => {
                paragraph
                    .as_mut()
                    .expect("paragraph exists")
                    .push(Event::End(end));
                let paragraph = paragraph.take().expect("paragraph exists");

                if paragraph_contains_policy(&paragraph)? {
                    rewritten = true;
                    if let Some(policy) = replacement {
                        write_hidden_policy_prompt(&mut writer, policy)?;
                    }
                } else {
                    for event in paragraph {
                        writer.write_event(event)?;
                    }
                }
            }
            event => {
                if let Some(events) = &mut paragraph {
                    events.push(event);
                } else {
                    writer.write_event(event)?;
                }
            }
        }

        buffer.clear();
    }

    if paragraph.is_some() {
        return Err(error::OfficeError::InvalidAiPolicy(
            "unterminated Word paragraph".into(),
        ));
    }

    Ok((rewritten, writer.into_inner()))
}

fn paragraph_contains_policy(events: &[Event<'_>]) -> error::Result<bool> {
    let mut in_text = false;
    let mut text = String::new();

    for event in events {
        match event {
            Event::Start(start) if start.name().as_ref() == WORD_TEXT => in_text = true,
            Event::End(end) if end.name().as_ref() == WORD_TEXT => in_text = false,
            Event::Text(value) if in_text => {
                let value = value
                    .xml10_content()
                    .map_err(|error| error::OfficeError::InvalidAiPolicy(error.to_string()))?;
                let value = quick_xml::escape::unescape(&value)
                    .map_err(|error| error::OfficeError::InvalidAiPolicy(error.to_string()))?;
                text.push_str(&value);
            }
            _ => {}
        }
    }

    Ok(text.starts_with(POLICY_PREFIX))
}

pub(super) fn write_hidden_policy(xml: &[u8], policy: &AiPolicy) -> error::Result<Vec<u8>> {
    let mut reader = Reader::from_reader(xml);
    let mut writer = Writer::new(Vec::new());
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
                    write_hidden_policy_prompt(&mut writer, policy)?;
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
                    write_hidden_policy_prompt(&mut writer, policy)?;
                    injected = true;
                }

                writer.write_event(Event::Empty(empty))?;
            }
            Event::End(end) => {
                let is_body = end.name().as_ref() == WORD_BODY;

                if is_body && !injected {
                    write_hidden_policy_prompt(&mut writer, policy)?;
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
    if !injected {
        return Err(error::OfficeError::InvalidDocument(
            "Word document has no body element".into(),
        ));
    }

    Ok(writer.into_inner())
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
    ooxml::write_text_element(writer, "w:t", &format!("{POLICY_PREFIX}{prompt}"))?;

    writer.write_event(Event::End(BytesEnd::new("w:r")))?;
    writer.write_event(Event::End(BytesEnd::new("w:p")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::docx::{AiPolicyDocument, tests::minimal_docx};

    #[test]
    fn manages_hidden_policy_prompt() {
        let mut document = minimal_docx();
        let policy = AiPolicy {
            id: "internal-use".into(),
            human_review_required: true,
            training_allowed: false,
            attribution_required: true,
            owner: Some("legal".into()),
        };

        document.inject_policy(&policy).unwrap();

        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();
        assert!(xml.contains("w:vanish"));
        assert!(xml.contains(POLICY_PREFIX));
        assert!(document.has_policy().unwrap());

        let updated = AiPolicy {
            id: "public-use".into(),
            ..policy
        };
        document.update_policy(&updated).unwrap();

        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();
        assert!(!xml.contains("Policy ID: internal-use"));
        assert!(xml.contains("Policy ID: public-use"));
        assert!(document.remove_policy().unwrap());
        assert!(!document.has_policy().unwrap());
        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();
        assert!(xml.contains("Hello world"));
        assert!(!xml.contains(POLICY_PREFIX));
        assert!(!document.remove_policy().unwrap());
        assert!(document.update_policy(&updated).is_err());
    }

    #[test]
    fn reinjects_policy_before_section_properties() {
        let mut document = minimal_docx();
        document.package.write_part(
            "word/document.xml".into(),
            br#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello world</w:t></w:r></w:p><w:sectPr/></w:body></w:document>"#.to_vec(),
        );
        let policy = AiPolicy::default();

        document.inject_policy(&policy).unwrap();
        let first_injection = document.document_xml().unwrap().to_vec();
        assert!(document.remove_policy().unwrap());
        document.inject_policy(&policy).unwrap();

        assert_eq!(first_injection, document.document_xml().unwrap());
    }
}
