//! Plain text output formatter.
//!
//! Renders the captured buffer as plain text without any styling.
//! This is the simplest format, ideal for text comparisons and assertions.

use crate::backend::CaptureBackend;

/// Renders the backend as plain text.
///
/// Each row is rendered on its own line. Trailing spaces on each line
/// are preserved to maintain the exact buffer representation.
pub fn render(backend: &CaptureBackend) -> String {
    let height = backend.height();
    let mut lines = Vec::with_capacity(height as usize);

    for y in 0..height {
        lines.push(backend.row_content(y));
    }

    lines.join("\n")
}

/// Renders the backend as plain text with trailing whitespace trimmed.
///
/// This is useful when you want a cleaner output for display,
/// but note that it may not exactly match the buffer contents.
pub fn render_trimmed(backend: &CaptureBackend) -> String {
    let height = backend.height();
    let mut lines = Vec::with_capacity(height as usize);

    for y in 0..height {
        lines.push(backend.row_content(y).trim_end().to_string());
    }

    // Also trim trailing empty lines
    while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_render() {
        let mut backend = CaptureBackend::new(10, 3);

        // Set some content
        for (i, c) in "Hello".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }

        let output = render(&backend);
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("Hello"));
        assert_eq!(lines[0].len(), 10); // Full width preserved
    }

    #[test]
    fn test_plain_render_trimmed() {
        let mut backend = CaptureBackend::new(10, 5);

        // Set content only in first row
        for (i, c) in "Hi".chars().enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, 0) {
                cell.set_char(c);
            }
        }

        let output = render_trimmed(&backend);
        let lines: Vec<&str> = output.lines().collect();

        // Should have trimmed trailing empty lines
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hi");
    }
}
