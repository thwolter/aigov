use super::parser::{MetadataProperty, write_text_element, write_timestamp_element};
use crate::{error::Result, office::OfficeMetadata, ooxml::package::OoxmlPackage};
use quick_xml::{
    Writer,
    events::{BytesDecl, BytesEnd, BytesStart, Event},
};

pub const DOC_PROPS_CORE: &str = "docProps/core.xml";

#[derive(Clone, Copy)]
pub enum CoreProperty {
    Title,
    Subject,
    Creator,
    LastModifiedBy,
    Description,
    Keywords,
    Category,
    ContentStatus,
    ContentType,
    Language,
    Created,
    Modified,
    LastPrinted,
    Revision,
    Identifier,
    Version,
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
            b"category" => Some(Self::Category),
            b"contentStatus" => Some(Self::ContentStatus),
            b"contentType" => Some(Self::ContentType),
            b"language" => Some(Self::Language),
            b"created" => Some(Self::Created),
            b"modified" => Some(Self::Modified),
            b"lastPrinted" => Some(Self::LastPrinted),
            b"revision" => Some(Self::Revision),
            b"identifier" => Some(Self::Identifier),
            b"version" => Some(Self::Version),
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
            Self::Category => b"category",
            Self::ContentStatus => b"contentStatus",
            Self::ContentType => b"contentType",
            Self::Language => b"language",
            Self::Created => b"created",
            Self::Modified => b"modified",
            Self::LastPrinted => b"lastPrinted",
            Self::Revision => b"revision",
            Self::Identifier => b"identifier",
            Self::Version => b"version",
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
            Self::Category => metadata.category = Some(value.to_owned()),
            Self::ContentStatus => metadata.content_status = Some(value.to_owned()),
            Self::ContentType => metadata.content_type = Some(value.to_owned()),
            Self::Language => metadata.language = Some(value.to_owned()),
            Self::Created => metadata.created = Some(value.to_owned()),
            Self::Modified => metadata.modified = Some(value.to_owned()),
            Self::LastPrinted => metadata.last_printed = Some(value.to_owned()),
            Self::Revision => metadata.revision = Some(value.to_owned()),
            Self::Identifier => metadata.identifier = Some(value.to_owned()),
            Self::Version => metadata.version = Some(value.to_owned()),
        }
    }
}

/// Reads core properties from the document package.
pub(super) fn read_from(package: &OoxmlPackage, metadata: &mut OfficeMetadata) -> Result<()> {
    if let Ok(core_xml) = package.read_part(&DOC_PROPS_CORE.into()) {
        CoreProperty::parse(core_xml, metadata)?;
    }
    Ok(())
}

/// Writes core properties to the document package.
pub(super) fn write_to(package: &mut OoxmlPackage, metadata: &OfficeMetadata) -> Result<()> {
    let core_xml = create_core_properties(metadata)?;
    package.write_part(DOC_PROPS_CORE.into(), core_xml);
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
        CoreProperty::parse(&xml, &mut parsed).unwrap();

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
