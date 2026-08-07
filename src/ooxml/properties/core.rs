use crate::{
    error::Result,
    office::OfficeMetadata,
    ooxml::package::OoxmlPackage,
};

use super::parser::MetadataProperty;

pub const DOC_PROPS_CORE: &str = "docProps/core.xml";

#[derive(Clone, Copy)]
pub enum CoreProperty {
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

pub(super) fn read_from(package: &OoxmlPackage, metadata: &mut OfficeMetadata) -> Result<()> {
    if let Ok(core_xml) = package.read_part(&DOC_PROPS_CORE.into()) {
        CoreProperty::parse(core_xml, metadata)?;
    }
    Ok(())
}

pub(super) fn write_to(package: &mut OoxmlPackage, metadata: &OfficeMetadata) -> Result<()> {
    if let Ok(core_xml) = create_core_properties(metadata) {
        package.write_part(DOC_PROPS_CORE.into(), core_xml);
    }
    Ok(())
}

fn create_core_properties(_metadata: &OfficeMetadata) -> Result<Vec<u8>> {
    todo!("Create core.xml")
}