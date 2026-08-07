use std::io::{self, IsTerminal};

use crate::Result;
use crate::cli::{MetadataArgs, metadata::SetArgs};
use crate::ooxml::OoxmlPackage;
use crate::ooxml::properties::{read_metadata, write_metadata};

pub fn show_metadata(filepath: &str, metadata_args: &MetadataArgs) -> Result<()> {
    println!("Metadata {}:", filepath);
    let package = OoxmlPackage::open(filepath)?;
    let metadata = read_metadata(&package)?;

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

pub fn set_metadata(filepath: &str, args: &SetArgs) -> Result<()> {
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

    write_metadata(&mut package, &metadata)?;
    package.save(std::path::Path::new(filepath))?;

    let message = format!("Metadata updated: {filepath}");
    if io::stdout().is_terminal() {
        println!("\x1b[32m✓ {message}\x1b[0m");
    } else {
        println!("✓ {message}");
    }

    Ok(())
}
