use super::*;
use crate::input::{Event, KeyCode};

#[test]
fn test_new() {
    let state = CheckboxState::new("Test label");
    assert_eq!(state.label(), "Test label");
    assert!(!state.is_checked());
    assert!(!state.is_disabled());
    assert!(!Checkbox::is_focused(&state));
}

#[test]
fn test_checked_constructor() {
    let state = CheckboxState::checked("Checked label");
    assert_eq!(state.label(), "Checked label");
    assert!(state.is_checked());
    assert!(!state.is_disabled());
}

#[test]
fn test_default() {
    let state = CheckboxState::default();
    assert_eq!(state.label(), "");
    assert!(!state.is_checked());
    assert!(!state.is_disabled());
    assert!(!Checkbox::is_focused(&state));
}

#[test]
fn test_toggle_unchecked() {
    let mut state = CheckboxState::new("Test");
    assert!(!state.is_checked());

    let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
    assert!(state.is_checked());
}

#[test]
fn test_toggle_checked() {
    let mut state = CheckboxState::checked("Test");
    assert!(state.is_checked());

    let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
    assert_eq!(output, Some(CheckboxOutput::Toggled(false)));
    assert!(!state.is_checked());
}

#[test]
fn test_toggle_disabled() {
    let mut state = CheckboxState::new("Test");
    state.set_disabled(true);

    let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
    assert_eq!(output, None);
    assert!(!state.is_checked()); // State unchanged
}

#[test]
fn test_toggle_disabled_when_checked() {
    let mut state = CheckboxState::checked("Test");
    state.set_disabled(true);

    let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
    assert_eq!(output, None);
    assert!(state.is_checked()); // State unchanged
}

#[test]
fn test_init() {
    let state = Checkbox::init();
    assert_eq!(state.label(), "");
    assert!(!state.is_checked());
    assert!(!state.is_disabled());
    assert!(!Checkbox::is_focused(&state));
}

#[test]
fn test_view_unchecked() {
    let state = CheckboxState::new("Unchecked");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);

    terminal
        .draw(|frame| {
            Checkbox::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_checked() {
    let state = CheckboxState::checked("Checked");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);

    terminal
        .draw(|frame| {
            Checkbox::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let mut state = CheckboxState::new("Focused");
    Checkbox::set_focused(&mut state, true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);

    terminal
        .draw(|frame| {
            Checkbox::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let mut state = CheckboxState::new("Disabled");
    state.set_disabled(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);

    terminal
        .draw(|frame| {
            Checkbox::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_multiple_toggles() {
    let mut state = CheckboxState::new("Test");

    // Toggle multiple times
    Checkbox::update(&mut state, CheckboxMessage::Toggle);
    assert!(state.is_checked());

    Checkbox::update(&mut state, CheckboxMessage::Toggle);
    assert!(!state.is_checked());

    Checkbox::update(&mut state, CheckboxMessage::Toggle);
    assert!(state.is_checked());
}

// handle_event tests

#[test]
fn test_handle_event_enter_when_focused() {
    let mut state = CheckboxState::new("Test");
    Checkbox::set_focused(&mut state, true);
    let msg = Checkbox::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(CheckboxMessage::Toggle));
}

#[test]
fn test_handle_event_space_when_focused() {
    let mut state = CheckboxState::new("Test");
    Checkbox::set_focused(&mut state, true);
    let msg = Checkbox::handle_event(&state, &Event::char(' '));
    assert_eq!(msg, Some(CheckboxMessage::Toggle));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = CheckboxState::new("Test");
    let msg = Checkbox::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let mut state = CheckboxState::new("Test");
    Checkbox::set_focused(&mut state, true);
    state.set_disabled(true);
    let msg = Checkbox::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);
}

#[test]
fn test_dispatch_event() {
    let mut state = CheckboxState::new("Test");
    Checkbox::set_focused(&mut state, true);
    let output = Checkbox::dispatch_event(&mut state, &Event::key(KeyCode::Enter));
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
}

#[test]
fn test_instance_is_focused() {
    let mut state = CheckboxState::new("Test");
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
}

#[test]
fn test_instance_handle_event() {
    let mut state = CheckboxState::new("Test");
    state.set_focused(true);
    let msg = state.handle_event(&Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(CheckboxMessage::Toggle));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = CheckboxState::new("Test");
    state.set_focused(true);
    let output = state.dispatch_event(&Event::key(KeyCode::Enter));
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
}

#[test]
fn test_instance_update() {
    let mut state = CheckboxState::new("Test");
    let output = state.update(CheckboxMessage::Toggle);
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
}
