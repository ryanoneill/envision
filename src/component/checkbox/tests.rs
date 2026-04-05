use super::*;
use crate::input::{Event, KeyCode};

#[test]
fn test_new() {
    let state = CheckboxState::new("Test label");
    assert_eq!(state.label(), "Test label");
    assert!(!state.is_checked());
}

#[test]
fn test_checked_constructor() {
    let state = CheckboxState::checked("Checked label");
    assert_eq!(state.label(), "Checked label");
    assert!(state.is_checked());
}

#[test]
fn test_default() {
    let state = CheckboxState::default();
    assert_eq!(state.label(), "");
    assert!(!state.is_checked());
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
fn test_init() {
    let state = Checkbox::init();
    assert_eq!(state.label(), "");
    assert!(!state.is_checked());
}

#[test]
fn test_view_unchecked() {
    let state = CheckboxState::new("Unchecked");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);

    terminal
        .draw(|frame| {
            Checkbox::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Checkbox::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = CheckboxState::new("Focused");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);

    terminal
        .draw(|frame| {
            Checkbox::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = CheckboxState::new("Disabled");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 5);

    terminal
        .draw(|frame| {
            Checkbox::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
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
    let state = CheckboxState::new("Test");
    let msg = Checkbox::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(CheckboxMessage::Toggle));
}

#[test]
fn test_handle_event_space_when_focused() {
    let state = CheckboxState::new("Test");
    let msg = Checkbox::handle_event(&state, &Event::char(' '), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(CheckboxMessage::Toggle));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = CheckboxState::new("Test");
    let msg = Checkbox::handle_event(&state, &Event::key(KeyCode::Enter), &ViewContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let state = CheckboxState::new("Test");
    let msg = Checkbox::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_dispatch_event() {
    let mut state = CheckboxState::new("Test");
    let output = Checkbox::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
}

#[test]
fn test_instance_update() {
    let mut state = CheckboxState::new("Test");
    let output = state.update(CheckboxMessage::Toggle);
    assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = CheckboxState::new("Accept TOS");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Checkbox::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Checkbox);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("Accept TOS".to_string()));
    assert!(!regions[0].annotation.selected);
}

#[test]
fn test_annotation_checked() {
    use crate::annotation::{WidgetType, with_annotations};
    let mut state = CheckboxState::new("Accept");
    Checkbox::update(&mut state, CheckboxMessage::Toggle);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Checkbox::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    let regions = registry.find_by_type(&WidgetType::Checkbox);
    assert!(regions[0].annotation.selected);
}
