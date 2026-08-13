use crate::cli::FileCommand;
use aigov::error;
use clap::{Args, ValueEnum};
use std::path::PathBuf;
use undoc::render::{CleanupPreset, TableFallback};

#[derive(Args)]
pub struct MarkdownArgs {
    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Include YAML frontmatter with metadata
    #[arg(short, long)]
    frontmatter: bool,

    /// Table rendering mode
    #[arg(long, default_value = "markdown")]
    table_mode: TableMode,

    /// Apply text cleanup
    #[arg(long)]
    cleanup: Option<CleanupMode>,

    /// Maximum heading level (1-6, default: 4)
    #[arg(long, default_value = "4")]
    max_heading: u8,

    /// Emit `\n\n---\n\n` for hard page breaks (default: off — markdown has
    /// no page concept).
    #[arg(long)]
    emit_page_breaks: bool,

    /// Include DOCX section headers/footers as blockquoted lines around
    /// the body (default: off — they are typically page-chrome noise).
    #[arg(long)]
    include_headers_footers: bool,

    /// Shortcut: enable both `--emit-page-breaks` and
    /// `--include-headers-footers` (i.e. `RenderOptions::lossless()`).
    #[arg(long)]
    lossless: bool,

    /// Insert HTML section boundary markers (<!-- slide N: Name -->, <!-- sheet N: Name -->).
    /// No effect on DOCX documents.
    #[arg(long)]
    section_markers: bool,
}

#[derive(Args)]
pub struct JsonArgs {
    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output compact JSON (no indentation)
    #[arg(long)]
    compact: bool,
}

#[derive(Args)]
pub struct ExtractArgs {
    /// Output directory for resources
    #[arg(short, long, default_value = ".")]
    output: PathBuf,
}

/// Table rendering mode
#[derive(Clone, ValueEnum)]
enum TableMode {
    /// Standard Markdown tables
    Markdown,
    /// HTML tables (for complex layouts)
    Html,
    /// ASCII art tables
    Ascii,
}

impl From<TableMode> for TableFallback {
    fn from(mode: TableMode) -> Self {
        match mode {
            TableMode::Markdown => TableFallback::Markdown,
            TableMode::Html => TableFallback::Html,
            TableMode::Ascii => TableFallback::Ascii,
        }
    }
}

/// Cleanup mode
#[derive(Clone, ValueEnum)]
enum CleanupMode {
    /// No cleanup
    None,
    /// Minimal cleanup
    Minimal,
    /// Standard cleanup (default)
    Standard,
    /// Aggressive cleanup
    Aggressive,
}

impl CleanupMode {
    fn to_preset(self) -> Option<CleanupPreset> {
        match self {
            Self::None => None,
            Self::Minimal => Some(CleanupPreset::Minimal),
            Self::Standard => Some(CleanupPreset::Default),
            Self::Aggressive => Some(CleanupPreset::Aggressive),
        }
    }
}

pub(crate) fn run_markdown(command: &FileCommand<MarkdownArgs>) -> error::Result<()> {
    return Ok(());
}

pub(crate) fn run_json(command: &FileCommand<JsonArgs>) -> error::Result<()> {
    return Ok(());
}

pub(crate) fn run_extract(command: &FileCommand<ExtractArgs>) -> error::Result<()> {
    return Ok(());
}
