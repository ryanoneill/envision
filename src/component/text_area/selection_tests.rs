use super::*;

fn focused_state(value: &str) -> TextAreaState {
    TextAreaState::new().with_value(value)
}

// =============================================================================
// Selection model tests
// =============================================================================

#[test]
fn test_no_selection_by_default() {
    let state = TextAreaState::new().with_value("hello");
    assert!(!state.has_selection());
    assert_eq!(state.selected_text(), None);
}

#[test]
fn test_select_left() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::SelectLeft);
    assert!(state.has_selection());
    assert_eq!(state.selected_text(), Some("o".to_string()));
}

#[test]
fn test_select_right() {
    let mut state = TextAreaState::new().with_value("hello");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::SelectRight);
    assert_eq!(state.selected_text(), Some("h".to_string()));
}

#[test]
fn test_select_home() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::SelectHome);
    assert_eq!(state.selected_text(), Some("hello".to_string()));
}

#[test]
fn test_select_end() {
    let mut state = TextAreaState::new().with_value("hello");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::SelectEnd);
    assert_eq!(state.selected_text(), Some("hello".to_string()));
}

#[test]
fn test_select_up() {
    let mut state = TextAreaState::new().with_value("line1\nline2");
    // Cursor at end of line2
    TextArea::update(&mut state, TextAreaMessage::SelectUp);
    assert!(state.has_selection());
    // Selected from (1, end) to (0, same_col)
    assert!(state.selected_text().is_some());
}

#[test]
fn test_select_down() {
    let mut state = TextAreaState::new().with_value("line1\nline2");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::SelectDown);
    assert!(state.has_selection());
}

#[test]
fn test_select_word_left() {
    let mut state = TextAreaState::new().with_value("hello world");
    TextArea::update(&mut state, TextAreaMessage::SelectWordLeft);
    assert_eq!(state.selected_text(), Some("world".to_string()));
}

#[test]
fn test_select_word_right() {
    let mut state = TextAreaState::new().with_value("hello world");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::SelectWordRight);
    assert_eq!(state.selected_text(), Some("hello ".to_string()));
}

#[test]
fn test_select_all() {
    let mut state = TextAreaState::new().with_value("line1\nline2");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    assert_eq!(state.selected_text(), Some("line1\nline2".to_string()));
}

#[test]
fn test_select_all_empty() {
    let mut state = TextAreaState::new();
    let output = TextArea::update(&mut state, TextAreaMessage::SelectAll);
    assert_eq!(output, None);
    assert!(!state.has_selection());
}

// =============================================================================
// Multi-line selection tests
// =============================================================================

#[test]
fn test_multiline_selection() {
    let mut state = TextAreaState::new().with_value("abc\ndef\nghi");
    state.set_cursor_position(0, 0);
    // Select from start to end of line 1
    TextArea::update(&mut state, TextAreaMessage::SelectDown);
    TextArea::update(&mut state, TextAreaMessage::SelectEnd);
    let text = state.selected_text().unwrap();
    assert!(text.contains("abc"));
    assert!(text.contains("def"));
}

#[test]
fn test_select_across_lines() {
    let mut state = TextAreaState::new().with_value("abc\ndef");
    state.set_cursor_position(0, 1); // After 'a'
    // Select from (0,1) to (1,2) using SelectDown then SelectRight
    TextArea::update(&mut state, TextAreaMessage::SelectDown);
    TextArea::update(&mut state, TextAreaMessage::SelectRight);
    let text = state.selected_text().unwrap();
    assert_eq!(text, "bc\nde");
}

// =============================================================================
// Selection clearing tests
// =============================================================================

#[test]
fn test_left_clears_selection() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::SelectLeft);
    assert!(state.has_selection());

    TextArea::update(&mut state, TextAreaMessage::Left);
    assert!(!state.has_selection());
}

#[test]
fn test_right_clears_selection() {
    let mut state = TextAreaState::new().with_value("hello");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::SelectRight);
    assert!(state.has_selection());

    TextArea::update(&mut state, TextAreaMessage::Right);
    assert!(!state.has_selection());
}

#[test]
fn test_up_clears_selection() {
    let mut state = TextAreaState::new().with_value("a\nb");
    TextArea::update(&mut state, TextAreaMessage::SelectUp);
    assert!(state.has_selection());

    TextArea::update(&mut state, TextAreaMessage::Up);
    assert!(!state.has_selection());
}

// =============================================================================
// Selection + editing tests
// =============================================================================

#[test]
fn test_insert_replaces_selection() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    TextArea::update(&mut state, TextAreaMessage::Insert('X'));
    assert_eq!(state.value(), "X");
}

#[test]
fn test_backspace_deletes_selection() {
    let mut state = TextAreaState::new().with_value("hello world");
    // Select "world"
    for _ in 0..5 {
        TextArea::update(&mut state, TextAreaMessage::SelectLeft);
    }
    TextArea::update(&mut state, TextAreaMessage::Backspace);
    assert_eq!(state.value(), "hello ");
}

#[test]
fn test_delete_deletes_selection() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    TextArea::update(&mut state, TextAreaMessage::Delete);
    assert_eq!(state.value(), "");
}

#[test]
fn test_multiline_delete_selection() {
    let mut state = TextAreaState::new().with_value("abc\ndef\nghi");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    TextArea::update(&mut state, TextAreaMessage::Delete);
    assert_eq!(state.value(), "");
    assert_eq!(state.line_count(), 1);
}

#[test]
fn test_newline_replaces_selection() {
    let mut state = TextAreaState::new().with_value("hello world");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    TextArea::update(&mut state, TextAreaMessage::NewLine);
    assert_eq!(state.value(), "\n");
}

// =============================================================================
// Copy/Cut/Paste tests
// =============================================================================

#[test]
fn test_copy() {
    let mut state = TextAreaState::new().with_value("hello world");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    let output = TextArea::update(&mut state, TextAreaMessage::Copy);
    assert_eq!(output, Some(TextAreaOutput::Copied("hello world".into())));
    assert_eq!(state.clipboard(), "hello world");
    assert_eq!(state.value(), "hello world"); // Unchanged
}

#[test]
fn test_copy_without_selection() {
    let mut state = TextAreaState::new().with_value("hello");
    let output = TextArea::update(&mut state, TextAreaMessage::Copy);
    assert_eq!(output, None);
}

#[test]
fn test_cut() {
    let mut state = TextAreaState::new().with_value("hello world");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    let output = TextArea::update(&mut state, TextAreaMessage::Cut);
    assert_eq!(output, Some(TextAreaOutput::Changed(String::new())));
    assert_eq!(state.clipboard(), "hello world");
    assert_eq!(state.value(), "");
}

#[test]
fn test_cut_without_selection() {
    let mut state = TextAreaState::new().with_value("hello");
    let output = TextArea::update(&mut state, TextAreaMessage::Cut);
    assert_eq!(output, None);
}

#[test]
fn test_paste() {
    let mut state = TextAreaState::new().with_value("hello");
    let output = TextArea::update(&mut state, TextAreaMessage::Paste(" world".into()));
    assert_eq!(state.value(), "hello world");
    assert!(output.is_some());
}

#[test]
fn test_paste_replaces_selection() {
    let mut state = TextAreaState::new().with_value("hello world");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    TextArea::update(&mut state, TextAreaMessage::Paste("goodbye".into()));
    assert_eq!(state.value(), "goodbye");
}

#[test]
fn test_paste_multiline() {
    let mut state = TextAreaState::new();
    TextArea::update(
        &mut state,
        TextAreaMessage::Paste("line1\nline2\nline3".into()),
    );
    assert_eq!(state.value(), "line1\nline2\nline3");
    assert_eq!(state.line_count(), 3);
}

#[test]
fn test_paste_empty() {
    let mut state = TextAreaState::new().with_value("hello");
    let output = TextArea::update(&mut state, TextAreaMessage::Paste(String::new()));
    assert_eq!(output, None);
}

#[test]
fn test_copy_then_paste() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    TextArea::update(&mut state, TextAreaMessage::Copy);
    TextArea::update(&mut state, TextAreaMessage::End);
    let clipboard = state.clipboard().to_string();
    TextArea::update(&mut state, TextAreaMessage::Paste(clipboard));
    assert_eq!(state.value(), "hellohello");
}

// =============================================================================
// Event mapping tests
// =============================================================================

#[test]
fn test_shift_left_event() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(
        &state,
        &Event::key_with(KeyCode::Left, KeyModifiers::SHIFT),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TextAreaMessage::SelectLeft));
}

#[test]
fn test_shift_right_event() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(
        &state,
        &Event::key_with(KeyCode::Right, KeyModifiers::SHIFT),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TextAreaMessage::SelectRight));
}

#[test]
fn test_shift_up_event() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(
        &state,
        &Event::key_with(KeyCode::Up, KeyModifiers::SHIFT),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TextAreaMessage::SelectUp));
}

#[test]
fn test_shift_down_event() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(
        &state,
        &Event::key_with(KeyCode::Down, KeyModifiers::SHIFT),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TextAreaMessage::SelectDown));
}

#[test]
fn test_ctrl_c_event() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(&state, &Event::ctrl('c'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(TextAreaMessage::Copy));
}

#[test]
fn test_ctrl_x_event() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(&state, &Event::ctrl('x'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(TextAreaMessage::Cut));
}

#[test]
fn test_ctrl_a_event() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(&state, &Event::ctrl('a'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(TextAreaMessage::SelectAll));
}

#[test]
fn test_paste_event() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(
        &state,
        &Event::Paste("text".into()),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TextAreaMessage::Paste("text".into())));
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_clear_clears_selection() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    TextArea::update(&mut state, TextAreaMessage::Clear);
    assert!(!state.has_selection());
    assert_eq!(state.value(), "");
}

#[test]
fn test_set_value_clears_selection() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    TextArea::update(&mut state, TextAreaMessage::SetValue("new".into()));
    assert!(!state.has_selection());
    assert_eq!(state.value(), "new");
}

#[test]
fn test_delete_partial_multiline_selection() {
    let mut state = TextAreaState::new().with_value("abc\ndef\nghi");
    // Select from middle of line 0 to middle of line 2
    state.set_cursor_position(0, 1); // After 'a'
    state.selection_anchor = Some((0, 1));
    state.cursor_row = 2;
    state.cursor_col = 2; // After 'gh'
    let deleted = state.selected_text();
    assert_eq!(deleted, Some("bc\ndef\ngh".to_string()));

    state.delete_selection();
    assert_eq!(state.value(), "ai");
    assert_eq!(state.line_count(), 1);
}
