use super::*;
use crate::component::test_utils;

fn sample_metrics() -> Vec<AlertMetric> {
    vec![
        AlertMetric::new("cpu", "CPU Usage", AlertThreshold::new(70.0, 90.0))
            .with_units("%")
            .with_value(45.2),
        AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
            .with_units("%")
            .with_value(82.5),
        AlertMetric::new("disk", "Disk I/O", AlertThreshold::new(100.0, 200.0))
            .with_units("MB/s")
            .with_value(12.0),
        AlertMetric::new("errors", "Error Rate", AlertThreshold::new(1.0, 5.0))
            .with_units("%")
            .with_value(6.2),
    ]
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state = AlertPanelState::new();
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_metrics() {
    let state = AlertPanelState::new()
        .with_metrics(sample_metrics())
        .with_columns(2)
        .with_title("System Alerts");
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused() {
    let state = AlertPanelState::new()
        .with_metrics(sample_metrics())
        .with_columns(2)
        .with_title("Alerts");
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            AlertPanel::view(
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
fn test_snapshot_disabled() {
    let state = AlertPanelState::new()
        .with_metrics(sample_metrics())
        .with_columns(2);
    let (mut terminal, theme) = test_utils::setup_render(80, 25);
    terminal
        .draw(|frame| {
            AlertPanel::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_all_ok() {
    let state = AlertPanelState::new()
        .with_metrics(vec![
            AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
                .with_units("%")
                .with_value(30.0),
            AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
                .with_units("%")
                .with_value(40.0),
        ])
        .with_columns(2)
        .with_title("All OK");
    let (mut terminal, theme) = test_utils::setup_render(80, 15);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_all_critical() {
    let state = AlertPanelState::new()
        .with_metrics(vec![
            AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
                .with_units("%")
                .with_value(95.0),
            AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
                .with_units("%")
                .with_value(98.0),
        ])
        .with_columns(2)
        .with_title("All Critical");
    let (mut terminal, theme) = test_utils::setup_render(80, 15);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_sparklines() {
    let mut metrics = vec![
        AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
            .with_units("%")
            .with_max_history(10),
        AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
            .with_units("%")
            .with_max_history(10),
    ];
    // Add history
    for i in 0..8 {
        metrics[0].update_value(30.0 + i as f64 * 5.0);
        metrics[1].update_value(60.0 + i as f64 * 3.0);
    }
    let state = AlertPanelState::new().with_metrics(metrics).with_columns(2);
    let (mut terminal, theme) = test_utils::setup_render(80, 15);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_metric() {
    let state = AlertPanelState::new()
        .with_metrics(vec![AlertMetric::new(
            "cpu",
            "CPU Usage",
            AlertThreshold::new(70.0, 90.0),
        )
        .with_units("%")
        .with_value(50.0)])
        .with_columns(1);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_thresholds() {
    let state = AlertPanelState::new()
        .with_metrics(vec![
            AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
                .with_units("%")
                .with_value(75.0),
            AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
                .with_units("%")
                .with_value(50.0),
        ])
        .with_columns(2)
        .with_show_thresholds(true);
    let (mut terminal, theme) = test_utils::setup_render(80, 15);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
