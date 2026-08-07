use crate::{
    error::Result,
    office::OfficeMetadata,
    ooxml::package::OoxmlPackage,
};

use super::parser::MetadataProperty;

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

pub(super) fn read_from(package: &OoxmlPackage, metadata: &mut OfficeMetadata) -> Result<()> {
    if let Ok(app_xml) = package.read_part(&DOC_PROPS_APP.into()) {
        ExtendedProperty::parse(app_xml, metadata)?;
    }
    Ok(())
}

pub(super) fn write_to(package: &mut OoxmlPackage, metadata: &OfficeMetadata) -> Result<()> {
    if let Ok(app_xml) = create_extended_properties(metadata) {
        package.write_part(DOC_PROPS_APP.into(), app_xml);
    }
    Ok(())
}

fn create_extended_properties(_metadata: &OfficeMetadata) -> Result<Vec<u8>> {
    todo!("Create app.xml")
}