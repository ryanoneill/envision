use super::*;

fn focused_state(value: &str) -> TextAreaState {
    TextAreaState::new().with_value(value)
}

// =============================================================================
// Basic undo/redo
// =============================================================================

#[test]
fn test_undo_single_insert() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::Insert('!'));
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_redo_after_undo() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::Insert('!'));
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hello");

    TextArea::update(&mut state, TextAreaMessage::Redo);
    assert_eq!(state.value(), "hello!");
}

#[test]
fn test_undo_empty_stack_no_change() {
    let mut state = TextAreaState::new().with_value("hello");
    let output = TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_redo_empty_stack_no_change() {
    let mut state = TextAreaState::new().with_value("hello");
    let output = TextArea::update(&mut state, TextAreaMessage::Redo);
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello");
}

// =============================================================================
// Insert grouping
// =============================================================================

#[test]
fn test_grouped_inserts_undo_together() {
    let mut state = TextAreaState::new();
    for c in "hello".chars() {
        TextArea::update(&mut state, TextAreaMessage::Insert(c));
    }
    assert_eq!(state.value(), "hello");

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "");
}

#[test]
fn test_whitespace_breaks_insert_group() {
    let mut state = TextAreaState::new();
    for c in "hi ".chars() {
        TextArea::update(&mut state, TextAreaMessage::Insert(c));
    }
    for c in "you".chars() {
        TextArea::update(&mut state, TextAreaMessage::Insert(c));
    }
    assert_eq!(state.value(), "hi you");

    // Undo "you"
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hi ");

    // Undo " "
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hi");

    // Undo "hi"
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "");
}

// =============================================================================
// Delete grouping
// =============================================================================

#[test]
fn test_grouped_backspace_undo_together() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::Backspace);
    TextArea::update(&mut state, TextAreaMessage::Backspace);
    assert_eq!(state.value(), "hel");

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_grouped_delete_undo_together() {
    let mut state = TextAreaState::new().with_value("hello");
    state.set_cursor_position(0, 0);
    TextArea::update(&mut state, TextAreaMessage::Delete);
    TextArea::update(&mut state, TextAreaMessage::Delete);
    assert_eq!(state.value(), "llo");

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hello");
}

// =============================================================================
// Newline
// =============================================================================

#[test]
fn test_newline_is_own_undo_entry() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::NewLine);
    assert_eq!(state.line_count(), 2);

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hello");
    assert_eq!(state.line_count(), 1);
}

#[test]
fn test_newline_breaks_insert_group() {
    let mut state = TextAreaState::new();
    for c in "abc".chars() {
        TextArea::update(&mut state, TextAreaMessage::Insert(c));
    }
    TextArea::update(&mut state, TextAreaMessage::NewLine);
    for c in "def".chars() {
        TextArea::update(&mut state, TextAreaMessage::Insert(c));
    }
    assert_eq!(state.value(), "abc\ndef");

    // Undo "def"
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "abc\n");

    // Undo newline
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "abc");

    // Undo "abc"
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "");
}

// =============================================================================
// Line operations
// =============================================================================

#[test]
fn test_delete_line_undo() {
    let mut state = TextAreaState::new().with_value("abc\ndef\nghi");
    state.set_cursor_position(1, 0);
    TextArea::update(&mut state, TextAreaMessage::DeleteLine);
    assert_eq!(state.value(), "abc\nghi");

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "abc\ndef\nghi");
}

#[test]
fn test_delete_to_end_undo() {
    let mut state = TextAreaState::new().with_value("hello world");
    state.set_cursor_position(0, 5);
    TextArea::update(&mut state, TextAreaMessage::DeleteToEnd);
    assert_eq!(state.value(), "hello");

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hello world");
}

#[test]
fn test_delete_to_start_undo() {
    let mut state = TextAreaState::new().with_value("hello world");
    state.set_cursor_position(0, 6);
    TextArea::update(&mut state, TextAreaMessage::DeleteToStart);
    assert_eq!(state.value(), "world");

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hello world");
}

// =============================================================================
// Clear and SetValue
// =============================================================================

#[test]
fn test_clear_undo() {
    let mut state = TextAreaState::new().with_value("hello\nworld");
    TextArea::update(&mut state, TextAreaMessage::Clear);
    assert_eq!(state.value(), "");

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "hello\nworld");
}

#[test]
fn test_set_value_undo() {
    let mut state = TextAreaState::new().with_value("original");
    TextArea::update(&mut state, TextAreaMessage::SetValue("replaced".into()));
    assert_eq!(state.value(), "replaced");

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "original");
}

// =============================================================================
// Cursor position restoration
// =============================================================================

#[test]
fn test_undo_restores_cursor_position() {
    let mut state = TextAreaState::new().with_value("hello");
    let (row_before, col_before) = (state.cursor_row(), state.cursor_col());
    TextArea::update(&mut state, TextAreaMessage::Insert('!'));
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.cursor_row(), row_before);
    assert_eq!(state.cursor_col(), col_before);
}

#[test]
fn test_undo_restores_multiline_cursor() {
    let mut state = TextAreaState::new().with_value("abc\ndef");
    // Cursor at end of "def" (row=1, col=3)
    TextArea::update(&mut state, TextAreaMessage::NewLine);
    // Now on row 2
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.cursor_row(), 1);
    assert_eq!(state.cursor_col(), 3);
}

// =============================================================================
// Redo invalidation
// =============================================================================

#[test]
fn test_new_edit_clears_redo() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::Insert('!'));
    TextArea::update(&mut state, TextAreaMessage::Undo);

    // New edit clears redo
    TextArea::update(&mut state, TextAreaMessage::Insert('?'));
    let output = TextArea::update(&mut state, TextAreaMessage::Redo);
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello?");
}

// =============================================================================
// Disabled state
// =============================================================================

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_ctrl_z_maps_to_undo() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(
        &state,
        &Event::ctrl('z'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(TextAreaMessage::Undo));
}

#[test]
fn test_ctrl_y_maps_to_redo() {
    let state = focused_state("hello");
    let msg = TextArea::handle_event(
        &state,
        &Event::ctrl('y'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(TextAreaMessage::Redo));
}

// =============================================================================
// State accessors
// =============================================================================

#[test]
fn test_can_undo() {
    let mut state = TextAreaState::new().with_value("hello");
    assert!(!state.can_undo());

    TextArea::update(&mut state, TextAreaMessage::Insert('!'));
    assert!(state.can_undo());

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert!(!state.can_undo());
}

#[test]
fn test_can_redo() {
    let mut state = TextAreaState::new().with_value("hello");
    assert!(!state.can_redo());

    TextArea::update(&mut state, TextAreaMessage::Insert('!'));
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert!(state.can_redo());

    TextArea::update(&mut state, TextAreaMessage::Redo);
    assert!(!state.can_redo());
}

// =============================================================================
// Backspace joining lines
// =============================================================================

#[test]
fn test_backspace_join_lines_undo() {
    let mut state = TextAreaState::new().with_value("abc\ndef");
    state.set_cursor_position(1, 0);
    TextArea::update(&mut state, TextAreaMessage::Backspace);
    assert_eq!(state.value(), "abcdef");
    assert_eq!(state.line_count(), 1);

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "abc\ndef");
    assert_eq!(state.line_count(), 2);
}

// =============================================================================
// Multiple undo/redo cycles
// =============================================================================

#[test]
fn test_multiple_undo_redo_cycles() {
    let mut state = TextAreaState::new();

    // Type "a"
    TextArea::update(&mut state, TextAreaMessage::Insert('a'));
    // Newline
    TextArea::update(&mut state, TextAreaMessage::NewLine);
    // Type "b"
    TextArea::update(&mut state, TextAreaMessage::Insert('b'));

    assert_eq!(state.value(), "a\nb");

    // Undo "b"
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "a\n");

    // Redo "b"
    TextArea::update(&mut state, TextAreaMessage::Redo);
    assert_eq!(state.value(), "a\nb");

    // Undo everything
    TextArea::update(&mut state, TextAreaMessage::Undo);
    TextArea::update(&mut state, TextAreaMessage::Undo);
    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert_eq!(state.value(), "");
}

// =============================================================================
// Undo clears selection
// =============================================================================

#[test]
fn test_undo_clears_selection() {
    let mut state = TextAreaState::new().with_value("hello");
    TextArea::update(&mut state, TextAreaMessage::Insert('!'));
    TextArea::update(&mut state, TextAreaMessage::SelectAll);
    assert!(state.has_selection());

    TextArea::update(&mut state, TextAreaMessage::Undo);
    assert!(!state.has_selection());
}
