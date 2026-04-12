use super::*;
use crate::input::{Event, Key};

#[test]
fn test_new() {
    let state = ButtonState::new("Click me");
    assert_eq!(state.label(), "Click me");
}

#[test]
fn test_default() {
    let state = ButtonState::default();
    assert_eq!(state.label(), "");
}

#[test]
fn test_press_enabled() {
    let mut state = ButtonState::new("Submit");

    let output = Button::update(&mut state, ButtonMessage::Press);
    assert_eq!(output, Some(ButtonOutput::Pressed));
}

#[test]
fn test_init() {
    let state = Button::init();
    assert_eq!(state.label(), "");
}

#[test]
fn test_view() {
    let state = ButtonState::new("Click");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 5);

    terminal
        .draw(|frame| {
            Button::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = ButtonState::new("Focused");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 5);

    terminal
        .draw(|frame| {
            Button::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = ButtonState::new("Disabled");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 5);

    terminal
        .draw(|frame| {
            Button::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// handle_event tests

#[test]
fn test_handle_event_enter_when_focused() {
    let state = ButtonState::new("OK");
    let msg = Button::handle_event(
        &state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(ButtonMessage::Press));
}

#[test]
fn test_handle_event_space_when_focused() {
    let state = ButtonState::new("OK");
    let msg = Button::handle_event(
        &state,
        &Event::char(' '),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(ButtonMessage::Press));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = ButtonState::new("OK");
    let msg = Button::handle_event(&state, &Event::key(Key::Enter), &EventContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let state = ButtonState::new("OK");
    let msg = Button::handle_event(
        &state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_irrelevant_key() {
    let state = ButtonState::new("OK");
    let msg = Button::handle_event(
        &state,
        &Event::char('x'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_dispatch_event() {
    let mut state = ButtonState::new("OK");
    let output = Button::dispatch_event(
        &mut state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(ButtonOutput::Pressed));
}

#[test]
fn test_instance_update() {
    let mut state = ButtonState::new("OK");
    let output = state.update(ButtonMessage::Press);
    assert_eq!(output, Some(ButtonOutput::Pressed));
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = ButtonState::new("Submit");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Button::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Button);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("Submit".to_string()));
    assert!(!regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}

#[test]
fn test_annotation_focused() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = ButtonState::new("OK");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(20, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Button::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
                );
            })
            .unwrap();
    });
    let regions = registry.find_by_type(&WidgetType::Button);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.focused);
}
