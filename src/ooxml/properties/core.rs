use crate::ooxml::xml::{TextElement, parse_text_elements, write_text_element};
use crate::{error::Result, office::metadata::OfficeMetadata, ooxml::package::OoxmlPackage};
use quick_xml::events::BytesText;
use quick_xml::{
    Writer,
    events::{BytesDecl, BytesEnd, BytesStart, Event},
};

pub const DOC_PROPS_CORE: &str = "docProps/core.xml";

/// Reads core properties from the document package.
pub(super) fn read_from(package: &OoxmlPackage, metadata: &mut OfficeMetadata) -> Result<()> {
    let Ok(xml) = package.read_part(&DOC_PROPS_CORE.into()) else {
        return Ok(())
    };
    apply_properties(parse_text_elements(xml)?, metadata);
    Ok(())
}

fn apply_properties(properties: Vec<TextElement>, metadata: &mut OfficeMetadata) {
    for TextElement { name, value } in properties {
        match name.as_slice() {
            b"title" => metadata.title = Some(value),
            b"subject" => metadata.subject = Some(value),
            b"creator" => metadata.creator = Some(value),
            b"lastModifiedBy" => metadata.last_modified_by = Some(value),
            b"description" => metadata.description = Some(value),

            b"keywords" => {
                metadata.keywords = value
                    .split(',')
                    .map(str::trim)
                    .filter(|keyword| !keyword.is_empty())
                    .map(str::to_owned)
                    .collect();
            }

            b"category" => metadata.category = Some(value),
            b"contentStatus" => metadata.content_status = Some(value),
            b"contentType" => metadata.content_type = Some(value),
            b"language" => metadata.language = Some(value),
            b"created" => metadata.created = Some(value),
            b"modified" => metadata.modified = Some(value),
            b"lastPrinted" => metadata.last_printed = Some(value),
            b"revision" => metadata.revision = Some(value),
            b"identifier" => metadata.identifier = Some(value),
            b"version" => metadata.version = Some(value),

            // Unknown core properties are intentionally ignored.
            _ => {}
        }
    }
}

/// Writes core properties to the document package.
pub(super) fn write_to(package: &mut OoxmlPackage, metadata: &OfficeMetadata) -> Result<()> {
    package.write_part(DOC_PROPS_CORE.into(), create_core_properties(metadata)?);
    Ok(())
}

fn create_core_properties(metadata: &OfficeMetadata) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());

    writer.write_event(Event::Decl(BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        Some("yes"),
    )))?;

    let mut root = BytesStart::new("cp:coreProperties");
    root.push_attribute((
        "xmlns:cp",
        "http://schemas.openxmlformats.org/package/2006/metadata/core-properties",
    ));
    root.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
    root.push_attribute(("xmlns:dcterms", "http://purl.org/dc/terms/"));
    root.push_attribute(("xmlns:dcmitype", "http://purl.org/dc/dcmitype/"));
    root.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));

    writer.write_event(Event::Start(root))?;

    if let Some(title) = metadata.title.as_deref().filter(|value| !value.is_empty()) {
        write_text_element(&mut writer, "dc:title", title)?;
    }

    if let Some(subject) = metadata
        .subject
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        write_text_element(&mut writer, "dc:subject", subject)?;
    }

    if let Some(creator) = metadata
        .creator
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        write_text_element(&mut writer, "dc:creator", creator)?;
    }

    if let Some(last_modified_by) = metadata
        .last_modified_by
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        write_text_element(&mut writer, "cp:lastModifiedBy", last_modified_by)?;
    }

    if let Some(description) = metadata
        .description
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        write_text_element(&mut writer, "dc:description", description)?;
    }

    if !metadata.keywords.is_empty() {
        let keywords = metadata.keywords.join(", ");
        write_text_element(&mut writer, "cp:keywords", &keywords)?;
    }

    write_optional(&mut writer, "cp:category", metadata.category.as_deref())?;
    write_optional(
        &mut writer,
        "cp:contentStatus",
        metadata.content_status.as_deref(),
    )?;
    write_optional(
        &mut writer,
        "cp:contentType",
        metadata.content_type.as_deref(),
    )?;
    write_optional(&mut writer, "dc:language", metadata.language.as_deref())?;
    write_optional_timestamp(&mut writer, "dcterms:created", metadata.created.as_deref())?;
    write_optional_timestamp(
        &mut writer,
        "dcterms:modified",
        metadata.modified.as_deref(),
    )?;
    write_optional(
        &mut writer,
        "cp:lastPrinted",
        metadata.last_printed.as_deref(),
    )?;
    write_optional(&mut writer, "cp:revision", metadata.revision.as_deref())?;
    write_optional(&mut writer, "dc:identifier", metadata.identifier.as_deref())?;
    write_optional(&mut writer, "cp:version", metadata.version.as_deref())?;

    writer.write_event(Event::End(BytesEnd::new("cp:coreProperties")))?;

    Ok(writer.into_inner())
}

fn write_optional(writer: &mut Writer<Vec<u8>>, name: &str, value: Option<&str>) -> Result<()> {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        write_text_element(writer, name, value)?;
    }
    Ok(())
}

fn write_optional_timestamp(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: Option<&str>,
) -> Result<()> {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        write_timestamp_element(writer, name, value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_core_properties() {
        let metadata = OfficeMetadata {
            title: Some("A & B".into()),
            creator: Some("Jane Doe".into()),
            keywords: vec!["planning".into(), "product".into()],
            category: Some("Strategy".into()),
            content_status: Some("Draft".into()),
            content_type: Some("Report".into()),
            language: Some("en-US".into()),
            created: Some("2026-08-07T10:00:00Z".into()),
            modified: Some("2026-08-07T11:00:00Z".into()),
            last_printed: Some("2026-08-07T12:00:00Z".into()),
            revision: Some("3".into()),
            identifier: Some("report-42".into()),
            version: Some("1.2".into()),
            ..Default::default()
        };

        let xml = create_core_properties(&metadata).unwrap();
        let mut parsed = OfficeMetadata::default();
        apply_properties(parse_text_elements(&xml).unwrap(), &mut parsed);

        assert_eq!(parsed.title, metadata.title);
        assert_eq!(parsed.creator, metadata.creator);
        assert_eq!(parsed.keywords, metadata.keywords);
        assert_eq!(parsed.category, metadata.category);
        assert_eq!(parsed.content_status, metadata.content_status);
        assert_eq!(parsed.content_type, metadata.content_type);
        assert_eq!(parsed.language, metadata.language);
        assert_eq!(parsed.created, metadata.created);
        assert_eq!(parsed.modified, metadata.modified);
        assert_eq!(parsed.last_printed, metadata.last_printed);
        assert_eq!(parsed.revision, metadata.revision);
        assert_eq!(parsed.identifier, metadata.identifier);
        assert_eq!(parsed.version, metadata.version);
    }
}

/// Writes a timestamp element to the XML writer.
pub(in crate::ooxml::properties) fn write_timestamp_element(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: &str,
) -> Result<()> {
    let mut element = BytesStart::new(name);
    element.push_attribute(("xsi:type", "dcterms:W3CDTF"));
    writer.write_event(Event::Start(element))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;

    Ok(())
}