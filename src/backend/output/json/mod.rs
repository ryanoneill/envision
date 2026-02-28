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
#[allow(dead_code)]
pub fn render_lines_only(backend: &CaptureBackend) -> String {
    let height = backend.height();
    let lines: Vec<String> = (0..height).map(|y| backend.row_content(y)).collect();
    serde_json::to_string(&lines).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

#[cfg(test)]
mod tests;
