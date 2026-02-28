//! JSON output formatter.
//!
//! Renders the captured buffer as JSON for machine consumption
//! or integration with other tools.

use crate::backend::CaptureBackend;
use serde::Serialize;

/// JSON-serializable frame representation.
///
/// This is a simpler representation than `FrameSnapshot` that focuses
/// on the visual content rather than internal state.
#[derive(Serialize)]
struct JsonFrame {
    /// Frame number
    frame: u64,

    /// Terminal dimensions
    size: JsonSize,

    /// Cursor state
    cursor: JsonCursor,

    /// Content as lines of text (without styling)
    lines: Vec<String>,

    /// Styled cells (only non-empty cells with non-default styling)
    styled_cells: Vec<JsonStyledCell>,
}

#[derive(Serialize)]
struct JsonSize {
    width: u16,
    height: u16,
}

#[derive(Serialize)]
struct JsonCursor {
    x: u16,
    y: u16,
    visible: bool,
}

#[derive(Serialize)]
struct JsonStyledCell {
    x: u16,
    y: u16,
    symbol: String,
    fg: Option<String>,
    bg: Option<String>,
    bold: bool,
    italic: bool,
    underlined: bool,
}

/// Renders the backend as JSON.
///
/// If `pretty` is true, the output is formatted with indentation.
/// Otherwise, it's compact single-line JSON.
pub fn render(backend: &CaptureBackend, pretty: bool) -> String {
    let width = backend.width();
    let height = backend.height();
    let cursor_pos = backend.cursor_position();

    // Collect lines
    let lines: Vec<String> = (0..height).map(|y| backend.row_content(y)).collect();

    // Collect styled cells (cells with non-default styling)
    let mut styled_cells = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if let Some(cell) = backend.cell(x, y) {
                // Only include cells with styling or non-space content
                let has_styling = cell.fg != crate::backend::cell::SerializableColor::Reset
                    || cell.bg != crate::backend::cell::SerializableColor::Reset
                    || !cell.modifiers.is_empty();

                if has_styling && cell.symbol() != " " {
                    styled_cells.push(JsonStyledCell {
                        x,
                        y,
                        symbol: cell.symbol().to_string(),
                        fg: if cell.fg != crate::backend::cell::SerializableColor::Reset {
                            Some(format!("{:?}", cell.fg))
                        } else {
                            None
                        },
                        bg: if cell.bg != crate::backend::cell::SerializableColor::Reset {
                            Some(format!("{:?}", cell.bg))
                        } else {
                            None
                        },
                        bold: cell.modifiers.bold,
                        italic: cell.modifiers.italic,
                        underlined: cell.modifiers.underlined,
                    });
                }
            }
        }
    }

    let frame = JsonFrame {
        frame: backend.current_frame(),
        size: JsonSize { width, height },
        cursor: JsonCursor {
            x: cursor_pos.x,
            y: cursor_pos.y,
            visible: backend.is_cursor_visible(),
        },
        lines,
        styled_cells,
    };

    if pretty {
        serde_json::to_string_pretty(&frame).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    } else {
        serde_json::to_string(&frame).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }
}

/// Renders only the content lines as a JSON array.
///
/// This is a minimal representation useful for simple text comparisons.
pub fn render_lines_only(backend: &CaptureBackend) -> String {
    let height = backend.height();
    let lines: Vec<String> = (0..height).map(|y| backend.row_content(y)).collect();
    serde_json::to_string(&lines).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::cell::SerializableColor;

    #[test]
    fn test_json_render() {
        let mut backend = CaptureBackend::new(10, 3);

        // Set some content
        for (i, c) in "Hello".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }

        let output = render(&backend, false);

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["size"]["width"], 10);
        assert_eq!(parsed["size"]["height"], 3);
        assert!(parsed["lines"][0].as_str().unwrap().starts_with("Hello"));
    }

    #[test]
    fn test_json_render_pretty() {
        let backend = CaptureBackend::new(5, 2);
        let output = render(&backend, true);

        // Pretty output should have newlines
        assert!(output.contains('\n'));

        // Should still be valid JSON
        let _: serde_json::Value = serde_json::from_str(&output).unwrap();
    }

    #[test]
    fn test_json_styled_cells() {
        let mut backend = CaptureBackend::new(5, 1);

        // Add a styled cell
        if let Some(cell) = backend.cell_mut(0, 0) {
            cell.set_char('X');
            cell.fg = SerializableColor::Red;
            cell.modifiers.bold = true;
        }

        let output = render(&backend, false);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // Should have one styled cell
        let styled = &parsed["styled_cells"];
        assert_eq!(styled.as_array().unwrap().len(), 1);
        assert_eq!(styled[0]["symbol"], "X");
        assert_eq!(styled[0]["bold"], true);
    }

    #[test]
    fn test_json_lines_only() {
        let mut backend = CaptureBackend::new(5, 2);

        for (i, c) in "Hi".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }

        let output = render_lines_only(&backend);
        let parsed: Vec<String> = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed.len(), 2);
        assert!(parsed[0].starts_with("Hi"));
    }
}
