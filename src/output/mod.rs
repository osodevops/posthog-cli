pub mod csv_fmt;
pub mod json;
pub mod jsonl;
pub mod table;

use std::io::IsTerminal;

use crate::cli::Format;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Json,
    Table,
    Csv,
    Jsonl,
}

impl OutputFormat {
    pub fn from_cli(fmt: Option<Format>) -> Self {
        match fmt {
            Some(Format::Json) => OutputFormat::Json,
            Some(Format::Table) => OutputFormat::Table,
            Some(Format::Csv) => OutputFormat::Csv,
            Some(Format::Jsonl) => OutputFormat::Jsonl,
            None => {
                if std::io::stdout().is_terminal() {
                    OutputFormat::Table
                } else {
                    OutputFormat::Json
                }
            }
        }
    }
}

/// Render an API envelope response. Wraps `data` in the standard
/// `{"ok": true, "data": ..., "meta": ...}` envelope for JSON output.
pub fn render(format: OutputFormat, data: &serde_json::Value, meta: &serde_json::Value) -> String {
    match format {
        OutputFormat::Json => json::render(data, meta),
        OutputFormat::Table => table::render(data),
        OutputFormat::Csv => csv_fmt::render(data),
        OutputFormat::Jsonl => jsonl::render(data),
    }
}

/// Render a single value (e.g. auth status, whoami).
pub fn render_value(format: OutputFormat, value: &serde_json::Value) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(value).unwrap_or_default(),
        OutputFormat::Table => table::render(value),
        OutputFormat::Csv => csv_fmt::render(value),
        OutputFormat::Jsonl => jsonl::render(value),
    }
}
