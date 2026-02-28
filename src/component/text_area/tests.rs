use super::*;

// State Tests

#[test]
fn test_new() {
    let state = TextAreaState::new();
    assert!(state.is_empty());
    assert_eq!(state.line_count(), 1);
    assert_eq!(state.line(0), Some(""));
    assert_eq!(state.cursor_position(), (0, 0));
}

#[test]
fn test_default() {
    let state = TextAreaState::default();
    assert!(state.is_empty());
    assert_eq!(state.line_count(), 1);
}

#[test]
fn test_with_value() {
    let state = TextAreaState::with_value("Hello\nWorld");
    assert_eq!(state.line_count(), 2);
    assert_eq!(state.line(0), Some("Hello"));
    assert_eq!(state.line(1), Some("World"));
    // Cursor at end of last line
    assert_eq!(state.cursor_position(), (1, 5));
}

#[test]
fn test_with_value_empty() {
    let state = TextAreaState::with_value("");
    assert!(state.is_empty());
    assert_eq!(state.line_count(), 1);
}

#[test]
fn test_with_placeholder() {
    let state = TextAreaState::with_placeholder("Enter text...");
    assert_eq!(state.placeholder(), "Enter text...");
    assert!(state.is_empty());
}

// Content Accessors

#[test]
fn test_value() {
    let state = TextAreaState::with_value("Line 1\nLine 2\nLine 3");
    assert_eq!(state.value(), "Line 1\nLine 2\nLine 3");
}

#[test]
fn test_set_value() {
    let mut state = TextAreaState::new();
    state.set_value("New\nContent");
    assert_eq!(state.line_count(), 2);
    assert_eq!(state.line(0), Some("New"));
    assert_eq!(state.line(1), Some("Content"));
    assert_eq!(state.cursor_position(), (1, 7));
}

#[test]
fn test_line() {
    let state = TextAreaState::with_value("a\nb\nc");
    assert_eq!(state.line(0), Some("a"));
    assert_eq!(state.line(1), Some("b"));
    assert_eq!(state.line(2), Some("c"));
    assert_eq!(state.line(3), None);
}

#[test]
fn test_current_line() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(0, 0);
    assert_eq!(state.current_line(), "Hello");
    state.set_cursor(1, 0);
    assert_eq!(state.current_line(), "World");
}

#[test]
fn test_line_count() {
    assert_eq!(TextAreaState::new().line_count(), 1);
    assert_eq!(TextAreaState::with_value("a").line_count(), 1);
    assert_eq!(TextAreaState::with_value("a\nb").line_count(), 2);
    assert_eq!(TextAreaState::with_value("a\nb\nc").line_count(), 3);
}

#[test]
fn test_is_empty() {
    assert!(TextAreaState::new().is_empty());
    assert!(!TextAreaState::with_value("a").is_empty());
    assert!(!TextAreaState::with_value("\n").is_empty()); // Two empty lines
}

// Cursor Tests

#[test]
fn test_cursor_position() {
    let state = TextAreaState::with_value("Hello\nWorld");
    assert_eq!(state.cursor_position(), (1, 5));
}

#[test]
fn test_set_cursor() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(0, 2);
    assert_eq!(state.cursor_position(), (0, 2));
}

#[test]
fn test_cursor_clamp_row() {
    let mut state = TextAreaState::with_value("Hello");
    state.set_cursor(10, 0); // Row out of bounds
    assert_eq!(state.cursor_row(), 0);
}

#[test]
fn test_cursor_clamp_col() {
    let mut state = TextAreaState::with_value("Hi");
    state.set_cursor(0, 100); // Col out of bounds
    assert_eq!(state.cursor_position(), (0, 2));
}

// Character Editing

#[test]
fn test_insert() {
    let mut state = TextArea::init();
    let output = TextArea::update(&mut state, TextAreaMessage::Insert('H'));
    assert_eq!(state.value(), "H");
    assert!(matches!(output, Some(TextAreaOutput::Changed(_))));

    TextArea::update(&mut state, TextAreaMessage::Insert('i'));
    assert_eq!(state.value(), "Hi");
}

#[test]
fn test_insert_unicode() {
    let mut state = TextArea::init();
    TextArea::update(&mut state, TextAreaMessage::Insert('日'));
    TextArea::update(&mut state, TextAreaMessage::Insert('本'));
    assert_eq!(state.value(), "日本");
    assert_eq!(state.cursor_position(), (0, 2));
}

#[test]
fn test_newline() {
    let mut state = TextAreaState::with_value("Hello");
    state.set_cursor(0, 2);
    TextArea::update(&mut state, TextAreaMessage::NewLine);
    assert_eq!(state.line_count(), 2);
    assert_eq!(state.line(0), Some("He"));
    assert_eq!(state.line(1), Some("llo"));
    assert_eq!(state.cursor_position(), (1, 0));
}

#[test]
fn test_newline_at_start() {
    let mut state = TextAreaState::with_value("Hello");
    state.set_cursor(0, 0);
    TextArea::update(&mut state, TextAreaMessage::NewLine);
    assert_eq!(state.line(0), Some(""));
    assert_eq!(state.line(1), Some("Hello"));
}

#[test]
fn test_newline_at_end() {
    let mut state = TextAreaState::with_value("Hello");
    TextArea::update(&mut state, TextAreaMessage::NewLine);
    assert_eq!(state.line(0), Some("Hello"));
    assert_eq!(state.line(1), Some(""));
}

#[test]
fn test_backspace() {
    let mut state = TextAreaState::with_value("Hello");
    let output = TextArea::update(&mut state, TextAreaMessage::Backspace);
    assert_eq!(state.value(), "Hell");
    assert!(matches!(output, Some(TextAreaOutput::Changed(_))));
}

#[test]
fn test_backspace_join_lines() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(1, 0); // Start of second line
    TextArea::update(&mut state, TextAreaMessage::Backspace);
    assert_eq!(state.value(), "HelloWorld");
    assert_eq!(state.cursor_position(), (0, 5));
}

#[test]
fn test_backspace_first_line_start() {
    let mut state = TextAreaState::with_value("Hello");
    state.set_cursor(0, 0);
    let output = TextArea::update(&mut state, TextAreaMessage::Backspace);
    assert_eq!(output, None);
    assert_eq!(state.value(), "Hello");
}

#[test]
fn test_delete() {
    let mut state = TextAreaState::with_value("Hello");
    state.set_cursor(0, 0);
    let output = TextArea::update(&mut state, TextAreaMessage::Delete);
    assert_eq!(state.value(), "ello");
    assert!(matches!(output, Some(TextAreaOutput::Changed(_))));
}

#[test]
fn test_delete_join_lines() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(0, 5); // End of first line
    TextArea::update(&mut state, TextAreaMessage::Delete);
    assert_eq!(state.value(), "HelloWorld");
}

#[test]
fn test_delete_last_line_end() {
    let mut state = TextAreaState::with_value("Hello");
    // Cursor is already at end
    let output = TextArea::update(&mut state, TextAreaMessage::Delete);
    assert_eq!(output, None);
}

// Navigation

#[test]
fn test_left() {
    let mut state = TextAreaState::with_value("Hello");
    TextArea::update(&mut state, TextAreaMessage::Left);
    assert_eq!(state.cursor_position(), (0, 4));
}

#[test]
fn test_left_wrap() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(1, 0);
    TextArea::update(&mut state, TextAreaMessage::Left);
    assert_eq!(state.cursor_position(), (0, 5)); // End of first line
}

#[test]
fn test_left_at_start() {
    let mut state = TextAreaState::with_value("Hello");
    state.set_cursor(0, 0);
    TextArea::update(&mut state, TextAreaMessage::Left);
    assert_eq!(state.cursor_position(), (0, 0)); // Stays at start
}

#[test]
fn test_right() {
    let mut state = TextAreaState::with_value("Hello");
    state.set_cursor(0, 0);
    TextArea::update(&mut state, TextAreaMessage::Right);
    assert_eq!(state.cursor_position(), (0, 1));
}

#[test]
fn test_right_wrap() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(0, 5); // End of first line
    TextArea::update(&mut state, TextAreaMessage::Right);
    assert_eq!(state.cursor_position(), (1, 0)); // Start of second line
}

#[test]
fn test_right_at_end() {
    let mut state = TextAreaState::with_value("Hello");
    // Already at end
    TextArea::update(&mut state, TextAreaMessage::Right);
    assert_eq!(state.cursor_position(), (0, 5)); // Stays at end
}

#[test]
fn test_up() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    TextArea::update(&mut state, TextAreaMessage::Up);
    assert_eq!(state.cursor_position(), (0, 5));
}

#[test]
fn test_up_clamps_column() {
    let mut state = TextAreaState::with_value("Hi\nHello");
    state.set_cursor(1, 5); // End of "Hello"
    TextArea::update(&mut state, TextAreaMessage::Up);
    assert_eq!(state.cursor_position(), (0, 2)); // Clamped to "Hi" length
}

#[test]
fn test_up_at_first_line() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(0, 2);
    TextArea::update(&mut state, TextAreaMessage::Up);
    assert_eq!(state.cursor_position(), (0, 2)); // Stays on first line
}

#[test]
fn test_down() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(0, 2);
    TextArea::update(&mut state, TextAreaMessage::Down);
    assert_eq!(state.cursor_position(), (1, 2));
}

#[test]
fn test_down_clamps_column() {
    let mut state = TextAreaState::with_value("Hello\nHi");
    state.set_cursor(0, 5); // End of "Hello"
    TextArea::update(&mut state, TextAreaMessage::Down);
    assert_eq!(state.cursor_position(), (1, 2)); // Clamped to "Hi" length
}

#[test]
fn test_down_at_last_line() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    // Already on last line
    TextArea::update(&mut state, TextAreaMessage::Down);
    assert_eq!(state.cursor_row(), 1); // Stays on last line
}

#[test]
fn test_home() {
    let mut state = TextAreaState::with_value("Hello");
    TextArea::update(&mut state, TextAreaMessage::Home);
    assert_eq!(state.cursor_position(), (0, 0));
}

#[test]
fn test_end() {
    let mut state = TextAreaState::with_value("Hello");
    state.set_cursor(0, 0);
    TextArea::update(&mut state, TextAreaMessage::End);
    assert_eq!(state.cursor_position(), (0, 5));
}

#[test]
fn test_text_start() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    TextArea::update(&mut state, TextAreaMessage::TextStart);
    assert_eq!(state.cursor_position(), (0, 0));
}

#[test]
fn test_text_end() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(0, 0);
    TextArea::update(&mut state, TextAreaMessage::TextEnd);
    assert_eq!(state.cursor_position(), (1, 5));
}

#[test]
fn test_word_left() {
    let mut state = TextAreaState::with_value("hello world");
    TextArea::update(&mut state, TextAreaMessage::WordLeft);
    assert_eq!(state.cursor_position(), (0, 6)); // Start of "world"
}

#[test]
fn test_word_right() {
    let mut state = TextAreaState::with_value("hello world");
    state.set_cursor(0, 0);
    TextArea::update(&mut state, TextAreaMessage::WordRight);
    assert_eq!(state.cursor_position(), (0, 6)); // After "hello "
}

// Line Operations

#[test]
fn test_delete_line() {
    let mut state = TextAreaState::with_value("Line 1\nLine 2\nLine 3");
    state.set_cursor(1, 0);
    TextArea::update(&mut state, TextAreaMessage::DeleteLine);
    assert_eq!(state.line_count(), 2);
    assert_eq!(state.value(), "Line 1\nLine 3");
}

#[test]
fn test_delete_line_single() {
    let mut state = TextAreaState::with_value("Hello");
    TextArea::update(&mut state, TextAreaMessage::DeleteLine);
    assert!(state.is_empty());
    assert_eq!(state.line_count(), 1);
}

#[test]
fn test_delete_to_end() {
    let mut state = TextAreaState::with_value("Hello World");
    state.set_cursor(0, 5);
    TextArea::update(&mut state, TextAreaMessage::DeleteToEnd);
    assert_eq!(state.value(), "Hello");
}

#[test]
fn test_delete_to_start() {
    let mut state = TextAreaState::with_value("Hello World");
    state.set_cursor(0, 6);
    TextArea::update(&mut state, TextAreaMessage::DeleteToStart);
    assert_eq!(state.value(), "World");
    assert_eq!(state.cursor_position(), (0, 0));
}

// Bulk Operations

#[test]
fn test_clear() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    let output = TextArea::update(&mut state, TextAreaMessage::Clear);
    assert!(state.is_empty());
    assert!(matches!(output, Some(TextAreaOutput::Changed(_))));
}

#[test]
fn test_clear_empty() {
    let mut state = TextArea::init();
    let output = TextArea::update(&mut state, TextAreaMessage::Clear);
    assert_eq!(output, None);
}

#[test]
fn test_set_value_message() {
    let mut state = TextArea::init();
    let output = TextArea::update(
        &mut state,
        TextAreaMessage::SetValue("New\nValue".to_string()),
    );
    assert_eq!(state.value(), "New\nValue");
    assert!(matches!(output, Some(TextAreaOutput::Changed(_))));
}

#[test]
fn test_set_value_same() {
    let mut state = TextAreaState::with_value("Same");
    let output = TextArea::update(&mut state, TextAreaMessage::SetValue("Same".to_string()));
    assert_eq!(output, None);
}

#[test]
fn test_submit() {
    let mut state = TextAreaState::with_value("My content");
    let output = TextArea::update(&mut state, TextAreaMessage::Submit);
    assert_eq!(
        output,
        Some(TextAreaOutput::Submitted("My content".to_string()))
    );
}

// Scroll Tests

#[test]
fn test_scroll_offset() {
    let state = TextAreaState::new();
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_ensure_cursor_visible_down() {
    let mut state = TextAreaState::with_value("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
    state.set_cursor(9, 0); // Last line
    state.ensure_cursor_visible(5);
    assert!(state.scroll_offset > 0);
    assert!(state.cursor_row >= state.scroll_offset);
    assert!(state.cursor_row < state.scroll_offset + 5);
}

#[test]
fn test_ensure_cursor_visible_up() {
    let mut state = TextAreaState::with_value("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
    state.scroll_offset = 5;
    state.set_cursor(2, 0);
    state.ensure_cursor_visible(5);
    assert_eq!(state.scroll_offset, 2);
}

#[test]
fn test_view_focused() {
    let mut state = TextAreaState::with_value("Hello");
    state.focused = true;
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_unfocused() {
    let state = TextAreaState::with_value("Hello");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_placeholder() {
    let state = TextAreaState::with_placeholder("Enter text...");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// Integration

#[test]
fn test_view_renders() {
    let state = TextAreaState::with_value("Line 1\nLine 2\nLine 3");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_full_workflow() {
    let mut state = TextAreaState::new();
    TextArea::set_focused(&mut state, true);

    // Type "Hello"
    TextArea::update(&mut state, TextAreaMessage::Insert('H'));
    TextArea::update(&mut state, TextAreaMessage::Insert('e'));
    TextArea::update(&mut state, TextAreaMessage::Insert('l'));
    TextArea::update(&mut state, TextAreaMessage::Insert('l'));
    TextArea::update(&mut state, TextAreaMessage::Insert('o'));

    // New line
    TextArea::update(&mut state, TextAreaMessage::NewLine);

    // Type "World"
    TextArea::update(&mut state, TextAreaMessage::Insert('W'));
    TextArea::update(&mut state, TextAreaMessage::Insert('o'));
    TextArea::update(&mut state, TextAreaMessage::Insert('r'));
    TextArea::update(&mut state, TextAreaMessage::Insert('l'));
    TextArea::update(&mut state, TextAreaMessage::Insert('d'));

    assert_eq!(state.value(), "Hello\nWorld");
    assert_eq!(state.line_count(), 2);

    // Navigate up
    TextArea::update(&mut state, TextAreaMessage::Up);
    assert_eq!(state.cursor_position(), (0, 5));

    // Go to start of line
    TextArea::update(&mut state, TextAreaMessage::Home);
    assert_eq!(state.cursor_position(), (0, 0));

    // Delete line
    TextArea::update(&mut state, TextAreaMessage::DeleteLine);
    assert_eq!(state.value(), "World");

    // Clear
    TextArea::update(&mut state, TextAreaMessage::Clear);
    assert!(state.is_empty());
}

#[test]
fn test_init() {
    let state = TextArea::init();
    assert!(state.is_empty());
    assert!(!state.focused);
}

#[test]
fn test_set_value_empty_string() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_value("");
    assert!(state.is_empty());
    assert_eq!(state.line_count(), 1);
    assert_eq!(state.cursor_position(), (0, 0));
}

#[test]
fn test_set_placeholder_method() {
    let mut state = TextAreaState::new();
    state.set_placeholder("Type here...");
    assert_eq!(state.placeholder(), "Type here...");
}

#[test]
fn test_cursor_col_accessor() {
    let state = TextAreaState::with_value("Hello");
    assert_eq!(state.cursor_col(), 5);
}

#[test]
fn test_word_left_at_line_start() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(1, 0); // Start of "World"
    TextArea::update(&mut state, TextAreaMessage::WordLeft);
    // Should wrap to end of previous line
    assert_eq!(state.cursor_position(), (0, 5));
}

#[test]
fn test_word_left_skip_whitespace() {
    let mut state = TextAreaState::with_value("hello   world");
    state.set_cursor(0, 8); // In the middle of spaces
    TextArea::update(&mut state, TextAreaMessage::WordLeft);
    assert!(state.cursor_col() < 8);
}

#[test]
fn test_word_right_at_line_end() {
    let mut state = TextAreaState::with_value("Hello\nWorld");
    state.set_cursor(0, 5); // End of "Hello"
    TextArea::update(&mut state, TextAreaMessage::WordRight);
    // Should wrap to start of next line
    assert_eq!(state.cursor_position(), (1, 0));
}

#[test]
fn test_word_right_skip_word() {
    let mut state = TextAreaState::with_value("abc def");
    state.set_cursor(0, 0);
    TextArea::update(&mut state, TextAreaMessage::WordRight);
    // Should skip past "abc " to start of "def"
    assert_eq!(state.cursor_position(), (0, 4));
}

#[test]
fn test_delete_line_last_line() {
    let mut state = TextAreaState::with_value("Line 1\nLine 2");
    state.set_cursor(1, 3); // On last line
    TextArea::update(&mut state, TextAreaMessage::DeleteLine);
    // Should adjust cursor_row when deleting the last line
    assert_eq!(state.line_count(), 1);
    assert_eq!(state.cursor_row(), 0);
}

#[test]
fn test_delete_line_single_empty() {
    let mut state = TextArea::init();
    // Single empty line - should return None
    let output = TextArea::update(&mut state, TextAreaMessage::DeleteLine);
    assert_eq!(output, None);
}

#[test]
fn test_delete_to_end_at_end() {
    let mut state = TextAreaState::with_value("Hello");
    // Cursor already at end
    let output = TextArea::update(&mut state, TextAreaMessage::DeleteToEnd);
    assert_eq!(output, None);
}

#[test]
fn test_delete_to_start_at_start() {
    let mut state = TextAreaState::with_value("Hello");
    state.set_cursor(0, 0);
    let output = TextArea::update(&mut state, TextAreaMessage::DeleteToStart);
    assert_eq!(output, None);
}

#[test]
fn test_view_with_scroll() {
    // Create a long content that needs scrolling
    let mut state = TextAreaState::with_value(
        "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10",
    );
    state.focused = true;
    // Small height to trigger scrolling
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_cursor_above_scroll() {
    let mut state = TextAreaState::with_value("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
    state.scroll_offset = 5; // Scroll down
    state.set_cursor(2, 0); // Cursor above scroll
    state.focused = true;
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            TextArea::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_ensure_cursor_visible_zero_lines() {
    let mut state = TextAreaState::with_value("Hello");
    state.ensure_cursor_visible(0);
    // Should not panic or change anything
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_backspace_unicode() {
    let mut state = TextAreaState::with_value("日本");
    TextArea::update(&mut state, TextAreaMessage::Backspace);
    assert_eq!(state.value(), "日");
}

#[test]
fn test_delete_unicode() {
    let mut state = TextAreaState::with_value("日本");
    state.set_cursor(0, 0);
    TextArea::update(&mut state, TextAreaMessage::Delete);
    assert_eq!(state.value(), "本");
}

#[test]
fn test_insert_emoji() {
    let mut state = TextAreaState::new();
    TextArea::update(&mut state, TextAreaMessage::Insert('\u{1F600}'));
    assert!(state.value().contains('\u{1F600}'));
}

#[test]
fn test_backspace_emoji() {
    let mut state = TextAreaState::new();
    TextArea::update(&mut state, TextAreaMessage::Insert('A'));
    TextArea::update(&mut state, TextAreaMessage::Insert('\u{1F600}'));
    assert!(state.value().contains('\u{1F600}'));

    TextArea::update(&mut state, TextAreaMessage::Backspace);
    assert!(!state.value().contains('\u{1F600}'));
    assert_eq!(state.value(), "A");
}

#[test]
fn test_combining_diacritics() {
    let mut state = TextAreaState::new();
    TextArea::update(&mut state, TextAreaMessage::Insert('e'));
    TextArea::update(&mut state, TextAreaMessage::Insert('\u{0301}'));
    assert!(state.value().contains('e'));
    assert!(state.value().contains('\u{0301}'));
}

#[test]
fn test_multiline_mixed_unicode() {
    let mut state = TextAreaState::new();
    // Type CJK on first line
    for c in "日本語".chars() {
        TextArea::update(&mut state, TextAreaMessage::Insert(c));
    }
    // New line
    TextArea::update(&mut state, TextAreaMessage::NewLine);
    // Emoji on second line
    TextArea::update(&mut state, TextAreaMessage::Insert('\u{1F600}'));
    TextArea::update(&mut state, TextAreaMessage::Insert('\u{1F389}'));

    let value = state.value();
    assert!(value.contains("日本語"));
    assert!(value.contains('\u{1F600}'));
    assert!(value.contains('\u{1F389}'));
}
