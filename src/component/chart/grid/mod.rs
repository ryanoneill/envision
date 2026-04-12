//! Grid layout for rendering multiple charts simultaneously.
//!
//! [`ChartGrid`] arranges multiple [`ChartState`] instances in a rows-by-columns
//! grid, similar to matplotlib's `subplots(rows, cols)`. Each cell can hold an
//! optional chart; empty cells are left blank. The grid handles layout splitting
//! and delegates rendering to [`Chart::view`] for each occupied cell.
//!
//! # Example
//!
//! ```rust,ignore
//! let grid = ChartGrid::new(2, 2)
//!     .set(0, 0, loss_chart)
//!     .set(0, 1, accuracy_chart)
//!     .set(1, 0, lr_chart)
//!     .set(1, 1, throughput_chart);
//! ```

use ratatui::prelude::*;

use super::{Chart, ChartState};
use crate::component::{Component, RenderContext};

/// A grid layout for rendering multiple charts simultaneously.
///
/// Arranges charts in a fixed rows-by-columns grid. Each cell is addressed
/// by `(row, col)` and can hold an optional [`ChartState`]. Empty cells
/// render nothing.
///
/// # Example
///
/// ```rust
/// use envision::component::{ChartGrid, ChartState, DataSeries};
///
/// let grid = ChartGrid::new(2, 2)
///     .set(0, 0, ChartState::line(vec![DataSeries::new("Loss", vec![1.0, 0.5, 0.2])]))
///     .set(1, 1, ChartState::line(vec![DataSeries::new("Acc", vec![0.6, 0.8, 0.95])]));
/// assert_eq!(grid.rows(), 2);
/// assert_eq!(grid.cols(), 2);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ChartGrid {
    rows: usize,
    cols: usize,
    charts: Vec<Option<ChartState>>, // row-major order
}

impl ChartGrid {
    /// Creates a new grid with the given dimensions.
    ///
    /// All cells start empty. Use [`set`](Self::set) to place charts.
    ///
    /// # Panics
    ///
    /// Panics if `rows` or `cols` is zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartGrid;
    ///
    /// let grid = ChartGrid::new(3, 2);
    /// assert_eq!(grid.rows(), 3);
    /// assert_eq!(grid.cols(), 2);
    /// ```
    pub fn new(rows: usize, cols: usize) -> Self {
        assert!(rows > 0, "ChartGrid rows must be at least 1");
        assert!(cols > 0, "ChartGrid cols must be at least 1");
        Self {
            rows,
            cols,
            charts: vec![None; rows * cols],
        }
    }

    /// Places a chart at the given grid position (builder pattern).
    ///
    /// # Panics
    ///
    /// Panics if `row >= rows` or `col >= cols`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartGrid, ChartState, DataSeries};
    ///
    /// let grid = ChartGrid::new(1, 2)
    ///     .set(0, 0, ChartState::line(vec![DataSeries::new("A", vec![1.0])]));
    /// assert!(grid.get(0, 0).is_some());
    /// assert!(grid.get(0, 1).is_none());
    /// ```
    pub fn set(mut self, row: usize, col: usize, chart: ChartState) -> Self {
        assert!(
            row < self.rows,
            "row index {row} out of bounds for grid with {} rows",
            self.rows
        );
        assert!(
            col < self.cols,
            "col index {col} out of bounds for grid with {} cols",
            self.cols
        );
        self.charts[row * self.cols + col] = Some(chart);
        self
    }

    /// Returns a reference to the chart at the given position, if any.
    ///
    /// # Panics
    ///
    /// Panics if `row >= rows` or `col >= cols`.
    pub fn get(&self, row: usize, col: usize) -> Option<&ChartState> {
        assert!(
            row < self.rows,
            "row index {row} out of bounds for grid with {} rows",
            self.rows
        );
        assert!(
            col < self.cols,
            "col index {col} out of bounds for grid with {} cols",
            self.cols
        );
        self.charts[row * self.cols + col].as_ref()
    }

    /// Returns a mutable reference to the chart at the given position, if any.
    ///
    /// # Panics
    ///
    /// Panics if `row >= rows` or `col >= cols`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartGrid, ChartState, DataSeries};
    ///
    /// let mut grid = ChartGrid::new(1, 1)
    ///     .set(0, 0, ChartState::line(vec![DataSeries::new("A", vec![1.0])]));
    /// let chart = grid.get_mut(0, 0).unwrap();
    /// chart.set_title(Some("Updated".to_string()));
    /// assert_eq!(grid.get(0, 0).unwrap().title(), Some("Updated"));
    /// ```
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut ChartState> {
        assert!(
            row < self.rows,
            "row index {row} out of bounds for grid with {} rows",
            self.rows
        );
        assert!(
            col < self.cols,
            "col index {col} out of bounds for grid with {} cols",
            self.cols
        );
        self.charts[row * self.cols + col].as_mut()
    }

    /// Places a chart at the given grid position (mutating variant).
    ///
    /// # Panics
    ///
    /// Panics if `row >= rows` or `col >= cols`.
    pub fn set_chart(&mut self, row: usize, col: usize, chart: ChartState) {
        assert!(
            row < self.rows,
            "row index {row} out of bounds for grid with {} rows",
            self.rows
        );
        assert!(
            col < self.cols,
            "col index {col} out of bounds for grid with {} cols",
            self.cols
        );
        self.charts[row * self.cols + col] = Some(chart);
    }

    /// Removes the chart at the given position, returning it if present.
    ///
    /// # Panics
    ///
    /// Panics if `row >= rows` or `col >= cols`.
    pub fn take(&mut self, row: usize, col: usize) -> Option<ChartState> {
        assert!(
            row < self.rows,
            "row index {row} out of bounds for grid with {} rows",
            self.rows
        );
        assert!(
            col < self.cols,
            "col index {col} out of bounds for grid with {} cols",
            self.cols
        );
        self.charts[row * self.cols + col].take()
    }

    /// Returns the number of rows.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Returns the number of columns.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Returns the total number of cells in the grid.
    pub fn cell_count(&self) -> usize {
        self.rows * self.cols
    }

    /// Returns the number of cells that contain a chart.
    pub fn chart_count(&self) -> usize {
        self.charts.iter().filter(|c| c.is_some()).count()
    }

    /// Renders all charts in the grid.
    ///
    /// Splits the area into equal rows and columns, then delegates each
    /// occupied cell to [`Chart::view`].
    pub fn render(&self, ctx: &mut RenderContext<'_, '_>) {
        let row_constraints: Vec<Constraint> = (0..self.rows)
            .map(|_| Constraint::Ratio(1, self.rows as u32))
            .collect();

        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(ctx.area);

        let col_constraints: Vec<Constraint> = (0..self.cols)
            .map(|_| Constraint::Ratio(1, self.cols as u32))
            .collect();

        for (row_idx, row_area) in row_areas.iter().enumerate() {
            let col_areas = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints.clone())
                .split(*row_area);

            for (col_idx, col_area) in col_areas.iter().enumerate() {
                let cell_idx = row_idx * self.cols + col_idx;
                if let Some(chart) = &self.charts[cell_idx] {
                    Chart::view(chart, &mut ctx.with_area(*col_area));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
