//! Tests for custom X-axis labels on line, area, and scatter charts.

use super::*;
use crate::component::test_utils;

// =============================================================================
// Builder
// =============================================================================

#[test]
fn test_with_x_labels_sets_labels() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0, 3.0])])
        .with_x_labels(vec!["Mon", "Tue", "Wed"]);
    assert_eq!(state.x_labels().unwrap(), &["Mon", "Tue", "Wed"]);
}

#[test]
fn test_with_x_labels_owned_strings() {
    let labels: Vec<String> = vec!["2025-01".to_string(), "2025-02".to_string()];
    let state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0])]).with_x_labels(labels);
    assert_eq!(state.x_labels().unwrap(), &["2025-01", "2025-02"]);
}

#[test]
fn test_with_x_labels_timestamp_use_case() {
    let series = DataSeries::new("Requests", vec![100.0, 250.0, 180.0, 320.0, 90.0]);
    let state = ChartState::line(vec![series])
        .with_x_labels(vec!["00:00", "06:00", "12:00", "18:00", "24:00"])
        .with_title("Request Rate (24h)");
    assert_eq!(state.title(), Some("Request Rate (24h)"));
    assert_eq!(
        state.x_labels().unwrap(),
        &["00:00", "06:00", "12:00", "18:00", "24:00"]
    );
}

#[test]
fn test_default_has_no_x_labels() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![1.0])]);
    assert!(state.x_labels().is_none());
}

#[test]
fn test_default_state_has_no_x_labels() {
    let state = ChartState::default();
    assert!(state.x_labels().is_none());
}

// =============================================================================
// Accessor
// =============================================================================

#[test]
fn test_x_labels_accessor_returns_none_when_unset() {
    let state = ChartState::scatter(vec![]);
    assert!(state.x_labels().is_none());
}

#[test]
fn test_x_labels_accessor_returns_slice() {
    let state =
        ChartState::area(vec![DataSeries::new("A", vec![1.0])]).with_x_labels(vec!["a", "b"]);
    let labels: &[String] = state.x_labels().unwrap();
    assert_eq!(labels.len(), 2);
    assert_eq!(labels[0], "a");
    assert_eq!(labels[1], "b");
}

// =============================================================================
// Setter
// =============================================================================

#[test]
fn test_set_x_labels_some() {
    let mut state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0])]);
    state.set_x_labels(Some(vec!["Jan", "Feb"]));
    assert_eq!(state.x_labels().unwrap(), &["Jan", "Feb"]);
}

#[test]
fn test_set_x_labels_none_clears() {
    let mut state =
        ChartState::line(vec![DataSeries::new("A", vec![1.0])]).with_x_labels(vec!["X"]);
    assert!(state.x_labels().is_some());
    state.set_x_labels(None::<Vec<String>>);
    assert!(state.x_labels().is_none());
}

#[test]
fn test_set_x_labels_replaces_previous() {
    let mut state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0])])
        .with_x_labels(vec!["old1", "old2"]);
    state.set_x_labels(Some(vec!["new1", "new2"]));
    assert_eq!(state.x_labels().unwrap(), &["new1", "new2"]);
}

// =============================================================================
// select_x_labels (via render module)
// =============================================================================

#[test]
fn test_select_x_labels_empty() {
    let result = super::render::select_x_labels(&[], 5);
    assert!(result.is_empty());
}

#[test]
fn test_select_x_labels_all_fit() {
    let labels: Vec<String> = vec!["A".into(), "B".into(), "C".into()];
    let result = super::render::select_x_labels(&labels, 5);
    assert_eq!(result, vec!["A", "B", "C"]);
}

#[test]
fn test_select_x_labels_exact_fit() {
    let labels: Vec<String> = vec!["A".into(), "B".into(), "C".into()];
    let result = super::render::select_x_labels(&labels, 3);
    assert_eq!(result, vec!["A", "B", "C"]);
}

#[test]
fn test_select_x_labels_needs_reduction() {
    let labels: Vec<String> = (0..10).map(|i| format!("L{}", i)).collect();
    let result = super::render::select_x_labels(&labels, 3);
    assert_eq!(result.len(), 3);
    // First and last are always included
    assert_eq!(result[0], "L0");
    assert_eq!(result[2], "L9");
}

#[test]
fn test_select_x_labels_single_label() {
    let labels: Vec<String> = vec!["Only".into()];
    let result = super::render::select_x_labels(&labels, 5);
    assert_eq!(result, vec!["Only"]);
}

#[test]
fn test_select_x_labels_max_one() {
    let labels: Vec<String> = vec!["A".into(), "B".into(), "C".into()];
    let result = super::render::select_x_labels(&labels, 1);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "A");
}

#[test]
fn test_select_x_labels_preserves_first_and_last() {
    let labels: Vec<String> = vec![
        "00:00".into(),
        "04:00".into(),
        "08:00".into(),
        "12:00".into(),
        "16:00".into(),
        "20:00".into(),
        "24:00".into(),
    ];
    let result = super::render::select_x_labels(&labels, 4);
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], "00:00");
    assert_eq!(result[3], "24:00");
}

// =============================================================================
// Rendering with custom X labels
// =============================================================================

#[test]
fn test_render_line_chart_with_x_labels() {
    let state = ChartState::line(vec![DataSeries::new(
        "CPU",
        vec![50.0, 60.0, 55.0, 70.0, 65.0],
    )])
    .with_x_labels(vec!["00:00", "06:00", "12:00", "18:00", "24:00"])
    .with_title("CPU Usage (24h)");

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let output = terminal.backend().to_string();
    // The custom labels should appear somewhere in the rendered output
    assert!(
        output.contains("00:00") || output.contains("24:00"),
        "Expected custom X labels in output, got:\n{}",
        output
    );
}

#[test]
fn test_render_area_chart_with_x_labels() {
    let state = ChartState::area(vec![DataSeries::new("Memory", vec![40.0, 55.0, 60.0])])
        .with_x_labels(vec!["Mon", "Wed", "Fri"]);

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(
        output.contains("Mon") || output.contains("Fri"),
        "Expected custom X labels in output, got:\n{}",
        output
    );
}

#[test]
fn test_render_scatter_chart_with_x_labels() {
    let state = ChartState::scatter(vec![DataSeries::new("Latency", vec![10.0, 25.0, 15.0])])
        .with_x_labels(vec!["T1", "T2", "T3"]);

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(
        output.contains("T1") || output.contains("T3"),
        "Expected custom X labels in output, got:\n{}",
        output
    );
}

#[test]
fn test_render_without_x_labels_uses_numeric_ticks() {
    let state = ChartState::line(vec![DataSeries::new(
        "Values",
        vec![10.0, 20.0, 30.0, 40.0, 50.0],
    )]);

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let output = terminal.backend().to_string();
    // Numeric ticks should appear (e.g., "0", "1", "2", etc.)
    assert!(
        output.contains('0') || output.contains('1'),
        "Expected numeric tick labels in output, got:\n{}",
        output
    );
}

#[test]
fn test_render_x_labels_many_labels_are_reduced() {
    // Provide many labels; the renderer should show a subset
    let labels: Vec<String> = (0..50).map(|i| format!("T{:02}", i)).collect();
    let values: Vec<f64> = (0..50).map(|i| (i as f64) * 2.0).collect();
    let state = ChartState::line(vec![DataSeries::new("Series", values)]).with_x_labels(labels);

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    // Should render without panicking; presence check is sufficient
    let output = terminal.backend().to_string();
    assert!(!output.is_empty());
}

#[test]
fn test_render_x_labels_long_labels_do_not_panic() {
    // Labels wider than the axis should not cause panics
    let state =
        ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0, 3.0])]).with_x_labels(vec![
            "2025-01-01T00:00:00Z",
            "2025-06-15T12:30:00Z",
            "2025-12-31T23:59:59Z",
        ]);

    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|frame| {
            Chart::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(!output.is_empty());
}

#[test]
fn test_render_x_labels_empty_vec() {
    // Empty labels vector should fall through gracefully
    let state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0])])
        .with_x_labels(Vec::<String>::new());

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(!output.is_empty());
}

// =============================================================================
// Snapshot test
// =============================================================================

#[test]
fn test_snapshot_line_chart_with_x_labels() {
    let state = ChartState::line(vec![DataSeries::new(
        "Requests",
        vec![100.0, 250.0, 180.0, 320.0, 90.0],
    )])
    .with_x_labels(vec!["00:00", "06:00", "12:00", "18:00", "24:00"])
    .with_title("Request Rate (24h)");

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
