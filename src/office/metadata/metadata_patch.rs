use crate::error::Result;
use crate::office::metadata::OfficeMetadata;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

/// A partial update for editable [`OfficeMetadata`] fields.
///
/// Fields set to `None` leave existing values unchanged. Unknown JSON fields
/// are rejected. Custom properties are merged into the existing map, while an
/// incoming value replaces a value with the same name. Application-generated
/// extended properties, such as page and word counts, are not included.
///
/// # Examples
///
/// ```
/// use aigov::office::metadata::{MetadataPatch, OfficeMetadata};
///
/// let mut metadata = OfficeMetadata::default();
/// let patch = MetadataPatch::from_json(
///     r#"{"title":"New title","keywords":"rust, office","custom":{"Team":"Docs"}}"#,
/// )?;
/// patch.apply_to(&mut metadata)?;
///
/// assert_eq!(metadata.core.title.as_deref(), Some("New title"));
/// assert_eq!(metadata.core.keywords, ["rust", "office"]);
/// # Ok::<(), aigov::error::OfficeError>(())
/// ```
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetadataPatch {
    pub category: Option<String>,
    pub content_status: Option<String>,
    pub content_type: Option<String>,
    pub creator: Option<String>,
    pub custom: Option<BTreeMap<String, String>>,
    pub description: Option<String>,
    pub identifier: Option<String>,
    #[serde(default, deserialize_with = "deserialize_keywords")]
    pub keywords: Option<Vec<String>>,
    pub language: Option<String>,
    pub subject: Option<String>,
    pub title: Option<String>,
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
    /// Parses a JSON object into a metadata patch.
    ///
    /// Keywords may be an array or comma-separated text. Returns an error for
    /// malformed JSON or unsupported fields.
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Applies this patch, preserving fields that were not provided.
    ///
    /// Custom properties are merged rather than replacing the complete map.
    pub fn apply_to(self, metadata: &mut OfficeMetadata) -> Result<()> {
        metadata.core.category = self.category.or(metadata.core.category.clone());
        metadata.core.content_status = self.content_status.or(metadata.core.content_status.clone());
        metadata.core.content_type = self.content_type.or(metadata.core.content_type.clone());
        metadata.core.creator = self.creator.or(metadata.core.creator.clone());
        metadata.custom.extend(self.custom.unwrap_or_default());
        metadata.core.description = self.description.or(metadata.core.description.clone());
        metadata.core.identifier = self.identifier.or(metadata.core.identifier.clone());
        metadata.core.keywords = self.keywords.unwrap_or(metadata.core.keywords.clone());
        metadata.core.language = self.language.or(metadata.core.language.clone());
        metadata.core.subject = self.subject.or(metadata.core.subject.clone());
        metadata.core.title = self.title.or(metadata.core.title.clone());
        metadata.core.version = self.version.or(metadata.core.version.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MetadataPatch, OfficeMetadata};

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

    #[test]
    fn applies_custom_profile_properties_without_removing_existing_ones() {
        let patch = MetadataPatch::from_json(r#"{"custom":{"Client":"Acme"}}"#).unwrap();
        let mut metadata = OfficeMetadata::default();
        metadata.custom.insert("Owner".into(), "Ada".into());
        patch.apply_to(&mut metadata).unwrap();
        assert_eq!(metadata.custom["Client"], "Acme");
        assert_eq!(metadata.custom["Owner"], "Ada");
    }
}
