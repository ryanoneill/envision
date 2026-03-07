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

fn focused_state() -> MetricsDashboardState {
    let mut state = MetricsDashboardState::new(sample_widgets(), 3);
    MetricsDashboard::set_focused(&mut state, true);
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = MetricsDashboardState::new(sample_widgets(), 3);
    assert_eq!(state.widget_count(), 6);
    assert_eq!(state.columns(), 3);
    assert_eq!(state.selected_index(), Some(0));
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
}

#[test]
fn test_default() {
    let state = MetricsDashboardState::default();
    assert!(state.is_empty());
    assert_eq!(state.columns(), 3);
}

#[test]
fn test_columns_minimum() {
    let state = MetricsDashboardState::new(sample_widgets(), 0);
    assert_eq!(state.columns(), 1);
}

#[test]
fn test_with_title() {
    let state = MetricsDashboardState::new(sample_widgets(), 3).with_title("Dashboard");
    assert_eq!(state.title(), Some("Dashboard"));
}

#[test]
fn test_with_disabled() {
    let state = MetricsDashboardState::new(sample_widgets(), 3).with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// MetricWidget constructors
// =============================================================================

#[test]
fn test_counter_widget() {
    let w = MetricWidget::counter("Count", 42);
    assert_eq!(w.label(), "Count");
    assert_eq!(w.display_value(), "42");
    assert_eq!(w.kind(), &MetricKind::Counter { value: 42 });
}

#[test]
fn test_gauge_widget() {
    let w = MetricWidget::gauge("CPU", 75, 100);
    assert_eq!(w.display_value(), "75/100");
    assert_eq!(w.gauge_percentage(), Some(0.75));
}

#[test]
fn test_gauge_zero_max() {
    let w = MetricWidget::gauge("Empty", 0, 0);
    assert_eq!(w.gauge_percentage(), None);
}

#[test]
fn test_status_widget_up() {
    let w = MetricWidget::status("API", true);
    assert_eq!(w.display_value(), "UP");
}

#[test]
fn test_status_widget_down() {
    let w = MetricWidget::status("API", false);
    assert_eq!(w.display_value(), "DOWN");
}

#[test]
fn test_text_widget() {
    let w = MetricWidget::text("Version", "1.0.0");
    assert_eq!(w.display_value(), "1.0.0");
}

#[test]
fn test_with_max_history() {
    let w = MetricWidget::counter("Test", 0).with_max_history(50);
    assert_eq!(w.history().len(), 0);
}

// =============================================================================
// Widget value setters
// =============================================================================

#[test]
fn test_set_counter_value() {
    let mut w = MetricWidget::counter("Count", 0);
    w.set_counter_value(100);
    assert_eq!(w.display_value(), "100");
}

#[test]
fn test_set_counter_records_history() {
    let mut w = MetricWidget::counter("Count", 0);
    w.set_counter_value(10);
    w.set_counter_value(20);
    w.set_counter_value(30);
    assert_eq!(w.history().len(), 3);
    assert_eq!(w.history(), &[10, 20, 30]);
}

#[test]
fn test_set_counter_caps_history() {
    let mut w = MetricWidget::counter("Count", 0).with_max_history(3);
    for i in 0..5 {
        w.set_counter_value(i);
    }
    assert_eq!(w.history().len(), 3);
}

#[test]
fn test_set_gauge_value() {
    let mut w = MetricWidget::gauge("CPU", 0, 100);
    w.set_gauge_value(75);
    assert_eq!(w.display_value(), "75/100");
}

#[test]
fn test_set_gauge_value_clamped() {
    let mut w = MetricWidget::gauge("CPU", 0, 100);
    w.set_gauge_value(200);
    assert_eq!(w.display_value(), "100/100");
}

#[test]
fn test_set_gauge_records_history() {
    let mut w = MetricWidget::gauge("CPU", 0, 100);
    w.set_gauge_value(25);
    w.set_gauge_value(50);
    assert_eq!(w.history().len(), 2);
}

#[test]
fn test_set_status() {
    let mut w = MetricWidget::status("API", true);
    w.set_status(false);
    assert_eq!(w.display_value(), "DOWN");
}

#[test]
fn test_set_text() {
    let mut w = MetricWidget::text("Version", "1.0");
    w.set_text("2.0");
    assert_eq!(w.display_value(), "2.0");
}

#[test]
fn test_increment() {
    let mut w = MetricWidget::counter("Count", 10);
    w.increment(5);
    assert_eq!(w.display_value(), "15");
}

#[test]
fn test_increment_negative() {
    let mut w = MetricWidget::counter("Count", 10);
    w.increment(-3);
    assert_eq!(w.display_value(), "7");
}

#[test]
fn test_increment_records_history() {
    let mut w = MetricWidget::counter("Count", 0);
    w.increment(10);
    w.increment(5);
    assert_eq!(w.history().len(), 2);
}

#[test]
fn test_gauge_percentage() {
    let w = MetricWidget::gauge("CPU", 50, 100);
    assert_eq!(w.gauge_percentage(), Some(0.5));
}

// =============================================================================
// Grid layout
// =============================================================================

#[test]
fn test_rows_calculation() {
    let state = MetricsDashboardState::new(sample_widgets(), 3);
    assert_eq!(state.rows(), 2); // 6 widgets / 3 columns = 2 rows
}

#[test]
fn test_rows_partial() {
    let widgets = vec![
        MetricWidget::counter("A", 0),
        MetricWidget::counter("B", 0),
        MetricWidget::counter("C", 0),
        MetricWidget::counter("D", 0),
    ];
    let state = MetricsDashboardState::new(widgets, 3);
    assert_eq!(state.rows(), 2); // 4 widgets / 3 columns = 2 rows (ceil)
}

#[test]
fn test_rows_empty() {
    let state = MetricsDashboardState::default();
    assert_eq!(state.rows(), 0);
}

#[test]
fn test_selected_position() {
    let mut state = focused_state();
    assert_eq!(state.selected_position(), Some((0, 0)));

    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right);
    assert_eq!(state.selected_position(), Some((0, 1)));

    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Down);
    assert_eq!(state.selected_position(), Some((1, 1)));
}

// =============================================================================
// Navigation
// =============================================================================

#[test]
fn test_right() {
    let mut state = focused_state();
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(MetricsDashboardOutput::SelectionChanged(1)));
}

#[test]
fn test_left() {
    let mut state = focused_state();
    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right);
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Left);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(MetricsDashboardOutput::SelectionChanged(0)));
}

#[test]
fn test_right_at_row_end() {
    let mut state = focused_state();
    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right);
    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right);
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(output, None);
}

#[test]
fn test_left_at_row_start() {
    let mut state = focused_state();
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Left);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, None);
}

#[test]
fn test_down() {
    let mut state = focused_state();
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Down);
    assert_eq!(state.selected_index(), Some(3));
    assert_eq!(output, Some(MetricsDashboardOutput::SelectionChanged(3)));
}

#[test]
fn test_up() {
    let mut state = focused_state();
    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Down);
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Up);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(MetricsDashboardOutput::SelectionChanged(0)));
}

#[test]
fn test_up_at_top() {
    let mut state = focused_state();
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Up);
    assert_eq!(output, None);
}

#[test]
fn test_down_at_bottom() {
    let mut state = focused_state();
    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Down);
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Down);
    assert_eq!(output, None);
}

#[test]
fn test_first() {
    let mut state = focused_state();
    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Last);
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(MetricsDashboardOutput::SelectionChanged(0)));
}

#[test]
fn test_last() {
    let mut state = focused_state();
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Last);
    assert_eq!(state.selected_index(), Some(5));
    assert_eq!(output, Some(MetricsDashboardOutput::SelectionChanged(5)));
}

#[test]
fn test_first_at_first() {
    let mut state = focused_state();
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::First);
    assert_eq!(output, None);
}

#[test]
fn test_last_at_last() {
    let mut state = focused_state();
    MetricsDashboard::update(&mut state, MetricsDashboardMessage::Last);
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Last);
    assert_eq!(output, None);
}

#[test]
fn test_select() {
    let mut state = focused_state();
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Select);
    assert_eq!(output, Some(MetricsDashboardOutput::Selected(0)));
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_messages() {
    let mut state = focused_state();
    state.set_disabled(true);
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = MetricsDashboard::handle_event(&state, &Event::key(KeyCode::Right));
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = MetricsDashboardState::new(sample_widgets(), 3);
    let msg = MetricsDashboard::handle_event(&state, &Event::key(KeyCode::Right));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_key_maps() {
    let state = focused_state();
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::key(KeyCode::Left)),
        Some(MetricsDashboardMessage::Left)
    );
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::key(KeyCode::Right)),
        Some(MetricsDashboardMessage::Right)
    );
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(MetricsDashboardMessage::Up)
    );
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(MetricsDashboardMessage::Down)
    );
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(MetricsDashboardMessage::First)
    );
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::key(KeyCode::End)),
        Some(MetricsDashboardMessage::Last)
    );
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::key(KeyCode::Enter)),
        Some(MetricsDashboardMessage::Select)
    );
}

#[test]
fn test_vim_key_maps() {
    let state = focused_state();
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::char('h')),
        Some(MetricsDashboardMessage::Left)
    );
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::char('l')),
        Some(MetricsDashboardMessage::Right)
    );
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::char('k')),
        Some(MetricsDashboardMessage::Up)
    );
    assert_eq!(
        MetricsDashboard::handle_event(&state, &Event::char('j')),
        Some(MetricsDashboardMessage::Down)
    );
}

// =============================================================================
// Widget accessors
// =============================================================================

#[test]
fn test_widget_accessor() {
    let state = focused_state();
    assert_eq!(state.widget(0).unwrap().label(), "Requests");
    assert_eq!(state.widget(99), None);
}

#[test]
fn test_widget_mut_accessor() {
    let mut state = focused_state();
    state.widget_mut(0).unwrap().set_counter_value(100);
    assert_eq!(state.widget(0).unwrap().display_value(), "100");
}

#[test]
fn test_selected_widget() {
    let state = focused_state();
    assert_eq!(state.selected_widget().unwrap().label(), "Requests");
}

#[test]
fn test_widgets_mut() {
    let mut state = focused_state();
    state.widgets_mut()[1].set_gauge_value(90);
    assert_eq!(state.widget(1).unwrap().display_value(), "90/100");
}

#[test]
fn test_set_columns() {
    let mut state = focused_state();
    state.set_columns(2);
    assert_eq!(state.columns(), 2);
    assert_eq!(state.rows(), 3); // 6 / 2 = 3
}

#[test]
fn test_set_columns_minimum() {
    let mut state = focused_state();
    state.set_columns(0);
    assert_eq!(state.columns(), 1);
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::key(KeyCode::Right));
    assert_eq!(msg, Some(MetricsDashboardMessage::Right));
}

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    let output = state.update(MetricsDashboardMessage::Right);
    assert_eq!(output, Some(MetricsDashboardOutput::SelectionChanged(1)));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state();
    let output = state.dispatch_event(&Event::key(KeyCode::Right));
    assert_eq!(output, Some(MetricsDashboardOutput::SelectionChanged(1)));
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = MetricsDashboardState::default();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_widgets() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = MetricsDashboardState::new(sample_widgets(), 3).with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_history() {
    let mut widgets = sample_widgets();
    for i in 0..10 {
        widgets[0].set_counter_value(i * 10);
    }
    let state = MetricsDashboardState::new(widgets, 3);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 2);
    terminal
        .draw(|frame| {
            MetricsDashboard::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = MetricsDashboard::init();
    assert!(!MetricsDashboard::is_focused(&state));

    MetricsDashboard::focus(&mut state);
    assert!(MetricsDashboard::is_focused(&state));

    MetricsDashboard::blur(&mut state);
    assert!(!MetricsDashboard::is_focused(&state));
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = MetricsDashboardState::new(sample_widgets(), 3);
    let state2 = MetricsDashboardState::new(sample_widgets(), 3);
    assert_eq!(state1, state2);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_dashboard_selected_index_is_none() {
    let state = MetricsDashboardState::default();
    assert_eq!(state.selected_index(), None);
    assert!(state.selected_widget().is_none());
    assert!(state.selected_position().is_none());
}

#[test]
fn test_empty_dashboard_ignores_navigation() {
    let mut state = MetricsDashboardState::default();
    state.set_focused(true);
    let output = MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right);
    assert_eq!(output, None);
}

#[test]
fn test_single_widget_navigation() {
    let mut state = MetricsDashboardState::new(vec![MetricWidget::counter("A", 0)], 1);
    state.set_focused(true);
    assert_eq!(
        MetricsDashboard::update(&mut state, MetricsDashboardMessage::Right),
        None
    );
    assert_eq!(
        MetricsDashboard::update(&mut state, MetricsDashboardMessage::Down),
        None
    );
}

#[test]
fn test_set_title() {
    let mut state = focused_state();
    state.set_title(Some("Test".into()));
    assert_eq!(state.title(), Some("Test"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = MetricsDashboardState::new(sample_widgets(), 3);
    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                MetricsDashboard::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    assert!(registry.get_by_id("metrics_dashboard").is_some());
}
