mod extended;
mod core;
mod parser;

use super::package::OoxmlPackage;
use crate::{error::Result, office::OfficeMetadata};


pub fn read_metadata(package: &OoxmlPackage) -> Result<OfficeMetadata> {
    let mut metadata = OfficeMetadata::default();

    core::read_from(package, &mut metadata)?;
    extended::read_from(package, &mut metadata)?;

    Ok(metadata)
}

pub fn write_metadata(package: &mut OoxmlPackage, metadata: &OfficeMetadata) -> Result<()> {
    core::write_to(package, metadata)?;
    extended::write_to(package, metadata)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        core::CoreProperty,
        extended::ExtendedProperty,
        parser::MetadataProperty,
    };
    use crate::office::OfficeMetadata;

    #[test]
    fn parses_core_properties() {
        let xml = br#"
            <cp:coreProperties
                xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                xmlns:dc="http://purl.org/dc/elements/1.1/">
                <dc:title>A &amp; B</dc:title>
                <dc:subject>Roadmap</dc:subject>
                <dc:creator>Jane Doe</dc:creator>
                <cp:lastModifiedBy>John Doe</cp:lastModifiedBy>
                <dc:description><![CDATA[Plan <draft>]]></dc:description>
                <cp:keywords>planning, product, 2026</cp:keywords>
            </cp:coreProperties>
        "#;
        let mut metadata = OfficeMetadata::default();

        CoreProperty::parse(xml, &mut metadata).unwrap();

        assert_eq!(metadata.title.as_deref(), Some("A & B"));
        assert_eq!(metadata.subject.as_deref(), Some("Roadmap"));
        assert_eq!(metadata.creator.as_deref(), Some("Jane Doe"));
        assert_eq!(metadata.last_modified_by.as_deref(), Some("John Doe"));
        assert_eq!(metadata.description.as_deref(), Some("Plan <draft>"));
        assert_eq!(metadata.keywords, ["planning", "product", "2026"]);
    }

    #[test]
    fn parses_extended_properties() {
        let xml = br#"
            <Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
                <Application>Microsoft Office Word</Application>
                <Company>Acme &amp; Co</Company>
            </Properties>
        "#;
        let mut metadata = OfficeMetadata::default();

        ExtendedProperty::parse(xml, &mut metadata).unwrap();

        assert_eq!(metadata.company.as_deref(), Some("Acme & Co"));
    }
}
