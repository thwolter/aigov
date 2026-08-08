use crate::{
    error::{OfficeError, Result},
    office::{OfficeMetadata, PartName},
    ooxml::package::OoxmlPackage,
};
use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};

pub const DOC_PROPS_CUSTOM: &str = "docProps/custom.xml";
const CUSTOM_PROPERTIES_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties";
const VTYPES_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes";
const CUSTOM_PROPERTY_FMTID: &str = "{D5CDD505-2E9C-101B-9397-08002B2CF9AE}";
const CONTENT_TYPES: &str = "[Content_Types].xml";
const ROOT_RELATIONSHIPS: &str = "_rels/.rels";
const CUSTOM_PROPERTIES_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.custom-properties+xml";
const CUSTOM_PROPERTIES_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";

pub(super) fn read_from(package: &OoxmlPackage, metadata: &mut OfficeMetadata) -> Result<()> {
    let Ok(xml) = package.read_part(&DOC_PROPS_CUSTOM.into()) else {
        return Ok(());
    };

    parse(xml, metadata)
}

/// Writes custom document properties to the package.
pub(super) fn write_to(package: &mut OoxmlPackage, metadata: &OfficeMetadata) -> Result<()> {
    package.write_part(DOC_PROPS_CUSTOM.into(), create_custom_properties(metadata)?);
    register_part(package)?;
    Ok(())
}

fn register_part(package: &mut OoxmlPackage) -> Result<()> {
    let content_types = replace_before_closing_tag(
        package.read_part(&PartName::from(CONTENT_TYPES))?,
        "</Types>",
        &format!(
            "<Override PartName=\"/docProps/custom.xml\" ContentType=\"{CUSTOM_PROPERTIES_CONTENT_TYPE}\"/>"
        ),
        "PartName=\"/docProps/custom.xml\"",
    )?;
    package.write_part(CONTENT_TYPES.into(), content_types);

    let relationships = replace_before_closing_tag(
        package.read_part(&PartName::from(ROOT_RELATIONSHIPS))?,
        "</Relationships>",
        &format!(
            "<Relationship Id=\"rIdCustomProperties\" Type=\"{CUSTOM_PROPERTIES_RELATIONSHIP}\" Target=\"docProps/custom.xml\"/>"
        ),
        CUSTOM_PROPERTIES_RELATIONSHIP,
    )?;
    package.write_part(ROOT_RELATIONSHIPS.into(), relationships);

    Ok(())
}

fn replace_before_closing_tag(
    xml: &[u8],
    closing_tag: &str,
    addition: &str,
    already_present: &str,
) -> Result<Vec<u8>> {
    let xml = std::str::from_utf8(xml)
        .map_err(|error| OfficeError::InvalidDocument(format!("Invalid package XML: {error}")))?;

    if xml.contains(already_present) {
        return Ok(xml.as_bytes().to_vec());
    }

    let updated = xml.replacen(closing_tag, &format!("{addition}{closing_tag}"), 1);
    if updated != xml {
        return Ok(updated.into_bytes());
    }

    let root_name = closing_tag.trim_start_matches("</").trim_end_matches('>');
    let Some(index) = xml.rfind("/>") else {
        return Err(OfficeError::InvalidDocument(format!(
            "Invalid package XML: missing {closing_tag}"
        )));
    };
    let root = &xml[..index];
    if !root.trim_start().starts_with(&format!("<{root_name}"))
        && !root.contains(&format!("<{root_name}"))
    {
        return Err(OfficeError::InvalidDocument(format!(
            "Invalid package XML: missing {closing_tag}"
        )));
    }

    Ok(format!("{root}>{addition}</{root_name}>").into_bytes())
}

fn create_custom_properties(metadata: &OfficeMetadata) -> Result<Vec<u8>> {
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

    #[test]
    fn writes_custom_properties() {
        let mut metadata = OfficeMetadata::default();
        metadata.custom.insert("Client".into(), "Acme & Co".into());
        metadata.custom.insert("Approved".into(), "true".into());

        let xml = create_custom_properties(&metadata).unwrap();
        let mut parsed = OfficeMetadata::default();
        parse(&xml, &mut parsed).unwrap();

        assert_eq!(parsed.custom, metadata.custom);
        let xml = String::from_utf8(xml).unwrap();
        assert!(xml.contains("pid=\"2\""));
        assert!(xml.contains("pid=\"3\""));
        assert!(xml.contains("Acme &amp; Co"));
    }
}
