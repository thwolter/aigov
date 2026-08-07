use crate::{
    error::{OfficeError, Result},
    office::OfficeMetadata,
    ooxml::package::OoxmlPackage,
};
use quick_xml::{Reader, events::Event};

pub const DOC_PROPS_CUSTOM: &str = "docProps/custom.xml";

pub(super) fn read_from(package: &OoxmlPackage, metadata: &mut OfficeMetadata) -> Result<()> {
    let Ok(xml) = package.read_part(&DOC_PROPS_CUSTOM.into()) else {
        return Ok(());
    };

    parse(xml, metadata)
}

fn parse(xml: &[u8], metadata: &mut OfficeMetadata) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut name = None;
    let mut value = String::new();
    let mut value_depth = 0;

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.local_name().as_ref() == b"property" => {
                name = element
                    .attributes()
                    .filter_map(|attribute| attribute.ok())
                    .find(|attribute| attribute.key.as_ref() == b"name")
                    .map(|attribute| {
                        attribute
                            .normalized_value(quick_xml::XmlVersion::Explicit1_0)
                            .map(|value| value.into_owned())
                            .map_err(invalid_property)
                    })
                    .transpose()?;
                value.clear();
                value_depth = 0;
            }
            Event::Start(_) if name.is_some() => value_depth += 1,
            Event::Text(text) if value_depth > 0 => {
                let text = text.xml10_content().map_err(invalid_property)?;
                value.push_str(&text);
            }
            Event::GeneralRef(reference) if value_depth > 0 => {
                let reference = reference.xml10_content().map_err(invalid_property)?;
                value.push_str(
                    &quick_xml::escape::unescape(&format!("&{reference};"))
                        .map_err(invalid_property)?,
                );
            }
            Event::CData(text) if value_depth > 0 => {
                value.push_str(&text.xml10_content().map_err(invalid_property)?);
            }
            Event::End(element) if element.local_name().as_ref() == b"property" => {
                if let Some(name) = name.take() {
                    metadata.custom.insert(name, std::mem::take(&mut value));
                }
            }
            Event::End(_) if value_depth > 0 => value_depth -= 1,
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    Ok(())
}

fn invalid_property(error: impl std::fmt::Display) -> OfficeError {
    OfficeError::InvalidDocument(format!("Invalid custom metadata: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_custom_properties() {
        let xml = br#"
            <Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties"
                xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
                <property name="Client" fmtid="{x}" pid="2"><vt:lpwstr>Acme &amp; Co</vt:lpwstr></property>
                <property name="Approved" fmtid="{x}" pid="3"><vt:bool>true</vt:bool></property>
            </Properties>
        "#;
        let mut metadata = OfficeMetadata::default();

        parse(xml, &mut metadata).unwrap();

        assert_eq!(metadata.custom["Client"], "Acme & Co");
        assert_eq!(metadata.custom["Approved"], "true");
    }
}
