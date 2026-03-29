use super::*;
use crate::component::test_utils;

fn sample_events() -> Vec<TimelineEvent> {
    vec![
        TimelineEvent::new("e1", 100.0, "Start"),
        TimelineEvent::new("e2", 500.0, "Deploy").with_color(Color::Green),
        TimelineEvent::new("e3", 900.0, "Complete").with_color(Color::Blue),
    ]
}

fn sample_spans() -> Vec<TimelineSpan> {
    vec![
        TimelineSpan::new("s1", 200.0, 800.0, "request-1").with_color(Color::Cyan),
        TimelineSpan::new("s2", 300.0, 600.0, "db-query")
            .with_color(Color::Magenta)
            .with_lane(1),
    ]
}

fn focused_timeline() -> TimelineState {
    let mut state = TimelineState::new()
        .with_events(sample_events())
        .with_spans(sample_spans())
        .with_view_range(0.0, 1000.0);
    state.set_focused(true);
    state
}

// =============================================================================
// TimelineEvent
// =============================================================================

#[test]
fn test_timeline_event_new() {
    let event = TimelineEvent::new("e1", 100.0, "Start");
    assert_eq!(event.id, "e1");
    assert_eq!(event.timestamp, 100.0);
    assert_eq!(event.label, "Start");
    assert_eq!(event.color, Color::Yellow);
}

#[test]
fn test_timeline_event_with_color() {
    let event = TimelineEvent::new("e1", 0.0, "Test").with_color(Color::Red);
    assert_eq!(event.color, Color::Red);
}

#[test]
fn test_timeline_event_partial_eq() {
    let a = TimelineEvent::new("e1", 100.0, "Start");
    let b = TimelineEvent::new("e1", 100.0, "Start");
    assert_eq!(a, b);
}

// =============================================================================
// TimelineSpan
// =============================================================================

#[test]
fn test_timeline_span_new() {
    let span = TimelineSpan::new("s1", 200.0, 800.0, "request");
    assert_eq!(span.id, "s1");
    assert_eq!(span.start, 200.0);
    assert_eq!(span.end, 800.0);
    assert_eq!(span.label, "request");
    assert_eq!(span.color, Color::Cyan);
    assert_eq!(span.lane, 0);
}

#[test]
fn test_timeline_span_with_color() {
    let span = TimelineSpan::new("s1", 0.0, 100.0, "test").with_color(Color::Red);
    assert_eq!(span.color, Color::Red);
}

#[test]
fn test_timeline_span_with_lane() {
    let span = TimelineSpan::new("s1", 0.0, 100.0, "test").with_lane(3);
    assert_eq!(span.lane, 3);
}

#[test]
fn test_timeline_span_duration() {
    let span = TimelineSpan::new("s1", 100.0, 400.0, "test");
    assert_eq!(span.duration(), 300.0);
}

#[test]
fn test_timeline_span_partial_eq() {
    let a = TimelineSpan::new("s1", 0.0, 100.0, "test");
    let b = TimelineSpan::new("s1", 0.0, 100.0, "test");
    assert_eq!(a, b);
}

// =============================================================================
// SelectedType
// =============================================================================

#[test]
fn test_selected_type_default() {
    assert_eq!(SelectedType::default(), SelectedType::Event);
}

// =============================================================================
// TimelineState construction
// =============================================================================

#[test]
fn test_new() {
    let state = TimelineState::new();
    assert!(state.events().is_empty());
    assert!(state.spans().is_empty());
    assert_eq!(state.view_range(), (0.0, 1000.0));
    assert!(state.title().is_none());
    assert!(state.show_labels());
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert!(state.selected_index.is_none());
}

#[test]
fn test_default() {
    let state = TimelineState::default();
    assert!(state.events().is_empty());
    assert!(state.spans().is_empty());
}

#[test]
fn test_with_events() {
    let state = TimelineState::new().with_events(sample_events());
    assert_eq!(state.events().len(), 3);
}

#[test]
fn test_with_spans() {
    let state = TimelineState::new().with_spans(sample_spans());
    assert_eq!(state.spans().len(), 2);
}

#[test]
fn test_with_view_range() {
    let state = TimelineState::new().with_view_range(100.0, 900.0);
    assert_eq!(state.view_range(), (100.0, 900.0));
}

#[test]
fn test_with_title() {
    let state = TimelineState::new().with_title("My Timeline");
    assert_eq!(state.title(), Some("My Timeline"));
}

#[test]
fn test_with_show_labels() {
    let state = TimelineState::new().with_show_labels(false);
    assert!(!state.show_labels());
}

#[test]
fn test_with_disabled() {
    let state = TimelineState::new().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Event/span operations
// =============================================================================

#[test]
fn test_add_event() {
    let mut state = TimelineState::new();
    state.add_event(TimelineEvent::new("e1", 100.0, "Start"));
    assert_eq!(state.events().len(), 1);
    assert_eq!(state.events()[0].id, "e1");
}

#[test]
fn test_add_span() {
    let mut state = TimelineState::new();
    state.add_span(TimelineSpan::new("s1", 0.0, 200.0, "Init"));
    assert_eq!(state.spans().len(), 1);
    assert_eq!(state.spans()[0].id, "s1");
}

#[test]
fn test_clear() {
    let mut state = TimelineState::new()
        .with_events(sample_events())
        .with_spans(sample_spans());
    state.selected_index = Some(0);
    state.clear();
    assert!(state.events().is_empty());
    assert!(state.spans().is_empty());
    assert!(state.selected_index.is_none());
}

// =============================================================================
// View range and navigation
// =============================================================================

#[test]
fn test_set_view_range() {
    let mut state = TimelineState::new();
    state.set_view_range(50.0, 750.0);
    assert_eq!(state.view_range(), (50.0, 750.0));
}

#[test]
fn test_fit_all_with_data() {
    let mut state = TimelineState::new()
        .with_events(vec![TimelineEvent::new("e1", 100.0, "A")])
        .with_spans(vec![TimelineSpan::new("s1", 200.0, 800.0, "B")]);
    state.fit_all();
    let (start, end) = state.view_range();
    // Min is 100.0, max is 800.0, range is 700.0, padding is 35.0
    assert!(start < 100.0);
    assert!(end > 800.0);
    assert!((start - 65.0).abs() < 0.01);
    assert!((end - 835.0).abs() < 0.01);
}

#[test]
fn test_fit_all_empty() {
    let mut state = TimelineState::new();
    state.fit_all();
    assert_eq!(state.view_range(), (0.0, 1000.0));
}

#[test]
fn test_fit_all_single_event() {
    let mut state = TimelineState::new().with_events(vec![TimelineEvent::new("e1", 500.0, "Solo")]);
    state.fit_all();
    // Single point: min == max, so resets to default
    assert_eq!(state.view_range(), (0.0, 1000.0));
}

#[test]
fn test_zoom_in() {
    let mut state = TimelineState::new().with_view_range(0.0, 1000.0);
    state.zoom_in();
    let (start, end) = state.view_range();
    // Range should be 750 (75% of 1000), centered at 500
    assert!((start - 125.0).abs() < 0.01);
    assert!((end - 875.0).abs() < 0.01);
}

#[test]
fn test_zoom_in_minimum_range() {
    let mut state = TimelineState::new().with_view_range(499.0, 500.0);
    let original = state.view_range();
    state.zoom_in();
    // Range is 1.0, 75% would be 0.75 which is < 1.0, so no change
    assert_eq!(state.view_range(), original);
}

#[test]
fn test_zoom_out() {
    let mut state = TimelineState::new().with_view_range(250.0, 750.0);
    state.zoom_out();
    let (start, end) = state.view_range();
    // Original range = 500, new range = 500 / 0.75 = 666.67
    let expected_range = 500.0 / 0.75;
    let actual_range = end - start;
    assert!((actual_range - expected_range).abs() < 0.01);
    // Center should remain at 500
    let center = (start + end) / 2.0;
    assert!((center - 500.0).abs() < 0.01);
}

#[test]
fn test_pan_left() {
    let mut state = focused_timeline();
    let original_range = state.view_end - state.view_start;
    Timeline::update(&mut state, TimelineMessage::PanLeft);
    let new_range = state.view_end - state.view_start;
    // Range should be preserved
    assert!((new_range - original_range).abs() < 0.01);
    // View should shift left by 10%
    assert!(state.view_start < 0.0);
}

#[test]
fn test_pan_right() {
    let mut state = focused_timeline();
    let original_range = state.view_end - state.view_start;
    Timeline::update(&mut state, TimelineMessage::PanRight);
    let new_range = state.view_end - state.view_start;
    // Range should be preserved
    assert!((new_range - original_range).abs() < 0.01);
    // View should shift right by 10%
    assert!(state.view_start > 0.0);
}

// =============================================================================
// Selection
// =============================================================================

#[test]
fn test_select_next_from_none() {
    let mut state = focused_timeline();
    let output = Timeline::update(&mut state, TimelineMessage::SelectNext);
    // First item should be first event
    assert_eq!(state.selected_type, SelectedType::Event);
    assert_eq!(state.selected_index, Some(0));
    assert_eq!(output, Some(TimelineOutput::EventSelected("e1".into())));
}

#[test]
fn test_select_next_cycles_through_events() {
    let mut state = focused_timeline();
    Timeline::update(&mut state, TimelineMessage::SelectNext); // e1
    Timeline::update(&mut state, TimelineMessage::SelectNext); // e2
    assert_eq!(state.selected_type, SelectedType::Event);
    assert_eq!(state.selected_index, Some(1));

    Timeline::update(&mut state, TimelineMessage::SelectNext); // e3
    assert_eq!(state.selected_type, SelectedType::Event);
    assert_eq!(state.selected_index, Some(2));
}

#[test]
fn test_select_next_transitions_to_spans() {
    let mut state = focused_timeline();
    // Go through all events (3) to reach spans
    Timeline::update(&mut state, TimelineMessage::SelectNext); // e1
    Timeline::update(&mut state, TimelineMessage::SelectNext); // e2
    Timeline::update(&mut state, TimelineMessage::SelectNext); // e3
    let output = Timeline::update(&mut state, TimelineMessage::SelectNext); // s1
    assert_eq!(state.selected_type, SelectedType::Span);
    assert_eq!(state.selected_index, Some(0));
    assert_eq!(output, Some(TimelineOutput::SpanSelected("s1".into())));
}

#[test]
fn test_select_next_wraps() {
    let mut state = focused_timeline();
    // 3 events + 2 spans = 5 total
    // 5 calls go: e0, e1, e2, s0, s1
    // 6th call wraps to e0
    for _ in 0..6 {
        Timeline::update(&mut state, TimelineMessage::SelectNext);
    }
    assert_eq!(state.selected_type, SelectedType::Event);
    assert_eq!(state.selected_index, Some(0));
}

#[test]
fn test_select_prev_from_none() {
    let mut state = focused_timeline();
    let output = Timeline::update(&mut state, TimelineMessage::SelectPrev);
    // Should select last item (last span)
    assert_eq!(state.selected_type, SelectedType::Span);
    assert_eq!(state.selected_index, Some(1));
    assert_eq!(output, Some(TimelineOutput::SpanSelected("s2".into())));
}

#[test]
fn test_select_prev_wraps() {
    let mut state = focused_timeline();
    Timeline::update(&mut state, TimelineMessage::SelectNext); // e1 (index 0)
    let output = Timeline::update(&mut state, TimelineMessage::SelectPrev);
    // Should wrap to last span
    assert_eq!(state.selected_type, SelectedType::Span);
    assert_eq!(state.selected_index, Some(1));
    assert_eq!(output, Some(TimelineOutput::SpanSelected("s2".into())));
}

#[test]
fn test_select_empty_timeline() {
    let mut state = TimelineState::new();
    state.set_focused(true);
    let output = Timeline::update(&mut state, TimelineMessage::SelectNext);
    assert_eq!(output, None);
    assert!(state.selected_index.is_none());
}

#[test]
fn test_selected_event() {
    let mut state = focused_timeline();
    Timeline::update(&mut state, TimelineMessage::SelectNext);
    let event = state.selected_event().unwrap();
    assert_eq!(event.id, "e1");
}

#[test]
fn test_selected_span() {
    let mut state = focused_timeline();
    // Navigate to first span (past 3 events)
    for _ in 0..4 {
        Timeline::update(&mut state, TimelineMessage::SelectNext);
    }
    let span = state.selected_span().unwrap();
    assert_eq!(span.id, "s1");
}

#[test]
fn test_selected_event_returns_none_when_span_selected() {
    let mut state = focused_timeline();
    // Navigate to a span
    for _ in 0..4 {
        Timeline::update(&mut state, TimelineMessage::SelectNext);
    }
    assert!(state.selected_event().is_none());
}

#[test]
fn test_selected_span_returns_none_when_event_selected() {
    let mut state = focused_timeline();
    Timeline::update(&mut state, TimelineMessage::SelectNext);
    assert!(state.selected_span().is_none());
}

// =============================================================================
// Message handling
// =============================================================================

#[test]
fn test_add_event_message() {
    let mut state = TimelineState::new();
    let event = TimelineEvent::new("e1", 100.0, "Start");
    Timeline::update(&mut state, TimelineMessage::AddEvent(event));
    assert_eq!(state.events().len(), 1);
}

#[test]
fn test_add_span_message() {
    let mut state = TimelineState::new();
    let span = TimelineSpan::new("s1", 0.0, 200.0, "Init");
    Timeline::update(&mut state, TimelineMessage::AddSpan(span));
    assert_eq!(state.spans().len(), 1);
}

#[test]
fn test_set_events_message() {
    let mut state = focused_timeline();
    state.selected_index = Some(0);
    let new_events = vec![TimelineEvent::new("e_new", 50.0, "New")];
    Timeline::update(&mut state, TimelineMessage::SetEvents(new_events));
    assert_eq!(state.events().len(), 1);
    assert_eq!(state.events()[0].id, "e_new");
    assert!(state.selected_index.is_none());
}

#[test]
fn test_set_spans_message() {
    let mut state = focused_timeline();
    state.selected_index = Some(0);
    let new_spans = vec![TimelineSpan::new("s_new", 0.0, 100.0, "New")];
    Timeline::update(&mut state, TimelineMessage::SetSpans(new_spans));
    assert_eq!(state.spans().len(), 1);
    assert_eq!(state.spans()[0].id, "s_new");
    assert!(state.selected_index.is_none());
}

#[test]
fn test_clear_message() {
    let mut state = focused_timeline();
    Timeline::update(&mut state, TimelineMessage::Clear);
    assert!(state.events().is_empty());
    assert!(state.spans().is_empty());
}

#[test]
fn test_zoom_in_message_returns_view_changed() {
    let mut state = focused_timeline();
    let output = Timeline::update(&mut state, TimelineMessage::ZoomIn);
    match output {
        Some(TimelineOutput::ViewChanged { start, end }) => {
            assert!(start > 0.0);
            assert!(end < 1000.0);
        }
        _ => panic!("Expected ViewChanged output"),
    }
}

#[test]
fn test_zoom_out_message_returns_view_changed() {
    let mut state = focused_timeline();
    let output = Timeline::update(&mut state, TimelineMessage::ZoomOut);
    match output {
        Some(TimelineOutput::ViewChanged { start, end }) => {
            assert!(start < 0.0);
            assert!(end > 1000.0);
        }
        _ => panic!("Expected ViewChanged output"),
    }
}

#[test]
fn test_pan_left_message_returns_view_changed() {
    let mut state = focused_timeline();
    let output = Timeline::update(&mut state, TimelineMessage::PanLeft);
    assert!(matches!(output, Some(TimelineOutput::ViewChanged { .. })));
}

#[test]
fn test_pan_right_message_returns_view_changed() {
    let mut state = focused_timeline();
    let output = Timeline::update(&mut state, TimelineMessage::PanRight);
    assert!(matches!(output, Some(TimelineOutput::ViewChanged { .. })));
}

#[test]
fn test_fit_all_message_returns_view_changed() {
    let mut state = focused_timeline();
    let output = Timeline::update(&mut state, TimelineMessage::FitAll);
    assert!(matches!(output, Some(TimelineOutput::ViewChanged { .. })));
}

// =============================================================================
// Event handling (keyboard)
// =============================================================================

#[test]
fn test_left_maps_to_pan_left() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::key(KeyCode::Left)),
        Some(TimelineMessage::PanLeft)
    );
}

#[test]
fn test_h_maps_to_pan_left() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::char('h')),
        Some(TimelineMessage::PanLeft)
    );
}

#[test]
fn test_right_maps_to_pan_right() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::key(KeyCode::Right)),
        Some(TimelineMessage::PanRight)
    );
}

#[test]
fn test_l_maps_to_pan_right() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::char('l')),
        Some(TimelineMessage::PanRight)
    );
}

#[test]
fn test_plus_maps_to_zoom_in() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::char('+')),
        Some(TimelineMessage::ZoomIn)
    );
}

#[test]
fn test_equals_maps_to_zoom_in() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::char('=')),
        Some(TimelineMessage::ZoomIn)
    );
}

#[test]
fn test_minus_maps_to_zoom_out() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::char('-')),
        Some(TimelineMessage::ZoomOut)
    );
}

#[test]
fn test_up_maps_to_select_prev() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(TimelineMessage::SelectPrev)
    );
}

#[test]
fn test_k_maps_to_select_prev() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::char('k')),
        Some(TimelineMessage::SelectPrev)
    );
}

#[test]
fn test_down_maps_to_select_next() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(TimelineMessage::SelectNext)
    );
}

#[test]
fn test_j_maps_to_select_next() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::char('j')),
        Some(TimelineMessage::SelectNext)
    );
}

#[test]
fn test_home_maps_to_fit_all() {
    let state = focused_timeline();
    assert_eq!(
        Timeline::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(TimelineMessage::FitAll)
    );
}

// =============================================================================
// Disabled/unfocused state
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_timeline();
    state.set_disabled(true);
    let msg = Timeline::handle_event(&state, &Event::key(KeyCode::Left));
    assert_eq!(msg, None);
}

#[test]
fn test_unfocused_ignores_events() {
    let state = TimelineState::new()
        .with_events(sample_events())
        .with_spans(sample_spans());
    let msg = Timeline::handle_event(&state, &Event::key(KeyCode::Left));
    assert_eq!(msg, None);
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_timeline();
    let msg = state.handle_event(&Event::key(KeyCode::Left));
    assert_eq!(msg, Some(TimelineMessage::PanLeft));
}

#[test]
fn test_instance_update() {
    let mut state = focused_timeline();
    let output = state.update(TimelineMessage::SelectNext);
    assert_eq!(output, Some(TimelineOutput::EventSelected("e1".into())));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_timeline();
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(TimelineOutput::EventSelected("e1".into())));
}

// =============================================================================
// Focusable / Disableable
// =============================================================================

#[test]
fn test_focusable() {
    let mut state = TimelineState::new();
    assert!(!Timeline::is_focused(&state));
    Timeline::set_focused(&mut state, true);
    assert!(Timeline::is_focused(&state));
    Timeline::blur(&mut state);
    assert!(!Timeline::is_focused(&state));
    Timeline::focus(&mut state);
    assert!(Timeline::is_focused(&state));
}

#[test]
fn test_disableable() {
    let mut state = TimelineState::new();
    assert!(!Timeline::is_disabled(&state));
    Timeline::set_disabled(&mut state, true);
    assert!(Timeline::is_disabled(&state));
    Timeline::enable(&mut state);
    assert!(!Timeline::is_disabled(&state));
    Timeline::disable(&mut state);
    assert!(Timeline::is_disabled(&state));
}

// =============================================================================
// Effective lane count
// =============================================================================

#[test]
fn test_effective_lane_count_auto() {
    let state = TimelineState::new().with_spans(sample_spans());
    // Spans have lanes 0 and 1
    assert_eq!(state.effective_lane_count(), 2);
}

#[test]
fn test_effective_lane_count_no_spans() {
    let state = TimelineState::new();
    assert_eq!(state.effective_lane_count(), 0);
}

#[test]
fn test_effective_lane_count_manual() {
    let mut state = TimelineState::new().with_spans(sample_spans());
    state.lane_count = 5;
    assert_eq!(state.effective_lane_count(), 5);
}

// =============================================================================
// Rendering (view tests)
// =============================================================================

#[test]
fn test_render_empty() {
    let state = TimelineState::default();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_events() {
    let state = TimelineState::new()
        .with_events(sample_events())
        .with_view_range(0.0, 1000.0);
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_spans() {
    let state = TimelineState::new()
        .with_spans(sample_spans())
        .with_view_range(0.0, 1000.0);
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_events_and_spans() {
    let state = TimelineState::new()
        .with_events(sample_events())
        .with_spans(sample_spans())
        .with_view_range(0.0, 1000.0)
        .with_title("Trace Timeline");
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_selection() {
    let mut state = focused_timeline();
    state.update(TimelineMessage::SelectNext);
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = TimelineState::new()
        .with_events(sample_events())
        .with_spans(sample_spans())
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let state = focused_timeline();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = focused_timeline();
    let (mut terminal, theme) = test_utils::setup_render(60, 2);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_minimal_height() {
    let state = focused_timeline();
    let (mut terminal, theme) = test_utils::setup_render(60, 5);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_very_wide() {
    let state = focused_timeline();
    let (mut terminal, theme) = test_utils::setup_render(120, 15);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// Annotation tests
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = focused_timeline();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Timeline::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    assert!(registry.get_by_id("timeline").is_some());
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_events_only_no_spans() {
    let mut state = TimelineState::new().with_events(vec![TimelineEvent::new("e1", 100.0, "Solo")]);
    state.set_focused(true);
    let output = state.update(TimelineMessage::SelectNext);
    assert_eq!(output, Some(TimelineOutput::EventSelected("e1".into())));
    // Next should wrap back
    let output = state.update(TimelineMessage::SelectNext);
    assert_eq!(output, Some(TimelineOutput::EventSelected("e1".into())));
}

#[test]
fn test_spans_only_no_events() {
    let mut state =
        TimelineState::new().with_spans(vec![TimelineSpan::new("s1", 0.0, 100.0, "Solo")]);
    state.set_focused(true);
    let output = state.update(TimelineMessage::SelectNext);
    assert_eq!(output, Some(TimelineOutput::SpanSelected("s1".into())));
}

#[test]
fn test_overlapping_spans_same_lane() {
    let spans = vec![
        TimelineSpan::new("s1", 0.0, 500.0, "A"),
        TimelineSpan::new("s2", 200.0, 700.0, "B"),
    ];
    let state = TimelineState::new()
        .with_spans(spans)
        .with_view_range(0.0, 1000.0);
    // Should render without panic
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_data_bounds_events_only() {
    let state = TimelineState::new().with_events(vec![
        TimelineEvent::new("e1", 100.0, "A"),
        TimelineEvent::new("e2", 900.0, "B"),
    ]);
    let (min, max) = state.data_bounds();
    assert_eq!(min, 100.0);
    assert_eq!(max, 900.0);
}

#[test]
fn test_data_bounds_spans_only() {
    let state = TimelineState::new().with_spans(vec![
        TimelineSpan::new("s1", 50.0, 300.0, "A"),
        TimelineSpan::new("s2", 200.0, 800.0, "B"),
    ]);
    let (min, max) = state.data_bounds();
    assert_eq!(min, 50.0);
    assert_eq!(max, 800.0);
}

#[test]
fn test_data_bounds_mixed() {
    let state = TimelineState::new()
        .with_events(vec![TimelineEvent::new("e1", 10.0, "A")])
        .with_spans(vec![TimelineSpan::new("s1", 200.0, 900.0, "B")]);
    let (min, max) = state.data_bounds();
    assert_eq!(min, 10.0);
    assert_eq!(max, 900.0);
}

#[test]
fn test_data_bounds_empty() {
    let state = TimelineState::new();
    let (min, max) = state.data_bounds();
    assert_eq!(min, 0.0);
    assert_eq!(max, 0.0);
}

#[test]
fn test_partial_eq() {
    let state1 = focused_timeline();
    let state2 = focused_timeline();
    assert_eq!(state1, state2);
}
