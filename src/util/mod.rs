//! Utility functions for TUI rendering.
//!
//! This module provides utility functions for working with text and layout
//! in TUI applications.
//!
//! # Text Utilities
//!
//! - [`truncate_to_width`]: Truncate a string to fit within a maximum display width.
//! - [`wrapped_line_count`]: Count visual lines when text is wrapped at a given width.
//!
//! # Layout Utilities
//!
//! - [`centered_rect`]: Calculate a centered rectangle within a given area.
//!
//! # Example
//!
//! ```rust
//! use envision::util::{truncate_to_width, wrapped_line_count};
//!
//! // ASCII text
//! assert_eq!(truncate_to_width("hello world", 5), "hello");
//!
//! // CJK characters are 2 columns wide
//! assert_eq!(truncate_to_width("世界", 3), "世");
//!
//! // Wrapped line counting
//! assert_eq!(wrapped_line_count("hello world", 5), 3);
//! assert_eq!(wrapped_line_count("hello\nworld", 20), 2);
//! ```

use ratatui::prelude::Rect;
use unicode_width::UnicodeWidthChar;

/// Truncates a string to fit within `max_width` display columns.
///
/// Returns the longest prefix of `s` whose display width does not exceed
/// `max_width`. The returned slice is always a valid UTF-8 substring of `s`
/// and never splits a character.
///
/// Wide characters (e.g., CJK ideographs, some emoji) count as 2 display
/// columns. If a wide character would cause the accumulated width to exceed
/// `max_width`, it is excluded entirely.
///
/// # Examples
///
/// ```rust
/// use envision::util::truncate_to_width;
///
/// // Fits entirely
/// assert_eq!(truncate_to_width("hi", 10), "hi");
///
/// // Exact fit
/// assert_eq!(truncate_to_width("hello", 5), "hello");
///
/// // Truncated
/// assert_eq!(truncate_to_width("hello world", 5), "hello");
///
/// // Empty string
/// assert_eq!(truncate_to_width("", 5), "");
///
/// // Zero width
/// assert_eq!(truncate_to_width("hello", 0), "");
///
/// // CJK characters (2 columns each)
/// assert_eq!(truncate_to_width("世界", 4), "世界");
/// assert_eq!(truncate_to_width("世界", 3), "世");
/// assert_eq!(truncate_to_width("世界", 1), "");
/// ```
pub fn truncate_to_width(s: &str, max_width: usize) -> &str {
    let mut width = 0;
    for (i, ch) in s.char_indices() {
        let ch_width = ch.width().unwrap_or(0);
        if width + ch_width > max_width {
            return &s[..i];
        }
        width += ch_width;
    }
    s
}

/// Counts the number of visual lines when `s` is wrapped at `width` columns.
///
/// This function handles both explicit newlines (`\n`) and character-level
/// wrapping within each line. Wide characters (CJK, some emoji) that would
/// exceed the line width are bumped to the next line.
///
/// # Special cases
///
/// - Returns `0` if `width` is `0` (no columns available to display text).
/// - Returns `1` for an empty string (an empty line still occupies one row).
/// - A trailing newline adds an additional line.
///
/// # Examples
///
/// ```rust
/// use envision::util::wrapped_line_count;
///
/// // Single line fits
/// assert_eq!(wrapped_line_count("hello", 10), 1);
///
/// // Empty string
/// assert_eq!(wrapped_line_count("", 10), 1);
///
/// // Explicit newlines
/// assert_eq!(wrapped_line_count("a\nb\nc", 10), 3);
///
/// // Character-level wrapping
/// assert_eq!(wrapped_line_count("hello world", 5), 3);
///
/// // Zero width
/// assert_eq!(wrapped_line_count("hello", 0), 0);
///
/// // CJK wrapping (2 columns each)
/// assert_eq!(wrapped_line_count("世界你好", 5), 2);
/// ```
pub fn wrapped_line_count(s: &str, width: usize) -> usize {
    if width == 0 {
        return 0;
    }

    let mut total_lines = 0;

    for line in s.split('\n') {
        if line.is_empty() {
            total_lines += 1;
            continue;
        }

        let mut col = 0;
        let mut line_started = false;

        for ch in line.chars() {
            let ch_width = ch.width().unwrap_or(0);

            if !line_started {
                total_lines += 1;
                line_started = true;
            }

            if ch_width == 0 {
                continue;
            }

            if col + ch_width > width {
                // Wrap to next line
                total_lines += 1;
                col = ch_width;
            } else {
                col += ch_width;
            }
        }

        if !line_started {
            // Line had only zero-width characters
            total_lines += 1;
        }
    }

    // Handle case where s is completely empty (split produces one empty item)
    if total_lines == 0 { 1 } else { total_lines }
}

/// Calculates a centered rectangle within the given area.
///
/// Returns a `Rect` of the given `width` and `height` centered within `area`.
/// The dimensions are clamped to fit within the area if they exceed it.
///
/// # Examples
///
/// ```rust
/// use ratatui::prelude::Rect;
/// use envision::util::centered_rect;
///
/// let area = Rect::new(0, 0, 80, 24);
/// let centered = centered_rect(40, 10, area);
/// assert_eq!(centered.x, 20);
/// assert_eq!(centered.y, 7);
/// assert_eq!(centered.width, 40);
/// assert_eq!(centered.height, 10);
/// ```
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

#[cfg(test)]
mod tests;
