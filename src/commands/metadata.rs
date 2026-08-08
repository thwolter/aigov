use crate::Result;
use crate::cli::{MetadataArgs, metadata::SetArgs};
use crate::ooxml::OoxmlPackage;
use crate::ooxml::properties::{read_metadata, write_metadata};
use std::io::{self};
use std::path::Path;

pub fn show_metadata(filepath: &Path, metadata_args: &MetadataArgs) -> Result<()> {
    let package = OoxmlPackage::open(filepath)?;
    let metadata = read_metadata(&package)?;
    let metadata = serde_json::json!({
        "core": {
            "title": &metadata.title,
            "subject": &metadata.subject,
            "creator": &metadata.creator,
            "description": &metadata.description,
            "keywords": &metadata.keywords,
            "category": &metadata.category,
            "content_status": &metadata.content_status,
            "content_type": &metadata.content_type,
            "language": &metadata.language,
            "last_modified_by": &metadata.last_modified_by,
            "created": &metadata.created,
            "modified": &metadata.modified,
            "last_printed": &metadata.last_printed,
            "revision": &metadata.revision,
            "identifier": &metadata.identifier,
            "version": &metadata.version,
        },
        "extended": {
            "application": &metadata.application,
            "app_version": &metadata.app_version,
            "template": &metadata.template,
            "company": &metadata.company,
            "total_time": &metadata.total_time,
            "pages": &metadata.pages,
            "words": &metadata.words,
            "characters": &metadata.characters,
            "characters_with_spaces": &metadata.characters_with_spaces,
            "lines": &metadata.lines,
            "paragraphs": &metadata.paragraphs,
            "doc_security": &metadata.doc_security,
            "scale_crop": &metadata.scale_crop,
            "links_up_to_date": &metadata.links_up_to_date,
            "shared_doc": &metadata.shared_doc,
            "hyperlinks_changed": &metadata.hyperlinks_changed,
            "heading_pairs": &metadata.heading_pairs,
            "titles_of_parts": &metadata.titles_of_parts,
            "dig_sig": &metadata.dig_sig,
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
    let mut package = OoxmlPackage::open(filepath)?;
    let mut metadata = read_metadata(&package)?;

    if let Some(title) = &args.title {
        metadata.title = Some(title.clone());
    }
    if let Some(description) = &args.description {
        metadata.description = Some(description.clone());
    }
    if let Some(author) = &args.author {
        metadata.creator = Some(author.clone());
    }
    if let Some(creator) = &args.creator {
        metadata.creator = Some(creator.clone());
    }
    if let Some(keywords) = &args.keywords {
        metadata.keywords = keywords
            .split(',')
            .map(str::trim)
            .filter(|keyword| !keyword.is_empty())
            .map(str::to_owned)
            .collect();
    }
    if let Some(subject) = &args.subject {
        metadata.subject = Some(subject.clone());
    }
    if let Some(category) = &args.category {
        metadata.category = Some(category.clone());
    }
    if let Some(content_status) = &args.content_status {
        metadata.content_status = Some(content_status.clone());
    }
    if let Some(content_type) = &args.content_type {
        metadata.content_type = Some(content_type.clone());
    }
    if let Some(language) = &args.language {
        metadata.language = Some(language.clone());
    }
    if let Some(identifier) = &args.identifier {
        metadata.identifier = Some(identifier.clone());
    }
    if let Some(version) = &args.version {
        metadata.version = Some(version.clone());
    }

    write_metadata(&mut package, &metadata)?;
    package.save(Path::new(filepath))?;

    let filepath = filepath.to_string_lossy();
    super::print_success(format!("Metadata updated: {filepath}"));

    Ok(())
}

