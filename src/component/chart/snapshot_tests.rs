use super::*;
use crate::component::test_utils;

fn sample_series() -> Vec<DataSeries> {
    vec![
        DataSeries::new("Series A", vec![10.0, 20.0, 30.0, 25.0, 15.0]),
        DataSeries::new("Series B", vec![5.0, 15.0, 10.0, 20.0, 25.0]).with_color(Color::Red),
    ]
}

// =============================================================================
// Snapshot tests (existing)
// =============================================================================

#[test]
fn test_snapshot_default_empty() {
    let state = ChartState::default();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_populated_line_chart() {
    let state = ChartState::line(sample_series())
        .with_title("Temperature")
        .with_x_label("Time")
        .with_y_label("Value");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_line_chart() {
    let mut state = ChartState::line(sample_series()).with_title("CPU Usage");
    state.set_focused(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(
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
fn test_snapshot_bar_vertical() {
    let state = ChartState::bar_vertical(vec![DataSeries::new(
        "Sales",
        vec![10.0, 25.0, 15.0, 30.0, 20.0],
    )])
    .with_title("Quarterly Sales");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_bar_horizontal() {
    let state =
        ChartState::bar_horizontal(vec![DataSeries::new("Revenue", vec![100.0, 200.0, 150.0])])
            .with_title("Revenue by Region");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_series() {
    let state = ChartState::line(vec![DataSeries::new(
        "Memory",
        vec![40.0, 55.0, 60.0, 45.0, 70.0, 65.0],
    )]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// New chart type snapshots
// =============================================================================

#[test]
fn test_snapshot_area_chart() {
    let state = ChartState::area(vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 48.0, 65.0, 72.0, 58.0, 61.0],
    )])
    .with_title("CPU Usage (Area)");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_scatter_chart() {
    let state = ChartState::scatter(vec![DataSeries::new(
        "Latency",
        vec![50.0, 120.0, 80.0, 200.0, 90.0, 150.0, 110.0],
    )])
    .with_title("Request Latency (Scatter)");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_area_chart_with_thresholds() {
    let state = ChartState::area(vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 80.0, 92.0, 72.0, 85.0],
    )])
    .with_title("CPU with Thresholds")
    .with_threshold(90.0, "Warning", Color::Yellow)
    .with_threshold(95.0, "Critical", Color::Red);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_area_chart_with_y_range() {
    let state = ChartState::area(vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 48.0, 65.0, 72.0],
    )])
    .with_title("CPU (0-100 range)")
    .with_y_range(0.0, 100.0);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_multi_series_area() {
    let state = ChartState::area(sample_series()).with_title("Multi-Series Area");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_multi_series_scatter() {
    let state = ChartState::scatter(sample_series()).with_title("Multi-Series Scatter");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
