use super::*;
use crate::component::test_utils;
use crate::input::Event;

fn sample_metrics() -> Vec<AlertMetric> {
    vec![
        AlertMetric::new("cpu", "CPU Usage", AlertThreshold::new(70.0, 90.0))
            .with_units("%")
            .with_value(45.2),
        AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
            .with_units("%")
            .with_value(78.5),
        AlertMetric::new("disk", "Disk I/O", AlertThreshold::new(100.0, 200.0))
            .with_units("MB/s")
            .with_value(12.0),
        AlertMetric::new("errors", "Error Rate", AlertThreshold::new(1.0, 5.0))
            .with_units("%")
            .with_value(0.3),
    ]
}

fn focused_state() -> AlertPanelState {
    let mut state = AlertPanelState::new()
        .with_metrics(sample_metrics())
        .with_columns(2);
    state.set_focused(true);
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = AlertPanelState::new();
    assert!(state.metrics().is_empty());
    assert_eq!(state.columns(), 2);
    assert_eq!(state.selected(), None);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert!(state.show_sparklines());
    assert!(!state.show_thresholds());
    assert_eq!(state.title(), None);
}

#[test]
fn test_with_metrics() {
    let state = AlertPanelState::new().with_metrics(sample_metrics());
    assert_eq!(state.metrics().len(), 4);
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_with_columns() {
    let state = AlertPanelState::new().with_columns(3);
    assert_eq!(state.columns(), 3);
}

#[test]
fn test_with_columns_minimum() {
    let state = AlertPanelState::new().with_columns(0);
    assert_eq!(state.columns(), 1);
}

#[test]
fn test_with_title() {
    let state = AlertPanelState::new().with_title("System Alerts");
    assert_eq!(state.title(), Some("System Alerts"));
}

#[test]
fn test_with_show_sparklines() {
    let state = AlertPanelState::new().with_show_sparklines(false);
    assert!(!state.show_sparklines());
}

#[test]
fn test_with_show_thresholds() {
    let state = AlertPanelState::new().with_show_thresholds(true);
    assert!(state.show_thresholds());
}

#[test]
fn test_with_disabled() {
    let state = AlertPanelState::new().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// AlertMetric
// =============================================================================

#[test]
fn test_metric_new() {
    let metric = AlertMetric::new("cpu", "CPU Usage", AlertThreshold::new(70.0, 90.0));
    assert_eq!(metric.id(), "cpu");
    assert_eq!(metric.name(), "CPU Usage");
    assert_eq!(metric.value(), 0.0);
    assert_eq!(metric.state(), &AlertState::Ok);
    assert!(metric.history().is_empty());
    assert_eq!(metric.max_history(), 20);
}

#[test]
fn test_metric_with_units() {
    let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_units("%");
    assert_eq!(metric.units(), Some("%"));
}

#[test]
fn test_metric_with_value() {
    let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(85.0);
    assert_eq!(metric.value(), 85.0);
    assert_eq!(metric.state(), &AlertState::Warning);
}

#[test]
fn test_metric_with_max_history() {
    let metric =
        AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_max_history(50);
    assert_eq!(metric.max_history(), 50);
}

#[test]
fn test_metric_display_value_with_units() {
    let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
        .with_units("%")
        .with_value(45.2);
    assert_eq!(metric.display_value(), "45.2%");
}

#[test]
fn test_metric_display_value_without_units() {
    let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(45.2);
    assert_eq!(metric.display_value(), "45.2");
}

// =============================================================================
// AlertThreshold
// =============================================================================

#[test]
fn test_threshold_new() {
    let threshold = AlertThreshold::new(75.0, 95.0);
    assert_eq!(threshold.warning, 75.0);
    assert_eq!(threshold.critical, 95.0);
}

// =============================================================================
// Compute state
// =============================================================================

#[test]
fn test_compute_state_ok() {
    let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(69.9);
    assert_eq!(metric.compute_state(), AlertState::Ok);
}

#[test]
fn test_compute_state_at_warning_boundary() {
    let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(70.0);
    assert_eq!(metric.compute_state(), AlertState::Warning);
}

#[test]
fn test_compute_state_warning() {
    let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(85.0);
    assert_eq!(metric.compute_state(), AlertState::Warning);
}

#[test]
fn test_compute_state_at_critical_boundary() {
    let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(90.0);
    assert_eq!(metric.compute_state(), AlertState::Critical);
}

#[test]
fn test_compute_state_critical() {
    let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(95.0);
    assert_eq!(metric.compute_state(), AlertState::Critical);
}

// =============================================================================
// Update value and history
// =============================================================================

#[test]
fn test_update_value() {
    let mut metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0));
    metric.update_value(50.0);
    assert_eq!(metric.value(), 50.0);
    assert_eq!(metric.history(), &[50.0]);
}

#[test]
fn test_update_value_history_tracking() {
    let mut metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0));
    metric.update_value(10.0);
    metric.update_value(20.0);
    metric.update_value(30.0);
    assert_eq!(metric.history(), &[10.0, 20.0, 30.0]);
}

#[test]
fn test_update_value_history_cap() {
    let mut metric =
        AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_max_history(3);
    for i in 0..5 {
        metric.update_value(i as f64 * 10.0);
    }
    assert_eq!(metric.history().len(), 3);
    assert_eq!(metric.history(), &[20.0, 30.0, 40.0]);
}

#[test]
fn test_update_value_recomputes_state() {
    let mut metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0));
    assert_eq!(metric.state(), &AlertState::Ok);
    metric.update_value(75.0);
    assert_eq!(metric.state(), &AlertState::Warning);
    metric.update_value(95.0);
    assert_eq!(metric.state(), &AlertState::Critical);
    metric.update_value(50.0);
    assert_eq!(metric.state(), &AlertState::Ok);
}

// =============================================================================
// Metric operations on state
// =============================================================================

#[test]
fn test_add_metric() {
    let mut state = AlertPanelState::new();
    assert_eq!(state.selected(), None);
    state.add_metric(AlertMetric::new(
        "cpu",
        "CPU",
        AlertThreshold::new(70.0, 90.0),
    ));
    assert_eq!(state.metrics().len(), 1);
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_add_metric_preserves_selection() {
    let mut state = AlertPanelState::new().with_metrics(sample_metrics());
    state.add_metric(AlertMetric::new(
        "new",
        "New",
        AlertThreshold::new(50.0, 80.0),
    ));
    assert_eq!(state.metrics().len(), 5);
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_update_metric_state_change() {
    let mut state = AlertPanelState::new().with_metrics(vec![AlertMetric::new(
        "cpu",
        "CPU",
        AlertThreshold::new(70.0, 90.0),
    )
    .with_value(50.0)]);
    let result = state.update_metric("cpu", 80.0);
    assert_eq!(result, Some((AlertState::Ok, AlertState::Warning)));
}

#[test]
fn test_update_metric_no_state_change() {
    let mut state = AlertPanelState::new().with_metrics(vec![AlertMetric::new(
        "cpu",
        "CPU",
        AlertThreshold::new(70.0, 90.0),
    )
    .with_value(50.0)]);
    let result = state.update_metric("cpu", 60.0);
    assert_eq!(result, None);
}

#[test]
fn test_update_metric_not_found() {
    let mut state = AlertPanelState::new().with_metrics(sample_metrics());
    let result = state.update_metric("nonexistent", 50.0);
    assert_eq!(result, None);
}

#[test]
fn test_metric_by_id() {
    let state = AlertPanelState::new().with_metrics(sample_metrics());
    assert!(state.metric_by_id("cpu").is_some());
    assert_eq!(state.metric_by_id("cpu").unwrap().name(), "CPU Usage");
    assert!(state.metric_by_id("nonexistent").is_none());
}

// =============================================================================
// State change detection
// =============================================================================

#[test]
fn test_state_change_ok_to_warning() {
    let mut state = AlertPanelState::new().with_metrics(vec![AlertMetric::new(
        "cpu",
        "CPU",
        AlertThreshold::new(70.0, 90.0),
    )
    .with_value(50.0)]);
    let result = state.update_metric("cpu", 75.0);
    assert_eq!(result, Some((AlertState::Ok, AlertState::Warning)));
}

#[test]
fn test_state_change_warning_to_critical() {
    let mut state = AlertPanelState::new().with_metrics(vec![AlertMetric::new(
        "cpu",
        "CPU",
        AlertThreshold::new(70.0, 90.0),
    )
    .with_value(75.0)]);
    let result = state.update_metric("cpu", 95.0);
    assert_eq!(result, Some((AlertState::Warning, AlertState::Critical)));
}

#[test]
fn test_state_change_critical_to_ok() {
    let mut state = AlertPanelState::new().with_metrics(vec![AlertMetric::new(
        "cpu",
        "CPU",
        AlertThreshold::new(70.0, 90.0),
    )
    .with_value(95.0)]);
    let result = state.update_metric("cpu", 50.0);
    assert_eq!(result, Some((AlertState::Critical, AlertState::Ok)));
}

// =============================================================================
// Aggregate counts
// =============================================================================

#[test]
fn test_ok_count() {
    let state = AlertPanelState::new().with_metrics(sample_metrics());
    // cpu=45.2 (OK), mem=78.5 (OK, threshold 80), disk=12 (OK), errors=0.3 (OK)
    assert_eq!(state.ok_count(), 4);
}

#[test]
fn test_warning_count() {
    let state = AlertPanelState::new().with_metrics(vec![
        AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(75.0),
        AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0)).with_value(85.0),
    ]);
    assert_eq!(state.warning_count(), 2);
}

#[test]
fn test_critical_count() {
    let state = AlertPanelState::new().with_metrics(vec![
        AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(95.0),
        AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0)).with_value(96.0),
    ]);
    assert_eq!(state.critical_count(), 2);
}

#[test]
fn test_unknown_count() {
    let state = AlertPanelState::new();
    assert_eq!(state.unknown_count(), 0);
}

#[test]
fn test_mixed_state_counts() {
    let state = AlertPanelState::new().with_metrics(vec![
        AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(50.0), // OK
        AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0)).with_value(85.0), // Warning
        AlertMetric::new("disk", "Disk", AlertThreshold::new(100.0, 200.0)).with_value(250.0), // Critical
    ]);
    assert_eq!(state.ok_count(), 1);
    assert_eq!(state.warning_count(), 1);
    assert_eq!(state.critical_count(), 1);
}

// =============================================================================
// Navigation
// =============================================================================

#[test]
fn test_select_next() {
    let mut state = focused_state();
    let output = AlertPanel::update(&mut state, AlertPanelMessage::SelectNext);
    assert_eq!(state.selected(), Some(1));
    assert_eq!(output, Some(AlertPanelOutput::MetricSelected("mem".into())));
}

#[test]
fn test_select_prev() {
    let mut state = focused_state();
    AlertPanel::update(&mut state, AlertPanelMessage::SelectNext);
    let output = AlertPanel::update(&mut state, AlertPanelMessage::SelectPrev);
    assert_eq!(state.selected(), Some(0));
    assert_eq!(output, Some(AlertPanelOutput::MetricSelected("cpu".into())));
}

#[test]
fn test_select_next_at_row_end() {
    let mut state = focused_state();
    AlertPanel::update(&mut state, AlertPanelMessage::SelectNext);
    let output = AlertPanel::update(&mut state, AlertPanelMessage::SelectNext);
    assert_eq!(state.selected(), Some(1));
    assert_eq!(output, None);
}

#[test]
fn test_select_prev_at_row_start() {
    let mut state = focused_state();
    let output = AlertPanel::update(&mut state, AlertPanelMessage::SelectPrev);
    assert_eq!(state.selected(), Some(0));
    assert_eq!(output, None);
}

#[test]
fn test_select_down() {
    let mut state = focused_state();
    let output = AlertPanel::update(&mut state, AlertPanelMessage::SelectDown);
    assert_eq!(state.selected(), Some(2));
    assert_eq!(
        output,
        Some(AlertPanelOutput::MetricSelected("disk".into()))
    );
}

#[test]
fn test_select_up() {
    let mut state = focused_state();
    AlertPanel::update(&mut state, AlertPanelMessage::SelectDown);
    let output = AlertPanel::update(&mut state, AlertPanelMessage::SelectUp);
    assert_eq!(state.selected(), Some(0));
    assert_eq!(output, Some(AlertPanelOutput::MetricSelected("cpu".into())));
}

#[test]
fn test_select_up_at_top() {
    let mut state = focused_state();
    let output = AlertPanel::update(&mut state, AlertPanelMessage::SelectUp);
    assert_eq!(output, None);
}

#[test]
fn test_select_down_at_bottom() {
    let mut state = focused_state();
    AlertPanel::update(&mut state, AlertPanelMessage::SelectDown);
    let output = AlertPanel::update(&mut state, AlertPanelMessage::SelectDown);
    assert_eq!(output, None);
}

#[test]
fn test_select_enter() {
    let mut state = focused_state();
    let output = AlertPanel::update(&mut state, AlertPanelMessage::Select);
    assert_eq!(output, Some(AlertPanelOutput::MetricSelected("cpu".into())));
}

// =============================================================================
// Message handling
// =============================================================================

#[test]
fn test_update_metric_message() {
    let mut state = AlertPanelState::new().with_metrics(vec![AlertMetric::new(
        "cpu",
        "CPU",
        AlertThreshold::new(70.0, 90.0),
    )
    .with_value(50.0)]);
    let output = AlertPanel::update(
        &mut state,
        AlertPanelMessage::UpdateMetric {
            id: "cpu".into(),
            value: 80.0,
        },
    );
    assert_eq!(
        output,
        Some(AlertPanelOutput::StateChanged {
            id: "cpu".into(),
            old: AlertState::Ok,
            new_state: AlertState::Warning,
        })
    );
    assert_eq!(state.metric_by_id("cpu").unwrap().value(), 80.0);
}

#[test]
fn test_update_metric_message_no_state_change() {
    let mut state = AlertPanelState::new().with_metrics(vec![AlertMetric::new(
        "cpu",
        "CPU",
        AlertThreshold::new(70.0, 90.0),
    )
    .with_value(50.0)]);
    let output = AlertPanel::update(
        &mut state,
        AlertPanelMessage::UpdateMetric {
            id: "cpu".into(),
            value: 60.0,
        },
    );
    assert_eq!(output, None);
}

#[test]
fn test_add_metric_message() {
    let mut state = AlertPanelState::new();
    AlertPanel::update(
        &mut state,
        AlertPanelMessage::AddMetric(AlertMetric::new(
            "cpu",
            "CPU",
            AlertThreshold::new(70.0, 90.0),
        )),
    );
    assert_eq!(state.metrics().len(), 1);
}

#[test]
fn test_remove_metric_message() {
    let mut state = AlertPanelState::new().with_metrics(sample_metrics());
    AlertPanel::update(&mut state, AlertPanelMessage::RemoveMetric("cpu".into()));
    assert_eq!(state.metrics().len(), 3);
    assert!(state.metric_by_id("cpu").is_none());
}

#[test]
fn test_remove_last_metric_clears_selection() {
    let mut state = AlertPanelState::new().with_metrics(vec![AlertMetric::new(
        "cpu",
        "CPU",
        AlertThreshold::new(70.0, 90.0),
    )]);
    AlertPanel::update(&mut state, AlertPanelMessage::RemoveMetric("cpu".into()));
    assert_eq!(state.selected(), None);
}

#[test]
fn test_remove_metric_adjusts_selection() {
    let mut state = AlertPanelState::new().with_metrics(vec![
        AlertMetric::new("a", "A", AlertThreshold::new(70.0, 90.0)),
        AlertMetric::new("b", "B", AlertThreshold::new(70.0, 90.0)),
    ]);
    // Select last item
    AlertPanel::update(&mut state, AlertPanelMessage::SelectNext);
    assert_eq!(state.selected(), Some(1));
    // Remove last item
    AlertPanel::update(&mut state, AlertPanelMessage::RemoveMetric("b".into()));
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_set_metrics_message() {
    let mut state = AlertPanelState::new();
    let metrics = sample_metrics();
    AlertPanel::update(&mut state, AlertPanelMessage::SetMetrics(metrics));
    assert_eq!(state.metrics().len(), 4);
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_set_metrics_empty_message() {
    let mut state = AlertPanelState::new().with_metrics(sample_metrics());
    AlertPanel::update(&mut state, AlertPanelMessage::SetMetrics(vec![]));
    assert!(state.metrics().is_empty());
    assert_eq!(state.selected(), None);
}

#[test]
fn test_set_columns_message() {
    let mut state = AlertPanelState::new().with_columns(2);
    AlertPanel::update(&mut state, AlertPanelMessage::SetColumns(3));
    assert_eq!(state.columns(), 3);
}

#[test]
fn test_set_columns_minimum_message() {
    let mut state = AlertPanelState::new();
    AlertPanel::update(&mut state, AlertPanelMessage::SetColumns(0));
    assert_eq!(state.columns(), 1);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_key_maps() {
    let state = focused_state();
    assert_eq!(
        AlertPanel::handle_event(
            &state,
            &Event::key(KeyCode::Left),
            &ViewContext::new().focused(true)
        ),
        Some(AlertPanelMessage::SelectPrev)
    );
    assert_eq!(
        AlertPanel::handle_event(
            &state,
            &Event::key(KeyCode::Right),
            &ViewContext::new().focused(true)
        ),
        Some(AlertPanelMessage::SelectNext)
    );
    assert_eq!(
        AlertPanel::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &ViewContext::new().focused(true)
        ),
        Some(AlertPanelMessage::SelectUp)
    );
    assert_eq!(
        AlertPanel::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &ViewContext::new().focused(true)
        ),
        Some(AlertPanelMessage::SelectDown)
    );
    assert_eq!(
        AlertPanel::handle_event(
            &state,
            &Event::key(KeyCode::Enter),
            &ViewContext::new().focused(true)
        ),
        Some(AlertPanelMessage::Select)
    );
}

#[test]
fn test_vim_key_maps() {
    let state = focused_state();
    assert_eq!(
        AlertPanel::handle_event(&state, &Event::char('h'), &ViewContext::new().focused(true)),
        Some(AlertPanelMessage::SelectPrev)
    );
    assert_eq!(
        AlertPanel::handle_event(&state, &Event::char('l'), &ViewContext::new().focused(true)),
        Some(AlertPanelMessage::SelectNext)
    );
    assert_eq!(
        AlertPanel::handle_event(&state, &Event::char('k'), &ViewContext::new().focused(true)),
        Some(AlertPanelMessage::SelectUp)
    );
    assert_eq!(
        AlertPanel::handle_event(&state, &Event::char('j'), &ViewContext::new().focused(true)),
        Some(AlertPanelMessage::SelectDown)
    );
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = AlertPanel::handle_event(
        &state,
        &Event::key(KeyCode::Right),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = AlertPanelState::new().with_metrics(sample_metrics());
    let msg =
        AlertPanel::handle_event(&state, &Event::key(KeyCode::Right), &ViewContext::default());
    assert_eq!(msg, None);
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::key(KeyCode::Right));
    assert_eq!(msg, Some(AlertPanelMessage::SelectNext));
}

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    let output = state.update(AlertPanelMessage::SelectNext);
    assert_eq!(output, Some(AlertPanelOutput::MetricSelected("mem".into())));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state();
    let output = state.dispatch_event(&Event::key(KeyCode::Right));
    assert_eq!(output, Some(AlertPanelOutput::MetricSelected("mem".into())));
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = AlertPanelState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_metrics() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = AlertPanelState::new()
        .with_metrics(sample_metrics())
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
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
}

#[test]
fn test_render_small_area() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 2);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_history() {
    let mut metrics = sample_metrics();
    for metric in &mut metrics {
        for i in 0..10 {
            metric.update_value(i as f64 * 5.0);
        }
    }
    let state = AlertPanelState::new().with_metrics(metrics).with_columns(2);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = AlertPanel::init();
    assert!(!AlertPanel::is_focused(&state));

    AlertPanel::focus(&mut state);
    assert!(AlertPanel::is_focused(&state));

    AlertPanel::blur(&mut state);
    assert!(!AlertPanel::is_focused(&state));
}

// =============================================================================
// Disableable trait
// =============================================================================

#[test]
fn test_disableable_trait() {
    let mut state = AlertPanel::init();
    assert!(!AlertPanel::is_disabled(&state));

    AlertPanel::disable(&mut state);
    assert!(AlertPanel::is_disabled(&state));

    AlertPanel::enable(&mut state);
    assert!(!AlertPanel::is_disabled(&state));
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_metrics_selected_none() {
    let state = AlertPanelState::new();
    assert_eq!(state.selected(), None);
    assert!(state.selected_metric().is_none());
}

#[test]
fn test_empty_metrics_ignores_navigation() {
    let mut state = AlertPanelState::new();
    state.set_focused(true);
    let output = AlertPanel::update(&mut state, AlertPanelMessage::SelectNext);
    assert_eq!(output, None);
}

#[test]
fn test_single_metric_navigation() {
    let mut state = AlertPanelState::new()
        .with_metrics(vec![AlertMetric::new(
            "cpu",
            "CPU",
            AlertThreshold::new(70.0, 90.0),
        )])
        .with_columns(1);
    state.set_focused(true);
    assert_eq!(
        AlertPanel::update(&mut state, AlertPanelMessage::SelectNext),
        None
    );
    assert_eq!(
        AlertPanel::update(&mut state, AlertPanelMessage::SelectDown),
        None
    );
}

#[test]
fn test_all_critical() {
    let state = AlertPanelState::new().with_metrics(vec![
        AlertMetric::new("a", "A", AlertThreshold::new(10.0, 20.0)).with_value(25.0),
        AlertMetric::new("b", "B", AlertThreshold::new(10.0, 20.0)).with_value(30.0),
        AlertMetric::new("c", "C", AlertThreshold::new(10.0, 20.0)).with_value(50.0),
    ]);
    assert_eq!(state.ok_count(), 0);
    assert_eq!(state.warning_count(), 0);
    assert_eq!(state.critical_count(), 3);
}

#[test]
fn test_title_with_counts_empty() {
    let state = AlertPanelState::new();
    assert_eq!(state.title_with_counts(), "Alerts");
}

#[test]
fn test_title_with_counts_mixed() {
    let state = AlertPanelState::new()
        .with_metrics(vec![
            AlertMetric::new("a", "A", AlertThreshold::new(10.0, 20.0)).with_value(5.0),
            AlertMetric::new("b", "B", AlertThreshold::new(10.0, 20.0)).with_value(15.0),
            AlertMetric::new("c", "C", AlertThreshold::new(10.0, 20.0)).with_value(25.0),
        ])
        .with_title("System");
    assert_eq!(state.title_with_counts(), "System (1 OK, 1 WARN, 1 CRIT)");
}

#[test]
fn test_rows_calculation() {
    let state = AlertPanelState::new()
        .with_metrics(sample_metrics())
        .with_columns(2);
    assert_eq!(state.rows(), 2); // 4 metrics / 2 columns = 2 rows
}

#[test]
fn test_rows_empty() {
    let state = AlertPanelState::new();
    assert_eq!(state.rows(), 0);
}

#[test]
fn test_rows_partial() {
    let state = AlertPanelState::new()
        .with_metrics(vec![
            AlertMetric::new("a", "A", AlertThreshold::new(10.0, 20.0)),
            AlertMetric::new("b", "B", AlertThreshold::new(10.0, 20.0)),
            AlertMetric::new("c", "C", AlertThreshold::new(10.0, 20.0)),
        ])
        .with_columns(2);
    assert_eq!(state.rows(), 2); // 3 metrics / 2 columns = 2 rows (ceil)
}

// =============================================================================
// AlertState display
// =============================================================================

#[test]
fn test_alert_state_display() {
    assert_eq!(AlertState::Ok.to_string(), "OK");
    assert_eq!(AlertState::Warning.to_string(), "WARN");
    assert_eq!(AlertState::Critical.to_string(), "CRIT");
    assert_eq!(AlertState::Unknown.to_string(), "UNKNOWN");
}

#[test]
fn test_alert_state_default() {
    let state = AlertState::default();
    assert_eq!(state, AlertState::Ok);
}

// =============================================================================
// Annotation
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = AlertPanelState::new().with_metrics(sample_metrics());
    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                AlertPanel::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert!(registry.get_by_id("alert_panel").is_some());
}
