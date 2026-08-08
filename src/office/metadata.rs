use crate::error::Result;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HeadingPair {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct OfficeMetadata {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub last_modified_by: Option<String>,
    pub company: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub category: Option<String>,
    pub content_status: Option<String>,
    pub content_type: Option<String>,
    pub language: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub last_printed: Option<String>,
    pub revision: Option<String>,
    pub identifier: Option<String>,
    pub version: Option<String>,
    pub application: Option<String>,
    pub app_version: Option<String>,
    pub template: Option<String>,
    pub total_time: Option<u32>,
    pub pages: Option<u32>,
    pub words: Option<u32>,
    pub characters: Option<u32>,
    pub characters_with_spaces: Option<u32>,
    pub lines: Option<u32>,
    pub paragraphs: Option<u32>,
    pub doc_security: Option<u32>,
    pub scale_crop: Option<bool>,
    pub links_up_to_date: Option<bool>,
    pub shared_doc: Option<bool>,
    pub hyperlinks_changed: Option<bool>,
    pub heading_pairs: Vec<HeadingPair>,
    pub titles_of_parts: Vec<String>,
    pub dig_sig: Option<String>,
    pub custom: BTreeMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub creator: Option<String>,
    #[serde(deserialize_with = "deserialize_keywords")]
    pub keywords: Option<Vec<String>>,
    pub subject: Option<String>,
    pub category: Option<String>,
    pub content_status: Option<String>,
    pub content_type: Option<String>,
    pub language: Option<String>,
    pub identifier: Option<String>,
    pub version: Option<String>,
}

fn deserialize_keywords<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Keywords {
        List(Vec<String>),
        CommaSeparated(String),
    }

    Option::<Keywords>::deserialize(deserializer).map(|keywords| {
        keywords.map(|keywords| match keywords {
            Keywords::List(keywords) => keywords,
            Keywords::CommaSeparated(keywords) => keywords
                .split(',')
                .map(str::trim)
                .filter(|keyword| !keyword.is_empty())
                .map(str::to_owned)
                .collect(),
        })
    })
}

impl MetadataPatch {
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn apply_to(self, metadata: &mut OfficeMetadata) -> Result<()> {
        metadata.title = self.title.or(metadata.title.clone());
        metadata.description = self.description.or(metadata.description.clone());
        metadata.creator = self.creator.or(metadata.creator.clone());
        metadata.keywords = self.keywords.unwrap_or(metadata.keywords.clone());
        metadata.subject = self.subject.or(metadata.subject.clone());
        metadata.category = self.category.or(metadata.category.clone());
        metadata.content_status = self.content_status.or(metadata.content_status.clone());
        metadata.content_type = self.content_type.or(metadata.content_type.clone());
        metadata.language = self.language.or(metadata.language.clone());
        metadata.identifier = self.identifier.or(metadata.identifier.clone());
        metadata.version = self.version.or(metadata.version.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MetadataPatch;

    #[test]
    fn accepts_comma_separated_profile_keywords() {
        let patch = MetadataPatch::from_json(r#"{"keywords":"test, profile, office"}"#).unwrap();

        assert_eq!(
            patch.keywords,
            Some(vec!["test".into(), "profile".into(), "office".into()])
        );
    }

    #[test]
    fn accepts_keyword_array_profile_keywords() {
        let patch = MetadataPatch::from_json(r#"{"keywords":["test","profile"]}"#).unwrap();

        assert_eq!(patch.keywords, Some(vec!["test".into(), "profile".into()]));
    }
}
