use super::*;
use crate::input::{Event, KeyCode, KeyModifiers};

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
    // Single row -> cursor on row 0 -> Up = HistoryPrev
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
    // "hello" | " worl" | "d!" -> cursor at end (row 2)
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
    // Single row -> cursor on last row -> Down = HistoryNext
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
    // cursor at row 0, multiple rows -> Down = VisualDown
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
