use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct OfficeMetadata {
    pub core: CoreMetadata,
    pub extended: ExtendedMetadata,
    pub custom: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CoreMetadata {
    pub category: Option<String>,
    pub content_status: Option<String>,
    pub content_type: Option<String>,
    pub created: Option<String>,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub identifier: Option<String>,
    pub keywords: Vec<String>,
    pub language: Option<String>,
    pub last_modified_by: Option<String>,
    pub last_printed: Option<String>,
    pub modified: Option<String>,
    pub revision: Option<String>,
    pub subject: Option<String>,
    pub title: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ExtendedMetadata {
    pub app_version: Option<String>,
    pub application: Option<String>,
    pub characters: Option<u32>,
    pub characters_with_spaces: Option<u32>,
    pub company: Option<String>,
    pub dig_sig: Option<String>,
    pub doc_security: Option<u32>,
    pub heading_pairs: Vec<HeadingPair>,
    pub hyperlinks_changed: Option<bool>,
    pub lines: Option<u32>,
    pub links_up_to_date: Option<bool>,
    pub pages: Option<u32>,
    pub paragraphs: Option<u32>,
    pub scale_crop: Option<bool>,
    pub shared_doc: Option<bool>,
    pub template: Option<String>,
    pub titles_of_parts: Vec<String>,
    pub total_time: Option<u32>,
    pub words: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HeadingPair {
    pub name: String,
    pub count: u32,
}
