use crate::{
    office::OfficeMetadata,
    error::Result,
};

use quick_xml::{Reader, events::Event};

use super::package::OoxmlPackage;

const DOC_PROPS_CORE: &str = "docProps/core.xml";
const DOC_PROPS_APP: &str = "docProps/app.xml";

pub fn read_metadata(
    package: &OoxmlPackage,
) -> Result<OfficeMetadata> {
    let mut metadata = OfficeMetadata::default();

    if let Ok(core_xml) = package.read_part(&DOC_PROPS_CORE.into()) {
        parse_properties::<CoreProperty>(core_xml, &mut metadata)?
    }

    if let Ok(app_xml) = package.read_part(&DOC_PROPS_APP.into()) {
        parse_properties::<ExtendedProperty>(app_xml, &mut metadata)?
    }

    Ok(metadata)
}

pub fn write_metadata(package: &mut OoxmlPackage, metadata: &OfficeMetadata) -> Result<()> {
    let core_xml = create_core_properties(metadata)?;
    let app_xml = create_extended_properties(metadata)?;

    package.write_part(
        DOC_PROPS_CORE.into(),
        core_xml,
    );

    package.write_part(
        DOC_PROPS_APP.into(),
        app_xml,
    );

    Ok(())
}


fn parse_properties<Property: MetadataProperty>(
    xml: &[u8],
    metadata: &mut OfficeMetadata,
) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut property = None;
    let mut value = String::new();

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                if let Some(next_property) =
                    Property::from_local_name(element.local_name().as_ref())
                {
                    property = Some(next_property);
                    value.clear();
                }
            }
            Event::Text(text) if property.is_some() => {
                let text = text.xml10_content().map_err(invalid_property_text)?;
                let text =
                    quick_xml::escape::unescape(&text).map_err(invalid_property_text)?;
                value.push_str(&text);
            }
            Event::GeneralRef(reference) if property.is_some() => {
                let reference = reference
                    .xml10_content()
                    .map_err(invalid_property_text)?;
                let reference = format!("&{reference};");
                let reference =
                    quick_xml::escape::unescape(&reference).map_err(invalid_property_text)?;
                value.push_str(&reference);
            }
            Event::CData(text) if property.is_some() => {
                value.push_str(&text.xml10_content().map_err(invalid_property_text)?);
            }
            Event::End(element) => {
                if let Some(current_property) = property
                    && current_property.local_name() == element.local_name().as_ref()
                {
                    current_property.set(metadata, &value);
                    property = None;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    Ok(())
}

fn invalid_property_text(error: impl std::fmt::Display) -> crate::error::OfficeError {
    crate::error::OfficeError::InvalidDocument(format!("Invalid metadata property text: {error}"))
}

trait MetadataProperty: Copy {
    fn from_local_name(name: &[u8]) -> Option<Self>;
    fn local_name(self) -> &'static [u8];
    fn set(self, metadata: &mut OfficeMetadata, value: &str);
}

#[derive(Clone, Copy)]
enum CoreProperty {
    Title,
    Subject,
    Creator,
    LastModifiedBy,
    Description,
    Keywords,
}

impl MetadataProperty for CoreProperty {
    fn from_local_name(name: &[u8]) -> Option<Self> {
        match name {
            b"title" => Some(Self::Title),
            b"subject" => Some(Self::Subject),
            b"creator" => Some(Self::Creator),
            b"lastModifiedBy" => Some(Self::LastModifiedBy),
            b"description" => Some(Self::Description),
            b"keywords" => Some(Self::Keywords),
            _ => None,
        }
    }

    fn local_name(self) -> &'static [u8] {
        match self {
            Self::Title => b"title",
            Self::Subject => b"subject",
            Self::Creator => b"creator",
            Self::LastModifiedBy => b"lastModifiedBy",
            Self::Description => b"description",
            Self::Keywords => b"keywords",
        }
    }

    fn set(self, metadata: &mut OfficeMetadata, value: &str) {
        match self {
            Self::Title => metadata.title = Some(value.to_owned()),
            Self::Subject => metadata.subject = Some(value.to_owned()),
            Self::Creator => metadata.creator = Some(value.to_owned()),
            Self::LastModifiedBy => metadata.last_modified_by = Some(value.to_owned()),
            Self::Description => metadata.description = Some(value.to_owned()),
            Self::Keywords => {
                metadata.keywords = value
                    .split(',')
                    .map(str::trim)
                    .filter(|keyword| !keyword.is_empty())
                    .map(str::to_owned)
                    .collect();
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ExtendedProperty {
    Company,
}

impl MetadataProperty for ExtendedProperty {
    fn from_local_name(name: &[u8]) -> Option<Self> {
        match name {
            b"Company" => Some(Self::Company),
            _ => None,
        }
    }

    fn local_name(self) -> &'static [u8] {
        match self {
            Self::Company => b"Company",
        }
    }

    fn set(self, metadata: &mut OfficeMetadata, value: &str) {
        match self {
            Self::Company => metadata.company = Some(value.to_owned()),
        }
    }
}

fn create_core_properties(
    _metadata: &OfficeMetadata,
) -> Result<Vec<u8>> {
    todo!("Create core.xml")
}

fn create_extended_properties(
    _metadata: &OfficeMetadata,
) -> Result<Vec<u8>> {
    todo!("Create app.xml")
}

#[cfg(test)]
mod tests {
    use super::*;

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

        parse_properties::<CoreProperty>(xml, &mut metadata).unwrap();

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

        parse_properties::<ExtendedProperty>(xml, &mut metadata).unwrap();

        assert_eq!(metadata.company.as_deref(), Some("Acme & Co"));
    }
}
