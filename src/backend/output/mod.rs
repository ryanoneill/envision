//! Output formatters for CaptureBackend.
//!
//! This module provides various output formats for rendering captured frames:
//!
//! - **Plain**: Simple text output without styling
//! - **Ansi**: Full ANSI escape codes for colored terminal output
//! - **Json**: Machine-readable JSON format
//! - **JsonPretty**: Human-readable pretty-printed JSON

mod ansi;
mod json;
mod plain;

use crate::backend::CaptureBackend;

/// Available output formats for rendering captured frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text without any styling information.
    /// Best for simple assertions and text comparisons.
    Plain,

    /// ANSI escape codes for full color and styling.
    /// Renders correctly in terminals that support ANSI codes.
    Ansi,

    /// Compact JSON format for machine consumption.
    Json,

    /// Pretty-printed JSON format for human readability.
    JsonPretty,
}

impl OutputFormat {
    /// Renders the backend using this format.
    pub fn render(self, backend: &CaptureBackend) -> String {
        match self {
            OutputFormat::Plain => plain::render(backend),
            OutputFormat::Ansi => ansi::render(backend),
            OutputFormat::Json => json::render(backend, false),
            OutputFormat::JsonPretty => json::render(backend, true),
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Plain);
    }
}
