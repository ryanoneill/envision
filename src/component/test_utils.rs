//! Test utilities for component rendering tests.

use crate::backend::CaptureBackend;
use crate::theme::Theme;
use ratatui::{Frame, Terminal};
use std::fmt::Write;

/// Creates a terminal and theme for rendering tests.
///
/// Returns a `(Terminal<CaptureBackend>, Theme)` tuple configured
/// with the given dimensions and the default theme.
///
/// # Example
///
/// ```rust
/// use envision::component::test_utils::setup_render;
///
/// let (mut terminal, theme) = setup_render(80, 24);
/// assert_eq!(terminal.backend().width(), 80);
/// ```
pub fn setup_render(width: u16, height: u16) -> (Terminal<CaptureBackend>, Theme) {
    let backend = CaptureBackend::new(width, height);
    let terminal = Terminal::new(backend).unwrap();
    (terminal, Theme::default())
}

/// Renders a component and returns the output as a string.
///
/// This is a convenience wrapper around [`setup_render`] that creates a terminal,
/// invokes the provided draw closure, and returns the backend's plain text output.
///
/// # Example
///
/// ```rust,ignore
/// use envision::component::test_utils::render_to_string;
///
/// let output = render_to_string(40, 5, |frame| {
///     // draw your component here
/// });
/// assert!(!output.is_empty());
/// ```
pub fn render_to_string<F>(width: u16, height: u16, draw: F) -> String
where
    F: FnOnce(&mut Frame),
{
    let (mut terminal, _theme) = setup_render(width, height);
    terminal.draw(draw).unwrap();
    terminal.backend().to_string()
}

/// Compares rendered terminal output against an expected string.
///
/// Returns `None` if the actual output matches `expected` exactly,
/// or `Some(diff)` with a human-readable, line-by-line diff showing
/// which lines differ.
///
/// # Example
///
/// ```rust,ignore
/// use envision::component::test_utils::{setup_render, render_diff};
///
/// let (mut terminal, theme) = setup_render(40, 5);
/// terminal.draw(|frame| {
///     // draw your component here
/// }).unwrap();
/// let diff = render_diff(terminal.backend(), "expected output here");
/// assert!(diff.is_none(), "Visual regression:\n{}", diff.unwrap());
/// ```
pub fn render_diff(backend: &CaptureBackend, expected: &str) -> Option<String> {
    let actual = backend.to_string();
    if actual == expected {
        return None;
    }

    let actual_lines: Vec<&str> = actual.split('\n').collect();
    let expected_lines: Vec<&str> = expected.split('\n').collect();
    let max_lines = actual_lines.len().max(expected_lines.len());

    let mut differing_count = 0;
    for i in 0..max_lines {
        let actual_line = actual_lines.get(i).copied();
        let expected_line = expected_lines.get(i).copied();
        if actual_line != expected_line {
            differing_count += 1;
        }
    }

    let mut output = String::new();
    let noun = if differing_count == 1 {
        "line"
    } else {
        "lines"
    };
    writeln!(
        output,
        "Visual diff ({differing_count} {noun} differ, {max_lines} total):"
    )
    .unwrap();

    for i in 0..max_lines {
        let line_num = i + 1;
        let actual_line = actual_lines.get(i).copied();
        let expected_line = expected_lines.get(i).copied();

        match (expected_line, actual_line) {
            (Some(exp), Some(act)) if exp == act => {
                writeln!(output, "  Line {line_num}: OK").unwrap();
            }
            (Some(exp), Some(act)) => {
                writeln!(output, "- Line {line_num}: {exp:?}").unwrap();
                writeln!(output, "+ Line {line_num}: {act:?}").unwrap();
            }
            (Some(exp), None) => {
                writeln!(output, "- Line {line_num}: {exp:?}").unwrap();
                writeln!(output, "+ Line {line_num}: <missing>").unwrap();
            }
            (None, Some(act)) => {
                writeln!(output, "- Line {line_num}: <missing>").unwrap();
                writeln!(output, "+ Line {line_num}: {act:?}").unwrap();
            }
            (None, None) => unreachable!(),
        }
    }

    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_diff_returns_none_when_matching() {
        let backend = CaptureBackend::new(5, 2);
        let expected = backend.to_string();
        assert!(render_diff(&backend, &expected).is_none());
    }

    #[test]
    fn render_diff_returns_some_when_content_differs() {
        let mut backend = CaptureBackend::new(5, 1);
        for (i, c) in "Hello".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }
        let diff = render_diff(&backend, "World");
        assert!(diff.is_some());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("1 line differ"));
        assert!(diff_text.contains("- Line 1:"));
        assert!(diff_text.contains("+ Line 1:"));
    }

    #[test]
    fn render_diff_handles_different_line_counts_actual_longer() {
        let mut backend = CaptureBackend::new(3, 2);
        for (i, c) in "abc".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }
        for (i, c) in "def".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 1) {
                cell.set_char(c);
            }
        }

        // Expected has only one line
        let diff = render_diff(&backend, "abc");
        assert!(diff.is_some());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("2 total"));
        assert!(diff_text.contains("Line 1: OK"));
        assert!(diff_text.contains("- Line 2: <missing>"));
        assert!(diff_text.contains("+ Line 2:"));
    }

    #[test]
    fn render_diff_handles_different_line_counts_expected_longer() {
        let backend = CaptureBackend::new(3, 1);
        let expected = "   \nExtra line";

        let diff = render_diff(&backend, expected);
        assert!(diff.is_some());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("2 total"));
        assert!(diff_text.contains("Line 1: OK"));
        assert!(diff_text.contains("- Line 2:"));
        assert!(diff_text.contains("+ Line 2: <missing>"));
    }

    #[test]
    fn render_diff_handles_empty_expected() {
        let backend = CaptureBackend::new(3, 1);
        let diff = render_diff(&backend, "");
        assert!(diff.is_some());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("1 line differ"));
    }

    #[test]
    fn render_diff_handles_empty_backend() {
        let backend = CaptureBackend::new(0, 0);
        let diff = render_diff(&backend, "");
        assert!(diff.is_none());
    }

    #[test]
    fn render_diff_detects_trailing_whitespace_differences() {
        let mut backend = CaptureBackend::new(5, 1);
        // Backend will have "Hi   " (padded to width 5)
        for (i, c) in "Hi".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }
        // Expected without trailing spaces
        let diff = render_diff(&backend, "Hi");
        assert!(diff.is_some());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("1 line differ"));
        // The quoted strings should show the whitespace difference
        assert!(diff_text.contains("\"Hi\""));
        assert!(diff_text.contains("\"Hi   \""));
    }

    #[test]
    fn render_diff_shows_plural_lines_in_header() {
        let mut backend = CaptureBackend::new(3, 3);
        for (i, c) in "abc".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }
        let diff = render_diff(&backend, "xxx\nyyy\nzzz");
        assert!(diff.is_some());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("3 lines differ"));
    }

    #[test]
    fn render_diff_shows_singular_line_in_header() {
        let mut backend = CaptureBackend::new(3, 2);
        for (i, c) in "abc".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }
        // First line matches, second line differs
        let diff = render_diff(&backend, "abc\nxxx");
        assert!(diff.is_some());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("1 line differ"));
    }

    #[test]
    fn render_to_string_returns_backend_output() {
        let output = render_to_string(5, 2, |_frame| {
            // Empty draw, should still produce default buffer content
        });
        assert!(!output.is_empty());
        // Output should have 2 lines
        let lines: Vec<&str> = output.split('\n').collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn render_to_string_dimensions_match() {
        let output = render_to_string(10, 3, |_frame| {});
        let lines: Vec<&str> = output.split('\n').collect();
        assert_eq!(lines.len(), 3);
        // Each line should be 10 characters (spaces for empty buffer)
        for line in &lines {
            assert_eq!(line.len(), 10);
        }
    }
}
