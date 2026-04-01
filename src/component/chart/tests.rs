use super::*;
use crate::component::test_utils;

fn sample_series() -> Vec<DataSeries> {
    vec![
        DataSeries::new("Series A", vec![10.0, 20.0, 30.0, 25.0, 15.0]),
        DataSeries::new("Series B", vec![5.0, 15.0, 10.0, 20.0, 25.0]).with_color(Color::Red),
    ]
}

fn focused_line_chart() -> ChartState {
    let mut state = ChartState::line(sample_series());
    state.set_focused(true);
    state
}

// =============================================================================
// DataSeries
// =============================================================================

#[test]
fn test_data_series_new() {
    let s = DataSeries::new("CPU", vec![1.0, 2.0, 3.0]);
    assert_eq!(s.label(), "CPU");
    assert_eq!(s.values(), &[1.0, 2.0, 3.0]);
    assert_eq!(s.color(), Color::Cyan);
}

#[test]
fn test_data_series_with_color() {
    let s = DataSeries::new("Test", vec![]).with_color(Color::Red);
    assert_eq!(s.color(), Color::Red);
}

#[test]
fn test_data_series_push() {
    let mut s = DataSeries::new("Test", vec![1.0]);
    s.push(2.0);
    assert_eq!(s.values(), &[1.0, 2.0]);
}

#[test]
fn test_data_series_push_bounded() {
    let mut s = DataSeries::new("Test", vec![1.0, 2.0, 3.0]);
    s.push_bounded(4.0, 3);
    assert_eq!(s.values(), &[2.0, 3.0, 4.0]);
}

#[test]
fn test_data_series_min_max() {
    let s = DataSeries::new("Test", vec![5.0, 1.0, 10.0, 3.0]);
    assert_eq!(s.min(), 1.0);
    assert_eq!(s.max(), 10.0);
}

#[test]
fn test_data_series_min_max_empty() {
    let s = DataSeries::new("Empty", vec![]);
    assert_eq!(s.min(), 0.0);
    assert_eq!(s.max(), 0.0);
}

#[test]
fn test_data_series_last() {
    let s = DataSeries::new("Test", vec![1.0, 2.0, 3.0]);
    assert_eq!(s.last(), Some(3.0));
}

#[test]
fn test_data_series_last_empty() {
    let s = DataSeries::new("Empty", vec![]);
    assert_eq!(s.last(), None);
}

#[test]
fn test_data_series_len() {
    let s = DataSeries::new("Test", vec![1.0, 2.0]);
    assert_eq!(s.len(), 2);
    assert!(!s.is_empty());
}

#[test]
fn test_data_series_is_empty() {
    let s = DataSeries::new("Empty", vec![]);
    assert!(s.is_empty());
}

#[test]
fn test_data_series_clear() {
    let mut s = DataSeries::new("Test", vec![1.0, 2.0]);
    s.clear();
    assert!(s.is_empty());
}

#[test]
fn test_data_series_set_label() {
    let mut s = DataSeries::new("Old", vec![]);
    s.set_label("New");
    assert_eq!(s.label(), "New");
}

#[test]
fn test_data_series_set_color() {
    let mut s = DataSeries::new("Test", vec![]);
    s.set_color(Color::Green);
    assert_eq!(s.color(), Color::Green);
}

// =============================================================================
// ChartState construction
// =============================================================================

#[test]
fn test_line_chart() {
    let state = ChartState::line(sample_series());
    assert_eq!(state.kind(), &ChartKind::Line);
    assert_eq!(state.series_count(), 2);
    assert_eq!(state.active_series(), 0);
}

#[test]
fn test_bar_vertical() {
    let state = ChartState::bar_vertical(sample_series());
    assert_eq!(state.kind(), &ChartKind::BarVertical);
}

#[test]
fn test_bar_horizontal() {
    let state = ChartState::bar_horizontal(sample_series());
    assert_eq!(state.kind(), &ChartKind::BarHorizontal);
}

#[test]
fn test_area_chart() {
    let state = ChartState::area(sample_series());
    assert_eq!(state.kind(), &ChartKind::Area);
    assert_eq!(state.series_count(), 2);
}

#[test]
fn test_scatter_chart() {
    let state = ChartState::scatter(sample_series());
    assert_eq!(state.kind(), &ChartKind::Scatter);
    assert_eq!(state.series_count(), 2);
}

#[test]
fn test_default() {
    let state = ChartState::default();
    assert!(state.is_empty());
    assert_eq!(state.kind(), &ChartKind::Line);
    assert_eq!(state.max_display_points(), 50);
    assert_eq!(state.bar_width(), 3);
    assert_eq!(state.bar_gap(), 1);
    assert!(state.show_legend());
    assert!(state.thresholds().is_empty());
    assert_eq!(state.y_min(), None);
    assert_eq!(state.y_max(), None);
}

#[test]
fn test_with_title() {
    let state = ChartState::line(vec![]).with_title("Chart");
    assert_eq!(state.title(), Some("Chart"));
}

#[test]
fn test_with_x_label() {
    let state = ChartState::line(vec![]).with_x_label("Time");
    assert_eq!(state.x_label(), Some("Time"));
}

#[test]
fn test_with_y_label() {
    let state = ChartState::line(vec![]).with_y_label("Value");
    assert_eq!(state.y_label(), Some("Value"));
}

#[test]
fn test_with_legend() {
    let state = ChartState::line(vec![]).with_legend(false);
    assert!(!state.show_legend());
}

#[test]
fn test_with_max_display_points() {
    let state = ChartState::line(vec![]).with_max_display_points(100);
    assert_eq!(state.max_display_points(), 100);
}

#[test]
fn test_with_bar_width() {
    let state = ChartState::bar_vertical(vec![]).with_bar_width(5);
    assert_eq!(state.bar_width(), 5);
}

#[test]
fn test_with_bar_width_minimum() {
    let state = ChartState::bar_vertical(vec![]).with_bar_width(0);
    assert_eq!(state.bar_width(), 1);
}

#[test]
fn test_with_bar_gap() {
    let state = ChartState::bar_vertical(vec![]).with_bar_gap(2);
    assert_eq!(state.bar_gap(), 2);
}

#[test]
fn test_with_disabled() {
    let state = ChartState::line(vec![]).with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// State manipulation
// =============================================================================

#[test]
fn test_add_series() {
    let mut state = ChartState::line(vec![]);
    state.add_series(DataSeries::new("New", vec![1.0]));
    assert_eq!(state.series_count(), 1);
}

#[test]
fn test_clear_series() {
    let mut state = ChartState::line(sample_series());
    state.clear_series();
    assert!(state.is_empty());
    assert_eq!(state.active_series(), 0);
}

#[test]
fn test_get_series() {
    let state = ChartState::line(sample_series());
    assert_eq!(state.get_series(0).unwrap().label(), "Series A");
    assert_eq!(state.get_series(99), None);
}

#[test]
fn test_get_series_mut() {
    let mut state = ChartState::line(sample_series());
    state.get_series_mut(0).unwrap().push(40.0);
    assert_eq!(state.get_series(0).unwrap().len(), 6);
}

#[test]
fn test_series_mut() {
    let mut state = ChartState::line(sample_series());
    state.series_mut()[0].set_label("Modified");
    assert_eq!(state.series()[0].label(), "Modified");
}

#[test]
fn test_set_kind() {
    let mut state = ChartState::line(vec![]);
    state.set_kind(ChartKind::BarVertical);
    assert_eq!(state.kind(), &ChartKind::BarVertical);
}

#[test]
fn test_set_title() {
    let mut state = ChartState::line(vec![]);
    state.set_title(Some("Test".into()));
    assert_eq!(state.title(), Some("Test"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_global_min_max() {
    let state = ChartState::line(sample_series());
    assert_eq!(state.global_min(), 5.0);
    assert_eq!(state.global_max(), 30.0);
}

#[test]
fn test_global_min_max_empty() {
    let state = ChartState::line(vec![]);
    assert_eq!(state.global_min(), 0.0);
    assert_eq!(state.global_max(), 0.0);
}

// =============================================================================
// Series cycling
// =============================================================================

#[test]
fn test_next_series() {
    let mut state = focused_line_chart();
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(state.active_series(), 1);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(1)));
}

#[test]
fn test_next_series_wraps() {
    let mut state = focused_line_chart();
    Chart::update(&mut state, ChartMessage::NextSeries);
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(state.active_series(), 0);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(0)));
}

#[test]
fn test_prev_series() {
    let mut state = focused_line_chart();
    Chart::update(&mut state, ChartMessage::NextSeries);
    let output = Chart::update(&mut state, ChartMessage::PrevSeries);
    assert_eq!(state.active_series(), 0);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(0)));
}

#[test]
fn test_prev_series_wraps() {
    let mut state = focused_line_chart();
    let output = Chart::update(&mut state, ChartMessage::PrevSeries);
    assert_eq!(state.active_series(), 1);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(1)));
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_messages() {
    let mut state = focused_line_chart();
    state.set_disabled(true);
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_line_chart();
    state.set_disabled(true);
    let msg = Chart::handle_event(&state, &Event::key(KeyCode::Tab));
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = ChartState::line(sample_series());
    let msg = Chart::handle_event(&state, &Event::key(KeyCode::Tab));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_tab_maps_to_next() {
    let state = focused_line_chart();
    assert_eq!(
        Chart::handle_event(&state, &Event::key(KeyCode::Tab)),
        Some(ChartMessage::NextSeries)
    );
}

#[test]
fn test_backtab_maps_to_prev() {
    let state = focused_line_chart();
    assert_eq!(
        Chart::handle_event(&state, &Event::key(KeyCode::BackTab)),
        Some(ChartMessage::PrevSeries)
    );
}

// =============================================================================
// Sparkline data conversion (delegated to render module, tested there too)
// =============================================================================

#[test]
fn test_series_to_sparkline_data() {
    let s = DataSeries::new("Test", vec![0.0, 50.0, 100.0]);
    let data = render::series_to_sparkline_data(&s, 50);
    assert_eq!(data, vec![0, 50, 100]);
}

#[test]
fn test_series_to_sparkline_data_constant() {
    let s = DataSeries::new("Test", vec![5.0, 5.0, 5.0]);
    let data = render::series_to_sparkline_data(&s, 50);
    assert_eq!(data, vec![50, 50, 50]);
}

#[test]
fn test_series_to_sparkline_data_bounded() {
    let s = DataSeries::new("Test", vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let data = render::series_to_sparkline_data(&s, 3);
    assert_eq!(data.len(), 3); // Only last 3 points
}

#[test]
fn test_series_to_sparkline_data_empty() {
    let s = DataSeries::new("Test", vec![]);
    let data = render::series_to_sparkline_data(&s, 50);
    assert!(data.is_empty());
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_line_chart();
    let msg = state.handle_event(&Event::key(KeyCode::Tab));
    assert_eq!(msg, Some(ChartMessage::NextSeries));
}

#[test]
fn test_instance_update() {
    let mut state = focused_line_chart();
    let output = state.update(ChartMessage::NextSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(1)));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_line_chart();
    let output = state.dispatch_event(&Event::key(KeyCode::Tab));
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(1)));
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = ChartState::default();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_line_chart() {
    let state = focused_line_chart();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_line_chart_with_labels() {
    let state = ChartState::line(sample_series())
        .with_title("Temperature")
        .with_x_label("Time")
        .with_y_label("°C");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_bar_vertical() {
    let state = ChartState::bar_vertical(vec![DataSeries::new(
        "Values",
        vec![10.0, 20.0, 30.0, 15.0],
    )]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_bar_horizontal() {
    let state = ChartState::bar_horizontal(vec![DataSeries::new("Values", vec![10.0, 20.0, 30.0])]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = ChartState::line(sample_series()).with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = focused_line_chart();
    let (mut terminal, theme) = test_utils::setup_render(60, 2);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_single_series_no_legend() {
    let state = ChartState::line(vec![DataSeries::new("Solo", vec![1.0, 2.0, 3.0])]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = ChartState::line(sample_series());
    let state2 = ChartState::line(sample_series());
    assert_eq!(state1, state2);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_chart_ignores_messages() {
    let mut state = ChartState::line(vec![]);
    state.set_focused(true);
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, None);
}

#[test]
fn test_single_series_cycling() {
    let mut state = ChartState::line(vec![DataSeries::new("Solo", vec![1.0])]);
    state.set_focused(true);
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(state.active_series(), 0);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(0)));
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = ChartState::line(sample_series());
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert!(registry.get_by_id("chart").is_some());
}
