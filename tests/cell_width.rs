//! symbol_width() uses ratatui's width, and never panics on control bytes.
use envision::EnhancedCell;

#[test]
fn halfwidth_katakana_with_sound_mark_is_two_wide() {
    assert_eq!(EnhancedCell::with_symbol("ﾊﾟ").symbol_width(), 2);
}

#[test]
fn control_bytes_are_one_wide_without_debug_panic() {
    for c in ['\n', '\t', '\x1b'] {
        let mut cell = EnhancedCell::new();
        cell.set_char(c);
        assert_eq!(cell.symbol_width(), 1, "{c:?}");
    }
}
