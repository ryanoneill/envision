use proptest::prelude::*;
use unicode_width::UnicodeWidthChar;

use super::chunking::{chunk_buffer, cursor_to_visual};

/// Generates a string with mixed ASCII, CJK, and emoji characters.
fn arbitrary_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            prop::char::range('\u{20}', '\u{7E}'),
            prop::char::range('\u{4E00}', '\u{4FFF}'),
        ],
        0..50,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

proptest! {
    /// Joining all chunks reproduces the original buffer exactly.
    #[test]
    fn partition_is_exact(
        buffer in arbitrary_text(),
        width in 1..20usize,
    ) {
        let chunks = chunk_buffer(&buffer, width);
        let reconstructed: String = chunks
            .iter()
            .map(|r| &buffer[r.clone()])
            .collect();
        prop_assert_eq!(reconstructed, buffer);
    }

    /// Every chunk's display width fits within the allocated width.
    /// Note: width must be >= 2 to accommodate CJK characters (display width 2).
    #[test]
    fn every_row_fits_display_width(
        buffer in arbitrary_text(),
        width in 2..20usize,
    ) {
        let chunks = chunk_buffer(&buffer, width);
        for chunk in &chunks {
            let display_width: usize = buffer[chunk.clone()]
                .chars()
                .map(|c| c.width().unwrap_or(0))
                .sum();
            prop_assert!(
                display_width <= width,
                "chunk {:?} has display width {} > {}",
                &buffer[chunk.clone()],
                display_width,
                width
            );
        }
    }

    /// For any valid cursor position, cursor_to_visual returns
    /// a (row, col) within the bounds of the rendered rectangle.
    /// Note: width must be >= 2 to accommodate CJK characters (display width 2).
    #[test]
    fn cursor_lands_inside_rendered_rect(
        buffer in arbitrary_text(),
        width in 2..20usize,
    ) {
        let chunks = chunk_buffer(&buffer, width);
        // Test every valid cursor position
        for cursor in 0..=buffer.len() {
            // Skip positions that land in the middle of a multi-byte char
            if !buffer.is_char_boundary(cursor) {
                continue;
            }
            let (row, col) = cursor_to_visual(&buffer, cursor, width);
            // Row must be within chunk count (+ possible phantom trailing row)
            prop_assert!(
                row <= chunks.len(),
                "row {} > chunks.len() {} for cursor {} in {:?}",
                row, chunks.len(), cursor, buffer
            );
            // Col must be within the display width
            prop_assert!(
                col <= width,
                "col {} > width {} for cursor {} in {:?}",
                col, width, cursor, buffer
            );
        }
    }

    /// Appending a character never changes the content of rows
    /// before the last row.
    #[test]
    fn append_never_moves_earlier_rows(
        buffer in arbitrary_text(),
        ch in prop::char::range('\u{20}', '\u{7E}'),
        width in 1..20usize,
    ) {
        let chunks_before = chunk_buffer(&buffer, width);
        let mut extended = buffer.clone();
        extended.push(ch);
        let chunks_after = chunk_buffer(&extended, width);

        // All rows before the last in the original should be unchanged
        let check_count = chunks_before.len().saturating_sub(1);
        for i in 0..check_count {
            let before_text = &buffer[chunks_before[i].clone()];
            let after_text = &extended[chunks_after[i].clone()];
            prop_assert_eq!(
                before_text, after_text,
                "row {} changed from {:?} to {:?} after appending {:?}",
                i, before_text, after_text, ch
            );
        }
    }
}
