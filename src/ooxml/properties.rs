use crate::{
    office::OfficeMetadata,
    error::Result,
};

use super::package::OoxmlPackage;

pub fn read_metadata(
    package: &OoxmlPackage,
) -> Result<OfficeMetadata> {
    let mut metadata = OfficeMetadata::default();

    if let Ok(core_xml) =
        package.read_part(&"docProps/core.xml".into())
    {
        parse_core_properties(core_xml, &mut metadata)?;
    }

    if let Ok(app_xml) =
        package.read_part(&"docProps/app.xml".into())
    {
        parse_extended_properties(app_xml, &mut metadata)?;
    }

    Ok(metadata)
}

pub fn write_metadata(
    package: &mut OoxmlPackage,
    metadata: &OfficeMetadata,
) -> Result<()> {
    let core_xml = create_core_properties(metadata)?;
    let app_xml = create_extended_properties(metadata)?;

    package.write_part(
        "docProps/core.xml".into(),
        core_xml,
    );

    package.write_part(
        "docProps/app.xml".into(),
        app_xml,
    );

    Ok(())
}

fn parse_core_properties(
    _xml: &[u8],
    _metadata: &mut OfficeMetadata,
) -> Result<()> {
    todo!("Parse core metadata using quick-xml")
}

fn parse_extended_properties(
    _xml: &[u8],
    _metadata: &mut OfficeMetadata,
) -> Result<()> {
    todo!("Parse extended metadata using quick-xml")
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