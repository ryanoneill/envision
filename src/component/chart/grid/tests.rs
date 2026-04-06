use super::*;
use crate::component::ViewContext;
use crate::component::chart::DataSeries;
use crate::component::test_utils;

fn sample_chart(title: &str) -> ChartState {
    ChartState::line(vec![DataSeries::new(title, vec![1.0, 2.0, 3.0])]).with_title(title)
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new_dimensions() {
    let grid = ChartGrid::new(3, 4);
    assert_eq!(grid.rows(), 3);
    assert_eq!(grid.cols(), 4);
    assert_eq!(grid.cell_count(), 12);
    assert_eq!(grid.chart_count(), 0);
}

#[test]
fn test_new_1x1() {
    let grid = ChartGrid::new(1, 1);
    assert_eq!(grid.rows(), 1);
    assert_eq!(grid.cols(), 1);
    assert_eq!(grid.cell_count(), 1);
}

#[test]
#[should_panic(expected = "ChartGrid rows must be at least 1")]
fn test_new_zero_rows_panics() {
    ChartGrid::new(0, 2);
}

#[test]
#[should_panic(expected = "ChartGrid cols must be at least 1")]
fn test_new_zero_cols_panics() {
    ChartGrid::new(2, 0);
}

// =============================================================================
// Set / Get
// =============================================================================

#[test]
fn test_set_and_get() {
    let grid = ChartGrid::new(2, 2)
        .set(0, 1, sample_chart("A"))
        .set(1, 0, sample_chart("B"));

    assert!(grid.get(0, 0).is_none());
    assert_eq!(grid.get(0, 1).unwrap().title(), Some("A"));
    assert_eq!(grid.get(1, 0).unwrap().title(), Some("B"));
    assert!(grid.get(1, 1).is_none());
    assert_eq!(grid.chart_count(), 2);
}

#[test]
fn test_set_overwrites_existing() {
    let grid =
        ChartGrid::new(1, 1)
            .set(0, 0, sample_chart("First"))
            .set(0, 0, sample_chart("Second"));

    assert_eq!(grid.get(0, 0).unwrap().title(), Some("Second"));
    assert_eq!(grid.chart_count(), 1);
}

#[test]
#[should_panic(expected = "row index 2 out of bounds")]
fn test_set_row_out_of_bounds_panics() {
    ChartGrid::new(2, 2).set(2, 0, sample_chart("X"));
}

#[test]
#[should_panic(expected = "col index 3 out of bounds")]
fn test_set_col_out_of_bounds_panics() {
    ChartGrid::new(2, 2).set(0, 3, sample_chart("X"));
}

// =============================================================================
// Get mut
// =============================================================================

#[test]
fn test_get_mut_modifies_chart() {
    let mut grid = ChartGrid::new(1, 2).set(0, 0, sample_chart("Original"));

    let chart = grid.get_mut(0, 0).unwrap();
    chart.set_title(Some("Modified".to_string()));

    assert_eq!(grid.get(0, 0).unwrap().title(), Some("Modified"));
}

#[test]
fn test_get_mut_empty_cell_returns_none() {
    let mut grid = ChartGrid::new(2, 2);
    assert!(grid.get_mut(0, 0).is_none());
    assert!(grid.get_mut(1, 1).is_none());
}

#[test]
#[should_panic(expected = "row index 2 out of bounds")]
fn test_get_mut_row_out_of_bounds_panics() {
    let mut grid = ChartGrid::new(2, 2);
    grid.get_mut(2, 0);
}

#[test]
#[should_panic(expected = "row index 1 out of bounds")]
fn test_get_row_out_of_bounds_panics() {
    let grid = ChartGrid::new(1, 1);
    grid.get(1, 0);
}

// =============================================================================
// Set chart (mutating)
// =============================================================================

#[test]
fn test_set_chart_mutating() {
    let mut grid = ChartGrid::new(2, 2);
    grid.set_chart(1, 1, sample_chart("Late"));
    assert_eq!(grid.get(1, 1).unwrap().title(), Some("Late"));
    assert_eq!(grid.chart_count(), 1);
}

// =============================================================================
// Take
// =============================================================================

#[test]
fn test_take_removes_chart() {
    let mut grid = ChartGrid::new(1, 2).set(0, 0, sample_chart("Taken"));

    let taken = grid.take(0, 0);
    assert!(taken.is_some());
    assert_eq!(taken.unwrap().title(), Some("Taken"));
    assert!(grid.get(0, 0).is_none());
    assert_eq!(grid.chart_count(), 0);
}

#[test]
fn test_take_empty_cell_returns_none() {
    let mut grid = ChartGrid::new(1, 1);
    assert!(grid.take(0, 0).is_none());
}

// =============================================================================
// Render
// =============================================================================

#[test]
fn test_render_2x2_grid_does_not_panic() {
    let grid = ChartGrid::new(2, 2)
        .set(0, 0, sample_chart("TL"))
        .set(0, 1, sample_chart("TR"))
        .set(1, 0, sample_chart("BL"))
        .set(1, 1, sample_chart("BR"));

    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    terminal
        .draw(|frame| {
            grid.render(frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_empty_cells_does_not_panic() {
    let grid = ChartGrid::new(2, 3).set(0, 0, sample_chart("Only"));

    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    terminal
        .draw(|frame| {
            grid.render(frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_all_empty_does_not_panic() {
    let grid = ChartGrid::new(2, 2);

    let (mut terminal, theme) = test_utils::setup_render(80, 24);
    terminal
        .draw(|frame| {
            grid.render(frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_1x1_single_chart() {
    let grid = ChartGrid::new(1, 1).set(0, 0, sample_chart("Solo"));

    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            grid.render(frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Solo"), "Title should be visible");
}

#[test]
fn test_render_small_area_does_not_panic() {
    let grid = ChartGrid::new(2, 2)
        .set(0, 0, sample_chart("A"))
        .set(1, 1, sample_chart("B"));

    let (mut terminal, theme) = test_utils::setup_render(10, 6);
    terminal
        .draw(|frame| {
            grid.render(frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_focused_context() {
    let grid = ChartGrid::new(1, 2)
        .set(0, 0, sample_chart("L"))
        .set(0, 1, sample_chart("R"));

    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    let ctx = ViewContext::new().focused(true);
    terminal
        .draw(|frame| {
            grid.render(frame, frame.area(), &theme, &ctx);
        })
        .unwrap();
}

#[test]
fn test_render_mixed_chart_kinds() {
    let line = ChartState::line(vec![DataSeries::new("Line", vec![1.0, 2.0, 3.0])])
        .with_title("Line Chart");
    let bar = ChartState::bar_vertical(vec![DataSeries::new("Bar", vec![10.0, 20.0, 15.0])])
        .with_title("Bar Chart");

    let grid = ChartGrid::new(1, 2).set(0, 0, line).set(0, 1, bar);

    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            grid.render(frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Line Chart"));
    assert!(output.contains("Bar Chart"));
}

// =============================================================================
// Clone / Debug
// =============================================================================

#[test]
fn test_clone() {
    let grid = ChartGrid::new(2, 2).set(0, 0, sample_chart("A"));
    let cloned = grid.clone();
    assert_eq!(cloned.rows(), 2);
    assert_eq!(cloned.cols(), 2);
    assert_eq!(cloned.chart_count(), 1);
    assert_eq!(cloned.get(0, 0).unwrap().title(), Some("A"));
}

#[test]
fn test_debug() {
    let grid = ChartGrid::new(1, 1);
    let debug = format!("{:?}", grid);
    assert!(debug.contains("ChartGrid"));
}
