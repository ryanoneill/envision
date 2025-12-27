//! ANSI escape code output formatter.
//!
//! Renders the captured buffer with full ANSI escape codes for colors
//! and text modifiers. The output can be displayed in any terminal
//! that supports ANSI escape sequences.

use crate::backend::cell::SerializableColor;
use crate::backend::CaptureBackend;

/// ANSI reset sequence
const RESET: &str = "\x1b[0m";

/// Renders the backend with ANSI escape codes for colors and styling.
///
/// This output can be printed directly to a terminal that supports ANSI
/// codes to see the full visual representation of the captured frame.
pub fn render(backend: &CaptureBackend) -> String {
    let width = backend.width();
    let height = backend.height();
    let mut output = String::new();

    for y in 0..height {
        if y > 0 {
            output.push('\n');
        }

        let mut last_fg = SerializableColor::Reset;
        let mut last_bg = SerializableColor::Reset;
        let mut last_modifiers = crate::backend::cell::SerializableModifier::empty();

        for x in 0..width {
            let cell = backend.cell(x, y).unwrap();

            // Check if we need to change styling
            let need_fg_change = cell.fg != last_fg;
            let need_bg_change = cell.bg != last_bg;
            let need_mod_change = cell.modifiers != last_modifiers;

            if need_fg_change || need_bg_change || need_mod_change {
                // Reset and apply new styles
                output.push_str(RESET);

                // Apply modifiers first
                if !cell.modifiers.is_empty() {
                    output.push_str(&cell.modifiers.to_ansi());
                }

                // Apply colors
                if cell.fg != SerializableColor::Reset {
                    output.push_str(&cell.fg.to_ansi_fg());
                }
                if cell.bg != SerializableColor::Reset {
                    output.push_str(&cell.bg.to_ansi_bg());
                }

                last_fg = cell.fg;
                last_bg = cell.bg;
                last_modifiers = cell.modifiers;
            }

            output.push_str(cell.symbol());
        }

        // Reset at end of each line
        if last_fg != SerializableColor::Reset
            || last_bg != SerializableColor::Reset
            || !last_modifiers.is_empty()
        {
            output.push_str(RESET);
        }
    }

    output
}

/// Renders the backend with ANSI codes and includes a legend explaining the styling.
///
/// This is useful for debugging, as it shows what colors and modifiers
/// are being used in the frame.
#[allow(dead_code)]
pub fn render_with_legend(backend: &CaptureBackend) -> String {
    let mut output = render(backend);

    // Collect unique styles used
    let width = backend.width();
    let height = backend.height();
    let mut colors_used: std::collections::HashSet<String> = std::collections::HashSet::new();

    for y in 0..height {
        for x in 0..width {
            let cell = backend.cell(x, y).unwrap();
            if cell.fg != SerializableColor::Reset {
                colors_used.insert(format!("FG: {:?}", cell.fg));
            }
            if cell.bg != SerializableColor::Reset {
                colors_used.insert(format!("BG: {:?}", cell.bg));
            }
        }
    }

    if !colors_used.is_empty() {
        output.push_str("\n\n--- Colors Used ---\n");
        for color in colors_used {
            output.push_str(&color);
            output.push('\n');
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::cell::SerializableColor;

    #[test]
    fn test_ansi_render_plain() {
        let mut backend = CaptureBackend::new(5, 1);

        for (i, c) in "Hello".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }

        let output = render(&backend);
        // Should contain "Hello" without any escape codes (all reset colors)
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_ansi_render_colored() {
        let mut backend = CaptureBackend::new(3, 1);

        // Set a red 'R'
        if let Some(cell) = backend.cell_mut(0, 0) {
            cell.set_char('R');
            cell.fg = SerializableColor::Red;
        }

        let output = render(&backend);
        // Should contain ANSI red foreground code
        assert!(output.contains("\x1b[31m"));
        assert!(output.contains("R"));
        assert!(output.contains(RESET));
    }

    #[test]
    fn test_ansi_render_styled() {
        let mut backend = CaptureBackend::new(1, 1);

        if let Some(cell) = backend.cell_mut(0, 0) {
            cell.set_char('B');
            cell.modifiers.bold = true;
        }

        let output = render(&backend);
        // Should contain bold modifier code (ESC[1m)
        assert!(output.contains("\x1b[1m"));
        assert!(output.contains("B"));
    }

    #[test]
    fn test_ansi_render_multi_row() {
        let mut backend = CaptureBackend::new(3, 2);

        // Row 0: "ABC" in red
        for (i, c) in "ABC".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
                cell.fg = SerializableColor::Red;
            }
        }

        // Row 1: "123" in blue
        for (i, c) in "123".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 1) {
                cell.set_char(c);
                cell.fg = SerializableColor::Blue;
            }
        }

        let output = render(&backend);
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("ABC"));
        assert!(lines[1].contains("123"));
    }
}
