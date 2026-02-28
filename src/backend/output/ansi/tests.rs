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

#[test]
fn test_ansi_render_with_background() {
    let mut backend = CaptureBackend::new(3, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('B');
        cell.bg = SerializableColor::Blue;
    }

    let output = render(&backend);
    // Should contain ANSI blue background code (44)
    assert!(output.contains("\x1b[44m"));
}

#[test]
fn test_ansi_render_combined_colors() {
    let mut backend = CaptureBackend::new(3, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('X');
        cell.fg = SerializableColor::Yellow;
        cell.bg = SerializableColor::Magenta;
    }

    let output = render(&backend);
    // Should contain both foreground and background codes
    assert!(output.contains("\x1b[33m")); // Yellow FG
    assert!(output.contains("\x1b[45m")); // Magenta BG
}

#[test]
fn test_ansi_render_no_style_change() {
    let mut backend = CaptureBackend::new(3, 1);

    // All cells same style (red)
    for i in 0..3 {
        if let Some(cell) = backend.cell_mut(i, 0) {
            cell.set_char('X');
            cell.fg = SerializableColor::Red;
        }
    }

    let output = render(&backend);
    // Should still have reset at end since color was applied
    assert!(output.contains("XXX"));
    assert!(output.ends_with(RESET));
}

#[test]
fn test_ansi_render_style_changes() {
    let mut backend = CaptureBackend::new(3, 1);

    // First cell red
    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('R');
        cell.fg = SerializableColor::Red;
    }
    // Second cell green
    if let Some(cell) = backend.cell_mut(1, 0) {
        cell.set_char('G');
        cell.fg = SerializableColor::Green;
    }
    // Third cell blue
    if let Some(cell) = backend.cell_mut(2, 0) {
        cell.set_char('B');
        cell.fg = SerializableColor::Blue;
    }

    let output = render(&backend);
    // Should have RESET between color changes
    let reset_count = output.matches(RESET).count();
    assert!(reset_count >= 3); // At least one for each change
}

#[test]
fn test_ansi_render_multiple_modifiers() {
    use crate::backend::cell::SerializableModifier;

    let mut backend = CaptureBackend::new(2, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('A');
        cell.modifiers = SerializableModifier {
            bold: true,
            italic: true,
            ..Default::default()
        };
    }

    let output = render(&backend);
    // Modifiers are combined: \x1b[1;3m (bold;italic)
    assert!(output.contains("\x1b[1;3m"));
}

#[test]
fn test_ansi_render_with_legend() {
    let mut backend = CaptureBackend::new(5, 1);

    // Set some colored cells
    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('R');
        cell.fg = SerializableColor::Red;
    }
    if let Some(cell) = backend.cell_mut(1, 0) {
        cell.set_char('G');
        cell.fg = SerializableColor::Green;
    }

    let output = render_with_legend(&backend);

    // Should contain the content
    assert!(output.contains("R"));
    assert!(output.contains("G"));

    // Should contain the legend section
    assert!(output.contains("--- Colors Used ---"));
    assert!(output.contains("FG:"));
}

#[test]
fn test_ansi_render_with_legend_includes_bg() {
    let mut backend = CaptureBackend::new(3, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('X');
        cell.bg = SerializableColor::Blue;
    }

    let output = render_with_legend(&backend);
    assert!(output.contains("BG:"));
}

#[test]
fn test_ansi_render_with_legend_no_colors() {
    let mut backend = CaptureBackend::new(3, 1);

    // Plain text, no colors
    for (i, c) in "ABC".chars().enumerate() {
        if let Some(cell) = backend.cell_mut(i as u16, 0) {
            cell.set_char(c);
        }
    }

    let output = render_with_legend(&backend);
    assert!(output.contains("ABC"));
    // No legend section if no colors used
    assert!(!output.contains("--- Colors Used ---"));
}

#[test]
fn test_ansi_render_rgb_color() {
    let mut backend = CaptureBackend::new(1, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('X');
        cell.fg = SerializableColor::Rgb {
            r: 255,
            g: 128,
            b: 64,
        };
    }

    let output = render(&backend);
    // RGB uses 38;2;R;G;B format
    assert!(output.contains("38;2;255;128;64"));
}

#[test]
fn test_ansi_render_indexed_color() {
    let mut backend = CaptureBackend::new(1, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('X');
        cell.fg = SerializableColor::Indexed(42);
    }

    let output = render(&backend);
    // Indexed uses 38;5;N format
    assert!(output.contains("38;5;42"));
}

#[test]
fn test_ansi_render_underline_modifier() {
    let mut backend = CaptureBackend::new(1, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('U');
        cell.modifiers.underlined = true;
    }

    let output = render(&backend);
    // Underline is ESC[4m
    assert!(output.contains("\x1b[4m"));
}

#[test]
fn test_ansi_render_dim_modifier() {
    let mut backend = CaptureBackend::new(1, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('D');
        cell.modifiers.dim = true;
    }

    let output = render(&backend);
    // Dim is ESC[2m
    assert!(output.contains("\x1b[2m"));
}

#[test]
fn test_ansi_render_no_reset_when_reset_colors() {
    let mut backend = CaptureBackend::new(3, 1);

    // All cells with reset colors and no modifiers
    for (i, c) in "ABC".chars().enumerate() {
        if let Some(cell) = backend.cell_mut(i as u16, 0) {
            cell.set_char(c);
            // Default is Reset colors and empty modifiers
        }
    }

    let output = render(&backend);
    // Should just be "ABC" with no ANSI codes
    assert_eq!(output, "ABC");
}

#[test]
fn test_ansi_render_bright_colors() {
    let mut backend = CaptureBackend::new(2, 1);

    if let Some(cell) = backend.cell_mut(0, 0) {
        cell.set_char('L');
        cell.fg = SerializableColor::LightRed;
    }
    if let Some(cell) = backend.cell_mut(1, 0) {
        cell.set_char('B');
        cell.bg = SerializableColor::LightBlue;
    }

    let output = render(&backend);
    // Light/bright colors use codes 90-97 (fg) and 100-107 (bg)
    assert!(output.contains("\x1b[91m")); // Light red FG
    assert!(output.contains("\x1b[104m")); // Light blue BG
}
