use super::*;
use crate::input::{Event, Key};

// ========== Construction Tests ==========

#[test]
fn test_new() {
    let state = CollapsibleState::new("Details");
    assert_eq!(state.header(), "Details");
    assert!(state.expanded());
    assert_eq!(state.content_height(), 5);
}

#[test]
fn test_new_from_string() {
    let state = CollapsibleState::new(String::from("Details"));
    assert_eq!(state.header(), "Details");
}

#[test]
fn test_default() {
    let state = CollapsibleState::default();
    assert_eq!(state.header(), "");
    assert!(state.expanded());
    assert_eq!(state.content_height(), 5);
}

#[test]
fn test_with_expanded_true() {
    let state = CollapsibleState::new("Details").with_expanded(true);
    assert!(state.expanded());
}

#[test]
fn test_with_expanded_false() {
    let state = CollapsibleState::new("Details").with_expanded(false);
    assert!(!state.expanded());
}

#[test]
fn test_with_content_height() {
    let state = CollapsibleState::new("Details").with_content_height(10);
    assert_eq!(state.content_height(), 10);
}

#[test]
fn test_builder_chaining() {
    let state = CollapsibleState::new("Settings")
        .with_expanded(false)
        .with_content_height(8);
    assert_eq!(state.header(), "Settings");
    assert!(!state.expanded());
    assert_eq!(state.content_height(), 8);
}

// ========== Accessor Tests ==========

#[test]
fn test_header() {
    let state = CollapsibleState::new("My Header");
    assert_eq!(state.header(), "My Header");
}

#[test]
fn test_set_header() {
    let mut state = CollapsibleState::new("Old");
    state.set_header("New");
    assert_eq!(state.header(), "New");
}

#[test]
fn test_set_header_from_string() {
    let mut state = CollapsibleState::new("Old");
    state.set_header(String::from("New"));
    assert_eq!(state.header(), "New");
}

#[test]
fn test_expanded() {
    let state = CollapsibleState::new("Details");
    assert!(state.expanded());
}

#[test]
fn test_is_expanded() {
    let state = CollapsibleState::new("Details");
    assert!(state.is_expanded());
    assert_eq!(state.expanded(), state.is_expanded());
}

#[test]
fn test_set_expanded() {
    let mut state = CollapsibleState::new("Details");
    state.set_expanded(false);
    assert!(!state.expanded());
    state.set_expanded(true);
    assert!(state.expanded());
}

#[test]
fn test_toggle() {
    let mut state = CollapsibleState::new("Details");
    assert!(state.expanded());
    state.toggle();
    assert!(!state.expanded());
    state.toggle();
    assert!(state.expanded());
}

#[test]
fn test_content_height() {
    let state = CollapsibleState::new("Details").with_content_height(7);
    assert_eq!(state.content_height(), 7);
}

#[test]
fn test_set_content_height() {
    let mut state = CollapsibleState::new("Details");
    state.set_content_height(12);
    assert_eq!(state.content_height(), 12);
}

// ========== content_area Tests ==========

#[test]
fn test_content_area_expanded() {
    let state = CollapsibleState::new("Details").with_content_height(5);
    let area = Rect::new(0, 0, 40, 10);
    let content = state.content_area(area);
    assert_eq!(content.x, 0);
    assert_eq!(content.y, 1);
    assert_eq!(content.width, 40);
    assert_eq!(content.height, 5);
}

#[test]
fn test_content_area_collapsed() {
    let state = CollapsibleState::new("Details").with_expanded(false);
    let area = Rect::new(0, 0, 40, 10);
    let content = state.content_area(area);
    assert_eq!(content.height, 0);
}

#[test]
fn test_content_area_limited_by_available_space() {
    let state = CollapsibleState::new("Details").with_content_height(20);
    let area = Rect::new(0, 0, 40, 6);
    let content = state.content_area(area);
    // 6 total - 1 header = 5 available
    assert_eq!(content.height, 5);
}

#[test]
fn test_content_area_no_height_for_content() {
    let state = CollapsibleState::new("Details");
    let area = Rect::new(0, 0, 40, 1);
    let content = state.content_area(area);
    assert_eq!(content.height, 0);
}

#[test]
fn test_content_area_zero_height_area() {
    let state = CollapsibleState::new("Details");
    let area = Rect::new(0, 0, 40, 0);
    let content = state.content_area(area);
    assert_eq!(content.height, 0);
}

#[test]
fn test_content_area_with_offset() {
    let state = CollapsibleState::new("Details").with_content_height(3);
    let area = Rect::new(5, 10, 30, 8);
    let content = state.content_area(area);
    assert_eq!(content.x, 5);
    assert_eq!(content.y, 11);
    assert_eq!(content.width, 30);
    assert_eq!(content.height, 3);
}

// ========== Update Tests ==========

#[test]
fn test_update_toggle_collapses() {
    let mut state = CollapsibleState::new("Details");
    assert!(state.expanded());
    let output = Collapsible::update(&mut state, CollapsibleMessage::Toggle);
    assert!(!state.expanded());
    assert_eq!(output, Some(CollapsibleOutput::Toggled(false)));
}

#[test]
fn test_update_toggle_expands() {
    let mut state = CollapsibleState::new("Details").with_expanded(false);
    let output = Collapsible::update(&mut state, CollapsibleMessage::Toggle);
    assert!(state.expanded());
    assert_eq!(output, Some(CollapsibleOutput::Toggled(true)));
}

#[test]
fn test_update_expand() {
    let mut state = CollapsibleState::new("Details").with_expanded(false);
    let output = Collapsible::update(&mut state, CollapsibleMessage::Expand);
    assert!(state.expanded());
    assert_eq!(output, Some(CollapsibleOutput::Expanded));
}

#[test]
fn test_update_expand_already_expanded() {
    let mut state = CollapsibleState::new("Details");
    let output = Collapsible::update(&mut state, CollapsibleMessage::Expand);
    assert!(state.expanded());
    assert_eq!(output, None);
}

#[test]
fn test_update_collapse() {
    let mut state = CollapsibleState::new("Details");
    let output = Collapsible::update(&mut state, CollapsibleMessage::Collapse);
    assert!(!state.expanded());
    assert_eq!(output, Some(CollapsibleOutput::Collapsed));
}

#[test]
fn test_update_collapse_already_collapsed() {
    let mut state = CollapsibleState::new("Details").with_expanded(false);
    let output = Collapsible::update(&mut state, CollapsibleMessage::Collapse);
    assert!(!state.expanded());
    assert_eq!(output, None);
}

#[test]
fn test_update_set_header() {
    let mut state = CollapsibleState::new("Old");
    let output = Collapsible::update(&mut state, CollapsibleMessage::SetHeader("New".to_string()));
    assert_eq!(state.header(), "New");
    assert_eq!(output, None);
}

#[test]
fn test_update_set_content_height() {
    let mut state = CollapsibleState::new("Details");
    let output = Collapsible::update(&mut state, CollapsibleMessage::SetContentHeight(15));
    assert_eq!(state.content_height(), 15);
    assert_eq!(output, None);
}

// ========== handle_event Tests ==========

#[test]
fn test_handle_event_space_toggles() {
    let state = CollapsibleState::new("Details");
    let msg = Collapsible::handle_event(
        &state,
        &Event::char(' '),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CollapsibleMessage::Toggle));
}

#[test]
fn test_handle_event_enter_toggles() {
    let state = CollapsibleState::new("Details");
    let msg = Collapsible::handle_event(
        &state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CollapsibleMessage::Toggle));
}

#[test]
fn test_handle_event_right_expands() {
    let state = CollapsibleState::new("Details");
    let msg = Collapsible::handle_event(
        &state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CollapsibleMessage::Expand));
}

#[test]
fn test_handle_event_left_collapses() {
    let state = CollapsibleState::new("Details");
    let msg = Collapsible::handle_event(
        &state,
        &Event::key(Key::Left),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(CollapsibleMessage::Collapse));
}

#[test]
fn test_handle_event_unfocused_ignores_events() {
    let state = CollapsibleState::new("Details");

    let msg = Collapsible::handle_event(&state, &Event::char(' '), &EventContext::default());
    assert_eq!(msg, None);

    let msg = Collapsible::handle_event(&state, &Event::key(Key::Enter), &EventContext::default());
    assert_eq!(msg, None);

    let msg = Collapsible::handle_event(&state, &Event::key(Key::Right), &EventContext::default());
    assert_eq!(msg, None);

    let msg = Collapsible::handle_event(&state, &Event::key(Key::Left), &EventContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled_ignores_events() {
    let state = CollapsibleState::new("Details");

    let msg = Collapsible::handle_event(
        &state,
        &Event::char(' '),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Collapsible::handle_event(
        &state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Collapsible::handle_event(
        &state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Collapsible::handle_event(
        &state,
        &Event::key(Key::Left),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_unrecognized_key_returns_none() {
    let state = CollapsibleState::new("Details");
    let msg = Collapsible::handle_event(
        &state,
        &Event::char('x'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, None);
}

// ========== dispatch_event Tests ==========

#[test]
fn test_dispatch_event_space_toggles() {
    let mut state = CollapsibleState::new("Details");
    let output = Collapsible::dispatch_event(
        &mut state,
        &Event::char(' '),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(CollapsibleOutput::Toggled(false)));
    assert!(!state.expanded());
}

#[test]
fn test_dispatch_event_enter_toggles() {
    let mut state = CollapsibleState::new("Details").with_expanded(false);
    let output = Collapsible::dispatch_event(
        &mut state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(CollapsibleOutput::Toggled(true)));
    assert!(state.expanded());
}

#[test]
fn test_dispatch_event_right_expands() {
    let mut state = CollapsibleState::new("Details").with_expanded(false);
    let output = Collapsible::dispatch_event(
        &mut state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(CollapsibleOutput::Expanded));
    assert!(state.expanded());
}

#[test]
fn test_dispatch_event_left_collapses() {
    let mut state = CollapsibleState::new("Details");
    let output = Collapsible::dispatch_event(
        &mut state,
        &Event::key(Key::Left),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(CollapsibleOutput::Collapsed));
    assert!(!state.expanded());
}

#[test]
fn test_dispatch_event_unfocused_returns_none() {
    let mut state = CollapsibleState::new("Details");
    let output =
        Collapsible::dispatch_event(&mut state, &Event::char(' '), &EventContext::default());
    assert_eq!(output, None);
    assert!(state.expanded()); // Unchanged
}

// ========== Instance Method Tests ==========

#[test]
fn test_instance_update() {
    let mut state = CollapsibleState::new("Details");
    let output = state.update(CollapsibleMessage::Collapse);
    assert_eq!(output, Some(CollapsibleOutput::Collapsed));
    assert!(!state.expanded());
}

// ========== Focusable Trait Tests ==========

// ========== Disableable Trait Tests ==========

// ========== Toggleable Trait Tests ==========

#[test]
fn test_toggleable_is_visible() {
    let state = CollapsibleState::new("Details");
    assert!(Collapsible::is_visible(&state));
}

#[test]
fn test_toggleable_is_visible_collapsed() {
    let state = CollapsibleState::new("Details").with_expanded(false);
    assert!(!Collapsible::is_visible(&state));
}

#[test]
fn test_toggleable_set_visible() {
    let mut state = CollapsibleState::new("Details");
    Collapsible::set_visible(&mut state, false);
    assert!(!state.expanded());
    Collapsible::set_visible(&mut state, true);
    assert!(state.expanded());
}

#[test]
fn test_toggleable_toggle() {
    let mut state = CollapsibleState::new("Details");
    assert!(Collapsible::is_visible(&state));
    Collapsible::toggle(&mut state);
    assert!(!Collapsible::is_visible(&state));
    Collapsible::toggle(&mut state);
    assert!(Collapsible::is_visible(&state));
}

#[test]
fn test_toggleable_show() {
    let mut state = CollapsibleState::new("Details").with_expanded(false);
    Collapsible::show(&mut state);
    assert!(state.expanded());
}

#[test]
fn test_toggleable_hide() {
    let mut state = CollapsibleState::new("Details");
    Collapsible::hide(&mut state);
    assert!(!state.expanded());
}

// ========== Init Tests ==========

#[test]
fn test_init() {
    let state = Collapsible::init();
    assert_eq!(state.header(), "");
    assert!(state.expanded());
}

// ========== View Tests ==========

#[test]
fn test_view_expanded() {
    let state = CollapsibleState::new("Details");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Collapsible::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_collapsed() {
    let state = CollapsibleState::new("Details").with_expanded(false);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Collapsible::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = CollapsibleState::new("Details");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Collapsible::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = CollapsibleState::new("Details");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Collapsible::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused_collapsed() {
    let state = CollapsibleState::new("Details").with_expanded(false);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Collapsible::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_zero_area() {
    let state = CollapsibleState::new("Details");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            let zero_area = Rect::new(0, 0, 0, 0);
            Collapsible::view(&state, &mut RenderContext::new(frame, zero_area, &theme));
        })
        .unwrap();
    // Should not panic
}

#[test]
fn test_view_height_one() {
    let state = CollapsibleState::new("Details");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 40, 1);
            Collapsible::view(&state, &mut RenderContext::new(frame, area, &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========== Annotation Tests ==========

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;

    let state = CollapsibleState::new("Details");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Collapsible::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.regions();
    assert_eq!(regions.len(), 1);
    let annotation = &regions[0].annotation;
    assert!(!annotation.focused);
    assert!(!annotation.disabled);
    assert_eq!(annotation.expanded, Some(true));
}

#[test]
fn test_annotation_reflects_state() {
    use crate::annotation::with_annotations;

    let state = CollapsibleState::new("Details").with_expanded(false);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Collapsible::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme)
                        .focused(true)
                        .disabled(true),
                );
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.regions();
    let annotation = &regions[0].annotation;
    assert!(annotation.focused);
    assert!(annotation.disabled);
    assert_eq!(annotation.expanded, Some(false));
}
