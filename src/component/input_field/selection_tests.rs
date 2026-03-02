use super::*;

// =============================================================================
// Selection model tests
// =============================================================================

#[test]
fn test_no_selection_by_default() {
    let state = InputFieldState::with_value("hello");
    assert!(!state.has_selection());
    assert_eq!(state.selection_range(), None);
    assert_eq!(state.selected_text(), None);
}

#[test]
fn test_select_left() {
    let mut state = InputFieldState::with_value("hello");
    // Cursor at end (5), select left
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    assert!(state.has_selection());
    assert_eq!(state.selected_text(), Some("o"));
    // Cursor moved left, anchor at original position
    assert_eq!(state.cursor_position(), 4);
}

#[test]
fn test_select_left_multiple() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    assert_eq!(state.selected_text(), Some("llo"));
    assert_eq!(state.cursor_position(), 2);
}

#[test]
fn test_select_right() {
    let mut state = InputFieldState::with_value("hello");
    state.set_cursor(0);
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    assert!(state.has_selection());
    assert_eq!(state.selected_text(), Some("h"));
    assert_eq!(state.cursor_position(), 1);
}

#[test]
fn test_select_right_multiple() {
    let mut state = InputFieldState::with_value("hello");
    state.set_cursor(0);
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    assert_eq!(state.selected_text(), Some("hel"));
}

#[test]
fn test_select_home() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectHome);
    assert_eq!(state.selected_text(), Some("hello"));
    assert_eq!(state.cursor_position(), 0);
}

#[test]
fn test_select_end() {
    let mut state = InputFieldState::with_value("hello");
    state.set_cursor(0);
    InputField::update(&mut state, InputFieldMessage::SelectEnd);
    assert_eq!(state.selected_text(), Some("hello"));
    assert_eq!(state.cursor_position(), 5);
}

#[test]
fn test_select_word_left() {
    let mut state = InputFieldState::with_value("hello world");
    // Cursor at end
    InputField::update(&mut state, InputFieldMessage::SelectWordLeft);
    assert_eq!(state.selected_text(), Some("world"));
}

#[test]
fn test_select_word_right() {
    let mut state = InputFieldState::with_value("hello world");
    state.set_cursor(0);
    InputField::update(&mut state, InputFieldMessage::SelectWordRight);
    assert_eq!(state.selected_text(), Some("hello "));
}

#[test]
fn test_select_all() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectAll);
    assert_eq!(state.selected_text(), Some("hello"));
    assert_eq!(state.cursor_position(), 5);
}

#[test]
fn test_select_all_empty() {
    let mut state = InputFieldState::new();
    let output = InputField::update(&mut state, InputFieldMessage::SelectAll);
    assert_eq!(output, None);
    assert!(!state.has_selection());
}

// =============================================================================
// Selection clearing tests
// =============================================================================

#[test]
fn test_left_clears_selection() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    assert!(state.has_selection());

    // Left without shift clears selection and moves to start of selection
    InputField::update(&mut state, InputFieldMessage::Left);
    assert!(!state.has_selection());
    assert_eq!(state.cursor_position(), 3);
}

#[test]
fn test_right_clears_selection() {
    let mut state = InputFieldState::with_value("hello");
    state.set_cursor(0);
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    assert!(state.has_selection());

    // Right without shift clears selection and moves to end of selection
    InputField::update(&mut state, InputFieldMessage::Right);
    assert!(!state.has_selection());
    assert_eq!(state.cursor_position(), 2);
}

#[test]
fn test_home_clears_selection() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    assert!(state.has_selection());

    InputField::update(&mut state, InputFieldMessage::Home);
    assert!(!state.has_selection());
}

#[test]
fn test_end_clears_selection() {
    let mut state = InputFieldState::with_value("hello");
    state.set_cursor(0);
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    assert!(state.has_selection());

    InputField::update(&mut state, InputFieldMessage::End);
    assert!(!state.has_selection());
}

// =============================================================================
// Selection + editing tests
// =============================================================================

#[test]
fn test_insert_replaces_selection() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectAll);
    let output = InputField::update(&mut state, InputFieldMessage::Insert('X'));
    assert_eq!(state.value(), "X");
    assert_eq!(output, Some(InputFieldOutput::Changed("X".into())));
    assert!(!state.has_selection());
}

#[test]
fn test_insert_replaces_partial_selection() {
    let mut state = InputFieldState::with_value("hello world");
    state.set_cursor(0);
    // Select "hello"
    for _ in 0..5 {
        InputField::update(&mut state, InputFieldMessage::SelectRight);
    }
    assert_eq!(state.selected_text(), Some("hello"));

    InputField::update(&mut state, InputFieldMessage::Insert('H'));
    assert_eq!(state.value(), "H world");
}

#[test]
fn test_backspace_deletes_selection() {
    let mut state = InputFieldState::with_value("hello world");
    // Select "world"
    for _ in 0..5 {
        InputField::update(&mut state, InputFieldMessage::SelectLeft);
    }
    assert_eq!(state.selected_text(), Some("world"));

    let output = InputField::update(&mut state, InputFieldMessage::Backspace);
    assert_eq!(state.value(), "hello ");
    assert_eq!(output, Some(InputFieldOutput::Changed("hello ".into())));
    assert!(!state.has_selection());
}

#[test]
fn test_delete_deletes_selection() {
    let mut state = InputFieldState::with_value("hello world");
    InputField::update(&mut state, InputFieldMessage::SelectAll);

    let output = InputField::update(&mut state, InputFieldMessage::Delete);
    assert_eq!(state.value(), "");
    assert_eq!(output, Some(InputFieldOutput::Changed(String::new())));
}

#[test]
fn test_delete_word_back_deletes_selection() {
    let mut state = InputFieldState::with_value("hello world");
    // Select "world"
    for _ in 0..5 {
        InputField::update(&mut state, InputFieldMessage::SelectLeft);
    }

    let output = InputField::update(&mut state, InputFieldMessage::DeleteWordBack);
    assert_eq!(state.value(), "hello ");
    assert!(output.is_some());
}

#[test]
fn test_delete_word_forward_deletes_selection() {
    let mut state = InputFieldState::with_value("hello world");
    InputField::update(&mut state, InputFieldMessage::SelectAll);

    let output = InputField::update(&mut state, InputFieldMessage::DeleteWordForward);
    assert_eq!(state.value(), "");
    assert!(output.is_some());
}

// =============================================================================
// Copy/Cut/Paste tests
// =============================================================================

#[test]
fn test_copy_with_selection() {
    let mut state = InputFieldState::with_value("hello world");
    // Select "hello"
    state.set_cursor(0);
    for _ in 0..5 {
        InputField::update(&mut state, InputFieldMessage::SelectRight);
    }

    let output = InputField::update(&mut state, InputFieldMessage::Copy);
    assert_eq!(output, Some(InputFieldOutput::Copied("hello".into())));
    assert_eq!(state.clipboard(), "hello");
    // Value unchanged, selection preserved
    assert_eq!(state.value(), "hello world");
    assert!(state.has_selection());
}

#[test]
fn test_copy_without_selection() {
    let mut state = InputFieldState::with_value("hello");
    let output = InputField::update(&mut state, InputFieldMessage::Copy);
    assert_eq!(output, None);
    assert_eq!(state.clipboard(), "");
}

#[test]
fn test_cut_with_selection() {
    let mut state = InputFieldState::with_value("hello world");
    // Select "hello"
    state.set_cursor(0);
    for _ in 0..5 {
        InputField::update(&mut state, InputFieldMessage::SelectRight);
    }

    let output = InputField::update(&mut state, InputFieldMessage::Cut);
    assert_eq!(output, Some(InputFieldOutput::Changed(" world".into())));
    assert_eq!(state.clipboard(), "hello");
    assert_eq!(state.value(), " world");
    assert!(!state.has_selection());
}

#[test]
fn test_cut_without_selection() {
    let mut state = InputFieldState::with_value("hello");
    let output = InputField::update(&mut state, InputFieldMessage::Cut);
    assert_eq!(output, None);
}

#[test]
fn test_paste() {
    let mut state = InputFieldState::with_value("hello");
    state.set_cursor(5);
    let output = InputField::update(&mut state, InputFieldMessage::Paste(" world".into()));
    assert_eq!(state.value(), "hello world");
    assert_eq!(output, Some(InputFieldOutput::Changed("hello world".into())));
}

#[test]
fn test_paste_replaces_selection() {
    let mut state = InputFieldState::with_value("hello world");
    InputField::update(&mut state, InputFieldMessage::SelectAll);

    let output = InputField::update(&mut state, InputFieldMessage::Paste("goodbye".into()));
    assert_eq!(state.value(), "goodbye");
    assert_eq!(output, Some(InputFieldOutput::Changed("goodbye".into())));
}

#[test]
fn test_paste_empty_string() {
    let mut state = InputFieldState::with_value("hello");
    let output = InputField::update(&mut state, InputFieldMessage::Paste(String::new()));
    assert_eq!(output, None);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_paste_at_cursor() {
    let mut state = InputFieldState::with_value("helo");
    state.set_cursor(3); // Between 'l' and 'o'
    InputField::update(&mut state, InputFieldMessage::Paste("l".into()));
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_copy_then_paste() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectAll);
    InputField::update(&mut state, InputFieldMessage::Copy);

    // Move to end and paste
    InputField::update(&mut state, InputFieldMessage::End);
    let clipboard = state.clipboard().to_string();
    InputField::update(&mut state, InputFieldMessage::Paste(clipboard));
    assert_eq!(state.value(), "hellohello");
}

#[test]
fn test_cut_then_paste() {
    let mut state = InputFieldState::with_value("hello world");
    // Select "hello"
    state.set_cursor(0);
    for _ in 0..5 {
        InputField::update(&mut state, InputFieldMessage::SelectRight);
    }
    InputField::update(&mut state, InputFieldMessage::Cut);
    assert_eq!(state.value(), " world");

    // Paste at beginning
    InputField::update(&mut state, InputFieldMessage::Home);
    let clipboard = state.clipboard().to_string();
    InputField::update(&mut state, InputFieldMessage::Paste(clipboard));
    assert_eq!(state.value(), "hello world");
}

// =============================================================================
// Event mapping tests
// =============================================================================

fn focused_state(value: &str) -> InputFieldState {
    let mut state = InputFieldState::with_value(value);
    state.set_focused(true);
    state
}

#[test]
fn test_shift_left_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(
        &state,
        &Event::key_with(KeyCode::Left, KeyModifiers::SHIFT),
    );
    assert_eq!(msg, Some(InputFieldMessage::SelectLeft));
}

#[test]
fn test_shift_right_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(
        &state,
        &Event::key_with(KeyCode::Right, KeyModifiers::SHIFT),
    );
    assert_eq!(msg, Some(InputFieldMessage::SelectRight));
}

#[test]
fn test_shift_home_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(
        &state,
        &Event::key_with(KeyCode::Home, KeyModifiers::SHIFT),
    );
    assert_eq!(msg, Some(InputFieldMessage::SelectHome));
}

#[test]
fn test_shift_end_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(
        &state,
        &Event::key_with(KeyCode::End, KeyModifiers::SHIFT),
    );
    assert_eq!(msg, Some(InputFieldMessage::SelectEnd));
}

#[test]
fn test_ctrl_shift_left_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(
        &state,
        &Event::key_with(KeyCode::Left, KeyModifiers::CONTROL | KeyModifiers::SHIFT),
    );
    assert_eq!(msg, Some(InputFieldMessage::SelectWordLeft));
}

#[test]
fn test_ctrl_shift_right_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(
        &state,
        &Event::key_with(KeyCode::Right, KeyModifiers::CONTROL | KeyModifiers::SHIFT),
    );
    assert_eq!(msg, Some(InputFieldMessage::SelectWordRight));
}

#[test]
fn test_ctrl_c_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(&state, &Event::ctrl('c'));
    assert_eq!(msg, Some(InputFieldMessage::Copy));
}

#[test]
fn test_ctrl_x_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(&state, &Event::ctrl('x'));
    assert_eq!(msg, Some(InputFieldMessage::Cut));
}

/// When the clipboard feature is enabled, system clipboard content takes
/// precedence over the internal clipboard, making this test environment-dependent.
/// This test validates the internal clipboard fallback path (no clipboard feature).
#[cfg(not(feature = "clipboard"))]
#[test]
fn test_ctrl_v_event_with_clipboard() {
    let mut state = focused_state("hello");
    state.clipboard = "world".into();
    let msg = InputField::handle_event(&state, &Event::ctrl('v'));
    assert_eq!(msg, Some(InputFieldMessage::Paste("world".into())));
}

/// When the clipboard feature is enabled, system clipboard may have content,
/// so Ctrl+V may produce a Paste message even with empty internal clipboard.
/// This test validates the no-content path (no clipboard feature).
#[cfg(not(feature = "clipboard"))]
#[test]
fn test_ctrl_v_event_empty_clipboard() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(&state, &Event::ctrl('v'));
    assert_eq!(msg, None);
}

/// With the clipboard feature enabled, Ctrl+V always produces a Paste message
/// when either system or internal clipboard has content.
#[cfg(feature = "clipboard")]
#[test]
fn test_ctrl_v_event_with_internal_clipboard() {
    let mut state = focused_state("hello");
    state.clipboard = "world".into();
    let msg = InputField::handle_event(&state, &Event::ctrl('v'));
    // System clipboard may override internal, but should still produce a Paste
    assert!(matches!(msg, Some(InputFieldMessage::Paste(_))));
}

#[test]
fn test_ctrl_a_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(&state, &Event::ctrl('a'));
    assert_eq!(msg, Some(InputFieldMessage::SelectAll));
}

#[test]
fn test_paste_event() {
    let state = focused_state("hello");
    let msg = InputField::handle_event(&state, &Event::Paste("pasted text".into()));
    assert_eq!(msg, Some(InputFieldMessage::Paste("pasted text".into())));
}

// =============================================================================
// UTF-8 selection tests
// =============================================================================

#[test]
fn test_select_utf8() {
    let mut state = InputFieldState::with_value("héllo");
    // Cursor at end, select left twice
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    assert_eq!(state.selected_text(), Some("lo"));
}

#[test]
fn test_select_all_utf8() {
    let mut state = InputFieldState::with_value("日本語");
    InputField::update(&mut state, InputFieldMessage::SelectAll);
    assert_eq!(state.selected_text(), Some("日本語"));
}

#[test]
fn test_cut_utf8() {
    let mut state = InputFieldState::with_value("héllo wörld");
    state.set_cursor(0);
    // Select "héllo"
    for _ in 0..5 {
        InputField::update(&mut state, InputFieldMessage::SelectRight);
    }
    InputField::update(&mut state, InputFieldMessage::Cut);
    assert_eq!(state.value(), " wörld");
    assert_eq!(state.clipboard(), "héllo");
}

#[test]
fn test_paste_utf8() {
    let mut state = InputFieldState::with_value("");
    InputField::update(&mut state, InputFieldMessage::Paste("日本語".into()));
    assert_eq!(state.value(), "日本語");
}

// =============================================================================
// Edge case tests
// =============================================================================

#[test]
fn test_select_left_at_start() {
    let mut state = InputFieldState::with_value("hello");
    state.set_cursor(0);
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    // Should set anchor but cursor can't move further
    assert!(!state.has_selection()); // anchor == cursor
}

#[test]
fn test_select_right_at_end() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    // Should set anchor but cursor can't move further
    assert!(!state.has_selection()); // anchor == cursor
}

#[test]
fn test_selection_preserved_across_multiple_shifts() {
    let mut state = InputFieldState::with_value("hello world");
    state.set_cursor(0);
    // Select "hello" then extend to "hello world"
    for _ in 0..5 {
        InputField::update(&mut state, InputFieldMessage::SelectRight);
    }
    assert_eq!(state.selected_text(), Some("hello"));

    for _ in 0..6 {
        InputField::update(&mut state, InputFieldMessage::SelectRight);
    }
    assert_eq!(state.selected_text(), Some("hello world"));
}

#[test]
fn test_select_then_reverse_direction() {
    let mut state = InputFieldState::with_value("hello");
    state.set_cursor(2); // After "he"
    // Select right twice: anchor=2, cursor=4, selected "ll"
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    InputField::update(&mut state, InputFieldMessage::SelectRight);
    assert_eq!(state.selected_text(), Some("ll"));

    // Select left three times: cursor goes 4→3→2→1
    // anchor=2, cursor=1, selected "e" (bytes 1..2)
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    InputField::update(&mut state, InputFieldMessage::SelectLeft);
    assert_eq!(state.selected_text(), Some("e"));
    assert_eq!(state.cursor_position(), 1);
}

#[test]
fn test_disabled_ignores_selection() {
    let mut state = InputFieldState::with_value("hello");
    state.set_disabled(true);
    let output = InputField::update(&mut state, InputFieldMessage::SelectAll);
    assert_eq!(output, None);
    assert!(!state.has_selection());
}

#[test]
fn test_clear_clears_selection() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectAll);
    assert!(state.has_selection());

    InputField::update(&mut state, InputFieldMessage::Clear);
    assert!(!state.has_selection());
    assert_eq!(state.value(), "");
}

#[test]
fn test_set_value_clears_selection() {
    let mut state = InputFieldState::with_value("hello");
    InputField::update(&mut state, InputFieldMessage::SelectAll);
    assert!(state.has_selection());

    InputField::update(&mut state, InputFieldMessage::SetValue("new".into()));
    assert!(!state.has_selection());
    assert_eq!(state.value(), "new");
}
