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

pub use ansi::render_with_legend;
pub use json::render_lines_only;
pub use plain::render_trimmed;

/// Available output formats for rendering captured frames.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Plain text without any styling information.
    /// Best for simple assertions and text comparisons.
    #[default]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Plain);
    }

    #[test]
    fn test_output_format_render_plain() {
        let mut backend = CaptureBackend::new(5, 1);
        for (i, c) in "Hello".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }

        let output = OutputFormat::Plain.render(&backend);
        assert_eq!(output.trim(), "Hello");
    }

    #[test]
    fn test_output_format_render_ansi() {
        use crate::backend::cell::SerializableColor;

        let mut backend = CaptureBackend::new(5, 1);
        if let Some(cell) = backend.cell_mut(0, 0) {
            cell.set_char('R');
            cell.fg = SerializableColor::Red;
        }

        let output = OutputFormat::Ansi.render(&backend);
        assert!(output.contains("\x1b[31m")); // Red color code
        assert!(output.contains("R"));
    }

    #[test]
    fn test_output_format_render_json() {
        let mut backend = CaptureBackend::new(3, 1);
        if let Some(cell) = backend.cell_mut(0, 0) {
            cell.set_char('X');
        }

        let output = OutputFormat::Json.render(&backend);
        assert!(output.starts_with("{"));
        assert!(output.ends_with("}"));
        assert!(output.contains("\"width\":3"));
        assert!(output.contains("\"height\":1"));
    }

    #[test]
    fn test_output_format_render_json_pretty() {
        let mut backend = CaptureBackend::new(3, 1);
        if let Some(cell) = backend.cell_mut(0, 0) {
            cell.set_char('Y');
        }

        let output = OutputFormat::JsonPretty.render(&backend);
        // Pretty JSON has newlines and indentation
        assert!(output.contains("\n"));
        assert!(output.contains("  ")); // Indentation
        assert!(output.contains("\"width\""));
    }

    #[test]
    fn test_output_format_clone() {
        let format = OutputFormat::Ansi;
        let cloned = format; // Copy, not clone
        assert_eq!(format, cloned);
    }

    #[test]
    fn test_output_format_copy() {
        let format = OutputFormat::Json;
        let copied = format; // Copy
        assert_eq!(format, copied);
    }
}
