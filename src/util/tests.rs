use super::*;

// ---- truncate_to_width tests ----

#[test]
fn test_truncate_fits_entirely() {
    assert_eq!(truncate_to_width("hi", 10), "hi");
}

#[test]
fn test_truncate_exact_fit() {
    assert_eq!(truncate_to_width("hello", 5), "hello");
}

#[test]
fn test_truncate_longer_than_max() {
    assert_eq!(truncate_to_width("hello world", 5), "hello");
}

#[test]
fn test_truncate_empty_string() {
    assert_eq!(truncate_to_width("", 5), "");
}

#[test]
fn test_truncate_zero_width() {
    assert_eq!(truncate_to_width("hello", 0), "");
}

#[test]
fn test_truncate_cjk_full_fit() {
    // Each CJK char is 2 columns: "\u{4e16}\u{754c}" = "世界"
    assert_eq!(truncate_to_width("世界", 4), "世界");
}

#[test]
fn test_truncate_cjk_partial() {
    // 3 columns can fit one CJK char (2 cols) but not two (4 cols)
    assert_eq!(truncate_to_width("世界", 3), "世");
}

#[test]
fn test_truncate_cjk_too_narrow() {
    // 1 column can't fit a 2-column CJK char
    assert_eq!(truncate_to_width("世界", 1), "");
}

#[test]
fn test_truncate_mixed_ascii_cjk() {
    // "a世b" = 1 + 2 + 1 = 4 columns
    assert_eq!(truncate_to_width("a世b", 4), "a世b");
    assert_eq!(truncate_to_width("a世b", 3), "a世");
    assert_eq!(truncate_to_width("a世b", 2), "a");
}

#[test]
fn test_truncate_boundary_cjk() {
    // Exactly 2 columns for a single CJK char
    assert_eq!(truncate_to_width("世", 2), "世");
    assert_eq!(truncate_to_width("世", 1), "");
}

#[test]
fn test_truncate_emoji() {
    // Simple emoji (typically 2 columns wide)
    let result = truncate_to_width("😀hello", 3);
    // Emoji is 2 cols, 'h' is 1 col = 3 total
    assert_eq!(result, "😀h");
}

// ---- wrapped_line_count tests ----

#[test]
fn test_wrapped_single_line_fits() {
    assert_eq!(wrapped_line_count("hello", 10), 1);
}

#[test]
fn test_wrapped_empty_string() {
    assert_eq!(wrapped_line_count("", 10), 1);
}

#[test]
fn test_wrapped_explicit_newlines() {
    assert_eq!(wrapped_line_count("a\nb\nc", 10), 3);
}

#[test]
fn test_wrapped_trailing_newline() {
    assert_eq!(wrapped_line_count("a\n", 10), 2);
}

#[test]
fn test_wrapped_character_wrapping() {
    // "hello world" = 11 chars, width 5
    // "hello" (5), " worl" (5), "d" (1) = 3 lines
    assert_eq!(wrapped_line_count("hello world", 5), 3);
}

#[test]
fn test_wrapped_exact_width() {
    // "hello" is exactly 5 chars, fits in width 5
    assert_eq!(wrapped_line_count("hello", 5), 1);
}

#[test]
fn test_wrapped_zero_width() {
    assert_eq!(wrapped_line_count("hello", 0), 0);
}

#[test]
fn test_wrapped_zero_width_empty() {
    assert_eq!(wrapped_line_count("", 0), 0);
}

#[test]
fn test_wrapped_cjk_wrapping() {
    // "世界你好" = 4 CJK chars, each 2 cols = 8 cols total
    // width 5: "世界" (4 cols), "你好" (4 cols) = 2 lines
    assert_eq!(wrapped_line_count("世界你好", 5), 2);
}

#[test]
fn test_wrapped_cjk_bump_to_next_line() {
    // "ab世" = 1 + 1 + 2 = 4 cols
    // width 3: "ab" fits (2 cols), "世" doesn't fit (2+2=4>3), bumps to next line
    assert_eq!(wrapped_line_count("ab世", 3), 2);
}

#[test]
fn test_wrapped_multiple_wraps() {
    // "abcdefghij" = 10 chars, width 3
    // "abc" (3), "def" (3), "ghi" (3), "j" (1) = 4 lines
    assert_eq!(wrapped_line_count("abcdefghij", 3), 4);
}

#[test]
fn test_wrapped_mixed_newlines_and_wrapping() {
    // "abcdef\nghij" at width 4
    // Line 1: "abcdef" wraps to "abcd" (4) + "ef" (2) = 2 visual lines
    // Line 2: "ghij" fits = 1 visual line
    // Total = 3
    assert_eq!(wrapped_line_count("abcdef\nghij", 4), 3);
}

#[test]
fn test_wrapped_width_one() {
    // Each character becomes its own line
    assert_eq!(wrapped_line_count("abc", 1), 3);
}

#[test]
fn test_wrapped_only_newlines() {
    assert_eq!(wrapped_line_count("\n\n", 10), 3);
}

#[test]
fn test_wrapped_single_char() {
    assert_eq!(wrapped_line_count("a", 1), 1);
    assert_eq!(wrapped_line_count("a", 10), 1);
}

#[test]
fn test_wrapped_long_cjk_line() {
    // "世界世界世界" = 6 CJK chars = 12 cols, width 4
    // "世界" (4), "世界" (4), "世界" (4) = 3 lines
    assert_eq!(wrapped_line_count("世界世界世界", 4), 3);
}

#[test]
fn test_wrapped_cjk_odd_width() {
    // width 3: each CJK char is 2 cols, so only 1 fits per line (2 <= 3)
    // "世界" = 2 CJK chars = 2 lines
    assert_eq!(wrapped_line_count("世界", 3), 2);
}
