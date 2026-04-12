use super::*;
use crate::input::Key;

fn sample_items() -> Vec<PaletteItem> {
    vec![
        PaletteItem::new("open", "Open File"),
        PaletteItem::new("save", "Save File"),
        PaletteItem::new("quit", "Quit"),
    ]
}

fn active_state() -> CommandPaletteState {
    let mut state = CommandPaletteState::new(sample_items());
    state.set_visible(true);
    state
}

#[test]
fn test_char_maps_to_type_char() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::char('a'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::TypeChar('a')));
}

#[test]
fn test_uppercase_char_maps_to_type_char() {
    let state = active_state();
    // Event::char('A') normalizes to Key::Char('a') with SHIFT and raw_char='A'
    let msg = CommandPalette::handle_event(
        &state,
        &Event::char('A'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::TypeChar('A')));
}

#[test]
fn test_backspace_maps_to_backspace() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::key(Key::Backspace),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::Backspace));
}

#[test]
fn test_enter_maps_to_confirm() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::Confirm));
}

#[test]
fn test_escape_maps_to_dismiss() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::key(Key::Esc),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::Dismiss));
}

#[test]
fn test_up_maps_to_select_prev() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::key(Key::Up),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::SelectPrev));
}

#[test]
fn test_down_maps_to_select_next() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::key(Key::Down),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::SelectNext));
}

#[test]
fn test_ctrl_p_maps_to_select_prev() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::ctrl('p'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::SelectPrev));
}

#[test]
fn test_ctrl_n_maps_to_select_next() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::ctrl('n'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::SelectNext));
}

#[test]
fn test_ctrl_u_maps_to_clear_query() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::ctrl('u'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CommandPaletteMessage::ClearQuery));
}

#[test]
fn test_unfocused_ignores_all_events() {
    let mut state = CommandPaletteState::new(sample_items());
    state.set_visible(true);
    // focused is false

    assert_eq!(
        CommandPalette::handle_event(&state, &Event::char('a'), &EventContext::default()),
        None
    );
    assert_eq!(
        CommandPalette::handle_event(&state, &Event::key(Key::Enter), &EventContext::default()),
        None
    );
    assert_eq!(
        CommandPalette::handle_event(&state, &Event::key(Key::Esc), &EventContext::default()),
        None
    );
}

#[test]
fn test_disabled_ignores_all_events() {
    let state = active_state();

    assert_eq!(
        CommandPalette::handle_event(
            &state,
            &Event::char('a'),
            &EventContext::new().focused(true).disabled(true)
        ),
        None
    );
    assert_eq!(
        CommandPalette::handle_event(
            &state,
            &Event::key(Key::Enter),
            &EventContext::new().focused(true).disabled(true)
        ),
        None
    );
}

#[test]
fn test_hidden_ignores_all_events() {
    let state = CommandPaletteState::new(sample_items());
    // visible is false

    assert_eq!(
        CommandPalette::handle_event(
            &state,
            &Event::char('a'),
            &EventContext::new().focused(true)
        ),
        None
    );
    assert_eq!(
        CommandPalette::handle_event(
            &state,
            &Event::key(Key::Enter),
            &EventContext::new().focused(true)
        ),
        None
    );
}

#[test]
fn test_unrecognized_key_returns_none() {
    let state = active_state();
    let msg = CommandPalette::handle_event(
        &state,
        &Event::key(Key::F(1)),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, None);
}
