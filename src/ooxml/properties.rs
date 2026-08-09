mod core;
mod custom;
mod extended;

use crate::{
    error::Result,
    office::metadata::OfficeMetadata,
    package::OoxmlPackage,
};

pub fn read_metadata(package: &OoxmlPackage) -> Result<OfficeMetadata> {
    let mut metadata = OfficeMetadata::default();

    core::read_from(package, &mut metadata)?;
    extended::read_from(package, &mut metadata)?;
    custom::read_from(package, &mut metadata)?;

    Ok(metadata)
}

pub fn write_metadata(package: &mut OoxmlPackage, metadata: &OfficeMetadata) -> Result<()> {
    core::write_to(package, metadata)?;
    extended::write_to(package, metadata)?;
    custom::write_to(package, metadata)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ooxml::xml::parse_text_elements;

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
        let properties = parse_text_elements(xml).unwrap();

        assert_eq!(properties[0].name, b"title");
        assert_eq!(properties[0].value, "A & B");
        assert_eq!(properties[1].name, b"subject");
        assert_eq!(properties[1].value, "Roadmap");
        assert_eq!(properties[2].name, b"creator");
        assert_eq!(properties[2].value, "Jane Doe");
        assert_eq!(properties[3].name, b"lastModifiedBy");
        assert_eq!(properties[3].value, "John Doe");
        assert_eq!(properties[4].name, b"description");
        assert_eq!(properties[4].value, "Plan <draft>");
        assert_eq!(properties[5].name, b"keywords");
        assert_eq!(properties[5].value, "planning, product, 2026");
    }

    #[test]
    fn parses_extended_properties() {
        let xml = br#"
            <Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
                <Application>Microsoft Office Word</Application>
                <Company>Acme &amp; Co</Company>
            </Properties>
        "#;
        let properties = parse_text_elements(xml).unwrap();

        let company = properties
            .iter()
            .find(|property| property.name == b"Company")
            .expect("Company property");

        assert_eq!(company.value, "Acme & Co");
    }
}
