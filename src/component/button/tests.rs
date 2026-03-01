use super::*;
use crate::input::{Event, KeyCode};

#[test]
fn test_new() {
    let state = ButtonState::new("Click me");
    assert_eq!(state.label(), "Click me");
    assert!(!state.is_disabled());
    assert!(!Button::is_focused(&state));
}

#[test]
fn test_default() {
    let state = ButtonState::default();
    assert_eq!(state.label(), "");
    assert!(!state.is_disabled());
    assert!(!Button::is_focused(&state));
}

#[test]
fn test_press_enabled() {
    let mut state = ButtonState::new("Submit");

    let output = Button::update(&mut state, ButtonMessage::Press);
    assert_eq!(output, Some(ButtonOutput::Pressed));
}

#[test]
fn test_press_disabled() {
    let mut state = ButtonState::new("Submit");
    state.set_disabled(true);

    let output = Button::update(&mut state, ButtonMessage::Press);
    assert_eq!(output, None);
}

#[test]
fn test_init() {
    let state = Button::init();
    assert_eq!(state.label(), "");
    assert!(!state.is_disabled());
    assert!(!Button::is_focused(&state));
}

#[test]
fn test_view() {
    let state = ButtonState::new("Click");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 5);

    terminal
        .draw(|frame| {
            Button::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let mut state = ButtonState::new("Focused");
    Button::set_focused(&mut state, true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 5);

    terminal
        .draw(|frame| {
            Button::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let mut state = ButtonState::new("Disabled");
    state.set_disabled(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 5);

    terminal
        .draw(|frame| {
            Button::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// handle_event tests

#[test]
fn test_handle_event_enter_when_focused() {
    let mut state = ButtonState::new("OK");
    Button::set_focused(&mut state, true);
    let msg = Button::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(ButtonMessage::Press));
}

#[test]
fn test_handle_event_space_when_focused() {
    let mut state = ButtonState::new("OK");
    Button::set_focused(&mut state, true);
    let msg = Button::handle_event(&state, &Event::char(' '));
    assert_eq!(msg, Some(ButtonMessage::Press));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = ButtonState::new("OK");
    let msg = Button::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let mut state = ButtonState::new("OK");
    Button::set_focused(&mut state, true);
    state.set_disabled(true);
    let msg = Button::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_irrelevant_key() {
    let mut state = ButtonState::new("OK");
    Button::set_focused(&mut state, true);
    let msg = Button::handle_event(&state, &Event::char('x'));
    assert_eq!(msg, None);
}

#[test]
fn test_dispatch_event() {
    let mut state = ButtonState::new("OK");
    Button::set_focused(&mut state, true);
    let output = Button::dispatch_event(&mut state, &Event::key(KeyCode::Enter));
    assert_eq!(output, Some(ButtonOutput::Pressed));
}

#[test]
fn test_instance_is_focused() {
    let mut state = ButtonState::new("OK");
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
}

#[test]
fn test_instance_handle_event() {
    let mut state = ButtonState::new("OK");
    state.set_focused(true);
    let msg = state.handle_event(&Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(ButtonMessage::Press));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = ButtonState::new("OK");
    state.set_focused(true);
    let output = state.dispatch_event(&Event::key(KeyCode::Enter));
    assert_eq!(output, Some(ButtonOutput::Pressed));
}

#[test]
fn test_instance_update() {
    let mut state = ButtonState::new("OK");
    let output = state.update(ButtonMessage::Press);
    assert_eq!(output, Some(ButtonOutput::Pressed));
}
