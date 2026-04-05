//! A heatmap component for 2D color-intensity grid display.
//!
//! [`Heatmap`] renders a grid of cells where each cell's color intensity
//! represents a value. Useful for GitHub-style contribution graphs,
//! correlation matrices, error rates by hour and day, and similar
//! visualizations. State is stored in [`HeatmapState`], updated via
//! [`HeatmapMessage`], and produces [`HeatmapOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Heatmap, HeatmapState, HeatmapMessage, HeatmapColorScale,
//! };
//!
//! let mut state = HeatmapState::new(3, 5);
//! state.set(0, 0, 1.0);
//! state.set(1, 2, 0.5);
//! assert_eq!(state.rows(), 3);
//! assert_eq!(state.cols(), 5);
//! assert_eq!(state.get(0, 0), Some(1.0));
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, Disableable, Focusable, ViewContext};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

mod render;

/// Color scale for the heatmap.
///
/// Determines how values are mapped to colors. Each variant defines a
/// different color gradient from low to high values.
///
/// # Example
///
/// ```rust
/// use envision::component::HeatmapColorScale;
/// use ratatui::style::Color;
///
/// let scale = HeatmapColorScale::GreenToRed;
/// assert_eq!(scale, HeatmapColorScale::default());
///
/// let custom = HeatmapColorScale::Intensity(Color::Cyan);
/// assert_ne!(custom, HeatmapColorScale::GreenToRed);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum HeatmapColorScale {
    /// Green (low) to Red (high), passing through yellow.
    #[default]
    GreenToRed,
    /// Blue (low) to Red (high), passing through magenta.
    BlueToRed,
    /// Cool blue (low) to warm yellow (high), passing through gray.
    CoolToWarm,
    /// Single color with varying intensity (dim to bright).
    Intensity(Color),
}

/// Messages that can be sent to a Heatmap.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, Heatmap, HeatmapState, HeatmapMessage,
/// };
///
/// let mut state = HeatmapState::new(3, 3);
/// state.set_focused(true);
/// state.update(HeatmapMessage::SelectDown);
/// assert_eq!(state.selected(), Some((1, 0)));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum HeatmapMessage {
    /// Replace all data.
    SetData(Vec<Vec<f64>>),
    /// Set a single cell value.
    SetCell {
        /// Row index.
        row: usize,
        /// Column index.
        col: usize,
        /// The value to set.
        value: f64,
    },
    /// Set row labels.
    SetRowLabels(Vec<String>),
    /// Set column labels.
    SetColLabels(Vec<String>),
    /// Change color scale.
    SetColorScale(HeatmapColorScale),
    /// Set manual value range.
    SetRange(Option<f64>, Option<f64>),
    /// Move selection up.
    SelectUp,
    /// Move selection down.
    SelectDown,
    /// Move selection left.
    SelectLeft,
    /// Move selection right.
    SelectRight,
    /// Clear all data.
    Clear,
}

/// Output messages from a Heatmap.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, Heatmap, HeatmapState, HeatmapOutput,
/// };
///
/// let mut state = HeatmapState::with_data(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
/// state.set_focused(true);
/// // Navigate to select a cell, then Enter to confirm
/// let output = state.dispatch_event(&envision::input::Event::key(
///     envision::input::KeyCode::Enter,
/// ));
/// assert_eq!(
///     output,
///     Some(HeatmapOutput::CellSelected {
///         row: 0,
///         col: 0,
///         value: 1.0,
///     })
/// );
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum HeatmapOutput {
    /// A cell was selected/confirmed with Enter.
    CellSelected {
        /// Row index.
        row: usize,
        /// Column index.
        col: usize,
        /// The value at the cell.
        value: f64,
    },
    /// Navigation changed the selected cell.
    SelectionChanged {
        /// Row index.
        row: usize,
        /// Column index.
        col: usize,
    },
}

/// State for a Heatmap component.
///
/// Contains the 2D data grid, labels, color scale, and selection state.
///
/// # Example
///
/// ```rust
/// use envision::component::HeatmapState;
///
/// let state = HeatmapState::new(4, 7);
/// assert_eq!(state.rows(), 4);
/// assert_eq!(state.cols(), 7);
/// assert_eq!(state.get(0, 0), Some(0.0));
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct HeatmapState {
    /// 2D grid of values (rows x columns).
    data: Vec<Vec<f64>>,
    /// Labels for each row.
    row_labels: Vec<String>,
    /// Labels for each column.
    col_labels: Vec<String>,
    /// How to map values to colors.
    color_scale: HeatmapColorScale,
    /// Manual minimum for scaling (None = auto).
    min_value: Option<f64>,
    /// Manual maximum for scaling (None = auto).
    max_value: Option<f64>,
    /// Selected row for navigation.
    selected_row: Option<usize>,
    /// Selected column for navigation.
    selected_col: Option<usize>,
    /// Display values in cells.
    show_values: bool,
    /// Optional title.
    title: Option<String>,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl HeatmapState {
    /// Creates a new empty heatmap grid with the given dimensions.
    ///
    /// All cells are initialized to 0.0. If the grid has at least one cell,
    /// the selection is set to (0, 0).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(3, 5);
    /// assert_eq!(state.rows(), 3);
    /// assert_eq!(state.cols(), 5);
    /// assert_eq!(state.get(2, 4), Some(0.0));
    /// assert_eq!(state.selected(), Some((0, 0)));
    /// ```
    pub fn new(rows: usize, cols: usize) -> Self {
        let data = vec![vec![0.0; cols]; rows];
        let (selected_row, selected_col) = if rows > 0 && cols > 0 {
            (Some(0), Some(0))
        } else {
            (None, None)
        };
        Self {
            data,
            selected_row,
            selected_col,
            ..Default::default()
        }
    }

    /// Creates a heatmap from existing 2D data.
    ///
    /// If the data has at least one row with at least one column,
    /// the selection is set to (0, 0).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let data = vec![
    ///     vec![1.0, 2.0, 3.0],
    ///     vec![4.0, 5.0, 6.0],
    /// ];
    /// let state = HeatmapState::with_data(data);
    /// assert_eq!(state.rows(), 2);
    /// assert_eq!(state.cols(), 3);
    /// assert_eq!(state.get(1, 2), Some(6.0));
    /// ```
    pub fn with_data(data: Vec<Vec<f64>>) -> Self {
        let has_cells = !data.is_empty() && data.iter().any(|row| !row.is_empty());
        let (selected_row, selected_col) = if has_cells {
            (Some(0), Some(0))
        } else {
            (None, None)
        };
        Self {
            data,
            selected_row,
            selected_col,
            ..Default::default()
        }
    }

    /// Sets row labels (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(2, 3)
    ///     .with_row_labels(vec!["Row A".into(), "Row B".into()]);
    /// assert_eq!(state.row_labels(), &["Row A", "Row B"]);
    /// ```
    pub fn with_row_labels(mut self, labels: Vec<String>) -> Self {
        self.row_labels = labels;
        self
    }

    /// Sets column labels (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(2, 3)
    ///     .with_col_labels(vec!["A".into(), "B".into(), "C".into()]);
    /// assert_eq!(state.col_labels(), &["A", "B", "C"]);
    /// ```
    pub fn with_col_labels(mut self, labels: Vec<String>) -> Self {
        self.col_labels = labels;
        self
    }

    /// Sets the color scale (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HeatmapState, HeatmapColorScale};
    ///
    /// let state = HeatmapState::new(2, 2)
    ///     .with_color_scale(HeatmapColorScale::BlueToRed);
    /// ```
    pub fn with_color_scale(mut self, scale: HeatmapColorScale) -> Self {
        self.color_scale = scale;
        self
    }

    /// Sets the manual value range (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(2, 2).with_range(0.0, 100.0);
    /// assert_eq!(state.effective_min(), 0.0);
    /// assert_eq!(state.effective_max(), 100.0);
    /// ```
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min_value = Some(min);
        self.max_value = Some(max);
        self
    }

    /// Sets whether to show values in cells (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(2, 2).with_show_values(true);
    /// ```
    pub fn with_show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(2, 2).with_title("Error Rates");
    /// assert_eq!(state.title(), Some("Error Rates"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(2, 2).with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Accessors ----

    /// Returns the 2D data grid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::with_data(vec![vec![1.0, 2.0]]);
    /// assert_eq!(state.data(), &[vec![1.0, 2.0]]);
    /// ```
    pub fn data(&self) -> &[Vec<f64>] {
        &self.data
    }

    /// Returns a mutable reference to the 2D data grid.
    ///
    /// This is safe because the heatmap has no derived indices or
    /// filter state; color mapping is computed on each render.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let mut state = HeatmapState::with_data(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
    /// state.data_mut()[0][1] = 10.0;
    /// assert_eq!(state.get(0, 1), Some(10.0));
    /// ```
    pub fn data_mut(&mut self) -> &mut Vec<Vec<f64>> {
        &mut self.data
    }

    /// Returns the number of rows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(4, 3);
    /// assert_eq!(state.rows(), 4);
    /// ```
    pub fn rows(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of columns.
    ///
    /// Returns the length of the first row, or 0 if the grid is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(3, 5);
    /// assert_eq!(state.cols(), 5);
    /// ```
    pub fn cols(&self) -> usize {
        self.data.first().map_or(0, |row| row.len())
    }

    /// Returns the value at the given position, or None if out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::with_data(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
    /// assert_eq!(state.get(0, 1), Some(2.0));
    /// assert_eq!(state.get(5, 0), None);
    /// ```
    pub fn get(&self, row: usize, col: usize) -> Option<f64> {
        self.data.get(row).and_then(|r| r.get(col)).copied()
    }

    /// Sets the value at the given position.
    ///
    /// Does nothing if the position is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let mut state = HeatmapState::new(2, 2);
    /// state.set(0, 1, 42.0);
    /// assert_eq!(state.get(0, 1), Some(42.0));
    /// ```
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        if let Some(r) = self.data.get_mut(row) {
            if let Some(cell) = r.get_mut(col) {
                *cell = value;
            }
        }
    }

    /// Returns the current selection as (row, col), or None if nothing is selected.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::new(3, 3);
    /// assert_eq!(state.selected(), Some((0, 0)));
    ///
    /// let empty = HeatmapState::default();
    /// assert_eq!(empty.selected(), None);
    /// ```
    pub fn selected(&self) -> Option<(usize, usize)> {
        match (self.selected_row, self.selected_col) {
            (Some(r), Some(c)) => Some((r, c)),
            _ => None,
        }
    }

    /// Returns the value of the currently selected cell.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::with_data(vec![vec![7.5, 3.2]]);
    /// assert_eq!(state.selected_value(), Some(7.5));
    /// ```
    pub fn selected_value(&self) -> Option<f64> {
        let (r, c) = self.selected()?;
        self.get(r, c)
    }

    /// Returns the effective minimum value for color scaling.
    ///
    /// If a manual minimum is set, that value is used. Otherwise, the
    /// minimum is computed from the data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::with_data(vec![vec![5.0, 10.0, 15.0]]);
    /// assert_eq!(state.effective_min(), 5.0);
    ///
    /// let manual = HeatmapState::new(2, 2).with_range(0.0, 100.0);
    /// assert_eq!(manual.effective_min(), 0.0);
    /// ```
    pub fn effective_min(&self) -> f64 {
        self.min_value.unwrap_or_else(|| {
            self.data
                .iter()
                .flat_map(|row| row.iter())
                .copied()
                .reduce(f64::min)
                .unwrap_or(0.0)
        })
    }

    /// Returns the effective maximum value for color scaling.
    ///
    /// If a manual maximum is set, that value is used. Otherwise, the
    /// maximum is computed from the data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let state = HeatmapState::with_data(vec![vec![5.0, 10.0, 15.0]]);
    /// assert_eq!(state.effective_max(), 15.0);
    ///
    /// let manual = HeatmapState::new(2, 2).with_range(0.0, 100.0);
    /// assert_eq!(manual.effective_max(), 100.0);
    /// ```
    pub fn effective_max(&self) -> f64 {
        self.max_value.unwrap_or_else(|| {
            self.data
                .iter()
                .flat_map(|row| row.iter())
                .copied()
                .reduce(f64::max)
                .unwrap_or(0.0)
        })
    }

    /// Returns the row labels.
    pub fn row_labels(&self) -> &[String] {
        &self.row_labels
    }

    /// Returns the column labels.
    pub fn col_labels(&self) -> &[String] {
        &self.col_labels
    }

    /// Returns the color scale.
    pub fn color_scale(&self) -> &HeatmapColorScale {
        &self.color_scale
    }

    /// Returns the title, if set.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let mut state = HeatmapState::new(3, 3);
    /// state.set_title("Error Rates");
    /// assert_eq!(state.title(), Some("Error Rates"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns whether values are shown in cells.
    pub fn show_values(&self) -> bool {
        self.show_values
    }

    /// Sets whether values are shown in cells.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HeatmapState;
    ///
    /// let mut state = HeatmapState::new(3, 3);
    /// state.set_show_values(true);
    /// assert!(state.show_values());
    /// ```
    pub fn set_show_values(&mut self, show: bool) {
        self.show_values = show;
    }

    // ---- Instance methods ----

    /// Returns true if the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Maps an input event to a heatmap message.
    pub fn handle_event(&self, event: &Event) -> Option<HeatmapMessage> {
        Heatmap::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<HeatmapOutput> {
        Heatmap::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: HeatmapMessage) -> Option<HeatmapOutput> {
        Heatmap::update(self, msg)
    }

    /// Returns the number of columns for a given row, handling uneven rows.
    fn row_cols(&self, row: usize) -> usize {
        self.data.get(row).map_or(0, |r| r.len())
    }
}

/// Maps a value to a color based on the heatmap color scale.
///
/// The value is normalized to the range [0.0, 1.0] based on the given
/// min and max, then mapped to an RGB color according to the scale.
///
/// # Example
///
/// ```rust
/// use envision::component::{HeatmapColorScale, value_to_color};
/// use ratatui::style::Color;
///
/// let color = value_to_color(0.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
/// assert_eq!(color, Color::Rgb(0, 255, 0));
///
/// let color = value_to_color(1.0, 0.0, 1.0, &HeatmapColorScale::GreenToRed);
/// assert_eq!(color, Color::Rgb(255, 0, 0));
/// ```
pub fn value_to_color(value: f64, min: f64, max: f64, scale: &HeatmapColorScale) -> Color {
    let t = if (max - min).abs() < f64::EPSILON {
        0.5
    } else {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    };

    match scale {
        HeatmapColorScale::GreenToRed => {
            // Green -> Yellow -> Red
            if t <= 0.5 {
                let s = t * 2.0; // 0..1 within first half
                let r = (255.0 * s) as u8;
                let g = 255u8;
                Color::Rgb(r, g, 0)
            } else {
                let s = (t - 0.5) * 2.0; // 0..1 within second half
                let r = 255u8;
                let g = (255.0 * (1.0 - s)) as u8;
                Color::Rgb(r, g, 0)
            }
        }
        HeatmapColorScale::BlueToRed => {
            // Blue -> Magenta -> Red
            if t <= 0.5 {
                let s = t * 2.0;
                let r = (255.0 * s) as u8;
                let b = 255u8;
                Color::Rgb(r, 0, b)
            } else {
                let s = (t - 0.5) * 2.0;
                let r = 255u8;
                let b = (255.0 * (1.0 - s)) as u8;
                Color::Rgb(r, 0, b)
            }
        }
        HeatmapColorScale::CoolToWarm => {
            // Blue(0,0,200) -> Gray(200,200,200) -> Yellow(200,200,0)
            if t <= 0.5 {
                let s = t * 2.0;
                let r = (200.0 * s) as u8;
                let g = (200.0 * s) as u8;
                let b = 200u8;
                Color::Rgb(r, g, b)
            } else {
                let s = (t - 0.5) * 2.0;
                let r = 200u8;
                let g = 200u8;
                let b = (200.0 * (1.0 - s)) as u8;
                Color::Rgb(r, g, b)
            }
        }
        HeatmapColorScale::Intensity(base_color) => {
            // Extract the base RGB and scale brightness
            let (br, bg, bb) = match base_color {
                Color::Rgb(r, g, b) => (*r, *g, *b),
                Color::Red => (255, 0, 0),
                Color::Green => (0, 255, 0),
                Color::Blue => (0, 0, 255),
                Color::Yellow => (255, 255, 0),
                Color::Cyan => (0, 255, 255),
                Color::Magenta => (255, 0, 255),
                Color::White => (255, 255, 255),
                _ => (128, 128, 128),
            };
            // Scale from dim (t=0) to full brightness (t=1)
            // Minimum brightness of ~20% so cells are always visible
            let factor = 0.2 + 0.8 * t;
            let r = (br as f64 * factor) as u8;
            let g = (bg as f64 * factor) as u8;
            let b = (bb as f64 * factor) as u8;
            Color::Rgb(r, g, b)
        }
    }
}

/// A heatmap component for 2D color-intensity grid display.
///
/// Renders a grid of cells where each cell's background color represents
/// a value. Supports arrow key / hjkl navigation and Enter to confirm
/// selection.
///
/// # Key Bindings
///
/// - `Up` / `k` -- Move selection up
/// - `Down` / `j` -- Move selection down
/// - `Left` / `h` -- Move selection left
/// - `Right` / `l` -- Move selection right
/// - `Enter` -- Confirm selection
pub struct Heatmap(PhantomData<()>);

impl Component for Heatmap {
    type State = HeatmapState;
    type Message = HeatmapMessage;
    type Output = HeatmapOutput;

    fn init() -> Self::State {
        HeatmapState::default()
    }

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(HeatmapMessage::SelectUp),
            KeyCode::Down | KeyCode::Char('j') => Some(HeatmapMessage::SelectDown),
            KeyCode::Left | KeyCode::Char('h') => Some(HeatmapMessage::SelectLeft),
            KeyCode::Right | KeyCode::Char('l') => Some(HeatmapMessage::SelectRight),
            KeyCode::Enter => {
                // Confirm selection -- handled in update to produce CellSelected output
                if let Some((row, col)) = state.selected() {
                    if let Some(value) = state.get(row, col) {
                        return Some(HeatmapMessage::SetCell { row, col, value });
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            HeatmapMessage::SetData(data) => {
                let has_cells = !data.is_empty() && data.iter().any(|row| !row.is_empty());
                state.data = data;
                if has_cells {
                    // Clamp selection to new bounds
                    let max_row = state.data.len().saturating_sub(1);
                    let max_col = state.cols().saturating_sub(1);
                    state.selected_row = Some(state.selected_row.unwrap_or(0).min(max_row));
                    state.selected_col = Some(state.selected_col.unwrap_or(0).min(max_col));
                } else {
                    state.selected_row = None;
                    state.selected_col = None;
                }
                None
            }
            HeatmapMessage::SetCell { row, col, value } => {
                // If this is triggered by Enter, emit CellSelected
                if state.selected() == Some((row, col)) {
                    return Some(HeatmapOutput::CellSelected { row, col, value });
                }
                state.set(row, col, value);
                None
            }
            HeatmapMessage::SetRowLabels(labels) => {
                state.row_labels = labels;
                None
            }
            HeatmapMessage::SetColLabels(labels) => {
                state.col_labels = labels;
                None
            }
            HeatmapMessage::SetColorScale(scale) => {
                state.color_scale = scale;
                None
            }
            HeatmapMessage::SetRange(min, max) => {
                state.min_value = min;
                state.max_value = max;
                None
            }
            HeatmapMessage::SelectUp => navigate_selection(state, Direction::Up),
            HeatmapMessage::SelectDown => navigate_selection(state, Direction::Down),
            HeatmapMessage::SelectLeft => navigate_selection(state, Direction::Left),
            HeatmapMessage::SelectRight => navigate_selection(state, Direction::Right),
            HeatmapMessage::Clear => {
                for row in &mut state.data {
                    for cell in row.iter_mut() {
                        *cell = 0.0;
                    }
                }
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if area.height < 3 || area.width < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::container("heatmap")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let border_style = if ctx.disabled {
            theme.disabled_style()
        } else if ctx.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(ref title) = state.title {
            block = block.title(title.as_str());
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 || state.data.is_empty() {
            return;
        }

        render::render_heatmap(state, frame, inner, theme, ctx.focused, ctx.disabled);
    }
}

/// Internal navigation direction.
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Navigates the selection, clamping at edges.
fn navigate_selection(state: &mut HeatmapState, direction: Direction) -> Option<HeatmapOutput> {
    if state.data.is_empty() {
        return None;
    }

    let num_rows = state.data.len();
    let current_row = state.selected_row.unwrap_or(0);
    let current_col = state.selected_col.unwrap_or(0);
    let num_cols = state.row_cols(current_row);

    if num_cols == 0 {
        return None;
    }

    let (new_row, new_col) = match direction {
        Direction::Up => {
            if current_row > 0 {
                let nr = current_row - 1;
                let nc = current_col.min(state.row_cols(nr).saturating_sub(1));
                (nr, nc)
            } else {
                return None;
            }
        }
        Direction::Down => {
            if current_row + 1 < num_rows {
                let nr = current_row + 1;
                let nc = current_col.min(state.row_cols(nr).saturating_sub(1));
                (nr, nc)
            } else {
                return None;
            }
        }
        Direction::Left => {
            if current_col > 0 {
                (current_row, current_col - 1)
            } else {
                return None;
            }
        }
        Direction::Right => {
            if current_col + 1 < num_cols {
                (current_row, current_col + 1)
            } else {
                return None;
            }
        }
    };

    state.selected_row = Some(new_row);
    state.selected_col = Some(new_col);

    Some(HeatmapOutput::SelectionChanged {
        row: new_row,
        col: new_col,
    })
}

impl Focusable for Heatmap {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for Heatmap {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
