mod document;

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use zip::ZipArchive;
use zip::write::ZipWriter;

const CUSTOM_PROPERTIES: &str = "docProps/custom.xml";
const CONTENT_TYPES: &str = "[Content_Types].xml";
const PACKAGE_RELATIONSHIPS: &str = "_rels/.rels";
const CUSTOM_PROPERTIES_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.custom-properties+xml";
const CUSTOM_PROPERTIES_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List the files stored inside a Word office.
    Inspect { filepath: String },

    /// Unzip a office and store its contents.
    Unzip {
        filepath: String,

        #[arg(short, long)]
        output: Option<String>,
    },

    /// Store a source citation in the office's custom properties.
    AddSource { filepath: String, source: String },
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Inspect { filepath } => inspect_docx(&filepath),
        Commands::Unzip { filepath, output } => unzip_document(&filepath, output.as_deref()),
        Commands::AddSource { filepath, source } => add_source_metadata(&filepath, &source),
    }
}

fn inspect_docx(filepath: &str) -> io::Result<()> {
    let file = File::open(filepath)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        println!("File: {}", file.name());
    }
    Ok(())
}

fn unzip_document(filepath: &str, output: Option<&str>) -> io::Result<()> {
    // unzip a office and store its contents in a subfolder
    let file = File::open(filepath)?;
    let mut archive = ZipArchive::new(file)?;
    let document_path = Path::new(filepath);
    let default_output_directory = document_path.with_extension("");
    let output_directory = output
        .map(PathBuf::from)
        .unwrap_or(default_output_directory);

    fs::create_dir_all(&output_directory)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(enclosed_name) = entry.enclosed_name() else {
            continue;
        };
        let output_path = output_directory.join(enclosed_name);

        if entry.is_dir() {
            fs::create_dir_all(&output_path)?;
        } else {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut output_file = File::create(&output_path)?;
            io::copy(&mut entry, &mut output_file)?;
        }
    }

    Ok(())
}

/// Adds or updates the custom property named `Source` in a `.docx` file.

/// Adds or updates the custom property named `Source` in a `.docx` file.
///
/// A Word office is a ZIP archive. This function copies every file to a
/// temporary archive, replacing `docProps/custom.xml` and registering that
/// part with Word before swapping the temporary archive into place.
fn add_source_metadata(filepath: &str, source: &str) -> io::Result<()> {
    let input = File::open(filepath)?;
    let mut archive = ZipArchive::new(input)?;
    let temporary_path = temporary_path_for(Path::new(filepath));
    let output = File::create(&temporary_path)?;
    let mut writer = ZipWriter::new(output);

    let existing_custom_properties = archive
        .by_name(CUSTOM_PROPERTIES)
        .ok()
        .map(|mut file| {
            let mut xml = Vec::new();
            file.read_to_end(&mut xml)?;
            Ok::<Vec<u8>, io::Error>(xml)
        })
        .transpose()?;

    let custom_properties =
        update_custom_properties(existing_custom_properties.as_deref(), source)?;

    let mut has_custom_properties = false;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let options = entry.options();

        if name == CUSTOM_PROPERTIES {
            has_custom_properties = true;
            writer.start_file(name, options)?;
            writer.write_all(&custom_properties)?;
        } else if name == CONTENT_TYPES {
            let mut xml = String::new();
            entry.read_to_string(&mut xml)?;
            writer.start_file(name, options)?;
            writer.write_all(add_content_type_override(&xml).as_bytes())?;
        } else if name == PACKAGE_RELATIONSHIPS {
            let mut xml = String::new();
            entry.read_to_string(&mut xml)?;
            writer.start_file(name, options)?;
            writer.write_all(add_custom_property_relationship(&xml).as_bytes())?;
        } else if entry.is_dir() {
            writer.add_directory(name, options)?;
        } else {
            writer.start_file(name, options)?;
            io::copy(&mut entry, &mut writer)?;
        }
    }

    if !has_custom_properties {
        writer.start_file(CUSTOM_PROPERTIES, zip::write::SimpleFileOptions::default())?;
        writer.write_all(&custom_properties)?;
    }

    writer.finish()?;
    drop(archive);
    fs::rename(temporary_path, filepath)?;
    Ok(())
}

fn temporary_path_for(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(".metadata-tmp");
    PathBuf::from(temporary)
}

fn update_custom_properties(existing: Option<&[u8]>, source: &str) -> io::Result<Vec<u8>> {
    let Some(existing) = existing else {
        return Ok(new_custom_properties(source));
    };

    let mut reader = Reader::from_reader(existing);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut buffer = Vec::new();
    let mut skipped_property_depth = 0;
    let mut highest_pid = 1;

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) if element.local_name().as_ref() == b"property" => {
                highest_pid = highest_pid.max(property_pid(&element)?);
                if property_name(&element)?.as_deref() == Some("Source") {
                    skipped_property_depth = 1;
                } else {
                    writer.write_event(Event::Start(element.into_owned()))?;
                }
            }
            Ok(Event::Start(element)) => {
                if skipped_property_depth > 0 {
                    skipped_property_depth += 1;
                } else {
                    writer.write_event(Event::Start(element.into_owned()))?;
                }
            }
            Ok(Event::Empty(element)) if element.local_name().as_ref() == b"property" => {
                highest_pid = highest_pid.max(property_pid(&element)?);
                if property_name(&element)?.as_deref() != Some("Source") {
                    writer.write_event(Event::Empty(element.into_owned()))?;
                }
            }
            Ok(Event::Empty(element)) => {
                if skipped_property_depth == 0 {
                    writer.write_event(Event::Empty(element.into_owned()))?;
                }
            }
            Ok(Event::End(element)) if element.local_name().as_ref() == b"Properties" => {
                write_source_property(&mut writer, highest_pid + 1, source)?;
                writer.write_event(Event::End(element.into_owned()))?;
            }
            Ok(Event::End(element)) => {
                if skipped_property_depth > 0 {
                    skipped_property_depth -= 1;
                } else {
                    writer.write_event(Event::End(element.into_owned()))?;
                }
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                if skipped_property_depth == 0 {
                    writer.write_event(event.into_owned())?;
                }
            }
            Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidData, error)),
        }
        buffer.clear();
    }

    Ok(writer.into_inner())
}

fn property_name(element: &BytesStart<'_>) -> io::Result<Option<String>> {
    for attribute in element.attributes() {
        let attribute =
            attribute.map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if attribute.key.as_ref() == b"name" {
            return std::str::from_utf8(attribute.value.as_ref())
                .map(str::to_owned)
                .map(Some)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
        }
    }
    Ok(None)
}

fn property_pid(element: &BytesStart<'_>) -> io::Result<i32> {
    for attribute in element.attributes() {
        let attribute =
            attribute.map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if attribute.key.as_ref() == b"pid" {
            return std::str::from_utf8(attribute.value.as_ref())
                .ok()
                .and_then(|value| value.parse().ok())
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "custom property has an invalid pid",
                    )
                });
        }
    }
    Ok(1)
}

fn new_custom_properties(source: &str) -> Vec<u8> {
    let mut writer = Writer::new(Vec::new());
    let mut root = BytesStart::new("Properties");
    root.push_attribute((
        "xmlns",
        "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties",
    ));
    root.push_attribute((
        "xmlns:vt",
        "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes",
    ));
    writer
        .write_event(Event::Start(root))
        .expect("writing to memory cannot fail");
    write_source_property(&mut writer, 2, source).expect("writing to memory cannot fail");
    writer
        .write_event(Event::End(BytesEnd::new("Properties")))
        .expect("writing to memory cannot fail");
    writer.into_inner()
}

fn write_source_property(writer: &mut Writer<Vec<u8>>, pid: i32, source: &str) -> io::Result<()> {
    let mut property = BytesStart::new("property");
    property.push_attribute(("fmtid", "{D5CDD505-2E9C-101B-9397-08002B2CF9AE}"));
    property.push_attribute(("pid", pid.to_string().as_str()));
    property.push_attribute(("name", "Source"));
    writer.write_event(Event::Start(property))?;
    writer.write_event(Event::Start(BytesStart::new("vt:lpwstr")))?;
    writer.write_event(Event::Text(BytesText::new(source)))?;
    writer.write_event(Event::End(BytesEnd::new("vt:lpwstr")))?;
    writer.write_event(Event::End(BytesEnd::new("property")))?;
    Ok(())
}

fn add_content_type_override(xml: &str) -> String {
    if xml.contains("PartName=\"/docProps/custom.xml\"") {
        return xml.to_owned();
    }
    xml.replacen(
        "</Types>",
        &format!("<Override PartName=\"/docProps/custom.xml\" ContentType=\"{CUSTOM_PROPERTIES_TYPE}\"/></Types>"),
        1,
    )
}

fn add_custom_property_relationship(xml: &str) -> String {
    if xml.contains(CUSTOM_PROPERTIES_RELATIONSHIP) {
        return xml.to_owned();
    }
    xml.replacen(
        "</Relationships>",
        &format!("<Relationship Id=\"rIdCustomProperties\" Type=\"{CUSTOM_PROPERTIES_RELATIONSHIP}\" Target=\"docProps/custom.xml\"/></Relationships>"),
        1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_source_without_removing_other_properties() {
        let existing = br#"<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"><property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="2" name="Owner"><vt:lpwstr>Ada</vt:lpwstr></property><property fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}" pid="3" name="Source"><vt:lpwstr>old</vt:lpwstr></property></Properties>"#;

        let updated =
            String::from_utf8(update_custom_properties(Some(existing), "new & verified").unwrap())
                .unwrap();

        assert!(updated.contains("name=\"Owner\""));
        assert!(updated.contains("new &amp; verified"));
        assert_eq!(updated.matches("name=\"Source\"").count(), 1);
    }
}
