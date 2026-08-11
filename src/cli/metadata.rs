use aigov::office::metadata::MetadataPatch;
use aigov::package::OoxmlPackage;
use aigov::{error, ooxml};
use clap::{Args, Subcommand};
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct MetadataArgs {
    #[arg(short, long)]
    pub pretty: bool,

    #[command(subcommand)]
    pub command: Option<MetadataCommand>,
}

#[derive(Subcommand)]
pub enum MetadataCommand {
    #[command(override_usage = "aigov <FILEPATH> metadata set [OPTIONS]")]
    Set(SetArgs),
}

#[derive(Args)]
#[command(
    about = "Update editable document metadata; application-generated statistics remain read-only"
)]
pub struct SetArgs {
    #[arg(long, short)]
    pub title: Option<String>,

    #[arg(long, short)]
    pub description: Option<String>,

    #[arg(long, short)]
    pub author: Option<String>,

    #[arg(long, short)]
    pub keywords: Option<String>,

    #[arg(long, short)]
    pub creator: Option<String>,

    #[arg(long, short)]
    pub subject: Option<String>,

    #[arg(long)]
    pub category: Option<String>,

    #[arg(long = "content-status")]
    pub content_status: Option<String>,

    #[arg(long = "content-type")]
    pub content_type: Option<String>,

    #[arg(long, short)]
    pub language: Option<String>,

    #[arg(long, short)]
    pub identifier: Option<String>,

    #[arg(long, short)]
    pub version: Option<String>,

    #[arg(long, value_name = "NAME=VALUE", value_parser = parse_custom_property)]
    pub custom: Vec<(String, String)>,

    #[arg(long, short)]
    pub profile: Option<PathBuf>,
}

fn parse_custom_property(value: &str) -> std::result::Result<(String, String), String> {
    let Some((name, value)) = value.split_once('=') else {
        return Err("must be in NAME=VALUE form".into());
    };
    if name.is_empty() {
        return Err("property name cannot be empty".into());
    }
    Ok((name.into(), value.into()))
}

impl SetArgs {
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.author.is_none()
            && self.keywords.is_none()
            && self.creator.is_none()
            && self.subject.is_none()
            && self.category.is_none()
            && self.content_status.is_none()
            && self.content_type.is_none()
            && self.language.is_none()
            && self.identifier.is_none()
            && self.version.is_none()
            && self.custom.is_empty()
            && self.profile.is_none()
    }
}

impl From<&SetArgs> for MetadataPatch {
    fn from(args: &SetArgs) -> Self {
        Self {
            title: args.title.clone(),
            description: args.description.clone(),
            creator: args.creator.clone().or_else(|| args.author.clone()),
            keywords: args.keywords.as_deref().map(|keywords| {
                keywords
                    .split(',')
                    .map(str::trim)
                    .filter(|keyword| !keyword.is_empty())
                    .map(str::to_owned)
                    .collect()
            }),
            subject: args.subject.clone(),
            category: args.category.clone(),
            content_status: args.content_status.clone(),
            content_type: args.content_type.clone(),
            language: args.language.clone(),
            identifier: args.identifier.clone(),
            version: args.version.clone(),
            custom: (!args.custom.is_empty())
                .then(|| args.custom.iter().cloned().collect::<BTreeMap<_, _>>()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MetadataPatch, SetArgs};

    #[test]
    fn detects_an_empty_update() {
        assert!(
            SetArgs {
                title: None,
                description: None,
                author: None,
                keywords: None,
                creator: None,
                subject: None,
                category: None,
                content_status: None,
                content_type: None,
                language: None,
                identifier: None,
                version: None,
                custom: Vec::new(),
                profile: None,
            }
            .is_empty()
        );
    }

    #[test]
    fn converts_custom_properties_to_a_metadata_patch() {
        let args = SetArgs {
            title: None,
            description: None,
            author: None,
            keywords: None,
            creator: None,
            subject: None,
            category: None,
            content_status: None,
            content_type: None,
            language: None,
            identifier: None,
            version: None,
            custom: vec![("Client".into(), "Acme".into())],
            profile: None,
        };

        let patch = MetadataPatch::from(&args);

        assert_eq!(patch.custom.unwrap()["Client"], "Acme");
    }
}

pub fn show_metadata(filepath: &Path, metadata_args: &MetadataArgs) -> error::Result<()> {
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

pub fn set_metadata(filepath: &Path, args: &SetArgs) -> error::Result<()> {
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
