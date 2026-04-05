use super::*;
use crate::input::{Event, KeyCode, KeyModifiers};

// ---- handle_event ----

#[test]
fn test_handle_event_unfocused() {
    let state = LineInputState::new();
    let event = Event::char('a');
    assert_eq!(
        LineInput::handle_event(&state, &event, &ViewContext::default()),
        None
    );
}

#[test]
fn test_handle_event_disabled() {
    let state = LineInputState::new();
    let event = Event::char('a');
    assert_eq!(
        LineInput::handle_event(
            &state,
            &event,
            &ViewContext::new().focused(true).disabled(true)
        ),
        None
    );
}

#[test]
fn test_handle_event_char() {
    let state = LineInputState::new();
    let event = Event::char('a');
    assert_eq!(
        LineInput::handle_event(&state, &event, &ViewContext::new().focused(true)),
        Some(LineInputMessage::Insert('a'))
    );
}

#[test]
fn test_handle_event_enter() {
    let state = LineInputState::new();
    let event = Event::key(KeyCode::Enter);
    assert_eq!(
        LineInput::handle_event(&state, &event, &ViewContext::new().focused(true)),
        Some(LineInputMessage::Submit)
    );
}

#[test]
fn test_handle_event_backspace() {
    let state = LineInputState::new();
    let event = Event::key(KeyCode::Backspace);
    assert_eq!(
        LineInput::handle_event(&state, &event, &ViewContext::new().focused(true)),
        Some(LineInputMessage::Backspace)
    );
}

#[test]
fn test_handle_event_delete() {
    let state = LineInputState::new();
    let event = Event::key(KeyCode::Delete);
    assert_eq!(
        LineInput::handle_event(&state, &event, &ViewContext::new().focused(true)),
        Some(LineInputMessage::Delete)
    );
}

#[test]
fn test_handle_event_arrows() {
    let state = LineInputState::new();
    let ctx = ViewContext::new().focused(true);
    assert_eq!(
        LineInput::handle_event(&state, &Event::key(KeyCode::Left), &ctx),
        Some(LineInputMessage::Left)
    );
    assert_eq!(
        LineInput::handle_event(&state, &Event::key(KeyCode::Right), &ctx),
        Some(LineInputMessage::Right)
    );
    assert_eq!(
        LineInput::handle_event(&state, &Event::key(KeyCode::Home), &ctx),
        Some(LineInputMessage::Home)
    );
    assert_eq!(
        LineInput::handle_event(&state, &Event::key(KeyCode::End), &ctx),
        Some(LineInputMessage::End)
    );
}

#[test]
fn test_handle_event_ctrl_keys() {
    let state = LineInputState::new();
    let ctx = ViewContext::new().focused(true);
    assert_eq!(
        LineInput::handle_event(&state, &Event::ctrl('z'), &ctx),
        Some(LineInputMessage::Undo)
    );
    assert_eq!(
        LineInput::handle_event(&state, &Event::ctrl('y'), &ctx),
        Some(LineInputMessage::Redo)
    );
    assert_eq!(
        LineInput::handle_event(&state, &Event::ctrl('a'), &ctx),
        Some(LineInputMessage::SelectAll)
    );
    assert_eq!(
        LineInput::handle_event(&state, &Event::ctrl('u'), &ctx),
        Some(LineInputMessage::Clear)
    );
    assert_eq!(
        LineInput::handle_event(&state, &Event::ctrl('c'), &ctx),
        Some(LineInputMessage::Copy)
    );
    assert_eq!(
        LineInput::handle_event(&state, &Event::ctrl('x'), &ctx),
        Some(LineInputMessage::Cut)
    );
}

#[test]
fn test_handle_event_shift_arrows() {
    let state = LineInputState::new();
    let ctx = ViewContext::new().focused(true);
    assert_eq!(
        LineInput::handle_event(
            &state,
            &Event::key_with(KeyCode::Left, KeyModifiers::SHIFT),
            &ctx
        ),
        Some(LineInputMessage::SelectLeft)
    );
    assert_eq!(
        LineInput::handle_event(
            &state,
            &Event::key_with(KeyCode::Right, KeyModifiers::SHIFT),
            &ctx
        ),
        Some(LineInputMessage::SelectRight)
    );
    assert_eq!(
        LineInput::handle_event(
            &state,
            &Event::key_with(KeyCode::Home, KeyModifiers::SHIFT),
            &ctx
        ),
        Some(LineInputMessage::SelectHome)
    );
    assert_eq!(
        LineInput::handle_event(
            &state,
            &Event::key_with(KeyCode::End, KeyModifiers::SHIFT),
            &ctx
        ),
        Some(LineInputMessage::SelectEnd)
    );
}

#[test]
fn test_handle_event_ctrl_arrows() {
    let state = LineInputState::new();
    let ctx = ViewContext::new().focused(true);
    assert_eq!(
        LineInput::handle_event(
            &state,
            &Event::key_with(KeyCode::Left, KeyModifiers::CONTROL),
            &ctx
        ),
        Some(LineInputMessage::WordLeft)
    );
    assert_eq!(
        LineInput::handle_event(
            &state,
            &Event::key_with(KeyCode::Right, KeyModifiers::CONTROL),
            &ctx
        ),
        Some(LineInputMessage::WordRight)
    );
}

#[test]
fn test_handle_event_ctrl_shift_arrows() {
    let state = LineInputState::new();
    let ctx = ViewContext::new().focused(true);
    let mods = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    assert_eq!(
        LineInput::handle_event(&state, &Event::key_with(KeyCode::Left, mods), &ctx),
        Some(LineInputMessage::SelectWordLeft)
    );
    assert_eq!(
        LineInput::handle_event(&state, &Event::key_with(KeyCode::Right, mods), &ctx),
        Some(LineInputMessage::SelectWordRight)
    );
}

#[test]
fn test_handle_event_ctrl_backspace() {
    let state = LineInputState::new();
    let ctx = ViewContext::new().focused(true);
    assert_eq!(
        LineInput::handle_event(
            &state,
            &Event::key_with(KeyCode::Backspace, KeyModifiers::CONTROL),
            &ctx
        ),
        Some(LineInputMessage::DeleteWordBack)
    );
}

#[test]
fn test_handle_event_ctrl_delete() {
    let state = LineInputState::new();
    let ctx = ViewContext::new().focused(true);
    assert_eq!(
        LineInput::handle_event(
            &state,
            &Event::key_with(KeyCode::Delete, KeyModifiers::CONTROL),
            &ctx
        ),
        Some(LineInputMessage::DeleteWordForward)
    );
}

// ---- handle_event: Up/Down context disambiguation ----

#[test]
fn test_up_on_first_row_is_history_prev() {
    let mut state = LineInputState::with_value("hello");
    state.set_display_width(80);
    let ctx = ViewContext::new().focused(true);
    // Single row -> cursor on row 0 -> Up = HistoryPrev
    assert_eq!(
        LineInput::handle_event(&state, &Event::key(KeyCode::Up), &ctx),
        Some(LineInputMessage::HistoryPrev)
    );
}

#[test]
fn test_up_on_second_row_is_visual_up() {
    let mut state = LineInputState::with_value("hello world!");
    state.set_display_width(5);
    let ctx = ViewContext::new().focused(true);
    // "hello" | " worl" | "d!" -> cursor at end (row 2)
    assert_eq!(
        LineInput::handle_event(&state, &Event::key(KeyCode::Up), &ctx),
        Some(LineInputMessage::VisualUp)
    );
}

#[test]
fn test_down_on_last_row_is_history_next() {
    let mut state = LineInputState::with_value("hello");
    state.set_display_width(80);
    let ctx = ViewContext::new().focused(true);
    // Single row -> cursor on last row -> Down = HistoryNext
    assert_eq!(
        LineInput::handle_event(&state, &Event::key(KeyCode::Down), &ctx),
        Some(LineInputMessage::HistoryNext)
    );
}

#[test]
fn test_down_on_first_row_is_visual_down() {
    let mut state = LineInputState::with_value("hello world!");
    state.set_display_width(5);
    state.cursor = 0;
    let ctx = ViewContext::new().focused(true);
    // cursor at row 0, multiple rows -> Down = VisualDown
    assert_eq!(
        LineInput::handle_event(&state, &Event::key(KeyCode::Down), &ctx),
        Some(LineInputMessage::VisualDown)
    );
}

// ---- handle_event: Paste ----

#[test]
fn test_handle_event_paste() {
    let state = LineInputState::new();
    let ctx = ViewContext::new().focused(true);
    let event = Event::Paste("pasted text".to_string());
    assert_eq!(
        LineInput::handle_event(&state, &event, &ctx),
        Some(LineInputMessage::Paste("pasted text".to_string()))
    );
}
