use crate::{error::Result, office::OfficeMetadata, ooxml::package::OoxmlPackage};

use super::parser::{MetadataProperty, write_text_element};
use quick_xml::{
    Writer,
    events::{BytesDecl, BytesEnd, BytesStart, Event},
};

pub const DOC_PROPS_APP: &str = "docProps/app.xml";

#[derive(Clone, Copy)]
pub enum ExtendedProperty {
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

/// Reads extended properties from the document package.
pub(super) fn read_from(package: &OoxmlPackage, metadata: &mut OfficeMetadata) -> Result<()> {
    if let Ok(app_xml) = package.read_part(&DOC_PROPS_APP.into()) {
        ExtendedProperty::parse(app_xml, metadata)?;
    }
    Ok(())
}

/// Writes extended properties to the document package.
pub(super) fn write_to(package: &mut OoxmlPackage, metadata: &OfficeMetadata) -> Result<()> {
    let app_xml = create_extended_properties(metadata)?;
    package.write_part(DOC_PROPS_APP.into(), app_xml);
    Ok(())
}

fn create_extended_properties(metadata: &OfficeMetadata) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());

    writer.write_event(Event::Decl(BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        Some("yes"),
    )))?;

    let mut root = BytesStart::new("Properties");
    root.push_attribute((
        "xmlns",
        "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties",
    ));
    root.push_attribute((
        "xmlns:vt",
        "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes",
    ));
    writer.write_event(Event::Start(root))?;

    if let Some(company) = metadata
        .company
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        write_text_element(&mut writer, "Company", company)?;
    }

    writer.write_event(Event::End(BytesEnd::new("Properties")))?;

    Ok(writer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_extended_properties() {
        let metadata = OfficeMetadata {
            company: Some("Acme & Co".into()),
            ..Default::default()
        };

        let xml = create_extended_properties(&metadata).unwrap();
        let mut parsed = OfficeMetadata::default();
        ExtendedProperty::parse(&xml, &mut parsed).unwrap();

        assert_eq!(parsed.company, metadata.company);
    }
}
