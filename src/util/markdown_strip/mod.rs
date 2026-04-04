//! Lightweight markdown stripping for plain-text fallback.
//!
//! When the `markdown` feature is disabled, this module strips common
//! markdown syntax so output is readable plain text rather than raw markup.

#[cfg(test)]
mod tests;

/// Strips common markdown syntax from text, returning readable plain text.
///
/// Handles: bold, italic, inline code, headings, blockquotes, unordered
/// lists, and links. Does not attempt full markdown parsing — just removes
/// the most common formatting markers.
///
/// # Examples
///
/// ```rust
/// use envision::util::strip_markdown;
///
/// assert_eq!(strip_markdown("**bold**"), "bold");
/// assert_eq!(strip_markdown("*italic*"), "italic");
/// assert_eq!(strip_markdown("`code`"), "code");
/// assert_eq!(strip_markdown("# Heading"), "Heading");
/// assert_eq!(strip_markdown("[text](url)"), "text");
/// ```
pub fn strip_markdown(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for line in input.split('\n') {
        if !result.is_empty() {
            result.push('\n');
        }
        let stripped = strip_line(line);
        result.push_str(&stripped);
    }

    result
}

/// Strips block-level markers from a single line, then inline markers.
fn strip_line(line: &str) -> String {
    let trimmed = line.trim_start();

    // Heading: # through ######
    let after_block = if trimmed.starts_with('#') {
        let rest = trimmed.trim_start_matches('#');
        rest.strip_prefix(' ').unwrap_or(rest)
    }
    // Blockquote: >
    else if let Some(rest) = trimmed.strip_prefix('>') {
        rest.strip_prefix(' ').unwrap_or(rest)
    }
    // Unordered list: - item or * item (but not ** bold)
    else if (trimmed.starts_with("- ") || trimmed.starts_with("* ")) && !trimmed.starts_with("**")
    {
        &trimmed[2..]
    } else {
        line
    };

    strip_inline(after_block)
}

/// Strips inline markdown markers from text.
fn strip_inline(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Backslash escape
        if chars[i] == '\\' && i + 1 < len {
            result.push(chars[i + 1]);
            i += 2;
            continue;
        }

        // Inline code: `code`
        if chars[i] == '`' {
            if let Some((content, end)) = parse_delimited(&chars, i, '`') {
                result.push_str(&content);
                i = end;
                continue;
            }
        }

        // Bold+italic: ***text***
        if i + 2 < len && chars[i] == '*' && chars[i + 1] == '*' && chars[i + 2] == '*' {
            if let Some((content, end)) = parse_triple_delimited(&chars, i, '*') {
                result.push_str(&strip_inline(&content));
                i = end;
                continue;
            }
        }

        // Bold: **text** or __text__
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some((content, end)) = parse_double_delimited(&chars, i, '*') {
                result.push_str(&strip_inline(&content));
                i = end;
                continue;
            }
        }
        if i + 1 < len && chars[i] == '_' && chars[i + 1] == '_' {
            if let Some((content, end)) = parse_double_delimited(&chars, i, '_') {
                result.push_str(&strip_inline(&content));
                i = end;
                continue;
            }
        }

        // Italic: *text* or _text_
        if chars[i] == '*' {
            if let Some((content, end)) = parse_delimited(&chars, i, '*') {
                result.push_str(&strip_inline(&content));
                i = end;
                continue;
            }
        }
        if chars[i] == '_' {
            if let Some((content, end)) = parse_delimited(&chars, i, '_') {
                result.push_str(&strip_inline(&content));
                i = end;
                continue;
            }
        }

        // Link: [text](url)
        if chars[i] == '[' {
            if let Some((text, end)) = parse_link(&chars, i) {
                result.push_str(&text);
                i = end;
                continue;
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Parses `*content*` or `` `content` `` starting at position `start`.
/// Returns (content, position_after_closing_delimiter).
fn parse_delimited(chars: &[char], start: usize, delim: char) -> Option<(String, usize)> {
    if chars[start] != delim {
        return None;
    }
    let mut i = start + 1;
    let mut content = String::new();
    while i < chars.len() {
        if chars[i] == delim {
            if !content.is_empty() {
                return Some((content, i + 1));
            }
            return None; // Empty delimiters like ** or ``
        }
        content.push(chars[i]);
        i += 1;
    }
    None // No closing delimiter
}

/// Parses `**content**` or `__content__` starting at position `start`.
fn parse_double_delimited(chars: &[char], start: usize, delim: char) -> Option<(String, usize)> {
    if start + 1 >= chars.len() || chars[start] != delim || chars[start + 1] != delim {
        return None;
    }
    let mut i = start + 2;
    let mut content = String::new();
    while i + 1 < chars.len() {
        if chars[i] == delim && chars[i + 1] == delim {
            if !content.is_empty() {
                return Some((content, i + 2));
            }
            return None;
        }
        content.push(chars[i]);
        i += 1;
    }
    None
}

/// Parses `***content***` starting at position `start`.
fn parse_triple_delimited(chars: &[char], start: usize, delim: char) -> Option<(String, usize)> {
    if start + 2 >= chars.len()
        || chars[start] != delim
        || chars[start + 1] != delim
        || chars[start + 2] != delim
    {
        return None;
    }
    let mut i = start + 3;
    let mut content = String::new();
    while i + 2 < chars.len() {
        if chars[i] == delim && chars[i + 1] == delim && chars[i + 2] == delim {
            if !content.is_empty() {
                return Some((content, i + 3));
            }
            return None;
        }
        content.push(chars[i]);
        i += 1;
    }
    None
}

/// Parses `[text](url)` starting at position `start`.
/// Returns (text, position_after_closing_paren).
fn parse_link(chars: &[char], start: usize) -> Option<(String, usize)> {
    if chars[start] != '[' {
        return None;
    }
    let mut i = start + 1;
    let mut text = String::new();

    // Find closing ]
    while i < chars.len() && chars[i] != ']' {
        text.push(chars[i]);
        i += 1;
    }
    if i >= chars.len() {
        return None;
    }
    i += 1; // skip ]

    // Expect (
    if i >= chars.len() || chars[i] != '(' {
        return None;
    }
    i += 1;

    // Find closing )
    while i < chars.len() && chars[i] != ')' {
        i += 1;
    }
    if i >= chars.len() {
        return None;
    }

    Some((text, i + 1))
}
