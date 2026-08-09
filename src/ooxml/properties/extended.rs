use crate::{
    error::{OfficeError, Result},
    office::metadata::{HeadingPair, OfficeMetadata},
    package::OoxmlPackage,
};


use crate::ooxml::xml::{TextElement, parse_text_elements, write_text_element};
use quick_xml::{
    Reader, Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};

pub const DOC_PROPS_APP: &str = "docProps/app.xml";

/// Reads extended properties from the document package.
pub(super) fn read_from(package: &OoxmlPackage, metadata: &mut OfficeMetadata) -> Result<()> {
    let Ok(xml) = package.read_part(&DOC_PROPS_APP.into()) else {
        return Ok(())
    };
    
    apply_properties(parse_text_elements(xml)?, metadata);
    read_vectors(xml, metadata)?;
    Ok(())
}

fn apply_properties(properties: Vec<TextElement>, metadata: &mut OfficeMetadata) {
    for TextElement { name, value } in properties {
        match name.as_slice() {
            b"Application" => metadata.extended.application = Some(value),
            b"AppVersion" => metadata.extended.app_version = Some(value),
            b"Template" => metadata.extended.template = Some(value),
            b"Company" => metadata.extended.company = Some(value),

            b"TotalTime" => metadata.extended.total_time = value.parse().ok(),
            b"Pages" => metadata.extended.pages = value.parse().ok(),
            b"Words" => metadata.extended.words = value.parse().ok(),
            b"Characters" => metadata.extended.characters = value.parse().ok(),
            b"CharactersWithSpaces" => {
                metadata.extended.characters_with_spaces = value.parse().ok()
            }
            b"Lines" => metadata.extended.lines = value.parse().ok(),
            b"Paragraphs" => metadata.extended.paragraphs = value.parse().ok(),
            b"DocSecurity" => metadata.extended.doc_security = value.parse().ok(),

            b"ScaleCrop" => metadata.extended.scale_crop = parse_bool(&value),
            b"LinksUpToDate" => {
                metadata.extended.links_up_to_date = parse_bool(&value)
            }
            b"SharedDoc" => metadata.extended.shared_doc = parse_bool(&value),
            b"HyperlinksChanged" => {
                metadata.extended.hyperlinks_changed = parse_bool(&value)
            }

            b"DigSig" => metadata.extended.dig_sig = Some(value),

            // HeadingPairs and TitlesOfParts are handled separately.
            _ => {}
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
                    metadata.extended.heading_pairs = values
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
                    metadata.extended.titles_of_parts = std::mem::take(&mut values);
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

    let extended = &metadata.extended;

    write_optional(&mut writer, "Application", extended.application.as_deref())?;
    write_optional(&mut writer, "AppVersion", extended.app_version.as_deref())?;
    write_optional(&mut writer, "Template", extended.template.as_deref())?;
    write_optional(&mut writer, "Company", extended.company.as_deref())?;
    write_optional_number(&mut writer, "TotalTime", extended.total_time)?;
    write_optional_number(&mut writer, "Pages", extended.pages)?;
    write_optional_number(&mut writer, "Words", extended.words)?;
    write_optional_number(&mut writer, "Characters", extended.characters)?;
    write_optional_number(
        &mut writer,
        "CharactersWithSpaces",
        extended.characters_with_spaces,
    )?;
    write_optional_number(&mut writer, "Lines", extended.lines)?;
    write_optional_number(&mut writer, "Paragraphs", extended.paragraphs)?;
    write_optional_number(&mut writer, "DocSecurity", extended.doc_security)?;
    write_optional_bool(&mut writer, "ScaleCrop", extended.scale_crop)?;
    write_optional_bool(&mut writer, "LinksUpToDate", extended.links_up_to_date)?;
    write_optional_bool(&mut writer, "SharedDoc", extended.shared_doc)?;
    write_optional_bool(
        &mut writer,
        "HyperlinksChanged",
        extended.hyperlinks_changed,
    )?;
    write_heading_pairs(&mut writer, &extended.heading_pairs)?;
    write_titles_of_parts(&mut writer, &extended.titles_of_parts)?;
    write_optional(&mut writer, "DigSig", extended.dig_sig.as_deref())?;

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
    use crate::office::metadata::ExtendedMetadata;

    use super::*;

    #[test]
    fn creates_extended_properties() {
        let metadata = OfficeMetadata {
            extended: ExtendedMetadata {
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
            },
            ..Default::default()
        };

        let xml = create_extended_properties(&metadata).unwrap();
        let mut parsed = OfficeMetadata::default();
        apply_properties(parse_text_elements(&xml).unwrap(), &mut parsed);
        read_vectors(&xml, &mut parsed).unwrap();

        assert_eq!(parsed, metadata);
    }
}
