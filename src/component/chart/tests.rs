use super::*;
use crate::component::test_utils;

fn sample_series() -> Vec<DataSeries> {
    vec![
        DataSeries::new("Series A", vec![10.0, 20.0, 30.0, 25.0, 15.0]),
        DataSeries::new("Series B", vec![5.0, 15.0, 10.0, 20.0, 25.0]).with_color(Color::Red),
    ]
}

fn focused_line_chart() -> ChartState {
    ChartState::line(sample_series())
}

// =============================================================================
// DataSeries
// =============================================================================

#[test]
fn test_data_series_new() {
    let s = DataSeries::new("CPU", vec![1.0, 2.0, 3.0]);
    assert_eq!(s.label(), "CPU");
    assert_eq!(s.values(), &[1.0, 2.0, 3.0]);
    // Default color is the first Tableau 20 palette color (blue)
    assert_eq!(s.color(), Color::Rgb(31, 119, 180));
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
    assert_eq!(state.max_display_points(), 500);
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
fn test_disabled_ignores_events() {
    let state = focused_line_chart();
    let msg = Chart::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = ChartState::line(sample_series());
    let msg = Chart::handle_event(&state, &Event::key(KeyCode::Tab), &ViewContext::default());
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_tab_maps_to_next() {
    let state = focused_line_chart();
    assert_eq!(
        Chart::handle_event(
            &state,
            &Event::key(KeyCode::Tab),
            &ViewContext::new().focused(true)
        ),
        Some(ChartMessage::NextSeries)
    );
}

#[test]
fn test_backtab_maps_to_prev() {
    let state = focused_line_chart();
    assert_eq!(
        Chart::handle_event(
            &state,
            &Event::key(KeyCode::BackTab),
            &ViewContext::new().focused(true)
        ),
        Some(ChartMessage::PrevSeries)
    );
}

// =============================================================================
// Braille line chart rendering
// =============================================================================

#[test]
fn test_render_line_chart_with_thresholds() {
    let state = ChartState::line(vec![DataSeries::new(
        "CPU",
        vec![45.0, 52.0, 80.0, 92.0, 72.0],
    )])
    .with_threshold(90.0, "Warning", Color::Yellow);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_line_chart_multi_series_overlay() {
    let state = ChartState::line(vec![
        DataSeries::new("Series A", vec![10.0, 20.0, 30.0, 25.0, 15.0]),
        DataSeries::new("Series B", vec![5.0, 15.0, 10.0, 20.0, 25.0]).with_color(Color::Red),
        DataSeries::new("Series C", vec![15.0, 10.0, 20.0, 30.0, 20.0]).with_color(Color::Green),
    ]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_update() {
    let mut state = focused_line_chart();
    let output = state.update(ChartMessage::NextSeries);
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
        .with_y_label("\u{b0}C");
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
    let state = ChartState::line(sample_series());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(
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
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, None);
}

#[test]
fn test_single_series_cycling() {
    let mut state = ChartState::line(vec![DataSeries::new("Solo", vec![1.0])]);
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

// =============================================================================
// Palette
// =============================================================================

#[test]
fn test_default_palette_has_20_colors() {
    assert_eq!(DEFAULT_PALETTE.len(), 20);
}

#[test]
fn test_default_palette_colors_are_all_distinct() {
    for i in 0..DEFAULT_PALETTE.len() {
        for j in (i + 1)..DEFAULT_PALETTE.len() {
            assert_ne!(
                DEFAULT_PALETTE[i], DEFAULT_PALETTE[j],
                "Palette colors at index {} and {} should be distinct",
                i, j
            );
        }
    }
}

#[test]
fn test_chart_palette_color_returns_correct_values() {
    assert_eq!(chart_palette_color(0), Color::Rgb(31, 119, 180)); // blue
    assert_eq!(chart_palette_color(1), Color::Rgb(255, 127, 14)); // orange
    assert_eq!(chart_palette_color(9), Color::Rgb(23, 190, 207)); // teal
    assert_eq!(chart_palette_color(10), Color::Rgb(174, 199, 232)); // light blue
    assert_eq!(chart_palette_color(19), Color::Rgb(158, 218, 229)); // light teal
}

#[test]
fn test_chart_palette_color_wraps_around() {
    assert_eq!(chart_palette_color(20), chart_palette_color(0));
    assert_eq!(chart_palette_color(21), chart_palette_color(1));
    assert_eq!(chart_palette_color(40), chart_palette_color(0));
}

#[test]
fn test_default_series_color_matches_palette() {
    let series = DataSeries::new("Test", vec![]);
    assert_eq!(series.color(), chart_palette_color(0));
}

// =============================================================================
// Category labels
// =============================================================================

#[test]
fn test_with_categories_builder() {
    let state = ChartState::bar_vertical(vec![DataSeries::new("Sales", vec![10.0, 20.0, 30.0])])
        .with_categories(vec!["Q1", "Q2", "Q3"]);
    assert_eq!(state.categories(), &["Q1", "Q2", "Q3"]);
}

#[test]
fn test_categories_accessor_empty_by_default() {
    let state = ChartState::bar_vertical(vec![]);
    assert!(state.categories().is_empty());
}

#[test]
fn test_set_categories() {
    let mut state = ChartState::bar_vertical(vec![DataSeries::new("Sales", vec![10.0, 20.0])]);
    assert!(state.categories().is_empty());
    state.set_categories(vec!["A", "B"]);
    assert_eq!(state.categories(), &["A", "B"]);
}

#[test]
fn test_categories_with_string_type() {
    let state = ChartState::bar_vertical(vec![])
        .with_categories(vec![String::from("Alpha"), String::from("Beta")]);
    assert_eq!(state.categories(), &["Alpha", "Beta"]);
}

#[test]
fn test_render_bar_chart_with_categories() {
    let state = ChartState::bar_vertical(vec![DataSeries::new(
        "Importance",
        vec![85.0, 72.0, 64.0, 51.0],
    )])
    .with_categories(vec!["Income", "Education", "Age", "Hours"])
    .with_title("Feature Importance");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    // Verify no panic and rendering succeeds
}

#[test]
fn test_render_bar_chart_falls_back_to_numeric_without_categories() {
    let state = ChartState::bar_vertical(vec![DataSeries::new("Values", vec![10.0, 20.0, 30.0])]);
    assert!(state.categories().is_empty());
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    // Verify no panic and rendering succeeds with numeric fallback
}

#[test]
fn test_categories_fewer_than_data_points() {
    // 2 categories but 4 data points: bars 3 and 4 should use numeric labels
    let state = ChartState::bar_vertical(vec![DataSeries::new(
        "Values",
        vec![10.0, 20.0, 30.0, 40.0],
    )])
    .with_categories(vec!["A", "B"]);
    assert_eq!(state.categories().len(), 2);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_categories_more_than_data_points() {
    // 5 categories but only 3 data points: extra categories are simply ignored
    let state = ChartState::bar_vertical(vec![DataSeries::new("Values", vec![10.0, 20.0, 30.0])])
        .with_categories(vec!["A", "B", "C", "D", "E"]);
    assert_eq!(state.categories().len(), 5);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_horizontal_bar_chart_with_categories() {
    let state =
        ChartState::bar_horizontal(vec![DataSeries::new("Revenue", vec![100.0, 200.0, 150.0])])
            .with_categories(vec!["East", "West", "Central"]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_default_state_has_empty_categories() {
    let state = ChartState::default();
    assert!(state.categories().is_empty());
}
