#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OfficeMetadata {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub last_modified_by: Option<String>,
    pub company: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
}