use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartName(String);

impl PartName {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();

        let normalized = value
            .trim_start_matches('/')
            .replace('\\', "/");

        Self(normalized)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for PartName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for PartName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for PartName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
