use super::*;
use crate::component::test_utils;

// =============================================================================
// ChartAnnotation struct
// =============================================================================

#[test]
fn test_annotation_new() {
    let ann = ChartAnnotation::new(3.0, 95.0, "Peak", Color::Yellow);
    assert_eq!(ann.x, 3.0);
    assert_eq!(ann.y, 95.0);
    assert_eq!(ann.label, "Peak");
    assert_eq!(ann.color, Color::Yellow);
}

#[test]
fn test_annotation_clone() {
    let ann = ChartAnnotation::new(1.0, 2.0, "Test", Color::Red);
    let ann2 = ann.clone();
    assert_eq!(ann, ann2);
}

#[test]
fn test_annotation_debug() {
    let ann = ChartAnnotation::new(0.0, 0.0, "Origin", Color::White);
    let debug = format!("{:?}", ann);
    assert!(debug.contains("ChartAnnotation"));
    assert!(debug.contains("Origin"));
}

#[test]
fn test_annotation_partial_eq() {
    let a = ChartAnnotation::new(1.0, 2.0, "A", Color::Red);
    let b = ChartAnnotation::new(1.0, 2.0, "A", Color::Red);
    let c = ChartAnnotation::new(1.0, 2.0, "B", Color::Red);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// =============================================================================
// Builder: with_annotation
// =============================================================================

#[test]
fn test_with_annotation_builder() {
    let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 90.0, 60.0])])
        .with_annotation(1.0, 90.0, "Peak", Color::Yellow);
    assert_eq!(state.annotations().len(), 1);
    assert_eq!(state.annotations()[0].x, 1.0);
    assert_eq!(state.annotations()[0].y, 90.0);
    assert_eq!(state.annotations()[0].label, "Peak");
    assert_eq!(state.annotations()[0].color, Color::Yellow);
}

#[test]
fn test_with_annotation_builder_chained() {
    let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 90.0, 60.0])])
        .with_annotation(1.0, 90.0, "Peak", Color::Yellow)
        .with_annotation(0.0, 50.0, "Start", Color::Green)
        .with_annotation(2.0, 60.0, "End", Color::Red);
    assert_eq!(state.annotations().len(), 3);
    assert_eq!(state.annotations()[0].label, "Peak");
    assert_eq!(state.annotations()[1].label, "Start");
    assert_eq!(state.annotations()[2].label, "End");
}

#[test]
fn test_with_annotation_builder_on_scatter() {
    let state = ChartState::scatter(vec![DataSeries::new("Points", vec![10.0, 25.0, 15.0])])
        .with_annotation(1.0, 25.0, "Max", Color::Cyan);
    assert_eq!(state.annotations().len(), 1);
    assert_eq!(state.kind(), &ChartKind::Scatter);
}

#[test]
fn test_with_annotation_builder_on_area() {
    let state = ChartState::area(vec![DataSeries::new("Load", vec![40.0, 80.0, 55.0])])
        .with_annotation(1.0, 80.0, "Peak Load", Color::Red);
    assert_eq!(state.annotations().len(), 1);
    assert_eq!(state.kind(), &ChartKind::Area);
}

// =============================================================================
// Accessor: annotations()
// =============================================================================

#[test]
fn test_annotations_accessor_empty() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![1.0])]);
    assert!(state.annotations().is_empty());
}

#[test]
fn test_annotations_accessor_returns_all() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0])])
        .with_annotation(0.0, 1.0, "First", Color::Red)
        .with_annotation(1.0, 2.0, "Second", Color::Blue);
    let anns = state.annotations();
    assert_eq!(anns.len(), 2);
    assert_eq!(anns[0].label, "First");
    assert_eq!(anns[1].label, "Second");
}

// =============================================================================
// Setter: add_annotation
// =============================================================================

#[test]
fn test_add_annotation() {
    let mut state = ChartState::line(vec![DataSeries::new("A", vec![1.0])]);
    assert!(state.annotations().is_empty());
    state.add_annotation(ChartAnnotation::new(0.0, 1.0, "Added", Color::Green));
    assert_eq!(state.annotations().len(), 1);
    assert_eq!(state.annotations()[0].label, "Added");
}

#[test]
fn test_add_annotation_multiple() {
    let mut state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0, 3.0])]);
    state.add_annotation(ChartAnnotation::new(0.0, 1.0, "First", Color::Red));
    state.add_annotation(ChartAnnotation::new(1.0, 2.0, "Second", Color::Green));
    state.add_annotation(ChartAnnotation::new(2.0, 3.0, "Third", Color::Blue));
    assert_eq!(state.annotations().len(), 3);
}

// =============================================================================
// Setter: clear_annotations
// =============================================================================

#[test]
fn test_clear_annotations() {
    let mut state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0])])
        .with_annotation(0.0, 1.0, "A", Color::Red)
        .with_annotation(1.0, 2.0, "B", Color::Blue);
    assert_eq!(state.annotations().len(), 2);
    state.clear_annotations();
    assert!(state.annotations().is_empty());
}

#[test]
fn test_clear_annotations_already_empty() {
    let mut state = ChartState::line(vec![DataSeries::new("A", vec![1.0])]);
    state.clear_annotations();
    assert!(state.annotations().is_empty());
}

// =============================================================================
// Rendering: no panic tests
// =============================================================================

fn render_chart_with_annotations(state: &ChartState) {
    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    terminal
        .draw(|frame| {
            Chart::view(state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_line_chart_with_annotation_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 90.0, 60.0, 70.0])])
        .with_annotation(1.0, 90.0, "Peak", Color::Yellow);
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_area_chart_with_annotation_no_panic() {
    let state = ChartState::area(vec![DataSeries::new("Load", vec![40.0, 80.0, 55.0, 65.0])])
        .with_annotation(1.0, 80.0, "Peak Load", Color::Red);
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_scatter_chart_with_annotation_no_panic() {
    let state = ChartState::scatter(vec![DataSeries::new(
        "Points",
        vec![10.0, 25.0, 15.0, 30.0],
    )])
    .with_annotation(3.0, 30.0, "Max", Color::Cyan);
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_multiple_annotations_no_panic() {
    let state = ChartState::line(vec![DataSeries::new(
        "Temp",
        vec![20.0, 22.0, 25.0, 23.0, 18.0],
    )])
    .with_annotation(2.0, 25.0, "Peak", Color::Red)
    .with_annotation(4.0, 18.0, "Trough", Color::Blue)
    .with_annotation(0.0, 20.0, "Start", Color::Green);
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_no_annotations_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 90.0, 60.0])]);
    render_chart_with_annotations(&state);
}

// =============================================================================
// Boundary conditions
// =============================================================================

#[test]
fn test_render_annotation_at_origin_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![0.0, 10.0, 20.0])])
        .with_annotation(0.0, 0.0, "Origin", Color::White);
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_annotation_at_max_boundary_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![0.0, 50.0, 100.0])])
        .with_annotation(2.0, 100.0, "Max", Color::Red);
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_annotation_outside_bounds_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![10.0, 20.0, 30.0])])
        .with_annotation(10.0, 200.0, "OutOfBounds", Color::Red);
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_annotation_negative_coords_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![-10.0, 0.0, 10.0])])
        .with_y_range(-20.0, 20.0)
        .with_annotation(-1.0, -10.0, "Negative", Color::Blue);
    // The annotation x=-1.0 is outside x bounds 0..2, so it should be skipped
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_annotation_with_long_label_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![10.0, 20.0, 30.0])])
        .with_annotation(
            1.0,
            20.0,
            "This is a very long annotation label that exceeds the chart width",
            Color::Yellow,
        );
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_annotation_empty_label_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![10.0, 20.0, 30.0])])
        .with_annotation(1.0, 20.0, "", Color::Yellow);
    render_chart_with_annotations(&state);
}

#[test]
fn test_render_annotation_small_chart_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![10.0, 20.0])]).with_annotation(
        0.0,
        10.0,
        "Tiny",
        Color::Red,
    );
    let (mut terminal, theme) = test_utils::setup_render(10, 5);
    terminal
        .draw(|frame| {
            Chart::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_annotation_with_log_scale_no_panic() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 10.0, 100.0, 1000.0])])
        .with_y_scale(Scale::Log10)
        .with_annotation(2.0, 100.0, "100x", Color::Cyan);
    render_chart_with_annotations(&state);
}

#[test]
fn test_default_chart_state_has_empty_annotations() {
    let state = ChartState::default();
    assert!(state.annotations().is_empty());
}

#[test]
fn test_annotations_preserved_through_other_builders() {
    let state = ChartState::line(vec![DataSeries::new("A", vec![1.0, 2.0, 3.0])])
        .with_annotation(1.0, 2.0, "Mid", Color::Green)
        .with_title("Test Chart")
        .with_y_range(0.0, 10.0)
        .with_grid(true);
    assert_eq!(state.annotations().len(), 1);
    assert_eq!(state.annotations()[0].label, "Mid");
    assert_eq!(state.title(), Some("Test Chart"));
}
