use crate::Result;
use crate::cli::{metadata::MetadataArgs, metadata::SetArgs};
use aigov::metadata_patch::MetadataPatch;
use aigov::ooxml;
use aigov::package::OoxmlPackage;
use std::fs::read_to_string;
use std::io::{self};
use std::path::Path;

pub fn show_metadata(filepath: &Path, metadata_args: &MetadataArgs) -> Result<()> {
    let package = OoxmlPackage::from_file(filepath)?;
    let metadata = ooxml::read_metadata(&package)?;
    let metadata = serde_json::json!({
        "core": {
            "title": &metadata.core.title,
            "subject": &metadata.core.subject,
            "creator": &metadata.core.creator,
            "description": &metadata.core.description,
            "keywords": &metadata.core.keywords,
            "category": &metadata.core.category,
            "content_status": &metadata.core.content_status,
            "content_type": &metadata.core.content_type,
            "language": &metadata.core.language,
            "last_modified_by": &metadata.core.last_modified_by,
            "created": &metadata.core.created,
            "modified": &metadata.core.modified,
            "last_printed": &metadata.core.last_printed,
            "revision": &metadata.core.revision,
            "identifier": &metadata.core.identifier,
            "version": &metadata.core.version,
        },
        "extended": {
            "application": &metadata.extended.application,
            "app_version": &metadata.extended.app_version,
            "template": &metadata.extended.template,
            "company": &metadata.extended.company,
            "total_time": &metadata.extended.total_time,
            "pages": &metadata.extended.pages,
            "words": &metadata.extended.words,
            "characters": &metadata.extended.characters,
            "characters_with_spaces": &metadata.extended.characters_with_spaces,
            "lines": &metadata.extended.lines,
            "paragraphs": &metadata.extended.paragraphs,
            "doc_security": &metadata.extended.doc_security,
            "scale_crop": &metadata.extended.scale_crop,
            "links_up_to_date": &metadata.extended.links_up_to_date,
            "shared_doc": &metadata.extended.shared_doc,
            "hyperlinks_changed": &metadata.extended.hyperlinks_changed,
            "heading_pairs": &metadata.extended.heading_pairs,
            "titles_of_parts": &metadata.extended.titles_of_parts,
            "dig_sig": &metadata.extended.dig_sig,
        },
        "custom": &metadata.custom,
    });

    let stdout = io::stdout();
    let mut out = stdout.lock();

    if metadata_args.pretty {
        serde_json::to_writer_pretty(&mut out, &metadata)?;
    } else {
        serde_json::to_writer(&mut out, &metadata)?;
    }
    println!();

    Ok(())
}

pub fn set_metadata(filepath: &Path, args: &SetArgs) -> Result<()> {
    let mut package = OoxmlPackage::from_file(filepath)?;
    let mut metadata = ooxml::read_metadata(&package)?;

    if let Some(profile) = &args.profile {
        let json = read_to_string(profile)?;
        MetadataPatch::from_json(&json)?.apply_to(&mut metadata)?;
    }
    MetadataPatch::from(args).apply_to(&mut metadata)?;

    ooxml::write_metadata(&mut package, &metadata)?;
    package.save(Path::new(filepath))?;

    let filepath = filepath.to_string_lossy();
    super::print_success(format!("Metadata updated: {filepath}"));

    Ok(())
}
