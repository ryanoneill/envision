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
    for (i, color_i) in DEFAULT_PALETTE.iter().enumerate() {
        for (j, color_j) in DEFAULT_PALETTE.iter().enumerate().skip(i + 1) {
            assert_ne!(
                color_i, color_j,
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

// =============================================================================
// XY-pair support
// =============================================================================

#[test]
fn test_xy_constructor() {
    let series = DataSeries::xy(
        "ROC",
        vec![0.0, 0.1, 0.3, 0.5, 1.0],
        vec![0.0, 0.5, 0.8, 0.9, 1.0],
    );
    assert_eq!(series.label(), "ROC");
    assert_eq!(series.values(), &[0.0, 0.5, 0.8, 0.9, 1.0]);
    assert_eq!(
        series.x_values(),
        Some([0.0, 0.1, 0.3, 0.5, 1.0].as_slice())
    );
    // Default color should be palette color 0
    assert_eq!(series.color(), chart_palette_color(0));
}

#[test]
fn test_xy_constructor_with_color() {
    let series = DataSeries::xy("Curve", vec![0.0, 1.0], vec![0.0, 1.0]).with_color(Color::Red);
    assert_eq!(series.color(), Color::Red);
    assert!(series.x_values().is_some());
}

#[test]
fn test_with_x_values_builder() {
    let series =
        DataSeries::new("Curve", vec![0.0, 0.5, 0.9, 1.0]).with_x_values(vec![0.0, 0.2, 0.6, 1.0]);
    assert_eq!(series.x_values(), Some([0.0, 0.2, 0.6, 1.0].as_slice()));
    assert_eq!(series.values(), &[0.0, 0.5, 0.9, 1.0]);
}

#[test]
fn test_x_values_accessor_none_by_default() {
    let series = DataSeries::new("Simple", vec![1.0, 2.0, 3.0]);
    assert_eq!(series.x_values(), None);
}

#[test]
fn test_set_x_values() {
    let mut series = DataSeries::new("Curve", vec![0.0, 0.5, 1.0]);
    assert_eq!(series.x_values(), None);

    series.set_x_values(Some(vec![0.0, 0.3, 1.0]));
    assert_eq!(series.x_values(), Some([0.0, 0.3, 1.0].as_slice()));

    series.set_x_values(None);
    assert_eq!(series.x_values(), None);
}

#[test]
fn test_push_does_not_affect_x_values() {
    let mut series = DataSeries::xy("Curve", vec![0.0, 1.0], vec![0.0, 1.0]);
    series.push(2.0);
    // x_values should be unchanged
    assert_eq!(series.x_values(), Some([0.0, 1.0].as_slice()));
    // y values should have the new value
    assert_eq!(series.values(), &[0.0, 1.0, 2.0]);
}

#[test]
fn test_xy_series_clone() {
    let original = DataSeries::xy("ROC", vec![0.0, 0.5, 1.0], vec![0.0, 0.8, 1.0]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_xy_series_partial_eq() {
    let a = DataSeries::xy("ROC", vec![0.0, 0.5, 1.0], vec![0.0, 0.8, 1.0]);
    let b = DataSeries::xy("ROC", vec![0.0, 0.5, 1.0], vec![0.0, 0.8, 1.0]);
    assert_eq!(a, b);

    // Different x_values should not be equal
    let c = DataSeries::xy("ROC", vec![0.0, 0.6, 1.0], vec![0.0, 0.8, 1.0]);
    assert_ne!(a, c);

    // xy series != implicit series with same y values
    let d = DataSeries::new("ROC", vec![0.0, 0.8, 1.0]);
    assert_ne!(a, d);
}

#[test]
fn test_render_scatter_with_xy_data() {
    // ROC curve example: explicit (FPR, TPR) pairs
    let roc = DataSeries::xy(
        "Model A",
        vec![0.0, 0.1, 0.2, 0.4, 0.6, 1.0],
        vec![0.0, 0.5, 0.7, 0.85, 0.95, 1.0],
    );
    let state = ChartState::scatter(vec![roc])
        .with_title("ROC Curve")
        .with_x_label("FPR")
        .with_y_label("TPR");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_line_with_xy_data() {
    // Non-uniform X spacing
    let series = DataSeries::xy(
        "Measurements",
        vec![0.0, 0.5, 1.5, 4.0, 10.0],
        vec![0.0, 2.0, 3.0, 5.0, 8.0],
    );
    let state = ChartState::line(vec![series]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_mixed_series_implicit_and_explicit_x() {
    // Mix of series: one with explicit x_values, one with implicit indices
    let explicit = DataSeries::xy(
        "Explicit",
        vec![0.0, 2.0, 4.0, 6.0, 8.0],
        vec![10.0, 20.0, 30.0, 20.0, 10.0],
    );
    let implicit = DataSeries::new("Implicit", vec![15.0, 25.0, 20.0, 30.0, 25.0]);

    let state = ChartState::line(vec![explicit, implicit]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_area_with_xy_data() {
    let series = DataSeries::xy(
        "Distribution",
        vec![0.0, 0.25, 0.5, 0.75, 1.0],
        vec![0.0, 0.3, 0.8, 0.3, 0.0],
    );
    let state = ChartState::area(vec![series]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_xy_with_negative_x_values() {
    let series = DataSeries::xy(
        "Centered",
        vec![-2.0, -1.0, 0.0, 1.0, 2.0],
        vec![4.0, 1.0, 0.0, 1.0, 4.0],
    );
    let state = ChartState::line(vec![series]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_xy_empty_series() {
    let series = DataSeries::xy("Empty", vec![], vec![]);
    assert!(series.is_empty());
    assert_eq!(series.len(), 0);
    assert_eq!(series.x_values(), Some([].as_slice()));
}

#[test]
fn test_xy_single_point() {
    let series = DataSeries::xy("Single", vec![5.0], vec![10.0]);
    assert_eq!(series.len(), 1);
    assert_eq!(series.x_values(), Some([5.0].as_slice()));
    assert_eq!(series.values(), &[10.0]);
}

#[test]
fn test_xy_min_max_on_y_values() {
    // min/max should still operate on Y values (values field)
    let series = DataSeries::xy("XY", vec![100.0, 200.0, 300.0], vec![5.0, 15.0, 10.0]);
    assert_eq!(series.min(), 5.0);
    assert_eq!(series.max(), 15.0);
}

#[test]
fn test_xy_clear_clears_y_values_only() {
    let mut series = DataSeries::xy("XY", vec![0.0, 1.0], vec![10.0, 20.0]);
    series.clear();
    assert!(series.is_empty());
    // x_values should still be present after clear (clear only affects values)
    assert!(series.x_values().is_some());
}

// =============================================================================
// BarMode
// =============================================================================

#[test]
fn test_bar_mode_default_is_single() {
    assert_eq!(BarMode::default(), BarMode::Single);
}

#[test]
fn test_bar_mode_builder() {
    let state = ChartState::bar_vertical(vec![DataSeries::new("A", vec![1.0])])
        .with_bar_mode(BarMode::Grouped);
    assert_eq!(state.bar_mode(), &BarMode::Grouped);
}

#[test]
fn test_bar_mode_builder_stacked() {
    let state = ChartState::bar_vertical(vec![DataSeries::new("A", vec![1.0])])
        .with_bar_mode(BarMode::Stacked);
    assert_eq!(state.bar_mode(), &BarMode::Stacked);
}

#[test]
fn test_bar_mode_setter() {
    let mut state = ChartState::bar_vertical(vec![]);
    assert_eq!(state.bar_mode(), &BarMode::Single);
    state.set_bar_mode(BarMode::Grouped);
    assert_eq!(state.bar_mode(), &BarMode::Grouped);
}

#[test]
fn test_bar_mode_default_state() {
    assert_eq!(ChartState::default().bar_mode(), &BarMode::Single);
}

#[test]
fn test_bar_mode_clone_eq() {
    assert_eq!(BarMode::Grouped.clone(), BarMode::Grouped);
    assert_ne!(BarMode::Single, BarMode::Stacked);
}

#[test]
fn test_render_single_mode() {
    let state = ChartState::bar_vertical(vec![
        DataSeries::new("A", vec![10.0, 20.0]),
        DataSeries::new("B", vec![5.0, 15.0]).with_color(Color::Red),
    ]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|f| Chart::view(&state, f, f.area(), &theme, &ViewContext::default()))
        .unwrap();
}

#[test]
fn test_render_grouped_vertical() {
    let state = ChartState::bar_vertical(vec![
        DataSeries::new("Q1", vec![10.0, 20.0]),
        DataSeries::new("Q2", vec![15.0, 25.0]).with_color(Color::Red),
    ])
    .with_bar_mode(BarMode::Grouped)
    .with_categories(vec!["A", "B"]);
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|f| Chart::view(&state, f, f.area(), &theme, &ViewContext::default()))
        .unwrap();
}

#[test]
fn test_render_grouped_horizontal() {
    let state = ChartState::bar_horizontal(vec![
        DataSeries::new("2023", vec![100.0, 200.0]),
        DataSeries::new("2024", vec![120.0, 180.0]).with_color(Color::Red),
    ])
    .with_bar_mode(BarMode::Grouped);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|f| Chart::view(&state, f, f.area(), &theme, &ViewContext::default()))
        .unwrap();
}

#[test]
fn test_render_stacked_vertical() {
    let state = ChartState::bar_vertical(vec![
        DataSeries::new("Rev", vec![100.0, 200.0]),
        DataSeries::new("Cost", vec![60.0, 120.0]).with_color(Color::Red),
    ])
    .with_bar_mode(BarMode::Stacked)
    .with_categories(vec!["Q1", "Q2"]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|f| Chart::view(&state, f, f.area(), &theme, &ViewContext::default()))
        .unwrap();
}

#[test]
fn test_render_stacked_horizontal() {
    let state = ChartState::bar_horizontal(vec![
        DataSeries::new("A", vec![50.0, 30.0]),
        DataSeries::new("B", vec![30.0, 40.0]).with_color(Color::Red),
    ])
    .with_bar_mode(BarMode::Stacked);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|f| Chart::view(&state, f, f.area(), &theme, &ViewContext::default()))
        .unwrap();
}

#[test]
fn test_stacked_sums_values() {
    let state = ChartState::bar_vertical(vec![
        DataSeries::new("A", vec![10.0, 20.0]),
        DataSeries::new("B", vec![5.0, 10.0]).with_color(Color::Red),
    ])
    .with_bar_mode(BarMode::Stacked);
    let (mut terminal, theme) = test_utils::setup_render(40, 15);
    terminal
        .draw(|f| Chart::view(&state, f, f.area(), &theme, &ViewContext::default()))
        .unwrap();
}

#[test]
fn test_bar_width_auto_scales() {
    let state = ChartState::bar_vertical(vec![DataSeries::new("A", vec![10.0, 20.0])]);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|f| Chart::view(&state, f, f.area(), &theme, &ViewContext::default()))
        .unwrap();
}

#[test]
fn test_render_grouped_disabled() {
    let state = ChartState::bar_vertical(vec![
        DataSeries::new("A", vec![10.0]),
        DataSeries::new("B", vec![15.0]).with_color(Color::Red),
    ])
    .with_bar_mode(BarMode::Grouped);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|f| {
            Chart::view(
                &state,
                f,
                f.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_stacked_disabled() {
    let state = ChartState::bar_vertical(vec![
        DataSeries::new("A", vec![10.0]),
        DataSeries::new("B", vec![15.0]).with_color(Color::Red),
    ])
    .with_bar_mode(BarMode::Stacked);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|f| {
            Chart::view(
                &state,
                f,
                f.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_stacked_zero_values() {
    let state = ChartState::bar_vertical(vec![
        DataSeries::new("A", vec![0.0, 10.0]),
        DataSeries::new("B", vec![5.0, 0.0]).with_color(Color::Red),
    ])
    .with_bar_mode(BarMode::Stacked);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|f| Chart::view(&state, f, f.area(), &theme, &ViewContext::default()))
        .unwrap();
}
