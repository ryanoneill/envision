use super::strip_markdown;

// =============================================================================
// Bold
// =============================================================================

#[test]
fn test_bold_asterisks() {
    assert_eq!(strip_markdown("**bold**"), "bold");
}

#[test]
fn test_bold_underscores() {
    assert_eq!(strip_markdown("__bold__"), "bold");
}

#[test]
fn test_bold_in_sentence() {
    assert_eq!(strip_markdown("this is **bold** text"), "this is bold text");
}

// =============================================================================
// Italic
// =============================================================================

#[test]
fn test_italic_asterisk() {
    assert_eq!(strip_markdown("*italic*"), "italic");
}

#[test]
fn test_italic_underscore() {
    assert_eq!(strip_markdown("_italic_"), "italic");
}

#[test]
fn test_italic_in_sentence() {
    assert_eq!(
        strip_markdown("this is *italic* text"),
        "this is italic text"
    );
}

// =============================================================================
// Inline code
// =============================================================================

#[test]
fn test_inline_code() {
    assert_eq!(strip_markdown("`code`"), "code");
}

#[test]
fn test_inline_code_in_sentence() {
    assert_eq!(
        strip_markdown("use `strip_markdown()` here"),
        "use strip_markdown() here"
    );
}

// =============================================================================
// Headings
// =============================================================================

#[test]
fn test_heading_h1() {
    assert_eq!(strip_markdown("# Heading"), "Heading");
}

#[test]
fn test_heading_h2() {
    assert_eq!(strip_markdown("## Subheading"), "Subheading");
}

#[test]
fn test_heading_h3() {
    assert_eq!(strip_markdown("### Third"), "Third");
}

// =============================================================================
// Blockquotes
// =============================================================================

#[test]
fn test_blockquote() {
    assert_eq!(strip_markdown("> quoted text"), "quoted text");
}

#[test]
fn test_blockquote_no_space() {
    assert_eq!(strip_markdown(">quoted"), "quoted");
}

// =============================================================================
// Lists
// =============================================================================

#[test]
fn test_unordered_list_dash() {
    assert_eq!(strip_markdown("- item one"), "item one");
}

#[test]
fn test_unordered_list_asterisk() {
    assert_eq!(strip_markdown("* item two"), "item two");
}

// =============================================================================
// Links
// =============================================================================

#[test]
fn test_link() {
    assert_eq!(
        strip_markdown("[click here](https://example.com)"),
        "click here"
    );
}

#[test]
fn test_link_in_sentence() {
    assert_eq!(
        strip_markdown("see [docs](https://docs.rs) for info"),
        "see docs for info"
    );
}

// =============================================================================
// Nested / combined
// =============================================================================

#[test]
fn test_bold_italic() {
    assert_eq!(strip_markdown("***bold italic***"), "bold italic");
}

#[test]
fn test_bold_in_heading() {
    assert_eq!(strip_markdown("# **Bold Heading**"), "Bold Heading");
}

#[test]
fn test_multiple_inline() {
    assert_eq!(
        strip_markdown("**bold** and *italic* and `code`"),
        "bold and italic and code"
    );
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_input() {
    assert_eq!(strip_markdown(""), "");
}

#[test]
fn test_no_markdown() {
    assert_eq!(strip_markdown("plain text"), "plain text");
}

#[test]
fn test_unmatched_bold() {
    assert_eq!(strip_markdown("**unmatched"), "**unmatched");
}

#[test]
fn test_unmatched_italic() {
    assert_eq!(strip_markdown("*unmatched"), "*unmatched");
}

#[test]
fn test_escaped_marker() {
    assert_eq!(strip_markdown("\\*not italic\\*"), "*not italic*");
}

#[test]
fn test_multiline() {
    let input = "# Title\n\n**bold** paragraph\n\n- list item";
    let expected = "Title\n\nbold paragraph\n\nlist item";
    assert_eq!(strip_markdown(input), expected);
}

#[test]
fn test_link_without_url_parens() {
    // Incomplete link syntax should be left as-is
    assert_eq!(strip_markdown("[text]"), "[text]");
}

#[test]
fn test_empty_bold() {
    // Empty bold markers should be left as-is
    assert_eq!(strip_markdown("****"), "****");
}

#[test]
fn test_single_asterisk_not_stripped() {
    // A lone * without a closing pair shouldn't be stripped
    assert_eq!(strip_markdown("5 * 3 = 15"), "5 * 3 = 15");
}
