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

#[test]
fn test_snapshot_empty() {
    let state = TimelineState::default();
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_events_only() {
    let state = TimelineState::new()
        .with_events(sample_events())
        .with_view_range(0.0, 1000.0)
        .with_title("Events Only");
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_spans_only() {
    let state = TimelineState::new()
        .with_spans(sample_spans())
        .with_view_range(0.0, 1000.0)
        .with_title("Spans Only");
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_full_timeline() {
    let state = TimelineState::new()
        .with_events(sample_events())
        .with_spans(sample_spans())
        .with_view_range(0.0, 1000.0)
        .with_title("Full Timeline");
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_selection() {
    let mut state = TimelineState::new()
        .with_events(sample_events())
        .with_spans(sample_spans())
        .with_view_range(0.0, 1000.0)
        .with_title("Selected Timeline");
    state.set_focused(true);
    state.update(TimelineMessage::SelectNext);
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let state = TimelineState::new()
        .with_events(sample_events())
        .with_spans(sample_spans())
        .with_view_range(0.0, 1000.0)
        .with_title("Disabled Timeline")
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_span_selected() {
    let mut state = TimelineState::new()
        .with_events(sample_events())
        .with_spans(sample_spans())
        .with_view_range(0.0, 1000.0)
        .with_title("Span Selected");
    state.set_focused(true);
    // Navigate past 3 events to first span
    for _ in 0..4 {
        state.update(TimelineMessage::SelectNext);
    }
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            Timeline::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
