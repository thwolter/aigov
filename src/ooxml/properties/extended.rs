use crate::{
    error::{OfficeError, Result},
    office::{HeadingPair, OfficeMetadata},
    ooxml::package::OoxmlPackage,
};

use super::parser::{MetadataProperty, write_text_element};
use quick_xml::{
    Reader, Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};

pub const DOC_PROPS_APP: &str = "docProps/app.xml";

#[derive(Clone, Copy)]
pub enum ExtendedProperty {
    Application,
    AppVersion,
    Template,
    Company,
    TotalTime,
    Pages,
    Words,
    Characters,
    CharactersWithSpaces,
    Lines,
    Paragraphs,
    DocSecurity,
    ScaleCrop,
    LinksUpToDate,
    SharedDoc,
    HyperlinksChanged,
    DigSig,
}

impl MetadataProperty for ExtendedProperty {
    fn from_local_name(name: &[u8]) -> Option<Self> {
        match name {
            b"Application" => Some(Self::Application),
            b"AppVersion" => Some(Self::AppVersion),
            b"Template" => Some(Self::Template),
            b"Company" => Some(Self::Company),
            b"TotalTime" => Some(Self::TotalTime),
            b"Pages" => Some(Self::Pages),
            b"Words" => Some(Self::Words),
            b"Characters" => Some(Self::Characters),
            b"CharactersWithSpaces" => Some(Self::CharactersWithSpaces),
            b"Lines" => Some(Self::Lines),
            b"Paragraphs" => Some(Self::Paragraphs),
            b"DocSecurity" => Some(Self::DocSecurity),
            b"ScaleCrop" => Some(Self::ScaleCrop),
            b"LinksUpToDate" => Some(Self::LinksUpToDate),
            b"SharedDoc" => Some(Self::SharedDoc),
            b"HyperlinksChanged" => Some(Self::HyperlinksChanged),
            b"DigSig" => Some(Self::DigSig),
            _ => None,
        }
    }

    fn local_name(self) -> &'static [u8] {
        match self {
            Self::Application => b"Application",
            Self::AppVersion => b"AppVersion",
            Self::Template => b"Template",
            Self::Company => b"Company",
            Self::TotalTime => b"TotalTime",
            Self::Pages => b"Pages",
            Self::Words => b"Words",
            Self::Characters => b"Characters",
            Self::CharactersWithSpaces => b"CharactersWithSpaces",
            Self::Lines => b"Lines",
            Self::Paragraphs => b"Paragraphs",
            Self::DocSecurity => b"DocSecurity",
            Self::ScaleCrop => b"ScaleCrop",
            Self::LinksUpToDate => b"LinksUpToDate",
            Self::SharedDoc => b"SharedDoc",
            Self::HyperlinksChanged => b"HyperlinksChanged",
            Self::DigSig => b"DigSig",
        }
    }

    fn set(self, metadata: &mut OfficeMetadata, value: &str) {
        match self {
            Self::Application => metadata.application = Some(value.to_owned()),
            Self::AppVersion => metadata.app_version = Some(value.to_owned()),
            Self::Template => metadata.template = Some(value.to_owned()),
            Self::Company => metadata.company = Some(value.to_owned()),
            Self::TotalTime => metadata.total_time = value.parse().ok(),
            Self::Pages => metadata.pages = value.parse().ok(),
            Self::Words => metadata.words = value.parse().ok(),
            Self::Characters => metadata.characters = value.parse().ok(),
            Self::CharactersWithSpaces => metadata.characters_with_spaces = value.parse().ok(),
            Self::Lines => metadata.lines = value.parse().ok(),
            Self::Paragraphs => metadata.paragraphs = value.parse().ok(),
            Self::DocSecurity => metadata.doc_security = value.parse().ok(),
            Self::ScaleCrop => metadata.scale_crop = parse_bool(value),
            Self::LinksUpToDate => metadata.links_up_to_date = parse_bool(value),
            Self::SharedDoc => metadata.shared_doc = parse_bool(value),
            Self::HyperlinksChanged => metadata.hyperlinks_changed = parse_bool(value),
            Self::DigSig => metadata.dig_sig = Some(value.to_owned()),
        }
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

/// Reads extended properties from the document package.
pub(super) fn read_from(package: &OoxmlPackage, metadata: &mut OfficeMetadata) -> Result<()> {
    if let Ok(app_xml) = package.read_part(&DOC_PROPS_APP.into()) {
        ExtendedProperty::parse(app_xml, metadata)?;
        read_vectors(app_xml, metadata)?;
    }
    Ok(())
}

fn read_vectors(xml: &[u8], metadata: &mut OfficeMetadata) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut property = None;
    let mut values = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => match element.local_name().as_ref() {
                b"HeadingPairs" => {
                    property = Some("HeadingPairs");
                    values.clear();
                }
                b"TitlesOfParts" => {
                    property = Some("TitlesOfParts");
                    values.clear();
                }
                _ => {}
            },
            Event::Text(text) if property.is_some() => {
                let value = text.xml10_content().map_err(invalid_vector)?;
                let value = quick_xml::escape::unescape(&value).map_err(invalid_vector)?;
                if !value.is_empty() {
                    values.push(value.into_owned());
                }
            }
            Event::End(element)
            if property == Some("HeadingPairs")
                && element.local_name().as_ref() == b"HeadingPairs" =>
                {
                    if values.len() % 2 != 0 {
                        return Err(OfficeError::InvalidDocument(
                            "HeadingPairs must contain name/count pairs".into(),
                        ));
                    }
                    metadata.heading_pairs = values
                        .chunks_exact(2)
                        .map(|pair| {
                            Ok(HeadingPair {
                                name: pair[0].clone(),
                                count: pair[1].parse().map_err(|_| {
                                    OfficeError::InvalidDocument(
                                        "HeadingPairs count must be an unsigned integer".into(),
                                    )
                                })?,
                            })
                        })
                        .collect::<Result<_>>()?;
                    property = None;
                }
            Event::End(element)
            if property == Some("TitlesOfParts")
                && element.local_name().as_ref() == b"TitlesOfParts" =>
                {
                    metadata.titles_of_parts = std::mem::take(&mut values);
                    property = None;
                }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    Ok(())
}

fn invalid_vector(error: impl std::fmt::Display) -> OfficeError {
    OfficeError::InvalidDocument(format!("Invalid extended metadata: {error}"))
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

    write_optional(&mut writer, "Application", metadata.application.as_deref())?;
    write_optional(&mut writer, "AppVersion", metadata.app_version.as_deref())?;
    write_optional(&mut writer, "Template", metadata.template.as_deref())?;
    write_optional(&mut writer, "Company", metadata.company.as_deref())?;
    write_optional_number(&mut writer, "TotalTime", metadata.total_time)?;
    write_optional_number(&mut writer, "Pages", metadata.pages)?;
    write_optional_number(&mut writer, "Words", metadata.words)?;
    write_optional_number(&mut writer, "Characters", metadata.characters)?;
    write_optional_number(
        &mut writer,
        "CharactersWithSpaces",
        metadata.characters_with_spaces,
    )?;
    write_optional_number(&mut writer, "Lines", metadata.lines)?;
    write_optional_number(&mut writer, "Paragraphs", metadata.paragraphs)?;
    write_optional_number(&mut writer, "DocSecurity", metadata.doc_security)?;
    write_optional_bool(&mut writer, "ScaleCrop", metadata.scale_crop)?;
    write_optional_bool(&mut writer, "LinksUpToDate", metadata.links_up_to_date)?;
    write_optional_bool(&mut writer, "SharedDoc", metadata.shared_doc)?;
    write_optional_bool(
        &mut writer,
        "HyperlinksChanged",
        metadata.hyperlinks_changed,
    )?;
    write_heading_pairs(&mut writer, &metadata.heading_pairs)?;
    write_titles_of_parts(&mut writer, &metadata.titles_of_parts)?;
    write_optional(&mut writer, "DigSig", metadata.dig_sig.as_deref())?;

    writer.write_event(Event::End(BytesEnd::new("Properties")))?;
    Ok(writer.into_inner())
}

fn write_optional(writer: &mut Writer<Vec<u8>>, name: &str, value: Option<&str>) -> Result<()> {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        write_text_element(writer, name, value)?;
    }
    Ok(())
}

fn write_optional_number(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: Option<u32>,
) -> Result<()> {
    if let Some(value) = value {
        write_text_element(writer, name, &value.to_string())?;
    }
    Ok(())
}

fn write_optional_bool(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: Option<bool>,
) -> Result<()> {
    if let Some(value) = value {
        write_text_element(writer, name, if value { "true" } else { "false" })?;
    }
    Ok(())
}

fn write_heading_pairs(writer: &mut Writer<Vec<u8>>, pairs: &[HeadingPair]) -> Result<()> {
    if pairs.is_empty() {
        return Ok(());
    }
    writer.write_event(Event::Start(BytesStart::new("HeadingPairs")))?;
    let mut vector = BytesStart::new("vt:vector");
    vector.push_attribute(("size", (pairs.len() * 2).to_string().as_str()));
    vector.push_attribute(("baseType", "variant"));
    writer.write_event(Event::Start(vector))?;
    for pair in pairs {
        writer.write_event(Event::Start(BytesStart::new("vt:variant")))?;
        writer.write_event(Event::Start(BytesStart::new("vt:lpstr")))?;
        writer.write_event(Event::Text(BytesText::new(&pair.name)))?;
        writer.write_event(Event::End(BytesEnd::new("vt:lpstr")))?;
        writer.write_event(Event::End(BytesEnd::new("vt:variant")))?;
        writer.write_event(Event::Start(BytesStart::new("vt:variant")))?;
        writer.write_event(Event::Start(BytesStart::new("vt:i4")))?;
        writer.write_event(Event::Text(BytesText::new(&pair.count.to_string())))?;
        writer.write_event(Event::End(BytesEnd::new("vt:i4")))?;
        writer.write_event(Event::End(BytesEnd::new("vt:variant")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("vt:vector")))?;
    writer.write_event(Event::End(BytesEnd::new("HeadingPairs")))?;
    Ok(())
}

fn write_titles_of_parts(writer: &mut Writer<Vec<u8>>, titles: &[String]) -> Result<()> {
    if titles.is_empty() {
        return Ok(());
    }
    writer.write_event(Event::Start(BytesStart::new("TitlesOfParts")))?;
    let mut vector = BytesStart::new("vt:vector");
    vector.push_attribute(("size", titles.len().to_string().as_str()));
    vector.push_attribute(("baseType", "lpstr"));
    writer.write_event(Event::Start(vector))?;
    for title in titles {
        writer.write_event(Event::Start(BytesStart::new("vt:lpstr")))?;
        writer.write_event(Event::Text(BytesText::new(title)))?;
        writer.write_event(Event::End(BytesEnd::new("vt:lpstr")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("vt:vector")))?;
    writer.write_event(Event::End(BytesEnd::new("TitlesOfParts")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_extended_properties() {
        let metadata = OfficeMetadata {
            application: Some("Microsoft Word".into()),
            app_version: Some("16.0".into()),
            template: Some("Normal.dotm".into()),
            company: Some("Acme & Co".into()),
            total_time: Some(42),
            pages: Some(3),
            words: Some(1200),
            characters: Some(6000),
            characters_with_spaces: Some(7200),
            lines: Some(80),
            paragraphs: Some(12),
            doc_security: Some(0),
            scale_crop: Some(true),
            links_up_to_date: Some(false),
            shared_doc: Some(true),
            hyperlinks_changed: Some(false),
            heading_pairs: vec![HeadingPair {
                name: "Heading 1".into(),
                count: 3,
            }],
            titles_of_parts: vec!["Introduction".into()],
            dig_sig: Some("signed".into()),
            ..Default::default()
        };

        let xml = create_extended_properties(&metadata).unwrap();
        let mut parsed = OfficeMetadata::default();
        ExtendedProperty::parse(&xml, &mut parsed).unwrap();
        read_vectors(&xml, &mut parsed).unwrap();

        assert_eq!(parsed, metadata);
    }
}
