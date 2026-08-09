mod core;
mod custom;
mod extended;

use crate::{
    error::Result,
    office::metadata::OfficeMetadata,
    package::OoxmlPackage,
};

/// Reads core, extended, and custom metadata from an OOXML package.
///
/// Missing metadata parts are represented by the corresponding default values
/// in [`OfficeMetadata`]. The package is not modified.
///
/// # Examples
///
/// Read the title from an existing Office document:
///
/// ```no_run
/// use aigov::{ooxml::read_metadata, package::OoxmlPackage};
///
/// # fn main() -> aigov::Result<()> {
/// let package = OoxmlPackage::open("report.docx")?;
/// let metadata = read_metadata(&package)?;
///
/// if let Some(title) = metadata.core.title {
///     println!("{title}");
/// }
/// # Ok(())
/// # }
/// ```
pub fn read_metadata(package: &OoxmlPackage) -> Result<OfficeMetadata> {
    let mut metadata = OfficeMetadata::default();

    core::read_from(package, &mut metadata)?;
    extended::read_from(package, &mut metadata)?;
    custom::read_from(package, &mut metadata)?;

    Ok(metadata)
}

/// Writes core, extended, and custom metadata to an OOXML package.
///
/// The package is modified in memory. Call [`OoxmlPackage::save`] to persist
/// the changes to a file. Fields left at their default values are omitted from
/// the corresponding metadata parts when possible.
///
/// # Examples
///
/// Update a document's title and add a custom property, then save the result:
///
/// ```no_run
/// use aigov::{
///     metadata::OfficeMetadata,
///     ooxml::{read_metadata, write_metadata},
///     package::OoxmlPackage,
/// };
/// use std::path::Path;
///
/// # fn main() -> aigov::Result<()> {
/// let mut package = OoxmlPackage::open("report.docx")?;
/// let mut metadata: OfficeMetadata = read_metadata(&package)?;
/// metadata.core.title = Some("Quarterly report".into());
/// metadata.custom.insert("Department".into(), "Finance".into());
///
/// write_metadata(&mut package, &metadata)?;
/// package.save(Path::new("report-with-metadata.docx"))?;
/// # Ok(())
/// # }
/// ```
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
