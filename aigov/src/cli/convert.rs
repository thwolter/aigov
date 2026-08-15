use crate::cli;
use crate::cli::{FileCommand, create_spinner, print_success};
use aigov::error;
use clap::{Args, ValueEnum};
use std::fs;
use std::path::PathBuf;
use undoc::render::{CleanupPreset, HeadingConfig, RenderOptions, TableFallback};

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
#[derive(Clone, Copy, ValueEnum)]
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
#[derive(Clone, Copy, ValueEnum)]
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
    let pb = create_spinner("Parsing document...");

    let output = cli::output_path(&command.filepath, command.args.output.as_deref(), "md");

    let doc = undoc::parse_file(&command.filepath)?;
    pb.set_message("Rendering to Markdown...");

    let heading_config = HeadingConfig::default().with_default_style_mapping();
    let base = if command.args.lossless {
        RenderOptions::lossless()
    } else {
        RenderOptions::new()
            .with_emit_page_breaks(command.args.emit_page_breaks)
            .with_include_headers_footers(command.args.include_headers_footers)
    };
    let mut options = base
        .with_frontmatter(command.args.frontmatter)
        .with_table_fallback(command.args.table_mode.into())
        .with_max_heading(command.args.max_heading)
        .with_heading_config(heading_config);

    if command.args.section_markers {
        options = options.with_section_markers(undoc::SectionMarkerStyle::Comment);
    }

    if let Some(preset) = command.args.cleanup.and_then(CleanupMode::to_preset) {
        options = options.with_cleanup_preset(preset);
    }

    let markdown = undoc::render::to_markdown(&doc, &options)?;

    pb.finish_and_clear();
    fs::write(&output, &markdown)?;

    if let Some(path) = command.args.output.as_deref() {
        print_success(format!("Converted to Markdown ({})", path.display()));
    } else {
        print_success("Converted to Markdown");
    }

    Ok(())
}

pub(crate) fn run_json(_command: &FileCommand<JsonArgs>) -> error::Result<()> {
    todo!()
}

pub(crate) fn run_extract(_command: &FileCommand<ExtractArgs>) -> error::Result<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    use clap::Parser;

    #[test]
    fn convert_docx_to_md() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output.md");
        let Some(Commands::Markdown(command)) = Cli::try_parse_from([
            "aigov",
            "markdown",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/convert.docx"),
            "-o",
            output.to_str().unwrap(),
        ])
        .unwrap()
        .command
        else {
            panic!("expected markdown command");
        };

        run_markdown(&command).unwrap();
        assert!(!fs::read_to_string(&output).unwrap().is_empty());
    }
}
