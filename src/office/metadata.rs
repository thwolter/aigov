use std::collections::BTreeMap;

use serde::Serialize;

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
