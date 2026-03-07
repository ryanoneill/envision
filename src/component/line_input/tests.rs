use super::*;
use crate::component::test_utils::setup_render;
use crate::component::{Component, Focusable};
use crate::input::{Event, KeyCode, KeyModifiers};

// ---- Construction / Accessors ----

#[test]
fn test_new_empty() {
    let state = LineInputState::new();
    assert!(state.is_empty());
    assert_eq!(state.value(), "");
    assert_eq!(state.len(), 0);
    assert_eq!(state.cursor_byte_offset(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
}

#[test]
fn test_with_value() {
    let state = LineInputState::with_value("hello");
    assert_eq!(state.value(), "hello");
    assert_eq!(state.cursor_byte_offset(), 5);
    assert_eq!(state.len(), 5);
}

#[test]
fn test_with_placeholder() {
    let state = LineInputState::new().with_placeholder("Type here...");
    assert_eq!(state.placeholder(), "Type here...");
}

#[test]
fn test_with_disabled() {
    let state = LineInputState::new().with_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_with_max_history() {
    let state = LineInputState::new().with_max_history(5);
    assert_eq!(state.history_count(), 0);
}

#[test]
fn test_set_value() {
    let mut state = LineInputState::with_value("hello");
    state.set_value("world");
    assert_eq!(state.value(), "world");
    assert_eq!(state.cursor_byte_offset(), 5);
}

#[test]
fn test_set_placeholder() {
    let mut state = LineInputState::new();
    state.set_placeholder("Enter command...");
    assert_eq!(state.placeholder(), "Enter command...");
}

#[test]
fn test_set_disabled() {
    let mut state = LineInputState::new();
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_display_width() {
    let mut state = LineInputState::new();
    assert_eq!(state.display_width(), 80);
    state.set_display_width(40);
    assert_eq!(state.display_width(), 40);
}

// ---- Focusable ----

#[test]
fn test_focusable() {
    let mut state = LineInput::init();
    assert!(!LineInput::is_focused(&state));
    LineInput::focus(&mut state);
    assert!(LineInput::is_focused(&state));
    LineInput::blur(&mut state);
    assert!(!LineInput::is_focused(&state));
}

#[test]
fn test_focus_instance_methods() {
    let mut state = LineInputState::new();
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());
}

// ---- Editing: Insert ----

#[test]
fn test_insert_char() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let output = state.update(LineInputMessage::Insert('h'));
    assert_eq!(state.value(), "h");
    assert_eq!(state.cursor_byte_offset(), 1);
    assert_eq!(output, Some(LineInputOutput::Changed("h".to_string())));
}

#[test]
fn test_insert_multiple_chars() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    state.update(LineInputMessage::Insert('h'));
    state.update(LineInputMessage::Insert('i'));
    assert_eq!(state.value(), "hi");
    assert_eq!(state.cursor_byte_offset(), 2);
}

#[test]
fn test_insert_unicode() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    state.update(LineInputMessage::Insert('世'));
    assert_eq!(state.value(), "世");
    assert_eq!(state.cursor_byte_offset(), 3);
}

#[test]
fn test_insert_emoji() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    state.update(LineInputMessage::Insert('🎉'));
    assert_eq!(state.value(), "🎉");
    assert_eq!(state.cursor_byte_offset(), 4);
}

// ---- Editing: Backspace ----

#[test]
fn test_backspace() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    let output = state.update(LineInputMessage::Backspace);
    assert_eq!(state.value(), "hell");
    assert_eq!(state.cursor_byte_offset(), 4);
    assert_eq!(output, Some(LineInputOutput::Changed("hell".to_string())));
}

#[test]
fn test_backspace_at_start() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let output = state.update(LineInputMessage::Backspace);
    assert_eq!(output, None);
}

#[test]
fn test_backspace_with_selection() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    // Select "ell"
    state.cursor = 1;
    state.selection_anchor = Some(4);
    let output = state.update(LineInputMessage::Backspace);
    assert_eq!(state.value(), "ho");
    assert_eq!(output, Some(LineInputOutput::Changed("ho".to_string())));
}

// ---- Editing: Delete ----

#[test]
fn test_delete() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.cursor = 0;
    let output = state.update(LineInputMessage::Delete);
    assert_eq!(state.value(), "ello");
    assert_eq!(output, Some(LineInputOutput::Changed("ello".to_string())));
}

#[test]
fn test_delete_at_end() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    let output = state.update(LineInputMessage::Delete);
    assert_eq!(output, None);
}

// ---- Editing: Word delete ----

#[test]
fn test_delete_word_back() {
    let mut state = LineInputState::with_value("hello world");
    state.set_focused(true);
    let output = state.update(LineInputMessage::DeleteWordBack);
    assert_eq!(state.value(), "hello ");
    assert_eq!(output, Some(LineInputOutput::Changed("hello ".to_string())));
}

#[test]
fn test_delete_word_forward() {
    let mut state = LineInputState::with_value("hello world");
    state.set_focused(true);
    state.cursor = 5;
    let output = state.update(LineInputMessage::DeleteWordForward);
    assert_eq!(state.value(), "helloworld");
    assert_eq!(
        output,
        Some(LineInputOutput::Changed("helloworld".to_string()))
    );
}

// ---- Editing: Clear ----

#[test]
fn test_clear() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    let output = state.update(LineInputMessage::Clear);
    assert_eq!(state.value(), "");
    assert_eq!(state.cursor_byte_offset(), 0);
    assert_eq!(output, Some(LineInputOutput::Changed(String::new())));
}

#[test]
fn test_clear_empty() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let output = state.update(LineInputMessage::Clear);
    assert_eq!(output, None);
}

// ---- Editing: Paste ----

#[test]
fn test_paste() {
    let mut state = LineInputState::with_value("ab");
    state.set_focused(true);
    state.cursor = 1;
    let output = state.update(LineInputMessage::Paste("xyz".to_string()));
    assert_eq!(state.value(), "axyzb");
    assert_eq!(state.cursor_byte_offset(), 4);
    assert_eq!(output, Some(LineInputOutput::Changed("axyzb".to_string())));
}

#[test]
fn test_paste_strips_newlines() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    state.update(LineInputMessage::Paste("line1\nline2\r\nline3".to_string()));
    assert_eq!(state.value(), "line1line2line3");
}

// ---- Editing: SetValue ----

#[test]
fn test_set_value_message() {
    let mut state = LineInputState::with_value("old");
    state.set_focused(true);
    let output = state.update(LineInputMessage::SetValue("new".to_string()));
    assert_eq!(state.value(), "new");
    assert_eq!(state.cursor_byte_offset(), 3);
    assert_eq!(output, Some(LineInputOutput::Changed("new".to_string())));
}

// ---- Movement ----

#[test]
fn test_move_left() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.update(LineInputMessage::Left);
    assert_eq!(state.cursor_byte_offset(), 4);
}

#[test]
fn test_move_right() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.cursor = 0;
    state.update(LineInputMessage::Right);
    assert_eq!(state.cursor_byte_offset(), 1);
}

#[test]
fn test_move_home() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.update(LineInputMessage::Home);
    assert_eq!(state.cursor_byte_offset(), 0);
}

#[test]
fn test_move_end() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.cursor = 0;
    state.update(LineInputMessage::End);
    assert_eq!(state.cursor_byte_offset(), 5);
}

#[test]
fn test_move_word_left() {
    let mut state = LineInputState::with_value("hello world");
    state.set_focused(true);
    state.update(LineInputMessage::WordLeft);
    assert_eq!(state.cursor_byte_offset(), 6);
}

#[test]
fn test_move_word_right() {
    let mut state = LineInputState::with_value("hello world");
    state.set_focused(true);
    state.cursor = 0;
    state.update(LineInputMessage::WordRight);
    assert_eq!(state.cursor_byte_offset(), 6);
}

#[test]
fn test_visual_up() {
    let mut state = LineInputState::with_value("hello world!");
    state.set_focused(true);
    state.set_display_width(5);
    // cursor at end: "hello" | " worl" | "d!"
    // cursor at byte 12 → row 2, col 2
    // VisualUp → row 1, col 2 → byte 7
    state.update(LineInputMessage::VisualUp);
    assert_eq!(state.cursor_byte_offset(), 7);
}

#[test]
fn test_visual_down() {
    let mut state = LineInputState::with_value("hello world!");
    state.set_focused(true);
    state.set_display_width(5);
    state.cursor = 0;
    // cursor at byte 0 → row 0, col 0
    // VisualDown → row 1, col 0 → byte 5
    state.update(LineInputMessage::VisualDown);
    assert_eq!(state.cursor_byte_offset(), 5);
}

#[test]
fn test_movement_clears_selection() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.selection_anchor = Some(0);
    state.update(LineInputMessage::Left);
    assert!(!state.has_selection());
}

// ---- Selection ----

#[test]
fn test_select_left() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.update(LineInputMessage::SelectLeft);
    assert!(state.has_selection());
    assert_eq!(state.selected_text(), Some("o"));
}

#[test]
fn test_select_right() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.cursor = 0;
    state.update(LineInputMessage::SelectRight);
    assert!(state.has_selection());
    assert_eq!(state.selected_text(), Some("h"));
}

#[test]
fn test_select_home() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.update(LineInputMessage::SelectHome);
    assert_eq!(state.selected_text(), Some("hello"));
}

#[test]
fn test_select_end() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.cursor = 0;
    state.update(LineInputMessage::SelectEnd);
    assert_eq!(state.selected_text(), Some("hello"));
}

#[test]
fn test_select_word_left() {
    let mut state = LineInputState::with_value("hello world");
    state.set_focused(true);
    state.update(LineInputMessage::SelectWordLeft);
    assert_eq!(state.selected_text(), Some("world"));
}

#[test]
fn test_select_word_right() {
    let mut state = LineInputState::with_value("hello world");
    state.set_focused(true);
    state.cursor = 0;
    state.update(LineInputMessage::SelectWordRight);
    assert_eq!(state.selected_text(), Some("hello "));
}

#[test]
fn test_select_all() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.update(LineInputMessage::SelectAll);
    assert_eq!(state.selected_text(), Some("hello"));
}

#[test]
fn test_select_all_empty() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let output = state.update(LineInputMessage::SelectAll);
    assert_eq!(output, None);
    assert!(!state.has_selection());
}

// ---- Clipboard ----

#[test]
fn test_copy() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.selection_anchor = Some(0);
    // cursor at 5, anchor at 0 → selects "hello"
    let output = state.update(LineInputMessage::Copy);
    assert_eq!(output, Some(LineInputOutput::Copied("hello".to_string())));
    assert_eq!(state.clipboard(), "hello");
    // Buffer unchanged
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_copy_no_selection() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    let output = state.update(LineInputMessage::Copy);
    assert_eq!(output, None);
}

#[test]
fn test_cut() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.selection_anchor = Some(0);
    let output = state.update(LineInputMessage::Cut);
    assert_eq!(state.clipboard(), "hello");
    assert_eq!(state.value(), "");
    assert_eq!(output, Some(LineInputOutput::Changed(String::new())));
}

#[test]
fn test_cut_no_selection() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    let output = state.update(LineInputMessage::Cut);
    assert_eq!(output, None);
}

// ---- History ----

#[test]
fn test_submit_pushes_to_history() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    let output = state.update(LineInputMessage::Submit);
    assert_eq!(
        output,
        Some(LineInputOutput::Submitted("hello".to_string()))
    );
    assert_eq!(state.value(), "");
    assert_eq!(state.cursor_byte_offset(), 0);
    assert_eq!(state.history_count(), 1);
}

#[test]
fn test_history_prev_next() {
    let mut state = LineInputState::with_value("first");
    state.set_focused(true);
    state.update(LineInputMessage::Submit);
    state.update(LineInputMessage::Insert('x'));

    // Navigate back
    let output = state.update(LineInputMessage::HistoryPrev);
    assert_eq!(state.value(), "first");
    assert_eq!(output, Some(LineInputOutput::Changed("first".to_string())));
    assert!(state.is_browsing_history());

    // Navigate forward (restores stashed "x")
    let output = state.update(LineInputMessage::HistoryNext);
    assert_eq!(state.value(), "x");
    assert_eq!(output, Some(LineInputOutput::Changed("x".to_string())));
    assert!(!state.is_browsing_history());
}

#[test]
fn test_history_prev_empty() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let output = state.update(LineInputMessage::HistoryPrev);
    assert_eq!(output, None);
}

#[test]
fn test_history_next_not_browsing() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let output = state.update(LineInputMessage::HistoryNext);
    assert_eq!(output, None);
}

#[test]
fn test_edit_exits_browse() {
    let mut state = LineInputState::with_value("first");
    state.set_focused(true);
    state.update(LineInputMessage::Submit);

    // Browse to "first"
    state.update(LineInputMessage::HistoryPrev);
    assert!(state.is_browsing_history());

    // Edit while browsing exits browse mode
    state.update(LineInputMessage::Insert('!'));
    assert!(!state.is_browsing_history());
    assert_eq!(state.value(), "first!");
}

// ---- Undo / Redo ----

#[test]
fn test_undo_insert() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    state.update(LineInputMessage::Insert('a'));
    state.update(LineInputMessage::Insert(' ')); // whitespace breaks group
    state.update(LineInputMessage::Insert('b'));

    // Undo "b" → back to "a "
    let output = state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "a ");
    assert!(output.is_some());

    // Undo " " → back to "a"
    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "a");

    // Undo "a" → back to ""
    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "");
}

#[test]
fn test_redo() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    state.update(LineInputMessage::Insert('a'));
    state.update(LineInputMessage::Insert(' '));
    state.update(LineInputMessage::Insert('b'));

    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "a ");

    state.update(LineInputMessage::Redo);
    assert_eq!(state.value(), "a b");
}

#[test]
fn test_undo_empty() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let output = state.update(LineInputMessage::Undo);
    assert_eq!(output, None);
}

#[test]
fn test_redo_empty() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let output = state.update(LineInputMessage::Redo);
    assert_eq!(output, None);
}

#[test]
fn test_undo_clear() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.update(LineInputMessage::Clear);
    assert_eq!(state.value(), "");

    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "hello");
}

#[test]
fn test_undo_paste() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    state.update(LineInputMessage::Paste("hello world".to_string()));
    assert_eq!(state.value(), "hello world");

    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "");
}

#[test]
fn test_undo_backspace() {
    let mut state = LineInputState::with_value("hi");
    state.set_focused(true);
    state.update(LineInputMessage::Backspace);
    assert_eq!(state.value(), "h");

    state.update(LineInputMessage::Undo);
    assert_eq!(state.value(), "hi");
}

// ---- handle_event ----

#[test]
fn test_handle_event_unfocused() {
    let state = LineInputState::new();
    let event = Event::char('a');
    assert_eq!(state.handle_event(&event), None);
}

#[test]
fn test_handle_event_disabled() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    state.set_disabled(true);
    let event = Event::char('a');
    assert_eq!(state.handle_event(&event), None);
}

#[test]
fn test_handle_event_char() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let event = Event::char('a');
    assert_eq!(
        state.handle_event(&event),
        Some(LineInputMessage::Insert('a'))
    );
}

#[test]
fn test_handle_event_enter() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let event = Event::key(KeyCode::Enter);
    assert_eq!(state.handle_event(&event), Some(LineInputMessage::Submit));
}

#[test]
fn test_handle_event_backspace() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let event = Event::key(KeyCode::Backspace);
    assert_eq!(
        state.handle_event(&event),
        Some(LineInputMessage::Backspace)
    );
}

#[test]
fn test_handle_event_delete() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let event = Event::key(KeyCode::Delete);
    assert_eq!(state.handle_event(&event), Some(LineInputMessage::Delete));
}

#[test]
fn test_handle_event_arrows() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    assert_eq!(
        state.handle_event(&Event::key(KeyCode::Left)),
        Some(LineInputMessage::Left)
    );
    assert_eq!(
        state.handle_event(&Event::key(KeyCode::Right)),
        Some(LineInputMessage::Right)
    );
    assert_eq!(
        state.handle_event(&Event::key(KeyCode::Home)),
        Some(LineInputMessage::Home)
    );
    assert_eq!(
        state.handle_event(&Event::key(KeyCode::End)),
        Some(LineInputMessage::End)
    );
}

#[test]
fn test_handle_event_ctrl_keys() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    assert_eq!(
        state.handle_event(&Event::ctrl('z')),
        Some(LineInputMessage::Undo)
    );
    assert_eq!(
        state.handle_event(&Event::ctrl('y')),
        Some(LineInputMessage::Redo)
    );
    assert_eq!(
        state.handle_event(&Event::ctrl('a')),
        Some(LineInputMessage::SelectAll)
    );
    assert_eq!(
        state.handle_event(&Event::ctrl('u')),
        Some(LineInputMessage::Clear)
    );
    assert_eq!(
        state.handle_event(&Event::ctrl('c')),
        Some(LineInputMessage::Copy)
    );
    assert_eq!(
        state.handle_event(&Event::ctrl('x')),
        Some(LineInputMessage::Cut)
    );
}

#[test]
fn test_handle_event_shift_arrows() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::Left, KeyModifiers::SHIFT)),
        Some(LineInputMessage::SelectLeft)
    );
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::Right, KeyModifiers::SHIFT)),
        Some(LineInputMessage::SelectRight)
    );
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::Home, KeyModifiers::SHIFT)),
        Some(LineInputMessage::SelectHome)
    );
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::End, KeyModifiers::SHIFT)),
        Some(LineInputMessage::SelectEnd)
    );
}

#[test]
fn test_handle_event_ctrl_arrows() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::Left, KeyModifiers::CONTROL)),
        Some(LineInputMessage::WordLeft)
    );
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::Right, KeyModifiers::CONTROL)),
        Some(LineInputMessage::WordRight)
    );
}

#[test]
fn test_handle_event_ctrl_shift_arrows() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let mods = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::Left, mods)),
        Some(LineInputMessage::SelectWordLeft)
    );
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::Right, mods)),
        Some(LineInputMessage::SelectWordRight)
    );
}

#[test]
fn test_handle_event_ctrl_backspace() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::Backspace, KeyModifiers::CONTROL)),
        Some(LineInputMessage::DeleteWordBack)
    );
}

#[test]
fn test_handle_event_ctrl_delete() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    assert_eq!(
        state.handle_event(&Event::key_with(KeyCode::Delete, KeyModifiers::CONTROL)),
        Some(LineInputMessage::DeleteWordForward)
    );
}

// ---- handle_event: Up/Down context disambiguation ----

#[test]
fn test_up_on_first_row_is_history_prev() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.set_display_width(80);
    // Single row → cursor on row 0 → Up = HistoryPrev
    assert_eq!(
        state.handle_event(&Event::key(KeyCode::Up)),
        Some(LineInputMessage::HistoryPrev)
    );
}

#[test]
fn test_up_on_second_row_is_visual_up() {
    let mut state = LineInputState::with_value("hello world!");
    state.set_focused(true);
    state.set_display_width(5);
    // "hello" | " worl" | "d!" → cursor at end (row 2)
    assert_eq!(
        state.handle_event(&Event::key(KeyCode::Up)),
        Some(LineInputMessage::VisualUp)
    );
}

#[test]
fn test_down_on_last_row_is_history_next() {
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    state.set_display_width(80);
    // Single row → cursor on last row → Down = HistoryNext
    assert_eq!(
        state.handle_event(&Event::key(KeyCode::Down)),
        Some(LineInputMessage::HistoryNext)
    );
}

#[test]
fn test_down_on_first_row_is_visual_down() {
    let mut state = LineInputState::with_value("hello world!");
    state.set_focused(true);
    state.set_display_width(5);
    state.cursor = 0;
    // cursor at row 0, multiple rows → Down = VisualDown
    assert_eq!(
        state.handle_event(&Event::key(KeyCode::Down)),
        Some(LineInputMessage::VisualDown)
    );
}

// ---- handle_event: Paste ----

#[test]
fn test_handle_event_paste() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let event = Event::Paste("pasted text".to_string());
    assert_eq!(
        state.handle_event(&event),
        Some(LineInputMessage::Paste("pasted text".to_string()))
    );
}

// ---- dispatch_event ----

#[test]
fn test_dispatch_event() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    let output = state.dispatch_event(&Event::char('x'));
    assert_eq!(output, Some(LineInputOutput::Changed("x".to_string())));
    assert_eq!(state.value(), "x");
}

// ---- Disabled update guard ----

#[test]
fn test_update_disabled() {
    let mut state = LineInputState::new();
    state.set_focused(true);
    state.set_disabled(true);
    let output = state.update(LineInputMessage::Insert('a'));
    assert_eq!(output, None);
    assert_eq!(state.value(), "");
}

// ---- Cursor visual position ----

#[test]
fn test_cursor_visual_position() {
    let mut state = LineInputState::with_value("hello world!");
    state.set_display_width(5);
    // cursor at end → "hello" | " worl" | "d!" → row 2, col 2
    assert_eq!(state.cursor_visual_position(), (2, 2));
}

// ---- Snapshot tests ----

#[test]
fn test_snapshot_focused() {
    let (mut terminal, theme) = setup_render(20, 3);
    let mut state = LineInputState::with_value("hello");
    state.set_focused(true);
    terminal
        .draw(|frame| {
            LineInput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_unfocused() {
    let (mut terminal, theme) = setup_render(20, 3);
    let state = LineInputState::with_value("hello");
    terminal
        .draw(|frame| {
            LineInput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let (mut terminal, theme) = setup_render(20, 3);
    let state = LineInputState::with_value("hello").with_disabled(true);
    terminal
        .draw(|frame| {
            LineInput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_placeholder() {
    let (mut terminal, theme) = setup_render(20, 3);
    let state = LineInputState::new().with_placeholder("Type here...");
    terminal
        .draw(|frame| {
            LineInput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_wrapped() {
    let (mut terminal, theme) = setup_render(12, 5);
    let mut state = LineInputState::with_value("hello world!");
    state.set_focused(true);
    terminal
        .draw(|frame| {
            LineInput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_wide_chars() {
    let (mut terminal, theme) = setup_render(12, 4);
    let mut state = LineInputState::with_value("世界你好ab");
    state.set_focused(true);
    terminal
        .draw(|frame| {
            LineInput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_selection() {
    let (mut terminal, theme) = setup_render(20, 3);
    let mut state = LineInputState::with_value("hello world");
    state.set_focused(true);
    state.cursor = 0;
    state.selection_anchor = Some(5);
    terminal
        .draw(|frame| {
            LineInput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
