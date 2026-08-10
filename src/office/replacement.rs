#[derive(Debug, Clone, Copy, Default)]
pub struct ReplaceOptions {
    pub case_matching: CaseMatching,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CaseMatching {
    #[default]
    Sensitive,
    UnicodeInsensitive,
}
