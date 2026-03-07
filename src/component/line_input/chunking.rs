//! Character-level chunking for visual line wrapping.
//!
//! Splits a single-line buffer into visual rows of at most `width` display
//! columns. Wide characters (CJK, emoji) that don't fit at the end of a row
//! are bumped to the next row.

use std::ops::Range;

use unicode_width::UnicodeWidthChar;

/// Splits `buffer` into byte-range chunks, each fitting within `width` display columns.
///
/// Returns `Vec<Range<usize>>` where each range indexes into the original buffer.
/// If the buffer is empty, returns a single empty range `0..0`.
/// If `width` is 0, returns a single empty range `0..0`.
pub fn chunk_buffer(buffer: &str, width: usize) -> Vec<Range<usize>> {
    if width == 0 || buffer.is_empty() {
        let empty: Range<usize> = 0..0;
        return vec![empty];
    }

    let mut chunks = Vec::new();
    let mut row_start = 0;
    let mut col = 0;

    for (i, ch) in buffer.char_indices() {
        let ch_width = ch.width().unwrap_or(0);

        if ch_width > 0 && col + ch_width > width {
            // Current char doesn't fit — start a new row
            chunks.push(row_start..i);
            row_start = i;
            col = ch_width;
        } else {
            col += ch_width;
        }
    }

    // Push the final chunk
    chunks.push(row_start..buffer.len());

    chunks
}

/// Converts a cursor byte offset into a (row, col) visual position.
///
/// `row` is 0-indexed into the chunk list. `col` is the display-column
/// offset within that chunk.
pub fn cursor_to_visual(buffer: &str, cursor: usize, width: usize) -> (usize, usize) {
    let chunks = chunk_buffer(buffer, width);

    // Handle phantom trailing row: if the buffer fills the last row exactly,
    // and cursor is at the end, it lands on a new (empty) row.
    if cursor == buffer.len() && width > 0 {
        let last = chunks.last().unwrap();
        let last_width: usize = buffer[last.clone()]
            .chars()
            .map(|c| c.width().unwrap_or(0))
            .sum();
        if last_width == width && !buffer.is_empty() {
            // Cursor is on the phantom trailing row
            return (chunks.len(), 0);
        }
    }

    for (row, chunk) in chunks.iter().enumerate() {
        // Use strict < for end so that a cursor at a chunk boundary
        // falls through to the next chunk (wraps to the next row).
        // The last chunk uses <= to capture cursor == buffer.len().
        let in_chunk = if row == chunks.len() - 1 {
            cursor >= chunk.start && cursor <= chunk.end
        } else {
            cursor >= chunk.start && cursor < chunk.end
        };
        if in_chunk {
            let col: usize = buffer[chunk.start..cursor]
                .chars()
                .map(|c| c.width().unwrap_or(0))
                .sum();
            return (row, col);
        }
    }

    // Fallback: cursor at end of last chunk
    let last = chunks.len().saturating_sub(1);
    let chunk = &chunks[last];
    let col: usize = buffer[chunk.start..cursor.min(chunk.end)]
        .chars()
        .map(|c| c.width().unwrap_or(0))
        .sum();
    (last, col)
}

/// Converts a (row, col) visual position back to a cursor byte offset.
///
/// If `row` exceeds the number of chunks, returns `buffer.len()`.
/// If `col` exceeds the display width of the row, returns the end of that row.
pub fn visual_to_cursor(buffer: &str, row: usize, col: usize, width: usize) -> usize {
    let chunks = chunk_buffer(buffer, width);

    // Handle phantom trailing row
    if row >= chunks.len() {
        return buffer.len();
    }

    let chunk = &chunks[row];
    let mut accumulated = 0;

    for (i, ch) in buffer[chunk.clone()].char_indices() {
        let ch_width = ch.width().unwrap_or(0);
        if accumulated + ch_width > col {
            // Snap to closest character boundary
            if col.saturating_sub(accumulated) > ch_width / 2 {
                // Closer to after the character
                return chunk.start + i + ch.len_utf8();
            }
            return chunk.start + i;
        }
        accumulated += ch_width;
    }

    chunk.end
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- chunk_buffer tests ----

    #[test]
    fn test_chunk_empty() {
        assert_eq!(chunk_buffer("", 10), vec![0..0]);
    }

    #[test]
    fn test_chunk_zero_width() {
        assert_eq!(chunk_buffer("hello", 0), vec![0..0]);
    }

    #[test]
    fn test_chunk_fits() {
        assert_eq!(chunk_buffer("hello", 10), vec![0..5]);
    }

    #[test]
    fn test_chunk_exact_fit() {
        assert_eq!(chunk_buffer("hello", 5), vec![0..5]);
    }

    #[test]
    fn test_chunk_wraps() {
        // "hello world" (11 chars), width 5
        // "hello" (5), " worl" (5), "d" (1)
        let chunks = chunk_buffer("hello world", 5);
        assert_eq!(chunks.len(), 3);
        assert_eq!(&"hello world"[chunks[0].clone()], "hello");
        assert_eq!(&"hello world"[chunks[1].clone()], " worl");
        assert_eq!(&"hello world"[chunks[2].clone()], "d");
    }

    #[test]
    fn test_chunk_cjk_bump() {
        // "a世b" = 1 + 2 + 1 = 4 cols, width 2
        // "a" (1 col) + "世" would be 3, doesn't fit → "a" then "世" then "b"
        let chunks = chunk_buffer("a世b", 2);
        assert_eq!(chunks.len(), 3);
        assert_eq!(&"a世b"[chunks[0].clone()], "a");
        assert_eq!(&"a世b"[chunks[1].clone()], "世");
        assert_eq!(&"a世b"[chunks[2].clone()], "b");
    }

    #[test]
    fn test_chunk_cjk_exact() {
        // "世界" = 4 cols, width 4
        let chunks = chunk_buffer("世界", 4);
        assert_eq!(chunks.len(), 1);
        assert_eq!(&"世界"[chunks[0].clone()], "世界");
    }

    #[test]
    fn test_chunk_single_char_per_row() {
        let chunks = chunk_buffer("abc", 1);
        assert_eq!(chunks.len(), 3);
        assert_eq!(&"abc"[chunks[0].clone()], "a");
        assert_eq!(&"abc"[chunks[1].clone()], "b");
        assert_eq!(&"abc"[chunks[2].clone()], "c");
    }

    // ---- cursor_to_visual tests ----

    #[test]
    fn test_cursor_visual_start() {
        assert_eq!(cursor_to_visual("hello", 0, 10), (0, 0));
    }

    #[test]
    fn test_cursor_visual_end() {
        assert_eq!(cursor_to_visual("hello", 5, 10), (0, 5));
    }

    #[test]
    fn test_cursor_visual_wrapped() {
        // "hello world" width 5: "hello" | " worl" | "d"
        assert_eq!(cursor_to_visual("hello world", 5, 5), (1, 0));
        assert_eq!(cursor_to_visual("hello world", 6, 5), (1, 1));
    }

    #[test]
    fn test_cursor_visual_empty() {
        assert_eq!(cursor_to_visual("", 0, 10), (0, 0));
    }

    #[test]
    fn test_cursor_visual_phantom_row() {
        // "hello" width 5: exactly fills one row, cursor at end lands on phantom row
        assert_eq!(cursor_to_visual("hello", 5, 5), (1, 0));
    }

    // ---- visual_to_cursor tests ----

    #[test]
    fn test_visual_to_cursor_start() {
        assert_eq!(visual_to_cursor("hello", 0, 0, 10), 0);
    }

    #[test]
    fn test_visual_to_cursor_end() {
        assert_eq!(visual_to_cursor("hello", 0, 5, 10), 5);
    }

    #[test]
    fn test_visual_to_cursor_wrapped() {
        // "hello world" width 5
        assert_eq!(visual_to_cursor("hello world", 1, 0, 5), 5);
    }

    #[test]
    fn test_visual_to_cursor_beyond_rows() {
        assert_eq!(visual_to_cursor("hello", 5, 0, 10), 5);
    }

    // ---- partition_is_exact property ----

    #[test]
    fn test_partition_is_exact() {
        let buffer = "hello world";
        let chunks = chunk_buffer(buffer, 5);
        let reconstructed: String = chunks.iter().map(|r| &buffer[r.clone()]).collect();
        assert_eq!(reconstructed, buffer);
    }

    #[test]
    fn test_partition_is_exact_cjk() {
        let buffer = "世界你好";
        let chunks = chunk_buffer(buffer, 3);
        let reconstructed: String = chunks.iter().map(|r| &buffer[r.clone()]).collect();
        assert_eq!(reconstructed, buffer);
    }
}
