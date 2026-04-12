use super::*;
use crate::component::test_utils;

fn vertical_state() -> SplitPanelState {
    SplitPanelState::new(SplitOrientation::Vertical)
}

fn horizontal_state() -> SplitPanelState {
    SplitPanelState::new(SplitOrientation::Horizontal)
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new_vertical() {
    let state = SplitPanelState::new(SplitOrientation::Vertical);
    assert_eq!(state.orientation(), &SplitOrientation::Vertical);
    assert!((state.ratio() - 0.5).abs() < f32::EPSILON);
    assert!(state.is_first_pane_focused());
    assert!(!state.is_second_pane_focused());
}

#[test]
fn test_new_horizontal() {
    let state = SplitPanelState::new(SplitOrientation::Horizontal);
    assert_eq!(state.orientation(), &SplitOrientation::Horizontal);
}

#[test]
fn test_with_ratio() {
    let state = SplitPanelState::new(SplitOrientation::Vertical).with_ratio(0.3);
    assert!((state.ratio() - 0.3).abs() < f32::EPSILON);
}

#[test]
fn test_with_ratio_clamped() {
    let state = SplitPanelState::new(SplitOrientation::Vertical).with_ratio(0.05);
    assert!((state.ratio() - 0.1).abs() < f32::EPSILON); // min is 0.1

    let state = SplitPanelState::new(SplitOrientation::Vertical).with_ratio(0.95);
    assert!((state.ratio() - 0.9).abs() < f32::EPSILON); // max is 0.9
}

#[test]
fn test_default() {
    let state = SplitPanelState::default();
    assert_eq!(state.orientation(), &SplitOrientation::Vertical);
    assert!((state.ratio() - 0.5).abs() < f32::EPSILON);
}

// =============================================================================
// Builder methods
// =============================================================================

#[test]
fn test_with_resize_step() {
    let state = SplitPanelState::new(SplitOrientation::Vertical).with_resize_step(0.05);
    assert!((state.resize_step() - 0.05).abs() < f32::EPSILON);
}

#[test]
fn test_with_bounds() {
    let state = SplitPanelState::new(SplitOrientation::Vertical).with_bounds(0.2, 0.8);
    // Ratio should still be 0.5 (within bounds)
    assert!((state.ratio() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn test_with_bounds_clamps_ratio() {
    let state = SplitPanelState::new(SplitOrientation::Vertical)
        .with_ratio(0.95)
        .with_bounds(0.2, 0.8);
    assert!((state.ratio() - 0.8).abs() < f32::EPSILON);
}
// =============================================================================
// Setters
// =============================================================================

#[test]
fn test_set_orientation() {
    let mut state = SplitPanelState::new(SplitOrientation::Vertical);
    state.set_orientation(SplitOrientation::Horizontal);
    assert_eq!(state.orientation(), &SplitOrientation::Horizontal);
}

#[test]
fn test_set_ratio() {
    let mut state = SplitPanelState::new(SplitOrientation::Vertical);
    state.set_ratio(0.7);
    assert!((state.ratio() - 0.7).abs() < f32::EPSILON);
}

#[test]
fn test_set_ratio_clamped() {
    let mut state = SplitPanelState::new(SplitOrientation::Vertical);
    state.set_ratio(0.0);
    assert!((state.ratio() - 0.1).abs() < f32::EPSILON);
    state.set_ratio(1.0);
    assert!((state.ratio() - 0.9).abs() < f32::EPSILON);
}
// =============================================================================
// Focus management
// =============================================================================

#[test]
fn test_focus_other_toggles() {
    let mut state = vertical_state();
    assert!(state.is_first_pane_focused());

    let output = SplitPanel::update(&mut state, SplitPanelMessage::FocusOther);
    assert!(state.is_second_pane_focused());
    assert_eq!(output, Some(SplitPanelOutput::FocusedSecond));

    let output = SplitPanel::update(&mut state, SplitPanelMessage::FocusOther);
    assert!(state.is_first_pane_focused());
    assert_eq!(output, Some(SplitPanelOutput::FocusedFirst));
}

#[test]
fn test_focus_first() {
    let mut state = vertical_state();
    SplitPanel::update(&mut state, SplitPanelMessage::FocusOther);
    assert!(state.is_second_pane_focused());

    let output = SplitPanel::update(&mut state, SplitPanelMessage::FocusFirst);
    assert!(state.is_first_pane_focused());
    assert_eq!(output, Some(SplitPanelOutput::FocusedFirst));
}

#[test]
fn test_focus_first_when_already_first() {
    let mut state = vertical_state();
    let output = SplitPanel::update(&mut state, SplitPanelMessage::FocusFirst);
    assert_eq!(output, None);
}

#[test]
fn test_focus_second() {
    let mut state = vertical_state();
    let output = SplitPanel::update(&mut state, SplitPanelMessage::FocusSecond);
    assert!(state.is_second_pane_focused());
    assert_eq!(output, Some(SplitPanelOutput::FocusedSecond));
}

#[test]
fn test_focus_second_when_already_second() {
    let mut state = vertical_state();
    SplitPanel::update(&mut state, SplitPanelMessage::FocusOther);
    let output = SplitPanel::update(&mut state, SplitPanelMessage::FocusSecond);
    assert_eq!(output, None);
}

// =============================================================================
// Resizing
// =============================================================================

#[test]
fn test_grow_first() {
    let mut state = vertical_state();
    let output = SplitPanel::update(&mut state, SplitPanelMessage::GrowFirst);
    assert!((state.ratio() - 0.6).abs() < f32::EPSILON);
    assert!(matches!(output, Some(SplitPanelOutput::RatioChanged(_))));
}

#[test]
fn test_shrink_first() {
    let mut state = vertical_state();
    let output = SplitPanel::update(&mut state, SplitPanelMessage::ShrinkFirst);
    assert!((state.ratio() - 0.4).abs() < f32::EPSILON);
    assert!(matches!(output, Some(SplitPanelOutput::RatioChanged(_))));
}

#[test]
fn test_grow_first_at_max() {
    let mut state = vertical_state();
    state.set_ratio(0.9);
    let output = SplitPanel::update(&mut state, SplitPanelMessage::GrowFirst);
    assert!((state.ratio() - 0.9).abs() < f32::EPSILON);
    assert_eq!(output, None);
}

#[test]
fn test_shrink_first_at_min() {
    let mut state = vertical_state();
    state.set_ratio(0.1);
    let output = SplitPanel::update(&mut state, SplitPanelMessage::ShrinkFirst);
    assert!((state.ratio() - 0.1).abs() < f32::EPSILON);
    assert_eq!(output, None);
}

#[test]
fn test_set_ratio_message() {
    let mut state = vertical_state();
    let output = SplitPanel::update(&mut state, SplitPanelMessage::SetRatio(0.7));
    assert!((state.ratio() - 0.7).abs() < f32::EPSILON);
    assert_eq!(output, Some(SplitPanelOutput::RatioChanged(0.7)));
}

#[test]
fn test_set_ratio_message_same_value() {
    let mut state = vertical_state();
    let output = SplitPanel::update(&mut state, SplitPanelMessage::SetRatio(0.5));
    assert_eq!(output, None);
}

#[test]
fn test_set_ratio_message_clamped() {
    let mut state = vertical_state();
    let output = SplitPanel::update(&mut state, SplitPanelMessage::SetRatio(0.0));
    assert!((state.ratio() - 0.1).abs() < f32::EPSILON);
    assert_eq!(output, Some(SplitPanelOutput::RatioChanged(0.1)));
}

#[test]
fn test_reset_ratio() {
    let mut state = vertical_state();
    state.set_ratio(0.8);
    let output = SplitPanel::update(&mut state, SplitPanelMessage::ResetRatio);
    assert!((state.ratio() - 0.5).abs() < f32::EPSILON);
    assert_eq!(output, Some(SplitPanelOutput::RatioChanged(0.5)));
}

#[test]
fn test_reset_ratio_already_50() {
    let mut state = vertical_state();
    let output = SplitPanel::update(&mut state, SplitPanelMessage::ResetRatio);
    assert_eq!(output, None);
}

#[test]
fn test_custom_resize_step() {
    let mut state = SplitPanelState::new(SplitOrientation::Vertical).with_resize_step(0.05);
    SplitPanel::update(&mut state, SplitPanelMessage::GrowFirst);
    assert!((state.ratio() - 0.55).abs() < f32::EPSILON);
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let state = vertical_state();

    let msg = SplitPanel::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = SplitPanelState::new(SplitOrientation::Vertical);
    let msg = SplitPanel::handle_event(&state, &Event::key(KeyCode::Tab), &EventContext::default());
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_tab_maps_to_focus_other() {
    let state = vertical_state();
    let msg = SplitPanel::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(SplitPanelMessage::FocusOther));
}

#[test]
fn test_backtab_maps_to_focus_other() {
    let state = vertical_state();
    let msg = SplitPanel::handle_event(
        &state,
        &Event::key(KeyCode::BackTab),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(SplitPanelMessage::FocusOther));
}

#[test]
fn test_ctrl_right_maps_to_grow_first() {
    let state = vertical_state();
    let msg = SplitPanel::handle_event(
        &state,
        &Event::key_with(KeyCode::Right, KeyModifiers::CONTROL),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(SplitPanelMessage::GrowFirst));
}

#[test]
fn test_ctrl_left_maps_to_shrink_first() {
    let state = vertical_state();
    let msg = SplitPanel::handle_event(
        &state,
        &Event::key_with(KeyCode::Left, KeyModifiers::CONTROL),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(SplitPanelMessage::ShrinkFirst));
}

#[test]
fn test_ctrl_down_maps_to_grow_first() {
    let state = vertical_state();
    let msg = SplitPanel::handle_event(
        &state,
        &Event::key_with(KeyCode::Down, KeyModifiers::CONTROL),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(SplitPanelMessage::GrowFirst));
}

#[test]
fn test_ctrl_up_maps_to_shrink_first() {
    let state = vertical_state();
    let msg = SplitPanel::handle_event(
        &state,
        &Event::key_with(KeyCode::Up, KeyModifiers::CONTROL),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(SplitPanelMessage::ShrinkFirst));
}

#[test]
fn test_ctrl_0_maps_to_reset_ratio() {
    let state = vertical_state();
    let msg = SplitPanel::handle_event(
        &state,
        &Event::ctrl('0'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(SplitPanelMessage::ResetRatio));
}

#[test]
fn test_arrow_without_ctrl_ignored() {
    let state = vertical_state();
    let msg = SplitPanel::handle_event(
        &state,
        &Event::key(KeyCode::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, None);
}

// =============================================================================
// Layout
// =============================================================================

#[test]
fn test_layout_vertical_50_50() {
    let state = SplitPanelState::new(SplitOrientation::Vertical);
    let area = Rect::new(0, 0, 80, 24);
    let (first, second) = state.layout(area);
    assert_eq!(first.width, 40);
    assert_eq!(second.width, 40);
    assert_eq!(first.height, 24);
    assert_eq!(second.height, 24);
}

#[test]
fn test_layout_horizontal_50_50() {
    let state = SplitPanelState::new(SplitOrientation::Horizontal);
    let area = Rect::new(0, 0, 80, 24);
    let (first, second) = state.layout(area);
    assert_eq!(first.height, 12);
    assert_eq!(second.height, 12);
    assert_eq!(first.width, 80);
    assert_eq!(second.width, 80);
}

#[test]
fn test_layout_vertical_70_30() {
    let state = SplitPanelState::new(SplitOrientation::Vertical).with_ratio(0.7);
    let area = Rect::new(0, 0, 100, 24);
    let (first, second) = state.layout(area);
    assert_eq!(first.width, 70);
    assert_eq!(second.width, 30);
}

#[test]
fn test_layout_zero_area() {
    let state = SplitPanelState::new(SplitOrientation::Vertical);
    let area = Rect::new(0, 0, 0, 0);
    let (first, second) = state.layout(area);
    assert_eq!(first.width, 0);
    assert_eq!(second.width, 0);
}

// =============================================================================
// dispatch_event
// =============================================================================

#[test]
fn test_dispatch_event_resize() {
    let mut state = vertical_state();
    let output = SplitPanel::dispatch_event(
        &mut state,
        &Event::key_with(KeyCode::Right, KeyModifiers::CONTROL),
        &EventContext::new().focused(true),
    );
    assert!(matches!(output, Some(SplitPanelOutput::RatioChanged(_))));
}
#[test]
fn test_instance_update() {
    let mut state = vertical_state();
    let output = state.update(SplitPanelMessage::FocusOther);
    assert_eq!(output, Some(SplitPanelOutput::FocusedSecond));
}
// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_vertical() {
    let state = vertical_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    terminal
        .draw(|frame| {
            SplitPanel::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_horizontal() {
    let state = horizontal_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    terminal
        .draw(|frame| {
            SplitPanel::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_second_pane_focused() {
    let mut state = vertical_state();
    SplitPanel::update(&mut state, SplitPanelMessage::FocusOther);
    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    terminal
        .draw(|frame| {
            SplitPanel::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = SplitPanelState::new(SplitOrientation::Vertical);
    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    terminal
        .draw(|frame| {
            SplitPanel::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
}
// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = SplitPanelState::new(SplitOrientation::Vertical);
    let state2 = SplitPanelState::new(SplitOrientation::Vertical);
    assert_eq!(state1, state2);
}

#[test]
fn test_partial_eq_different_ratio() {
    let state1 = SplitPanelState::new(SplitOrientation::Vertical);
    let state2 = SplitPanelState::new(SplitOrientation::Vertical).with_ratio(0.3);
    assert_ne!(state1, state2);
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = SplitPanelState::new(SplitOrientation::Vertical);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                SplitPanel::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::SplitPanel);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.has_id("split_panel"));
}
