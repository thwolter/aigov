use crate::error::{OoxmlError, Result};
use crate::opc::OoxmlPackage;
use crate::properties::metadata::Metadata;
use quick_xml::{
    Reader, Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};

pub const DOC_PROPS_CUSTOM: &str = "docProps/custom.xml";
const CUSTOM_PROPERTIES_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties";
const VTYPES_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes";
const CUSTOM_PROPERTY_FMTID: &str = "{D5CDD505-2E9C-101B-9397-08002B2CF9AE}";
const CUSTOM_PROPERTIES_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.custom-properties+xml";
const CUSTOM_PROPERTIES_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";

pub fn read_from(package: &OoxmlPackage, metadata: &mut Metadata) -> Result<()> {
    let Ok(xml) = package.read_part(&DOC_PROPS_CUSTOM.into()) else {
        return Ok(());
    };

    parse(xml, metadata)
}

/// Writes custom document properties to the package.
pub fn write_to(package: &mut OoxmlPackage, metadata: &Metadata) -> Result<()> {
    package.write_part(DOC_PROPS_CUSTOM.into(), create_custom_properties(metadata)?);
    package.ensure_content_type_override(
        &format!("/{DOC_PROPS_CUSTOM}"),
        CUSTOM_PROPERTIES_CONTENT_TYPE,
    )?;
    package.ensure_root_relationship(
        "rIdCustomProperties",
        CUSTOM_PROPERTIES_RELATIONSHIP,
        DOC_PROPS_CUSTOM,
    )?;

    Ok(())
}

fn create_custom_properties(metadata: &Metadata) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());
    writer.write_event(Event::Decl(BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        Some("yes"),
    )))?;

    let mut root = BytesStart::new("Properties");
    root.push_attribute(("xmlns", CUSTOM_PROPERTIES_NAMESPACE));
    root.push_attribute(("xmlns:vt", VTYPES_NAMESPACE));
    writer.write_event(Event::Start(root))?;

    for (index, (name, value)) in metadata.custom.iter().enumerate() {
        let mut property = BytesStart::new("property");
        property.push_attribute(("fmtid", CUSTOM_PROPERTY_FMTID));
        property.push_attribute(("pid", (index + 2).to_string().as_str()));
        property.push_attribute(("name", name.as_str()));
        writer.write_event(Event::Start(property))?;
        writer.write_event(Event::Start(BytesStart::new("vt:lpwstr")))?;
        writer.write_event(Event::Text(BytesText::new(value)))?;
        writer.write_event(Event::End(BytesEnd::new("vt:lpwstr")))?;
        writer.write_event(Event::End(BytesEnd::new("property")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("Properties")))?;
    Ok(writer.into_inner())
}

fn parse(xml: &[u8], metadata: &mut Metadata) -> Result<()> {
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

fn invalid_property(error: impl std::fmt::Display) -> OoxmlError {
    OoxmlError::InvalidDocument(format!("Invalid custom metadata: {error}"))
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
        let mut metadata = Metadata::default();

        parse(xml, &mut metadata).unwrap();

        assert_eq!(metadata.custom["Client"], "Acme & Co");
        assert_eq!(metadata.custom["Approved"], "true");
    }

    #[test]
    fn writes_custom_properties() {
        let mut metadata = Metadata::default();
        metadata.custom.insert("Client".into(), "Acme & Co".into());
        metadata.custom.insert("Approved".into(), "true".into());

        let xml = create_custom_properties(&metadata).unwrap();
        let mut parsed = Metadata::default();
        parse(&xml, &mut parsed).unwrap();

        assert_eq!(parsed.custom, metadata.custom);
        let xml = String::from_utf8(xml).unwrap();
        assert!(xml.contains("pid=\"2\""));
        assert!(xml.contains("pid=\"3\""));
        assert!(xml.contains("Acme &amp; Co"));
    }
}
