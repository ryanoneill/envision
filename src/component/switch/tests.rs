use super::*;
use crate::input::{Event, KeyCode};

// ========================================
// Construction Tests
// ========================================

#[test]
fn test_new() {
    let state = SwitchState::new();
    assert!(!state.is_on());
    assert!(state.label().is_none());
    assert!(!state.is_disabled());
    assert!(!state.is_focused());
}

#[test]
fn test_default() {
    let state = SwitchState::default();
    assert!(!state.is_on());
    assert!(state.label().is_none());
    assert!(!state.is_disabled());
    assert!(!Switch::is_focused(&state));
}

#[test]
fn test_init() {
    let state = Switch::init();
    assert!(!state.is_on());
    assert!(state.label().is_none());
    assert!(!state.is_disabled());
    assert!(!Switch::is_focused(&state));
}

#[test]
fn test_with_on_true() {
    let state = SwitchState::new().with_on(true);
    assert!(state.is_on());
}

#[test]
fn test_with_on_false() {
    let state = SwitchState::new().with_on(false);
    assert!(!state.is_on());
}

#[test]
fn test_with_label() {
    let state = SwitchState::new().with_label("Dark Mode");
    assert_eq!(state.label(), Some("Dark Mode"));
}

#[test]
fn test_with_on_label() {
    let state = SwitchState::new().with_on_label("YES");
    assert_eq!(state.on_label, "YES");
}

#[test]
fn test_with_off_label() {
    let state = SwitchState::new().with_off_label("NO");
    assert_eq!(state.off_label, "NO");
}

#[test]
fn test_with_disabled() {
    let state = SwitchState::new().with_disabled(true);
    assert!(state.is_disabled());

    let state = SwitchState::new().with_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_builder_chaining() {
    let state = SwitchState::new()
        .with_on(true)
        .with_label("Wi-Fi")
        .with_on_label("ENABLED")
        .with_off_label("DISABLED")
        .with_disabled(false);
    assert!(state.is_on());
    assert_eq!(state.label(), Some("Wi-Fi"));
    assert_eq!(state.on_label, "ENABLED");
    assert_eq!(state.off_label, "DISABLED");
    assert!(!state.is_disabled());
}

// ========================================
// Toggle / State Mutation Tests
// ========================================

#[test]
fn test_toggle_off_to_on() {
    let mut state = SwitchState::new();
    assert!(!state.is_on());

    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, Some(SwitchOutput::On));
    assert!(state.is_on());
}

#[test]
fn test_toggle_on_to_off() {
    let mut state = SwitchState::new().with_on(true);
    assert!(state.is_on());

    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, Some(SwitchOutput::Off));
    assert!(!state.is_on());
}

#[test]
fn test_toggle_disabled() {
    let mut state = SwitchState::new().with_disabled(true);

    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, None);
    assert!(!state.is_on());
}

#[test]
fn test_toggle_disabled_when_on() {
    let mut state = SwitchState::new().with_on(true).with_disabled(true);

    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, None);
    assert!(state.is_on());
}

#[test]
fn test_set_on_true() {
    let mut state = SwitchState::new();
    let output = Switch::update(&mut state, SwitchMessage::SetOn(true));
    assert_eq!(output, Some(SwitchOutput::Toggled(true)));
    assert!(state.is_on());
}

#[test]
fn test_set_on_false() {
    let mut state = SwitchState::new().with_on(true);
    let output = Switch::update(&mut state, SwitchMessage::SetOn(false));
    assert_eq!(output, Some(SwitchOutput::Toggled(false)));
    assert!(!state.is_on());
}

#[test]
fn test_set_on_same_value() {
    let mut state = SwitchState::new();
    let output = Switch::update(&mut state, SwitchMessage::SetOn(false));
    assert_eq!(output, None);
    assert!(!state.is_on());
}

#[test]
fn test_set_on_disabled() {
    let mut state = SwitchState::new().with_disabled(true);
    let output = Switch::update(&mut state, SwitchMessage::SetOn(true));
    assert_eq!(output, None);
    assert!(!state.is_on());
}

#[test]
fn test_set_label_message() {
    let mut state = SwitchState::new();
    let output = Switch::update(
        &mut state,
        SwitchMessage::SetLabel(Some("Test".to_string())),
    );
    assert_eq!(output, None);
    assert_eq!(state.label(), Some("Test"));
}

#[test]
fn test_set_label_message_none() {
    let mut state = SwitchState::new().with_label("Test");
    let output = Switch::update(&mut state, SwitchMessage::SetLabel(None));
    assert_eq!(output, None);
    assert_eq!(state.label(), None);
}

#[test]
fn test_set_on_direct() {
    let mut state = SwitchState::new();
    state.set_on(true);
    assert!(state.is_on());
    state.set_on(false);
    assert!(!state.is_on());
}

#[test]
fn test_toggle_direct() {
    let mut state = SwitchState::new();
    assert!(!state.is_on());
    state.toggle();
    assert!(state.is_on());
    state.toggle();
    assert!(!state.is_on());
}

#[test]
fn test_set_label_direct() {
    let mut state = SwitchState::new();
    state.set_label(Some("Label".to_string()));
    assert_eq!(state.label(), Some("Label"));
    state.set_label(None);
    assert_eq!(state.label(), None);
}

#[test]
fn test_multiple_toggles() {
    let mut state = SwitchState::new();

    Switch::update(&mut state, SwitchMessage::Toggle);
    assert!(state.is_on());

    Switch::update(&mut state, SwitchMessage::Toggle);
    assert!(!state.is_on());

    Switch::update(&mut state, SwitchMessage::Toggle);
    assert!(state.is_on());
}

// ========================================
// Event Handling Tests
// ========================================

#[test]
fn test_handle_event_enter_when_focused() {
    let mut state = SwitchState::new();
    Switch::set_focused(&mut state, true);
    let msg = Switch::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(SwitchMessage::Toggle));
}

#[test]
fn test_handle_event_space_when_focused() {
    let mut state = SwitchState::new();
    Switch::set_focused(&mut state, true);
    let msg = Switch::handle_event(&state, &Event::char(' '));
    assert_eq!(msg, Some(SwitchMessage::Toggle));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = SwitchState::new();
    let msg = Switch::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let mut state = SwitchState::new();
    Switch::set_focused(&mut state, true);
    state.set_disabled(true);
    let msg = Switch::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_other_key_ignored() {
    let mut state = SwitchState::new();
    Switch::set_focused(&mut state, true);
    let msg = Switch::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);
}

#[test]
fn test_dispatch_event() {
    let mut state = SwitchState::new();
    Switch::set_focused(&mut state, true);
    let output = Switch::dispatch_event(&mut state, &Event::key(KeyCode::Enter));
    assert_eq!(output, Some(SwitchOutput::On));
    assert!(state.is_on());
}

#[test]
fn test_dispatch_event_unfocused() {
    let mut state = SwitchState::new();
    let output = Switch::dispatch_event(&mut state, &Event::key(KeyCode::Enter));
    assert_eq!(output, None);
    assert!(!state.is_on());
}

// ========================================
// Instance Method Tests
// ========================================

#[test]
fn test_instance_is_focused() {
    let mut state = SwitchState::new();
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
}

#[test]
fn test_instance_handle_event() {
    let mut state = SwitchState::new();
    state.set_focused(true);
    let msg = state.handle_event(&Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(SwitchMessage::Toggle));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = SwitchState::new();
    state.set_focused(true);
    let output = state.dispatch_event(&Event::key(KeyCode::Enter));
    assert_eq!(output, Some(SwitchOutput::On));
}

#[test]
fn test_instance_update() {
    let mut state = SwitchState::new();
    let output = state.update(SwitchMessage::Toggle);
    assert_eq!(output, Some(SwitchOutput::On));
    assert!(state.is_on());
}

// ========================================
// View / Snapshot Tests
// ========================================

#[test]
fn test_view_off() {
    let state = SwitchState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_on() {
    let state = SwitchState::new().with_on(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let mut state = SwitchState::new();
    Switch::set_focused(&mut state, true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused_on() {
    let mut state = SwitchState::new().with_on(true);
    Switch::set_focused(&mut state, true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = SwitchState::new().with_disabled(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled_on() {
    let state = SwitchState::new().with_on(true).with_disabled(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_label() {
    let state = SwitchState::new().with_label("Dark Mode");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_label_on() {
    let state = SwitchState::new().with_label("Dark Mode").with_on(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_custom_labels() {
    let state = SwitchState::new()
        .with_on_label("YES")
        .with_off_label("NO")
        .with_on(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);

    terminal
        .draw(|frame| {
            Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========================================
// Toggleable Trait Tests
// ========================================

#[test]
fn test_toggleable_is_visible() {
    let state = SwitchState::new();
    assert!(!Switch::is_visible(&state));

    let state = SwitchState::new().with_on(true);
    assert!(Switch::is_visible(&state));
}

#[test]
fn test_toggleable_set_visible() {
    let mut state = SwitchState::new();
    Switch::set_visible(&mut state, true);
    assert!(state.is_on());

    Switch::set_visible(&mut state, false);
    assert!(!state.is_on());
}

#[test]
fn test_toggleable_toggle() {
    let mut state = SwitchState::new();
    assert!(!Switch::is_visible(&state));

    Switch::toggle(&mut state);
    assert!(Switch::is_visible(&state));
    assert!(state.is_on());

    Switch::toggle(&mut state);
    assert!(!Switch::is_visible(&state));
    assert!(!state.is_on());
}

#[test]
fn test_toggleable_show_hide() {
    let mut state = SwitchState::new();
    Switch::show(&mut state);
    assert!(state.is_on());

    Switch::hide(&mut state);
    assert!(!state.is_on());
}

// ========================================
// Focusable Trait Tests
// ========================================

#[test]
fn test_focusable_is_focused() {
    let state = SwitchState::new();
    assert!(!Switch::is_focused(&state));
}

#[test]
fn test_focusable_set_focused() {
    let mut state = SwitchState::new();
    Switch::set_focused(&mut state, true);
    assert!(Switch::is_focused(&state));
    Switch::set_focused(&mut state, false);
    assert!(!Switch::is_focused(&state));
}

#[test]
fn test_focusable_focus_blur() {
    let mut state = SwitchState::new();
    Switch::focus(&mut state);
    assert!(Switch::is_focused(&state));
    Switch::blur(&mut state);
    assert!(!Switch::is_focused(&state));
}

// ========================================
// Disableable Trait Tests
// ========================================

#[test]
fn test_disableable_is_disabled() {
    let state = SwitchState::new();
    assert!(!Switch::is_disabled(&state));
}

#[test]
fn test_disableable_set_disabled() {
    let mut state = SwitchState::new();
    Switch::set_disabled(&mut state, true);
    assert!(Switch::is_disabled(&state));
    Switch::set_disabled(&mut state, false);
    assert!(!Switch::is_disabled(&state));
}

#[test]
fn test_disableable_disable_enable() {
    let mut state = SwitchState::new();
    Switch::disable(&mut state);
    assert!(Switch::is_disabled(&state));
    Switch::enable(&mut state);
    assert!(!Switch::is_disabled(&state));
}

#[test]
fn test_with_disabled_prevents_toggle() {
    let mut state = SwitchState::new().with_disabled(true);
    state.set_focused(true);
    let output = Switch::update(&mut state, SwitchMessage::Toggle);
    assert_eq!(output, None);
    assert!(!state.is_on());
}

#[test]
fn test_with_disabled_prevents_handle_event() {
    let mut state = SwitchState::new().with_disabled(true);
    state.set_focused(true);
    let msg = Switch::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);
}

// ========================================
// Annotation Tests
// ========================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = SwitchState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Switch);
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].annotation.selected);
}

#[test]
fn test_annotation_on() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = SwitchState::new().with_on(true);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    let regions = registry.find_by_type(&WidgetType::Switch);
    assert!(regions[0].annotation.selected);
}

#[test]
fn test_annotation_with_label() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = SwitchState::new().with_label("Dark Mode");
    let (mut terminal, theme) = crate::component::test_utils::setup_render(30, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Switch::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    let regions = registry.find_by_type(&WidgetType::Switch);
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].annotation.label, Some("Dark Mode".to_string()));
}
