use std::path::Path;

use crate::office::document::{CaseMatching, OoxmlDocument, ReplaceOptions};
use crate::{
    document::ValidationIssue,
    error::Result,
    office::OfficeDocument,
    office::metadata::OfficeMetadata,
    ooxml,
    package::{OoxmlPackage, PartName},
};
use quick_xml::{
    Reader, Writer,
    events::{BytesText, Event},
};

const DOCUMENT_XML: &str = "word/document.xml";
const STYLES_XML: &str = "word/styles.xml";
const CONTENT_TYPES_XML: &str = "[Content_Types].xml";
const ROOT_RELS: &str = "_rels/.rels";
const WORD_PARAGRAPH: &[u8] = b"w:p";
const WORD_TEXT: &[u8] = b"w:t";

pub struct DocxDocument {
    package: OoxmlPackage,
}

impl DocxDocument {
    pub fn document_xml(&self) -> Result<&[u8]> {
        self.package.read_part(&DOCUMENT_XML.into())
    }

    pub fn styles_xml(&self) -> Result<Option<&[u8]>> {
        let part = PartName::new(STYLES_XML);

        if self.package.contains_part(&part) {
            Ok(Some(self.package.read_part(&part)?))
        } else {
            Ok(None)
        }
    }
}

impl OoxmlDocument for DocxDocument {
    fn from_package(package: OoxmlPackage) -> Self {
        Self { package }
    }

    fn package(&self) -> &OoxmlPackage {
        &self.package
    }

    fn mut_package(&mut self) -> &mut OoxmlPackage {
        &mut self.package
    }
}

impl OfficeDocument for DocxDocument {
    fn replace_text(
        &mut self,
        search: &str,
        replacement: &str,
        options: ReplaceOptions,
    ) -> Result<usize> {
        let part = PartName::new(DOCUMENT_XML);
        let (count, updated) = replace_document_text(
            self.package.read_part(&part)?,
            search,
            replacement,
            options.case_matching,
        )?;

        if count > 0 {
            self.package.write_part(part, updated);
        }

        Ok(count)
    }

    fn source_path(&self) -> Option<&Path> {
        self.package.source_path()
    }

    fn metadata(&self) -> Result<OfficeMetadata> {
        ooxml::read_metadata(&self.package)
    }

    fn set_metadata(&mut self, metadata: &OfficeMetadata) -> Result<()> {
        ooxml::write_metadata(&mut self.package, metadata)
    }

    fn validate(&self) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        let required_parts = [CONTENT_TYPES_XML, ROOT_RELS, DOCUMENT_XML];

        for required_part in required_parts {
            let part = PartName::new(required_part);

            if !self.contains_part(&part) {
                issues.push(ValidationIssue::error(
                    Some(part),
                    "Required DOCX part is missing",
                ));
            }
        }

        Ok(issues)
    }

    fn save(&self, destination: &Path) -> Result<()> {
        self.package.save(destination)
    }
}

fn replace_document_text(
    xml: &[u8],
    search: &str,
    replacement: &str,
    case_matching: CaseMatching,
) -> Result<(usize, Vec<u8>)> {
    if search.is_empty() {
        return Ok((0, xml.to_vec()));
    }

    let mut reader = Reader::from_reader(xml);
    let mut writer = Writer::new(Vec::new());
    let mut buffer = Vec::new();
    let mut paragraph_events: Option<Vec<Event<'static>>> = None;
    let mut count = 0;

    loop {
        let event = reader.read_event_into(&mut buffer)?.into_owned();

        match event {
            Event::Eof => break,
            Event::Start(start) if start.name().as_ref() == WORD_PARAGRAPH => {
                if let Some(events) = paragraph_events.as_mut() {
                    events.push(Event::Start(start));
                } else {
                    paragraph_events = Some(vec![Event::Start(start)]);
                }
            }
            Event::End(end)
                if paragraph_events.is_some() && end.name().as_ref() == WORD_PARAGRAPH =>
            {
                paragraph_events
                    .as_mut()
                    .expect("paragraph exists")
                    .push(Event::End(end));

                let (replacements, events) = replace_paragraph_text(
                    paragraph_events.take().expect("paragraph exists"),
                    search,
                    replacement,
                    case_matching,
                )?;
                count += replacements;

                for event in events {
                    writer.write_event(event)?;
                }
            }
            event => {
                if let Some(events) = paragraph_events.as_mut() {
                    events.push(event);
                } else {
                    writer.write_event(event)?;
                }
            }
        }

        buffer.clear();
    }

    if paragraph_events.is_some() {
        return Err(invalid_document("unterminated Word paragraph"));
    }

    Ok((count, writer.into_inner()))
}

fn replace_paragraph_text(
    mut events: Vec<Event<'static>>,
    search: &str,
    replacement: &str,
    case_matching: CaseMatching,
) -> Result<(usize, Vec<Event<'static>>)> {
    let mut in_text = false;
    let mut event_indices = Vec::new();
    let mut texts = Vec::new();

    for (index, event) in events.iter().enumerate() {
        match event {
            Event::Start(start) if start.name().as_ref() == WORD_TEXT => in_text = true,
            Event::End(end) if end.name().as_ref() == WORD_TEXT => in_text = false,
            Event::Text(text) if in_text => {
                event_indices.push(index);
                texts.push(decode_text(text)?);
            }
            _ => {}
        }
    }

    let text = texts.concat();
    let ranges = match_ranges(&text, search, case_matching);
    apply_replacements(&mut texts, &ranges, replacement);

    for (event_index, text) in event_indices.into_iter().zip(texts) {
        events[event_index] = Event::Text(BytesText::new(&text).into_owned());
    }

    Ok((ranges.len(), events))
}

fn decode_text(text: &BytesText<'_>) -> Result<String> {
    let text = text
        .xml10_content()
        .map_err(|error| invalid_document(format!("invalid Word text: {error}")))?;
    quick_xml::escape::unescape(&text)
        .map(|text| text.into_owned())
        .map_err(|error| invalid_document(format!("invalid Word text: {error}")))
}

fn match_ranges(text: &str, search: &str, case_matching: CaseMatching) -> Vec<(usize, usize)> {
    match case_matching {
        CaseMatching::Sensitive => text
            .match_indices(search)
            .map(|(start, matched)| (start, start + matched.len()))
            .collect(),
        CaseMatching::UnicodeInsensitive => case_insensitive_ranges(text, search),
    }
}

fn case_insensitive_ranges(text: &str, search: &str) -> Vec<(usize, usize)> {
    let folded_search = search.to_lowercase();
    let mut folded_text = String::new();
    let mut boundaries = vec![(0, 0)];

    for (start, character) in text.char_indices() {
        folded_text.extend(character.to_lowercase());
        boundaries.push((folded_text.len(), start + character.len_utf8()));
    }

    folded_text
        .match_indices(&folded_search)
        .filter_map(|(start, matched)| {
            let end = start + matched.len();
            let start_index = boundaries
                .binary_search_by_key(&start, |entry| entry.0)
                .ok()?;
            let end_index = boundaries
                .binary_search_by_key(&end, |entry| entry.0)
                .ok()?;
            Some((boundaries[start_index].1, boundaries[end_index].1))
        })
        .collect()
}

fn apply_replacements(texts: &mut [String], ranges: &[(usize, usize)], replacement: &str) {
    let spans = text_spans(texts);

    for &(start, end) in ranges.iter().rev() {
        let first = spans
            .iter()
            .position(|span| span.0 <= start && start < span.1)
            .expect("match starts in a text node");
        let last = spans
            .iter()
            .position(|span| span.0 < end && end <= span.1)
            .expect("match ends in a text node");
        let start_offset = start - spans[first].0;
        let end_offset = end - spans[last].0;

        if first == last {
            texts[first].replace_range(start_offset..end_offset, replacement);
        } else {
            texts[first].replace_range(start_offset.., replacement);
            for text in &mut texts[first + 1..last] {
                text.clear();
            }
            texts[last].replace_range(..end_offset, "");
        }
    }
}

fn text_spans(texts: &[String]) -> Vec<(usize, usize)> {
    let mut offset = 0;

    texts
        .iter()
        .map(|text| {
            let start = offset;
            offset += text.len();
            (start, offset)
        })
        .collect()
}

fn invalid_document(message: impl Into<String>) -> crate::OfficeError {
    crate::OfficeError::InvalidDocument(message.into())
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io::{Cursor, Write};

    use zip::{ZipWriter, write::SimpleFileOptions};

    use super::*;

    fn docx_document(parts: &[(&str, &[u8])]) -> DocxDocument {
        let mut buffer = Cursor::new(Vec::new());
        let mut archive = ZipWriter::new(&mut buffer);
        let options = SimpleFileOptions::default();

        for (name, content) in parts {
            archive.start_file(name, options).unwrap();
            archive.write_all(content).unwrap();
        }

        archive.finish().unwrap();

        DocxDocument {
            package: OoxmlPackage::from_bytes(buffer.get_ref()).unwrap(),
        }
    }

    pub(crate) fn minimal_docx() -> DocxDocument {
        docx_document(&[
            (
                CONTENT_TYPES_XML,
                br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>"#,
            ),
            (
                ROOT_RELS,
                br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#,
            ),
            (
                DOCUMENT_XML,
                br#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello world</w:t></w:r></w:p></w:body></w:document>"#,
            ),
        ])
    }

    #[test]
    fn validates_complete_minimal_docx() {
        let document = minimal_docx();

        let issues = document.validate().unwrap();

        assert!(issues.is_empty());
    }

    #[test]
    fn validates_missing_required_parts() {
        let document = docx_document(&[(DOCUMENT_XML, br#"<w:office>Hello world</w:office>"#)]);

        let issues = document.validate().unwrap();

        assert_eq!(2, issues.len());
    }

    #[test]
    fn reads_document_xml() {
        let document = minimal_docx();

        let xml = document.document_xml().unwrap();

        assert_eq!(
            br#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hello world</w:t></w:r></w:p></w:body></w:document>"#,
            xml,
        );
    }

    #[test]
    fn reads_optional_styles_xml_when_present() {
        let document = docx_document(&[
            ("[Content_Types].xml", br#"<Types />"#),
            ("_rels/.rels", br#"<Relationships />"#),
            ("word/office.xml", br#"<w:office>Hello world</w:office>"#),
            ("word/styles.xml", br#"<w:styles />"#),
        ]);

        let styles = document.styles_xml().unwrap();

        assert_eq!(Some(br#"<w:styles />"#.as_slice()), styles);
    }

    #[test]
    fn returns_none_when_styles_xml_is_missing() {
        let document = minimal_docx();

        let styles = document.styles_xml().unwrap();

        assert!(styles.is_none());
    }

    #[test]
    fn replaces_text_in_document_xml_case_sensitive() {
        let mut document = minimal_docx();

        let options = ReplaceOptions::default();
        let count = document.replace_text("Hello", "Goodbye", options).unwrap();

        assert_eq!(1, count);

        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();

        assert!(xml.contains("Goodbye world"));
        assert!(!xml.contains("Hello world"));
    }

    #[test]
    fn replaces_text_in_document_xml_case_insensitive() {
        let mut document = minimal_docx();

        let options = ReplaceOptions {
            case_matching: CaseMatching::UnicodeInsensitive,
        };
        let count = document.replace_text("hello", "Goodbye", options).unwrap();

        assert_eq!(1, count);

        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();

        assert_eq!(
            xml,
            r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Goodbye world</w:t></w:r></w:p></w:body></w:document>"#
        );
    }

    #[test]
    fn replaces_text_across_word_runs_without_touching_markup() {
        let mut document = docx_document(&[
            (CONTENT_TYPES_XML, br#"<Types />"#),
            (ROOT_RELS, br#"<Relationships />"#),
            (
                DOCUMENT_XML,
                br#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p w:rsidR="Hallo"><w:r><w:t>H</w:t></w:r><w:r><w:t>allo</w:t></w:r></w:p></w:body></w:document>"#,
            ),
        ]);

        let count = document
            .replace_text("Hallo", "Goodbye & <all>", ReplaceOptions::default())
            .unwrap();

        assert_eq!(1, count);
        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();
        assert!(xml.contains(r#"w:rsidR="Hallo""#));
        assert!(xml.contains("Goodbye &amp; &lt;all&gt;"));
        assert!(!xml.contains(">H<"));
        assert!(!xml.contains(">allo<"));
    }

    #[test]
    fn does_not_update_document_xml_when_text_is_missing() {
        let mut document = minimal_docx();

        let options = ReplaceOptions::default();
        let count = document
            .replace_text("Missing", "Replacement", options)
            .unwrap();

        assert_eq!(0, count);

        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();

        assert!(xml.contains("Hello world"));
    }

    #[test]
    fn writes_and_registers_custom_metadata() {
        let mut document = minimal_docx();
        let mut metadata = OfficeMetadata::default();
        metadata.custom.insert("Client".into(), "Acme & Co".into());

        document.set_metadata(&metadata).unwrap();

        let custom = std::str::from_utf8(
            document
                .read_part(&PartName::from("docProps/custom.xml"))
                .unwrap(),
        )
        .unwrap();
        let content_types = std::str::from_utf8(
            document
                .read_part(&PartName::from("[Content_Types].xml"))
                .unwrap(),
        )
        .unwrap();
        let relationships =
            std::str::from_utf8(document.read_part(&PartName::from("_rels/.rels")).unwrap())
                .unwrap();

        assert!(custom.contains("Acme &amp; Co"));
        assert!(content_types.contains("/docProps/custom.xml"));
        assert!(relationships.contains("custom-properties"));
    }
}
