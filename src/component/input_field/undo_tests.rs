use super::*;

fn focused_state(value: &str) -> InputFieldState {
    let mut state = InputFieldState::with_value(value);
    state.set_focused(true);
    state
}

// =============================================================================
// Basic undo/redo
// =============================================================================

#[test]
fn test_undo_single_insert() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::Insert(' '));
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_redo_after_undo() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::Insert('!'));
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello");

    InputField::update(&mut state, InputFieldMessage::Redo);
    assert_eq!(state.value(), "hello!");
}

#[test]
fn test_undo_empty_stack_no_change() {
    let mut state = InputFieldState::with_value("hello");
    let output = InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_redo_empty_stack_no_change() {
    let mut state = InputFieldState::with_value("hello");
    let output = InputField::update(&mut state, InputFieldMessage::Redo);
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello");
}

// =============================================================================
// Insert grouping
// =============================================================================

#[test]
fn test_grouped_inserts_undo_together() {
    let mut state = InputFieldState::new();
    // Type "hello" - should group into one undo entry
    for c in "hello".chars() {
        InputField::update(&mut state, InputFieldMessage::Insert(c));
    }
    assert_eq!(state.value(), "hello");

    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "");
}

#[test]
fn test_whitespace_breaks_insert_group() {
    let mut state = InputFieldState::new();
    // Type "hi " - whitespace should break the group
    for c in "hi ".chars() {
        InputField::update(&mut state, InputFieldMessage::Insert(c));
    }
    // Type "you"
    for c in "you".chars() {
        InputField::update(&mut state, InputFieldMessage::Insert(c));
    }
    assert_eq!(state.value(), "hi you");

    // Undo "you"
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hi ");

    // Undo " " (whitespace is its own group)
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hi");

    // Undo "hi"
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "");
}

// =============================================================================
// Delete grouping
// =============================================================================

#[test]
fn test_grouped_deletes_undo_together() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::Backspace);
    InputField::update(&mut state, InputFieldMessage::Backspace);
    assert_eq!(state.value(), "hel");

    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_delete_forward_grouped() {
    let mut state = InputFieldState::with_value("hello");
    state.set_cursor_position(0);
    InputField::update(&mut state, InputFieldMessage::Delete);
    InputField::update(&mut state, InputFieldMessage::Delete);
    assert_eq!(state.value(), "llo");

    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello");
}

// =============================================================================
// Mixed operations
// =============================================================================

#[test]
fn test_insert_then_delete_separate_groups() {
    let mut state = InputFieldState::new();
    for c in "hello".chars() {
        InputField::update(&mut state, InputFieldMessage::Insert(c));
    }
    InputField::update(&mut state, InputFieldMessage::Backspace);
    assert_eq!(state.value(), "hell");

    // Undo delete
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello");

    // Undo insert group
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "");
}

#[test]
fn test_clear_is_own_undo_entry() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::Clear);
    assert_eq!(state.value(), "");

    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_set_value_is_own_undo_entry() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SetValue("world".into()));
    assert_eq!(state.value(), "world");

    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_delete_word_back_is_own_undo_entry() {
    let mut state = InputFieldState::with_value("hello world");
    InputField::update(&mut state, InputFieldMessage::DeleteWordBack);
    assert_eq!(state.value(), "hello ");

    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello world");
}

#[test]
fn test_delete_word_forward_is_own_undo_entry() {
    let mut state = InputFieldState::with_value("hello world");
    state.set_cursor_position(0);
    InputField::update(&mut state, InputFieldMessage::DeleteWordForward);
    assert_eq!(state.value(), "world");

    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello world");
}

// =============================================================================
// Cursor position restoration
// =============================================================================

#[test]
fn test_undo_restores_cursor_position() {
    let mut state = InputFieldState::with_value("hello");
    // Cursor at end (5)
    InputField::update(&mut state, InputFieldMessage::Insert('!'));
    // Cursor at 6
    InputField::update(&mut state, InputFieldMessage::Undo);
    // Should restore cursor to 5
    assert_eq!(state.cursor_byte_offset(), 5);
}

#[test]
fn test_redo_restores_cursor_position() {
    let mut state = InputFieldState::new();
    for c in "hi".chars() {
        InputField::update(&mut state, InputFieldMessage::Insert(c));
    }
    let cursor_after = state.cursor_byte_offset();

    InputField::update(&mut state, InputFieldMessage::Undo);
    InputField::update(&mut state, InputFieldMessage::Redo);
    assert_eq!(state.cursor_byte_offset(), cursor_after);
}

// =============================================================================
// Redo invalidation
// =============================================================================

#[test]
fn test_new_edit_clears_redo() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::Insert('!'));
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello");

    // New edit should clear redo
    InputField::update(&mut state, InputFieldMessage::Insert('?'));
    let output = InputField::update(&mut state, InputFieldMessage::Redo);
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello?");
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_undo_ignored_when_disabled() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::Insert('!'));
    state.set_disabled(true);
    let output = InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello!");
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_ctrl_z_maps_to_undo() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(&state, &Event::ctrl('z'));
    assert_eq!(msg, Some(InputFieldMessage::Undo));
}

#[test]
fn test_ctrl_y_maps_to_redo() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(&state, &Event::ctrl('y'));
    assert_eq!(msg, Some(InputFieldMessage::Redo));
}

// =============================================================================
// State accessors
// =============================================================================

#[test]
fn test_can_undo() {
    let mut state = InputFieldState::with_value("hello");
    assert!(!state.can_undo());

    InputField::update(&mut state, InputFieldMessage::Insert('!'));
    assert!(state.can_undo());

    InputField::update(&mut state, InputFieldMessage::Undo);
    assert!(!state.can_undo());
}

#[test]
fn test_can_redo() {
    let mut state = InputFieldState::with_value("hello");
    assert!(!state.can_redo());

    InputField::update(&mut state, InputFieldMessage::Insert('!'));
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert!(state.can_redo());

    InputField::update(&mut state, InputFieldMessage::Redo);
    assert!(!state.can_redo());
}

// =============================================================================
// Multiple undo/redo cycles
// =============================================================================

#[test]
fn test_multiple_undo_redo_cycles() {
    let mut state = InputFieldState::new();

    // Type "a"
    InputField::update(&mut state, InputFieldMessage::Insert('a'));
    // Type " " (break group)
    InputField::update(&mut state, InputFieldMessage::Insert(' '));
    // Type "b"
    InputField::update(&mut state, InputFieldMessage::Insert('b'));

    assert_eq!(state.value(), "a b");

    // Undo "b"
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "a ");

    // Redo "b"
    InputField::update(&mut state, InputFieldMessage::Redo);
    assert_eq!(state.value(), "a b");

    // Undo "b" again
    InputField::update(&mut state, InputFieldMessage::Undo);
    // Undo " "
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "a");
}

#[test]
fn test_clear_history_on_set_value() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::Insert('!'));

    // SetValue creates its own undo entry
    InputField::update(&mut state, InputFieldMessage::SetValue("new".into()));
    assert_eq!(state.value(), "new");

    // Can undo SetValue
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert_eq!(state.value(), "hello!");
}

// =============================================================================
// Undo clears selection
// =============================================================================

#[test]
fn test_undo_clears_selection() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::Insert('!'));
    // Select all
    InputField::update(&mut state, InputFieldMessage::SelectAll);
    assert!(state.has_selection());

    // Undo should clear selection
    InputField::update(&mut state, InputFieldMessage::Undo);
    assert!(!state.has_selection());
}
