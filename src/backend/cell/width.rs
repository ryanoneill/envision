//! The single width definition used throughout envision.

use ratatui::buffer::CellWidth;

/// Display width of `symbol` in terminal cells, as ratatui computes it.
///
/// Every single-byte symbol is width 1. That is exactly what ratatui's
/// `CellWidth for str` returns for a one-byte string
/// (`ratatui-core-0.1.2/src/buffer/cell_width.rs:34-41`), so the result is
/// identical to upstream. Short-circuiting here has two benefits:
/// - ASCII skips the cross-crate call and the Unicode width tables, which are
///   the hot path when `CaptureBackend` emulates wide glyphs;
/// - an ASCII control byte (reachable via `set_char`/`set_symbol`) never
///   reaches upstream's `debug_assert!`, which panics on control bytes in
///   debug builds.
///
/// Longer symbols use ratatui's own width, including its halfwidth katakana
/// sound-mark adjustment.
pub(crate) fn symbol_cell_width(symbol: &str) -> u16 {
    if symbol.len() == 1 {
        1
    } else {
        symbol.cell_width()
    }
}

#[cfg(test)]
mod tests {
    use super::symbol_cell_width;

    #[test]
    fn ascii_is_one() {
        assert_eq!(symbol_cell_width("a"), 1);
    }

    #[test]
    fn cjk_is_two() {
        assert_eq!(symbol_cell_width("中"), 2);
    }

    #[test]
    fn empty_is_zero() {
        assert_eq!(symbol_cell_width(""), 0);
    }

    #[test]
    fn control_byte_is_one() {
        assert_eq!(symbol_cell_width("\u{7f}"), 1);
        assert_eq!(symbol_cell_width("\0"), 1);
        assert_eq!(symbol_cell_width("\t"), 1);
    }

    #[test]
    fn halfwidth_sound_mark_follows_ratatui() {
        assert_eq!(symbol_cell_width("ﾊﾟ"), 2);
    }
}
