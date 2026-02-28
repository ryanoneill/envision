use super::*;

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
fn test_label_accessors() {
    let mut state = ButtonState::new("Original");
    assert_eq!(state.label(), "Original");

    state.set_label("Updated");
    assert_eq!(state.label(), "Updated");
}

#[test]
fn test_disabled_accessors() {
    let mut state = ButtonState::new("Test");
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());

    state.set_disabled(false);
    assert!(!state.is_disabled());
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
fn test_focusable() {
    let mut state = ButtonState::new("Test");

    assert!(!Button::is_focused(&state));

    Button::set_focused(&mut state, true);
    assert!(Button::is_focused(&state));

    Button::blur(&mut state);
    assert!(!Button::is_focused(&state));

    Button::focus(&mut state);
    assert!(Button::is_focused(&state));
}

#[test]
fn test_init() {
    let state = Button::init();
    assert_eq!(state.label(), "");
    assert!(!state.is_disabled());
    assert!(!Button::is_focused(&state));
}

#[test]
fn test_clone() {
    let state = ButtonState::new("Clone me");
    let cloned = state.clone();
    assert_eq!(cloned.label(), "Clone me");
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
