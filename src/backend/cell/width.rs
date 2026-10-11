//! The single width definition used throughout envision.

use ratatui::buffer::CellWidth;

/// Display width of `symbol` in terminal cells, as ratatui computes it.
///
/// A single ASCII control byte (reachable via `set_char`/`set_symbol`) is
/// reported as 1 — the value ratatui's release builds return — without calling
/// `CellWidth`, whose `debug_assert!` panics on such bytes in debug builds.
pub(crate) fn symbol_cell_width(symbol: &str) -> u16 {
    if symbol.len() == 1 && symbol.as_bytes()[0].is_ascii_control() {
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
    }
}
