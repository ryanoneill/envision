//! Cursor movement and text editing helpers for LineInput.
//!
//! All operations work with byte offsets into a single-line `String` buffer.

/// Moves the cursor one character to the left.
///
/// Returns the new cursor position.
pub fn move_left(buffer: &str, cursor: usize) -> usize {
    buffer[..cursor]
        .char_indices()
        .next_back()
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Moves the cursor one character to the right.
///
/// Returns the new cursor position.
pub fn move_right(buffer: &str, cursor: usize) -> usize {
    buffer[cursor..]
        .char_indices()
        .nth(1)
        .map(|(i, _)| cursor + i)
        .unwrap_or(buffer.len())
}

/// Moves the cursor to the start of the previous word.
///
/// Skips trailing whitespace, then skips the word.
pub fn move_word_left(buffer: &str, cursor: usize) -> usize {
    let before = &buffer[..cursor];

    // Skip trailing whitespace
    let trimmed = before.trim_end();
    if trimmed.is_empty() {
        return 0;
    }

    // Find start of word
    for (i, ch) in trimmed.char_indices().rev() {
        if ch.is_whitespace() {
            return i + ch.len_utf8();
        }
    }

    0
}

/// Moves the cursor to the start of the next word.
///
/// Skips current word, then skips whitespace.
pub fn move_word_right(buffer: &str, cursor: usize) -> usize {
    let after = &buffer[cursor..];

    // Skip current word characters
    let mut chars = after.char_indices().peekable();

    // Skip non-whitespace
    while let Some(&(_, ch)) = chars.peek() {
        if ch.is_whitespace() {
            break;
        }
        chars.next();
    }

    // Skip whitespace
    for (i, ch) in chars {
        if !ch.is_whitespace() {
            return cursor + i;
        }
    }

    buffer.len()
}

/// Deletes the character before the cursor (backspace).
///
/// Returns `(new_buffer, new_cursor)`, or `None` if cursor is at start.
pub fn backspace(buffer: &str, cursor: usize) -> Option<(String, usize)> {
    if cursor == 0 {
        return None;
    }
    let new_cursor = move_left(buffer, cursor);
    let mut new_buffer = String::with_capacity(buffer.len());
    new_buffer.push_str(&buffer[..new_cursor]);
    new_buffer.push_str(&buffer[cursor..]);
    Some((new_buffer, new_cursor))
}

/// Deletes the character at the cursor (delete key).
///
/// Returns `Some(new_buffer)`, or `None` if cursor is at end.
pub fn delete_at(buffer: &str, cursor: usize) -> Option<String> {
    if cursor >= buffer.len() {
        return None;
    }
    let next = move_right(buffer, cursor);
    let mut new_buffer = String::with_capacity(buffer.len());
    new_buffer.push_str(&buffer[..cursor]);
    new_buffer.push_str(&buffer[next..]);
    Some(new_buffer)
}

/// Deletes the word before the cursor.
///
/// Returns `(new_buffer, new_cursor)`, or `None` if cursor is at start.
pub fn delete_word_back(buffer: &str, cursor: usize) -> Option<(String, usize)> {
    if cursor == 0 {
        return None;
    }
    let word_start = move_word_left(buffer, cursor);
    let mut new_buffer = String::with_capacity(buffer.len());
    new_buffer.push_str(&buffer[..word_start]);
    new_buffer.push_str(&buffer[cursor..]);
    Some((new_buffer, word_start))
}

/// Deletes the word after the cursor.
///
/// Returns `Some(new_buffer)`, or `None` if cursor is at end.
pub fn delete_word_forward(buffer: &str, cursor: usize) -> Option<String> {
    if cursor >= buffer.len() {
        return None;
    }
    let word_end = move_word_right(buffer, cursor);
    let mut new_buffer = String::with_capacity(buffer.len());
    new_buffer.push_str(&buffer[..cursor]);
    new_buffer.push_str(&buffer[word_end..]);
    Some(new_buffer)
}

/// Inserts a character at the cursor position.
///
/// Returns `(new_buffer, new_cursor)`.
pub fn insert_char(buffer: &str, cursor: usize, ch: char) -> (String, usize) {
    let mut new_buffer = String::with_capacity(buffer.len() + ch.len_utf8());
    new_buffer.push_str(&buffer[..cursor]);
    new_buffer.push(ch);
    new_buffer.push_str(&buffer[cursor..]);
    let new_cursor = cursor + ch.len_utf8();
    (new_buffer, new_cursor)
}

/// Inserts a string at the cursor position, stripping newlines.
///
/// Returns `(new_buffer, new_cursor)`.
pub fn insert_str(buffer: &str, cursor: usize, text: &str) -> (String, usize) {
    let clean: String = text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
    let mut new_buffer = String::with_capacity(buffer.len() + clean.len());
    new_buffer.push_str(&buffer[..cursor]);
    new_buffer.push_str(&clean);
    new_buffer.push_str(&buffer[cursor..]);
    let new_cursor = cursor + clean.len();
    (new_buffer, new_cursor)
}

/// Returns the display width of a character.
#[cfg(test)]
fn char_display_width(ch: char) -> usize {
    unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_left() {
        assert_eq!(move_left("hello", 3), 2);
        assert_eq!(move_left("hello", 0), 0);
    }

    #[test]
    fn test_move_left_multibyte() {
        // "a世b" — 世 is 3 bytes
        assert_eq!(move_left("a世b", 4), 1); // Before 'b' to before '世'
    }

    #[test]
    fn test_move_right() {
        assert_eq!(move_right("hello", 2), 3);
        assert_eq!(move_right("hello", 5), 5);
    }

    #[test]
    fn test_move_right_multibyte() {
        assert_eq!(move_right("a世b", 1), 4); // After 'a' to after '世'
    }

    #[test]
    fn test_move_word_left() {
        assert_eq!(move_word_left("hello world", 11), 6);
        assert_eq!(move_word_left("hello world", 6), 0);
        assert_eq!(move_word_left("hello world", 0), 0);
    }

    #[test]
    fn test_move_word_left_multiple_spaces() {
        assert_eq!(move_word_left("hello   world", 13), 8);
    }

    #[test]
    fn test_move_word_right() {
        assert_eq!(move_word_right("hello world", 0), 6);
        assert_eq!(move_word_right("hello world", 6), 11);
    }

    #[test]
    fn test_move_word_right_at_end() {
        assert_eq!(move_word_right("hello", 5), 5);
    }

    #[test]
    fn test_backspace() {
        let (buf, cur) = backspace("hello", 3).unwrap();
        assert_eq!(buf, "helo");
        assert_eq!(cur, 2);
    }

    #[test]
    fn test_backspace_at_start() {
        assert!(backspace("hello", 0).is_none());
    }

    #[test]
    fn test_delete_at() {
        let buf = delete_at("hello", 2).unwrap();
        assert_eq!(buf, "helo");
    }

    #[test]
    fn test_delete_at_end() {
        assert!(delete_at("hello", 5).is_none());
    }

    #[test]
    fn test_delete_word_back() {
        let (buf, cur) = delete_word_back("hello world", 11).unwrap();
        assert_eq!(buf, "hello ");
        assert_eq!(cur, 6);
    }

    #[test]
    fn test_delete_word_forward() {
        let buf = delete_word_forward("hello world", 5).unwrap();
        assert_eq!(buf, "helloworld");
    }

    #[test]
    fn test_insert_char() {
        let (buf, cur) = insert_char("hllo", 1, 'e');
        assert_eq!(buf, "hello");
        assert_eq!(cur, 2);
    }

    #[test]
    fn test_insert_str_strips_newlines() {
        let (buf, cur) = insert_str("ab", 1, "x\ny\nz");
        assert_eq!(buf, "axyzb");
        assert_eq!(cur, 4);
    }

    #[test]
    fn test_char_display_width() {
        assert_eq!(char_display_width('a'), 1);
        assert_eq!(char_display_width('世'), 2);
    }
}
