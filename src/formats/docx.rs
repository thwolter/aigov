use std::path::Path;

use crate::{
    document::ValidationIssue,
    error::Result,
    office::OfficeDocument,
    office::metadata::OfficeMetadata,
    office::word::WordDocument,
    ooxml,
    package::{OoxmlPackage, PartName},
};

const OFFICE_XML: &str = "word/office.xml";
const STYLES_XML: &str = "word/styles.xml";
const CONTENT_TYPES_XML: &str = "[Content_Types].xml";
const ROOT_RELS: &str = "_rels/.rels";

pub struct DocxDocument {
    package: OoxmlPackage,
}

impl DocxDocument {
    pub fn document_xml(&self) -> Result<&[u8]> {
        self.package.read_part(&OFFICE_XML.into())
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

impl OfficeDocument for DocxDocument {
    fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            package: OoxmlPackage::open(path)?,
        })
    }

    fn source_path(&self) -> Option<&Path> {
        self.package.source_path()
    }

    fn parts(&self) -> Vec<PartName> {
        self.package.parts()
    }

    fn read_part(&self, part: &PartName) -> Result<&[u8]> {
        self.package.read_part(part)
    }

    fn write_part(&mut self, part: PartName, content: Vec<u8>) -> Result<()> {
        self.package.write_part(part, content);
        Ok(())
    }

    fn remove_part(&mut self, part: &PartName) -> Result<()> {
        self.package.remove_part(part);
        Ok(())
    }

    fn contains_part(&self, part: &PartName) -> bool {
        self.package.contains_part(part)
    }

    fn metadata(&self) -> Result<OfficeMetadata> {
        ooxml::read_metadata(&self.package)
    }

    fn set_metadata(&mut self, metadata: &OfficeMetadata) -> Result<()> {
        ooxml::write_metadata(&mut self.package, metadata)
    }

    fn validate(&self) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        let required_parts = [CONTENT_TYPES_XML, ROOT_RELS, OFFICE_XML];

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

impl WordDocument for DocxDocument {
    fn replace_text(
        &mut self,
        search: &str,
        replacement: &str,
        ignore_case: bool,
    ) -> Result<usize> {
        let part = PartName::new(OFFICE_XML);
        let xml = String::from_utf8_lossy(self.package.read_part(&part)?);

        let (count, updated) = if ignore_case {
            replace_case_insensitive(&xml, search, replacement)
        } else {
            let count = xml.matches(search).count();
            (count, xml.replace(search, replacement))
        };

        if count > 0 {
            self.package.write_part(part, updated.into_bytes());
        }

        Ok(count)
    }
}

fn replace_case_insensitive(xml: &str, search: &str, replacement: &str) -> (usize, String) {
    if search.is_empty() {
        return (
            xml.matches(search).count(),
            xml.replace(search, replacement),
        );
    }

    let folded_search = search.to_lowercase();
    let mut folded_xml = String::new();
    let mut boundaries = vec![(0, 0)];

    for (start, character) in xml.char_indices() {
        folded_xml.extend(character.to_lowercase());
        boundaries.push((folded_xml.len(), start + character.len_utf8()));
    }

    let ranges: Vec<_> = folded_xml
        .match_indices(&folded_search)
        .filter_map(|(start, _)| {
            let end = start + folded_search.len();
            let original_start = boundaries
                .binary_search_by_key(&start, |entry| entry.0)
                .ok()?;
            let original_end = boundaries
                .binary_search_by_key(&end, |entry| entry.0)
                .ok()?;
            Some((boundaries[original_start].1, boundaries[original_end].1))
        })
        .collect();

    let mut updated = xml.to_owned();
    for (start, end) in ranges.iter().rev() {
        updated.replace_range(*start..*end, replacement);
    }

    (ranges.len(), updated)
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
                "[Content_Types].xml",
                br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>"#,
            ),
            (
                "_rels/.rels",
                br#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#,
            ),
            (
                "word/office.xml",
                br#"<?xml version="1.0" encoding="UTF-8"?><w:office>Hello world</w:office>"#,
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
        let document =
            docx_document(&[("word/office.xml", br#"<w:office>Hello world</w:office>"#)]);

        let issues = document.validate().unwrap();

        assert_eq!(2, issues.len());
    }

    #[test]
    fn reads_document_xml() {
        let document = minimal_docx();

        let xml = document.document_xml().unwrap();

        assert_eq!(
            br#"<?xml version="1.0" encoding="UTF-8"?><w:office>Hello world</w:office>"#,
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

        let count = document.replace_text("Hello", "Goodbye", false).unwrap();

        assert_eq!(1, count);

        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();

        assert!(xml.contains("Goodbye world"));
        assert!(!xml.contains("Hello world"));
    }

    #[test]
    fn replaces_text_in_document_xml_case_insensitive() {
        let mut document = minimal_docx();

        let count = document.replace_text("hello", "Goodbye", true).unwrap();

        assert_eq!(1, count);

        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();

        assert_eq!(
            xml,
            r#"<?xml version="1.0" encoding="UTF-8"?><w:office>Goodbye world</w:office>"#
        );
    }

    #[test]
    fn does_not_update_document_xml_when_text_is_missing() {
        let mut document = minimal_docx();

        let count = document
            .replace_text("Missing", "Replacement", false)
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
