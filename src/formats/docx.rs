mod policy;
mod replacement;

use crate::office::OoxmlDocument;
use crate::office::policy::{AiPolicy, AiPolicyDocument};
use crate::office::replacement::ReplaceOptions;
use crate::{
    document::ValidationIssue,
    error,
    error::Result,
    office::OfficeDocument,
    office::metadata::OfficeMetadata,
    ooxml,
    package::{OoxmlPackage, PartName},
};
use quick_xml::{Reader, Writer};
use std::path::Path;

const DOCUMENT_XML: &str = "word/document.xml";
const STYLES_XML: &str = "word/styles.xml";
const CONTENT_TYPES_XML: &str = "[Content_Types].xml";
const ROOT_RELS: &str = "_rels/.rels";

/// A mutable DOCX document backed by an OOXML package.
///
/// `DocxDocument` provides DOCX-specific access to `word/document.xml` and
/// `word/styles.xml`. For metadata, validation, and saving, use the
/// [`OfficeDocument`] trait. The [`OoxmlDocument`] trait provides construction
/// from a file, bytes, or an existing package.
///
/// # Examples
///
/// ```no_run
/// use aigov::{
///     document::Severity,
///     formats::DocxDocument,
///     office::{OfficeDocument, OoxmlDocument},
/// };
/// use std::path::Path;
///
/// let mut document = <DocxDocument as OoxmlDocument>::from_file(Path::new("report.docx"))?;
/// document.replace_text("Draft", "Final", Default::default())?;
///
/// if document
///     .validate()?
///     .iter()
///     .any(|issue| issue.severity == Severity::Error)
/// {
///     return Err("refusing to publish an invalid DOCX".into());
/// }
///
/// document.save(Path::new("published-report.docx"))?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct DocxDocument {
    package: OoxmlPackage,
}

impl DocxDocument {
    /// Returns the raw `word/document.xml` part.
    ///
    /// This is DOCX-specific. Prefer [`OfficeDocument::replace_text`] for
    /// ordinary visible-text edits.
    pub fn document_xml(&self) -> Result<&[u8]> {
        self.package.read_part(&DOCUMENT_XML.into())
    }

    /// Returns the raw `word/styles.xml` part when the document contains one.
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
        let (count, updated) = replacement::replace_document_text(
            self.package.read_part(&part)?,
            search,
            replacement,
            options,
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

fn invalid_document(message: impl Into<String>) -> error::OfficeError {
    error::OfficeError::InvalidDocument(message.into())
}

impl AiPolicyDocument for DocxDocument {
    fn policy(&self) -> error::Result<Option<AiPolicy>> {
        let mut reader = Reader::from_reader(self.package.read_part(&DOCUMENT_XML.into())?);
        policy::read_policy_from_document_xml(&mut reader)
    }

    fn inject_policy(&mut self, policy: &AiPolicy) -> error::Result<()> {
        let part = PartName::new(DOCUMENT_XML);
        let mut reader = Reader::from_reader(self.package.read_part(&part)?);
        let mut writer = Writer::new(Vec::new());

        let injected = policy::write_hidden_policy_paragraph(policy, &mut reader, &mut writer)?;

        if !injected {
            return Err(invalid_document("Word document has no body element"));
        }

        self.package.write_part(part, writer.into_inner());
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io::{Cursor, Write};

    use zip::{ZipWriter, write::SimpleFileOptions};

    use crate::office::replacement::CaseMatching;

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

    #[test]
    fn injects_and_reads_ai_policy() {
        let mut document = minimal_docx();
        let policy = AiPolicy {
            id: "internal-use".into(),
            human_review_required: true,
            training_allowed: false,
            attribution_required: true,
            owner: Some("legal".into()),
        };

        document.inject_policy(&policy).unwrap();

        assert_eq!(document.policy().unwrap(), Some(policy));
    }
}
