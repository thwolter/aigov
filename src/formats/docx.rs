use std::path::Path;

use crate::{
    error::Result,
    office::{OfficeDocument, OfficeFileType, OfficeMetadata, PartName, ValidationIssue},
    ooxml::{OoxmlPackage, properties},
};

use crate::office::word::WordDocument;

const OFFICE_XML: &str = "word/office.xml";
const STYLES_XML: &str = "word/styles.xml";
const CONTENT_TYPES_XML: &str = "[Content_Types].xml";
const ROOT_RELS: &str = "_rels/.rels";

pub struct DocxDocument {
    package: OoxmlPackage,
}

impl DocxDocument {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            package: OoxmlPackage::open(path)?,
        })
    }
}

impl OfficeDocument for DocxDocument {
    fn file_type(&self) -> OfficeFileType {
        OfficeFileType::Word
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
        properties::read_metadata(&self.package)
    }

    fn set_metadata(&mut self, metadata: OfficeMetadata) -> Result<()> {
        properties::write_metadata(&mut self.package, &metadata)
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
    fn document_xml(&self) -> Result<&[u8]> {
        self.package.read_part(&OFFICE_XML.into())
    }

    fn styles_xml(&self) -> Result<Option<&[u8]>> {
        let part = PartName::new(STYLES_XML);

        if self.package.contains_part(&part) {
            Ok(Some(self.package.read_part(&part)?))
        } else {
            Ok(None)
        }
    }

    fn replace_text(&mut self, search: &str, replacement: &str) -> Result<usize> {
        let part = PartName::new(OFFICE_XML);
        let xml = self.package.read_part(&part)?;
        let xml = String::from_utf8_lossy(xml);

        let count = xml.matches(search).count();

        if count > 0 {
            let updated = xml.replace(search, replacement);

            self.package.write_part(part, updated.into_bytes());
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
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

    fn minimal_docx() -> DocxDocument {
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
    fn reports_word_file_type() {
        let document = minimal_docx();

        assert_eq!(OfficeFileType::Word, document.file_type(),);
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
    fn replaces_text_in_document_xml() {
        let mut document = minimal_docx();

        let count = document.replace_text("Hello", "Goodbye").unwrap();

        assert_eq!(1, count);

        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();

        assert!(xml.contains("Goodbye world"));
        assert!(!xml.contains("Hello world"));
    }

    #[test]
    fn does_not_update_document_xml_when_text_is_missing() {
        let mut document = minimal_docx();

        let count = document.replace_text("Missing", "Replacement").unwrap();

        assert_eq!(0, count);

        let xml = std::str::from_utf8(document.document_xml().unwrap()).unwrap();

        assert!(xml.contains("Hello world"));
    }
}
