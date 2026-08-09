use crate::package::PartName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub part: Option<PartName>,
    pub message: String,
}

impl ValidationIssue {
    pub fn error(part: impl Into<Option<PartName>>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            part: part.into(),
            message: message.into(),
        }
    }

    pub fn warning(part: impl Into<Option<PartName>>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            part: part.into(),
            message: message.into(),
        }
    }
}
