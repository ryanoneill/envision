use super::*;
use crate::component::test_utils;

fn sample_series() -> Vec<DataSeries> {
    vec![
        DataSeries::new("Series A", vec![10.0, 20.0, 30.0, 25.0, 15.0]),
        DataSeries::new("Series B", vec![5.0, 15.0, 10.0, 20.0, 25.0]).with_color(Color::Red),
    ]
}

// =============================================================================
// ThresholdLine
// =============================================================================

#[test]
fn test_threshold_line_new() {
    let t = ThresholdLine::new(95.0, "SLO Target", Color::Yellow);
    assert_eq!(t.value, 95.0);
    assert_eq!(t.label, "SLO Target");
    assert_eq!(t.color, Color::Yellow);
}

#[test]
fn test_threshold_line_clone() {
    let t = ThresholdLine::new(50.0, "Midpoint", Color::Green);
    let t2 = t.clone();
    assert_eq!(t, t2);
}

// =============================================================================
// Threshold state management
// =============================================================================

#[test]
fn test_with_threshold_builder() {
    let state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])])
        .with_threshold(90.0, "Warning", Color::Yellow)
        .with_threshold(95.0, "Critical", Color::Red);
    assert_eq!(state.thresholds().len(), 2);
    assert_eq!(state.thresholds()[0].value, 90.0);
    assert_eq!(state.thresholds()[0].label, "Warning");
    assert_eq!(state.thresholds()[1].value, 95.0);
}

#[test]
fn test_add_threshold() {
    let mut state = ChartState::area(vec![]);
    state.add_threshold(ThresholdLine::new(100.0, "Max", Color::Red));
    assert_eq!(state.thresholds().len(), 1);
    assert_eq!(state.thresholds()[0].value, 100.0);
}

#[test]
fn test_clear_thresholds() {
    let mut state = ChartState::area(vec![])
        .with_threshold(50.0, "Mid", Color::Gray)
        .with_threshold(100.0, "Max", Color::Red);
    assert_eq!(state.thresholds().len(), 2);
    state.clear_thresholds();
    assert!(state.thresholds().is_empty());
}

#[test]
fn test_set_thresholds_message() {
    let mut state = ChartState::area(vec![DataSeries::new("X", vec![1.0])]);
    let thresholds = vec![
        ThresholdLine::new(10.0, "Low", Color::Green),
        ThresholdLine::new(90.0, "High", Color::Red),
    ];
    let output = Chart::update(&mut state, ChartMessage::SetThresholds(thresholds));
    assert_eq!(output, None);
    assert_eq!(state.thresholds().len(), 2);
    assert_eq!(state.thresholds()[0].value, 10.0);
}

#[test]
fn test_add_threshold_message() {
    let mut state = ChartState::area(vec![DataSeries::new("X", vec![1.0])]);
    let output = Chart::update(
        &mut state,
        ChartMessage::AddThreshold(ThresholdLine::new(50.0, "Mid", Color::Cyan)),
    );
    assert_eq!(output, None);
    assert_eq!(state.thresholds().len(), 1);
}

// =============================================================================
// Manual Y-axis range
// =============================================================================

#[test]
fn test_with_y_range_builder() {
    let state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])]).with_y_range(0.0, 100.0);
    assert_eq!(state.y_min(), Some(0.0));
    assert_eq!(state.y_max(), Some(100.0));
}

#[test]
fn test_set_y_range() {
    let mut state = ChartState::area(vec![]);
    state.set_y_range(Some(10.0), Some(90.0));
    assert_eq!(state.y_min(), Some(10.0));
    assert_eq!(state.y_max(), Some(90.0));
}

#[test]
fn test_set_y_range_partial() {
    let mut state = ChartState::area(vec![]);
    state.set_y_range(Some(0.0), None);
    assert_eq!(state.y_min(), Some(0.0));
    assert_eq!(state.y_max(), None);
}

#[test]
fn test_set_y_range_message() {
    let mut state = ChartState::area(vec![DataSeries::new("X", vec![1.0])]);
    let output = Chart::update(&mut state, ChartMessage::SetYRange(Some(0.0), Some(200.0)));
    assert_eq!(output, None);
    assert_eq!(state.y_min(), Some(0.0));
    assert_eq!(state.y_max(), Some(200.0));
}

#[test]
fn test_set_y_range_clear_message() {
    let mut state =
        ChartState::area(vec![DataSeries::new("X", vec![1.0])]).with_y_range(0.0, 100.0);
    Chart::update(&mut state, ChartMessage::SetYRange(None, None));
    assert_eq!(state.y_min(), None);
    assert_eq!(state.y_max(), None);
}

// =============================================================================
// Effective min/max
// =============================================================================

#[test]
fn test_effective_min_auto() {
    let state = ChartState::area(vec![
        DataSeries::new("A", vec![10.0, 20.0]),
        DataSeries::new("B", vec![5.0, 15.0]),
    ]);
    assert_eq!(state.effective_min(), 5.0);
}

#[test]
fn test_effective_max_auto() {
    let state = ChartState::area(vec![
        DataSeries::new("A", vec![10.0, 20.0]),
        DataSeries::new("B", vec![5.0, 15.0]),
    ]);
    assert_eq!(state.effective_max(), 20.0);
}

#[test]
fn test_effective_min_manual() {
    let state =
        ChartState::area(vec![DataSeries::new("A", vec![10.0, 20.0])]).with_y_range(0.0, 50.0);
    assert_eq!(state.effective_min(), 0.0);
}

#[test]
fn test_effective_max_manual() {
    let state =
        ChartState::area(vec![DataSeries::new("A", vec![10.0, 20.0])]).with_y_range(0.0, 50.0);
    assert_eq!(state.effective_max(), 50.0);
}

#[test]
fn test_effective_min_includes_thresholds() {
    let state = ChartState::area(vec![DataSeries::new("A", vec![10.0, 20.0])]).with_threshold(
        5.0,
        "Low",
        Color::Green,
    );
    // Threshold at 5.0 is below data min of 10.0
    assert_eq!(state.effective_min(), 5.0);
}

#[test]
fn test_effective_max_includes_thresholds() {
    let state = ChartState::area(vec![DataSeries::new("A", vec![10.0, 20.0])]).with_threshold(
        95.0,
        "SLO",
        Color::Yellow,
    );
    // Threshold at 95.0 is above data max of 20.0
    assert_eq!(state.effective_max(), 95.0);
}

#[test]
fn test_effective_range_manual_overrides_thresholds() {
    let state = ChartState::area(vec![DataSeries::new("A", vec![10.0, 20.0])])
        .with_threshold(5.0, "Low", Color::Green)
        .with_threshold(95.0, "High", Color::Red)
        .with_y_range(0.0, 100.0);
    // Manual range takes precedence
    assert_eq!(state.effective_min(), 0.0);
    assert_eq!(state.effective_max(), 100.0);
}

// =============================================================================
// Area chart rendering
// =============================================================================

#[test]
fn test_render_area_chart() {
    let state = ChartState::area(vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 48.0, 65.0, 72.0],
    )]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_area_chart_with_labels() {
    let state = ChartState::area(vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 48.0, 65.0, 72.0],
    )])
    .with_title("CPU Usage")
    .with_x_label("Time")
    .with_y_label("Percent");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_area_chart_multi_series() {
    let state = ChartState::area(sample_series());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Scatter chart rendering
// =============================================================================

#[test]
fn test_render_scatter_chart() {
    let state = ChartState::scatter(vec![DataSeries::new(
        "Points",
        vec![10.0, 25.0, 15.0, 30.0, 20.0],
    )]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_scatter_chart_multi_series() {
    let state = ChartState::scatter(sample_series()).with_title("Scatter Plot");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Threshold rendering
// =============================================================================

#[test]
fn test_render_area_chart_with_thresholds() {
    let state = ChartState::area(vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 80.0, 92.0, 72.0],
    )])
    .with_threshold(90.0, "Warning", Color::Yellow)
    .with_threshold(95.0, "Critical", Color::Red);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_scatter_with_thresholds() {
    let state = ChartState::scatter(vec![DataSeries::new(
        "Latency",
        vec![50.0, 120.0, 80.0, 200.0, 90.0],
    )])
    .with_threshold(100.0, "SLO", Color::Yellow)
    .with_y_range(0.0, 250.0);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Manual y_range rendering
// =============================================================================

#[test]
fn test_render_area_chart_with_y_range() {
    let state = ChartState::area(vec![DataSeries::new("CPU", vec![45.0, 52.0, 48.0])])
        .with_y_range(0.0, 100.0);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Edge cases with new features
// =============================================================================

#[test]
fn test_render_empty_area_chart() {
    let state = ChartState::area(vec![]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_empty_scatter_chart() {
    let state = ChartState::scatter(vec![]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_threshold_outside_data_range() {
    // Threshold at 200 when data max is 30
    let state = ChartState::area(sample_series()).with_threshold(200.0, "Way Above", Color::Red);
    assert_eq!(state.effective_max(), 200.0);
    assert_eq!(state.effective_min(), 5.0);
}

#[test]
fn test_threshold_below_data_range() {
    // Threshold at -10 when data min is 5
    let state = ChartState::area(sample_series()).with_threshold(-10.0, "Way Below", Color::Blue);
    assert_eq!(state.effective_min(), -10.0);
    assert_eq!(state.effective_max(), 30.0);
}

#[test]
fn test_empty_data_with_thresholds() {
    let state = ChartState::area(vec![]).with_threshold(50.0, "Mid", Color::Yellow);
    // With no data series, the chart won't render the inner content
    // but thresholds should still be accessible
    assert_eq!(state.thresholds().len(), 1);
    assert_eq!(state.effective_min(), 0.0); // global_min returns 0.0 for empty
    assert_eq!(state.effective_max(), 50.0); // threshold value
}

#[test]
fn test_area_chart_disabled() {
    let state =
        ChartState::area(vec![DataSeries::new("CPU", vec![45.0, 52.0, 48.0])]).with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_scatter_chart_disabled() {
    let state = ChartState::scatter(vec![DataSeries::new("Points", vec![10.0, 20.0, 30.0])])
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_set_kind_to_area() {
    let mut state = ChartState::line(sample_series());
    state.set_kind(ChartKind::Area);
    assert_eq!(state.kind(), &ChartKind::Area);
}

#[test]
fn test_set_kind_to_scatter() {
    let mut state = ChartState::line(sample_series());
    state.set_kind(ChartKind::Scatter);
    assert_eq!(state.kind(), &ChartKind::Scatter);
}

#[test]
fn test_chart_kind_eq() {
    assert_eq!(ChartKind::Area, ChartKind::Area);
    assert_eq!(ChartKind::Scatter, ChartKind::Scatter);
    assert_ne!(ChartKind::Area, ChartKind::Scatter);
    assert_ne!(ChartKind::Area, ChartKind::Line);
}

#[test]
fn test_disabled_still_processes_threshold_messages() {
    let mut state = ChartState::area(vec![DataSeries::new("X", vec![1.0])]);
    state.set_disabled(true);
    // Threshold and range messages should still work even when disabled
    let output = Chart::update(
        &mut state,
        ChartMessage::AddThreshold(ThresholdLine::new(50.0, "Mid", Color::Cyan)),
    );
    assert_eq!(output, None);
    assert_eq!(state.thresholds().len(), 1);
}

#[test]
fn test_disabled_still_processes_y_range_messages() {
    let mut state = ChartState::area(vec![DataSeries::new("X", vec![1.0])]);
    state.set_disabled(true);
    let output = Chart::update(&mut state, ChartMessage::SetYRange(Some(0.0), Some(100.0)));
    assert_eq!(output, None);
    assert_eq!(state.y_min(), Some(0.0));
    assert_eq!(state.y_max(), Some(100.0));
}
