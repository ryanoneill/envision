use super::*;
use crate::component::test_utils;

fn sample_widgets() -> Vec<MetricWidget> {
    vec![
        MetricWidget::counter("Requests", 42),
        MetricWidget::gauge("CPU %", 75, 100),
        MetricWidget::status("API", true),
        MetricWidget::counter("Errors", 3),
        MetricWidget::gauge("Memory", 512, 1024),
        MetricWidget::text("Version", "1.2.3"),
    ]
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_default_empty() {
    let state = MetricsDashboardState::default();
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_populated() {
    let state = MetricsDashboardState::new(sample_widgets(), 3);
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused() {
    let state = MetricsDashboardState::new(sample_widgets(), 3);
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(
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
fn test_snapshot_focused_second_widget() {
    let mut state = MetricsDashboardState::new(sample_widgets(), 3);
    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right);
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(
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
fn test_snapshot_two_columns() {
    let state = MetricsDashboardState::new(
        vec![
            MetricWidget::counter("Requests", 1500),
            MetricWidget::gauge("CPU", 85, 100),
            MetricWidget::status("Database", true),
            MetricWidget::status("Cache", false),
        ],
        2,
    );
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_sparkline_history() {
    let mut widgets = vec![
        MetricWidget::counter("Requests", 0).with_max_history(10),
        MetricWidget::gauge("CPU %", 0, 100).with_max_history(10),
    ];
    // Add some history values
    for i in 0..8 {
        widgets[0].set_counter_value((i * 10) as i64);
        widgets[1].set_gauge_value((i * 12) as u64);
    }
    let state = MetricsDashboardState::new(widgets, 2);
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
