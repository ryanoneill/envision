use super::*;
use crate::component::Component;
use crate::component::test_utils::setup_render;
use crate::input::{Event, KeyCode, KeyModifiers};
use ratatui::prelude::Rect;

// ========== PaneConfig Tests ==========

#[test]
fn test_pane_config_new() {
    let pane = PaneConfig::new("sidebar");
    assert_eq!(pane.id(), "sidebar");
    assert_eq!(pane.title(), None);
    assert_eq!(pane.proportion(), 1.0);
    assert_eq!(pane.min_size(), 1);
    assert_eq!(pane.max_size(), 0);
}

#[test]
fn test_pane_config_with_title() {
    let pane = PaneConfig::new("sidebar").with_title("Files");
    assert_eq!(pane.title(), Some("Files"));
}

#[test]
fn test_pane_config_with_proportion() {
    let pane = PaneConfig::new("sidebar").with_proportion(0.3);
    assert!((pane.proportion() - 0.3).abs() < f32::EPSILON);
}

#[test]
fn test_pane_config_with_min_size() {
    let pane = PaneConfig::new("sidebar").with_min_size(10);
    assert_eq!(pane.min_size(), 10);
}

#[test]
fn test_pane_config_with_max_size() {
    let pane = PaneConfig::new("sidebar").with_max_size(50);
    assert_eq!(pane.max_size(), 50);
}

#[test]
fn test_pane_config_min_size_clamped() {
    let pane = PaneConfig::new("sidebar").with_min_size(0);
    assert_eq!(pane.min_size(), 1);
}

#[test]
fn test_pane_config_proportion_clamped() {
    let pane = PaneConfig::new("sidebar").with_proportion(-1.0);
    assert!((pane.proportion() - 0.0).abs() < f32::EPSILON);
}

// ========== State Creation Tests ==========

#[test]
fn test_state_new() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    assert_eq!(state.pane_count(), 2);
    assert_eq!(state.focused_pane_index(), 0);
    assert_eq!(state.direction(), &PaneDirection::Horizontal);
}

#[test]
fn test_state_default() {
    let state = PaneLayoutState::default();
    assert_eq!(state.pane_count(), 0);
    assert_eq!(state.direction(), &PaneDirection::Horizontal);
}

#[test]
fn test_state_with_resize_step() {
    let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![]).with_resize_step(0.1);
    assert!((state.resize_step() - 0.1).abs() < f32::EPSILON);
}

#[test]
fn test_state_resize_step_clamped() {
    let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![]).with_resize_step(0.0);
    assert!((state.resize_step() - 0.01).abs() < f32::EPSILON);

    let state2 = PaneLayoutState::new(PaneDirection::Horizontal, vec![]).with_resize_step(1.0);
    assert!((state2.resize_step() - 0.5).abs() < f32::EPSILON);
}

// ========== Proportion Normalization Tests ==========

#[test]
fn test_proportions_normalized() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(1.0),
        PaneConfig::new("b").with_proportion(3.0),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let sum: f32 = state.panes().iter().map(|p| p.proportion()).sum();
    assert!((sum - 1.0).abs() < 0.01);
    assert!((state.panes()[0].proportion() - 0.25).abs() < 0.01);
    assert!((state.panes()[1].proportion() - 0.75).abs() < 0.01);
}

#[test]
fn test_equal_proportions() {
    let panes = vec![
        PaneConfig::new("a"),
        PaneConfig::new("b"),
        PaneConfig::new("c"),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    for pane in state.panes() {
        assert!((pane.proportion() - 1.0 / 3.0).abs() < 0.01);
    }
}

// ========== Layout Computation Tests ==========

#[test]
fn test_layout_two_equal_horizontal() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let area = Rect::new(0, 0, 80, 24);
    let rects = state.layout(area);
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].width, 40);
    assert_eq!(rects[1].width, 40);
    assert_eq!(rects[0].height, 24);
    assert_eq!(rects[1].height, 24);
    assert_eq!(rects[0].x, 0);
    assert_eq!(rects[1].x, 40);
}

#[test]
fn test_layout_two_equal_vertical() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let state = PaneLayoutState::new(PaneDirection::Vertical, panes);
    let area = Rect::new(0, 0, 80, 24);
    let rects = state.layout(area);
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].height, 12);
    assert_eq!(rects[1].height, 12);
    assert_eq!(rects[0].y, 0);
    assert_eq!(rects[1].y, 12);
}

#[test]
fn test_layout_three_panes() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.25),
        PaneConfig::new("b").with_proportion(0.5),
        PaneConfig::new("c").with_proportion(0.25),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let area = Rect::new(0, 0, 100, 40);
    let rects = state.layout(area);
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].width, 25);
    assert_eq!(rects[1].width, 50);
    assert_eq!(rects[2].width, 25);
}

#[test]
fn test_layout_with_min_size() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.1).with_min_size(20),
        PaneConfig::new("b").with_proportion(0.9),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let area = Rect::new(0, 0, 100, 24);
    let rects = state.layout(area);
    assert!(rects[0].width >= 20);
}

#[test]
fn test_layout_with_max_size() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.8).with_max_size(30),
        PaneConfig::new("b").with_proportion(0.2),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let area = Rect::new(0, 0, 100, 24);
    let rects = state.layout(area);
    assert!(rects[0].width <= 30);
}

#[test]
fn test_layout_empty() {
    let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![]);
    let area = Rect::new(0, 0, 100, 24);
    let rects = state.layout(area);
    assert!(rects.is_empty());
}

#[test]
fn test_pane_area_by_id() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let area = Rect::new(0, 0, 80, 24);
    let pane_a = state.pane_area(area, "a").unwrap();
    assert_eq!(pane_a.width, 40);
    assert!(state.pane_area(area, "nonexistent").is_none());
}

// ========== Accessor Tests ==========

#[test]
fn test_focused_pane_id() {
    let panes = vec![PaneConfig::new("alpha"), PaneConfig::new("beta")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    assert_eq!(state.focused_pane_id(), Some("alpha"));
}

#[test]
fn test_focused_pane_id_empty() {
    let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![]);
    assert_eq!(state.focused_pane_id(), None);
}

#[test]
fn test_pane_accessor() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    assert!(state.pane("a").is_some());
    assert!(state.pane("c").is_none());
}

// ========== Focus Navigation Tests ==========

#[test]
fn test_focus_next() {
    let panes = vec![
        PaneConfig::new("a"),
        PaneConfig::new("b"),
        PaneConfig::new("c"),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::FocusNext);
    assert_eq!(state.focused_pane_index(), 1);
    assert!(matches!(
        output,
        Some(PaneLayoutOutput::FocusChanged { ref pane_id, index: 1 }) if pane_id == "b"
    ));
}

#[test]
fn test_focus_next_wraps() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    state.focused_pane = 1;

    PaneLayout::update(&mut state, PaneLayoutMessage::FocusNext);
    assert_eq!(state.focused_pane_index(), 0);
}

#[test]
fn test_focus_prev() {
    let panes = vec![
        PaneConfig::new("a"),
        PaneConfig::new("b"),
        PaneConfig::new("c"),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    state.focused_pane = 2;

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::FocusPrev);
    assert_eq!(state.focused_pane_index(), 1);
    assert!(matches!(
        output,
        Some(PaneLayoutOutput::FocusChanged { ref pane_id, index: 1 }) if pane_id == "b"
    ));
}

#[test]
fn test_focus_prev_wraps() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    PaneLayout::update(&mut state, PaneLayoutMessage::FocusPrev);
    assert_eq!(state.focused_pane_index(), 1);
}

#[test]
fn test_focus_pane_by_id() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::FocusPane("b".to_string()));
    assert_eq!(state.focused_pane_index(), 1);
    assert!(matches!(
        output,
        Some(PaneLayoutOutput::FocusChanged { ref pane_id, index: 1 }) if pane_id == "b"
    ));
}

#[test]
fn test_focus_pane_by_id_nonexistent() {
    let panes = vec![PaneConfig::new("a")];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(
        &mut state,
        PaneLayoutMessage::FocusPane("nonexistent".to_string()),
    );
    assert_eq!(output, None);
}

#[test]
fn test_focus_pane_by_index() {
    let panes = vec![
        PaneConfig::new("a"),
        PaneConfig::new("b"),
        PaneConfig::new("c"),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::FocusPaneIndex(2));
    assert_eq!(state.focused_pane_index(), 2);
    assert!(matches!(
        output,
        Some(PaneLayoutOutput::FocusChanged { ref pane_id, index: 2 }) if pane_id == "c"
    ));
}

#[test]
fn test_focus_pane_by_index_out_of_bounds() {
    let panes = vec![PaneConfig::new("a")];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::FocusPaneIndex(5));
    assert_eq!(output, None);
}

// ========== Resize Tests ==========

#[test]
fn test_grow_focused() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::GrowFocused);
    assert!(output.is_some());
    assert!(state.panes()[0].proportion() > 0.5);
    assert!(state.panes()[1].proportion() < 0.5);
}

#[test]
fn test_shrink_focused() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::ShrinkFocused);
    assert!(output.is_some());
    assert!(state.panes()[0].proportion() < 0.5);
    assert!(state.panes()[1].proportion() > 0.5);
}

#[test]
fn test_grow_pane_by_id() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::GrowPane("b".to_string()));
    assert!(output.is_some());
    assert!(state.panes()[1].proportion() > 0.5);
}

#[test]
fn test_shrink_pane_by_id() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::ShrinkPane("a".to_string()));
    assert!(output.is_some());
    assert!(state.panes()[0].proportion() < 0.5);
}

#[test]
fn test_grow_at_boundary() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.95),
        PaneConfig::new("b").with_proportion(0.05),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    // Neighbor is too small to shrink further
    let output = PaneLayout::update(&mut state, PaneLayoutMessage::GrowFocused);
    assert_eq!(output, None);
}

#[test]
fn test_shrink_at_boundary() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.05),
        PaneConfig::new("b").with_proportion(0.95),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    // Focused pane is too small to shrink further
    let output = PaneLayout::update(&mut state, PaneLayoutMessage::ShrinkFocused);
    assert_eq!(output, None);
}

#[test]
fn test_set_proportion() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.5),
        PaneConfig::new("b").with_proportion(0.5),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(
        &mut state,
        PaneLayoutMessage::SetProportion {
            id: "a".to_string(),
            proportion: 0.7,
        },
    );
    assert!(output.is_some());
    // After normalization, proportions should sum to 1.0
    let sum: f32 = state.panes().iter().map(|p| p.proportion()).sum();
    assert!((sum - 1.0).abs() < 0.01);
}

#[test]
fn test_set_proportion_nonexistent() {
    let panes = vec![PaneConfig::new("a")];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(
        &mut state,
        PaneLayoutMessage::SetProportion {
            id: "z".to_string(),
            proportion: 0.5,
        },
    );
    assert_eq!(output, None);
}

#[test]
fn test_reset_proportions() {
    let panes = vec![
        PaneConfig::new("a").with_proportion(0.8),
        PaneConfig::new("b").with_proportion(0.2),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    let output = PaneLayout::update(&mut state, PaneLayoutMessage::ResetProportions);
    assert_eq!(output, Some(PaneLayoutOutput::ProportionsReset));
    assert!((state.panes()[0].proportion() - 0.5).abs() < 0.01);
    assert!((state.panes()[1].proportion() - 0.5).abs() < 0.01);
}

#[test]
fn test_reset_proportions_empty() {
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, vec![]);
    let output = PaneLayout::update(&mut state, PaneLayoutMessage::ResetProportions);
    assert_eq!(output, None);
}

// ========== Guard Tests ==========

#[test]
fn test_focus_next_empty_guard() {
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, vec![]);
    let output = PaneLayout::update(&mut state, PaneLayoutMessage::FocusNext);
    assert_eq!(output, None);
}

// ========== handle_event Tests ==========

#[test]
fn test_handle_event_tab() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let msg = PaneLayout::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaneLayoutMessage::FocusNext));
}

#[test]
fn test_handle_event_backtab() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let msg = PaneLayout::handle_event(
        &state,
        &Event::key(KeyCode::BackTab),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaneLayoutMessage::FocusPrev));
}

#[test]
fn test_handle_event_ctrl_right() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let msg = PaneLayout::handle_event(
        &state,
        &Event::key_with(KeyCode::Right, KeyModifiers::CONTROL),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaneLayoutMessage::GrowFocused));
}

#[test]
fn test_handle_event_ctrl_left() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let msg = PaneLayout::handle_event(
        &state,
        &Event::key_with(KeyCode::Left, KeyModifiers::CONTROL),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaneLayoutMessage::ShrinkFocused));
}

#[test]
fn test_handle_event_ctrl_0() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let msg = PaneLayout::handle_event(
        &state,
        &Event::ctrl('0'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(PaneLayoutMessage::ResetProportions));
}

#[test]
fn test_handle_event_unfocused_ignored() {
    let panes = vec![PaneConfig::new("a")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let msg = PaneLayout::handle_event(&state, &Event::key(KeyCode::Tab), &EventContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled_ignored() {
    let panes = vec![PaneConfig::new("a")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let msg = PaneLayout::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_unrecognized() {
    let panes = vec![PaneConfig::new("a")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let msg = PaneLayout::handle_event(
        &state,
        &Event::char('z'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, None);
}

// ========== dispatch_event and instance methods ==========

#[test]
fn test_dispatch_event() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let output = PaneLayout::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Tab),
        &EventContext::new().focused(true),
    );
    assert!(matches!(
        output,
        Some(PaneLayoutOutput::FocusChanged { .. })
    ));
}

#[test]
fn test_instance_update() {
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let output = state.update(PaneLayoutMessage::FocusNext);
    assert!(output.is_some());
}

// ========== Init Test ==========

#[test]
fn test_init() {
    let state = PaneLayout::init();
    assert_eq!(state.pane_count(), 0);
}

// ========== Rendering Snapshot Tests ==========

#[test]
fn test_view_two_panes_horizontal() {
    let (mut terminal, theme) = setup_render(60, 10);
    let panes = vec![
        PaneConfig::new("left").with_title("Left"),
        PaneConfig::new("right").with_title("Right"),
    ];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

    terminal
        .draw(|frame| {
            PaneLayout::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_two_panes_horizontal", display);
}

#[test]
fn test_view_two_panes_vertical() {
    let (mut terminal, theme) = setup_render(40, 12);
    let panes = vec![
        PaneConfig::new("top").with_title("Top"),
        PaneConfig::new("bottom").with_title("Bottom"),
    ];
    let state = PaneLayoutState::new(PaneDirection::Vertical, panes);

    terminal
        .draw(|frame| {
            PaneLayout::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_two_panes_vertical", display);
}

#[test]
fn test_view_three_panes_focused() {
    let (mut terminal, theme) = setup_render(60, 10);
    let panes = vec![
        PaneConfig::new("a").with_title("A"),
        PaneConfig::new("b").with_title("B"),
        PaneConfig::new("c").with_title("C"),
    ];
    let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    state.focused_pane = 1;

    terminal
        .draw(|frame| {
            PaneLayout::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_three_panes_focused", display);
}

#[test]
fn test_view_empty_panes() {
    let (mut terminal, theme) = setup_render(40, 5);
    let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![]);

    terminal
        .draw(|frame| {
            PaneLayout::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_empty_panes", display);
}

// ========== Annotation Tests ==========

#[test]
fn test_annotation_emission() {
    use crate::annotation::{WidgetType, with_annotations};
    let panes = vec![PaneConfig::new("a"), PaneConfig::new("b")];
    let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    let (mut terminal, theme) = setup_render(60, 10);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                PaneLayout::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
                );
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::PaneLayout);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.has_id("pane_layout"));
    assert!(regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}
