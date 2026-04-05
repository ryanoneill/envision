//! Box plot component for statistical distribution visualization.
//!
//! Displays box-and-whisker plots showing median, quartiles, and outliers.
//! Supports multiple box plots side-by-side for comparison (e.g., P50/P95/P99
//! latency across services).
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, BoxPlot, BoxPlotState, BoxPlotData, BoxPlotOrientation,
//! };
//! use ratatui::style::Color;
//!
//! let data = BoxPlotData::new("API Service", 5.0, 15.0, 25.0, 35.0, 45.0)
//!     .with_color(Color::Cyan);
//! let state = BoxPlotState::new(vec![data]);
//! assert_eq!(state.datasets().len(), 1);
//! assert_eq!(state.orientation(), &BoxPlotOrientation::Vertical);
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

mod render;

/// A single box plot dataset containing the five-number summary and optional outliers.
///
/// The five-number summary consists of minimum, first quartile (Q1), median,
/// third quartile (Q3), and maximum. Outliers are data points that fall outside
/// the whisker range.
///
/// # Example
///
/// ```rust
/// use envision::component::BoxPlotData;
/// use ratatui::style::Color;
///
/// let data = BoxPlotData::new("Latency", 2.0, 10.0, 25.0, 40.0, 55.0)
///     .with_color(Color::Green)
///     .with_outliers(vec![0.5, 70.0, 85.0]);
/// assert_eq!(data.label(), "Latency");
/// assert_eq!(data.min(), 2.0);
/// assert_eq!(data.q1(), 10.0);
/// assert_eq!(data.median(), 25.0);
/// assert_eq!(data.q3(), 40.0);
/// assert_eq!(data.max(), 55.0);
/// assert_eq!(data.outliers(), &[0.5, 70.0, 85.0]);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct BoxPlotData {
    /// The label for this dataset.
    label: String,
    /// Minimum value (lower whisker endpoint).
    min: f64,
    /// First quartile (25th percentile).
    q1: f64,
    /// Median (50th percentile).
    median: f64,
    /// Third quartile (75th percentile).
    q3: f64,
    /// Maximum value (upper whisker endpoint).
    max: f64,
    /// Outlier values outside the whisker range.
    outliers: Vec<f64>,
    /// Display color for this box plot.
    color: Color,
}

impl BoxPlotData {
    /// Creates a new box plot dataset with the five-number summary.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Response Time", 5.0, 20.0, 35.0, 50.0, 80.0);
    /// assert_eq!(data.label(), "Response Time");
    /// assert_eq!(data.median(), 35.0);
    /// ```
    pub fn new(
        label: impl Into<String>,
        min: f64,
        q1: f64,
        median: f64,
        q3: f64,
        max: f64,
    ) -> Self {
        Self {
            label: label.into(),
            min,
            q1,
            median,
            q3,
            max,
            outliers: Vec::new(),
            color: Color::Cyan,
        }
    }

    /// Sets the color (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    /// use ratatui::style::Color;
    ///
    /// let data = BoxPlotData::new("CPU", 1.0, 2.0, 3.0, 4.0, 5.0)
    ///     .with_color(Color::Red);
    /// assert_eq!(data.color(), Color::Red);
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the outliers (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Latency", 5.0, 15.0, 25.0, 35.0, 45.0)
    ///     .with_outliers(vec![1.0, 60.0]);
    /// assert_eq!(data.outliers(), &[1.0, 60.0]);
    /// ```
    pub fn with_outliers(mut self, outliers: Vec<f64>) -> Self {
        self.outliers = outliers;
        self
    }

    /// Returns the label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the minimum value.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Returns the first quartile (Q1).
    pub fn q1(&self) -> f64 {
        self.q1
    }

    /// Returns the median.
    pub fn median(&self) -> f64 {
        self.median
    }

    /// Returns the third quartile (Q3).
    pub fn q3(&self) -> f64 {
        self.q3
    }

    /// Returns the maximum value.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Returns the outliers.
    pub fn outliers(&self) -> &[f64] {
        &self.outliers
    }

    /// Returns the color.
    pub fn color(&self) -> Color {
        self.color
    }

    /// Returns the interquartile range (IQR = Q3 - Q1).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Test", 0.0, 10.0, 20.0, 30.0, 40.0);
    /// assert_eq!(data.iqr(), 20.0);
    /// ```
    pub fn iqr(&self) -> f64 {
        self.q3 - self.q1
    }

    /// Returns the range (max - min), not including outliers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Test", 5.0, 10.0, 20.0, 30.0, 45.0);
    /// assert_eq!(data.range(), 40.0);
    /// ```
    pub fn range(&self) -> f64 {
        self.max - self.min
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Sets the color.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Sets the outliers.
    pub fn set_outliers(&mut self, outliers: Vec<f64>) {
        self.outliers = outliers;
    }

    /// Adds a single outlier value.
    pub fn add_outlier(&mut self, value: f64) {
        self.outliers.push(value);
    }

    /// Returns the overall minimum including outliers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Test", 5.0, 10.0, 20.0, 30.0, 45.0)
    ///     .with_outliers(vec![1.0, 60.0]);
    /// assert_eq!(data.overall_min(), 1.0);
    /// ```
    pub fn overall_min(&self) -> f64 {
        let outlier_min = self
            .outliers
            .iter()
            .copied()
            .reduce(f64::min)
            .unwrap_or(self.min);
        f64::min(self.min, outlier_min)
    }

    /// Returns the overall maximum including outliers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Test", 5.0, 10.0, 20.0, 30.0, 45.0)
    ///     .with_outliers(vec![1.0, 60.0]);
    /// assert_eq!(data.overall_max(), 60.0);
    /// ```
    pub fn overall_max(&self) -> f64 {
        let outlier_max = self
            .outliers
            .iter()
            .copied()
            .reduce(f64::max)
            .unwrap_or(self.max);
        f64::max(self.max, outlier_max)
    }
}

/// Orientation of the box plot rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum BoxPlotOrientation {
    /// Vertical box plots (values on Y axis, datasets on X axis).
    Vertical,
    /// Horizontal box plots (values on X axis, datasets on Y axis).
    Horizontal,
}

/// Messages that can be sent to a BoxPlot.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum BoxPlotMessage {
    /// Select the next dataset.
    NextDataset,
    /// Select the previous dataset.
    PrevDataset,
    /// Toggle whether outliers are shown.
    ToggleOutliers,
    /// Replace all datasets.
    SetDatasets(Vec<BoxPlotData>),
    /// Add a single dataset.
    AddDataset(BoxPlotData),
    /// Clear all datasets.
    ClearDatasets,
    /// Set the orientation.
    SetOrientation(BoxPlotOrientation),
}

/// State for a BoxPlot component.
///
/// Contains the datasets, display configuration, and interaction state.
///
/// # Example
///
/// ```rust
/// use envision::component::{BoxPlotState, BoxPlotData, BoxPlotOrientation};
///
/// let state = BoxPlotState::new(vec![
///     BoxPlotData::new("Service A", 10.0, 20.0, 30.0, 40.0, 50.0),
///     BoxPlotData::new("Service B", 15.0, 25.0, 35.0, 45.0, 55.0),
/// ])
/// .with_title("Latency Distribution")
/// .with_show_outliers(true);
/// assert_eq!(state.datasets().len(), 2);
/// assert_eq!(state.title(), Some("Latency Distribution"));
/// assert!(state.show_outliers());
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct BoxPlotState {
    /// The box plot datasets to display.
    datasets: Vec<BoxPlotData>,
    /// Optional title.
    title: Option<String>,
    /// Whether to show outlier markers.
    show_outliers: bool,
    /// Plot orientation.
    orientation: BoxPlotOrientation,
    /// Index of the currently selected dataset.
    selected: usize,
}

impl Default for BoxPlotState {
    fn default() -> Self {
        Self {
            datasets: Vec::new(),
            title: None,
            show_outliers: true,
            orientation: BoxPlotOrientation::Vertical,
            selected: 0,
        }
    }
}

impl BoxPlotState {
    /// Creates a new box plot state with the given datasets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ]);
    /// assert_eq!(state.datasets().len(), 1);
    /// ```
    pub fn new(datasets: Vec<BoxPlotData>) -> Self {
        Self {
            datasets,
            ..Default::default()
        }
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ])
    /// .with_title("My Box Plot");
    /// assert_eq!(state.title(), Some("My Box Plot"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show outliers (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotState;
    ///
    /// let state = BoxPlotState::default().with_show_outliers(false);
    /// assert!(!state.show_outliers());
    /// ```
    pub fn with_show_outliers(mut self, show: bool) -> Self {
        self.show_outliers = show;
        self
    }

    /// Sets the orientation (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotOrientation};
    ///
    /// let state = BoxPlotState::default()
    ///     .with_orientation(BoxPlotOrientation::Horizontal);
    /// assert_eq!(state.orientation(), &BoxPlotOrientation::Horizontal);
    /// ```
    pub fn with_orientation(mut self, orientation: BoxPlotOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    // ---- Accessors ----

    /// Returns the datasets.
    pub fn datasets(&self) -> &[BoxPlotData] {
        &self.datasets
    }

    /// Returns a mutable reference to the datasets.
    pub fn datasets_mut(&mut self) -> &mut [BoxPlotData] {
        &mut self.datasets
    }

    /// Returns the dataset at the given index.
    pub fn get_dataset(&self, index: usize) -> Option<&BoxPlotData> {
        self.datasets.get(index)
    }

    /// Returns a mutable reference to the dataset at the given index.
    pub fn get_dataset_mut(&mut self, index: usize) -> Option<&mut BoxPlotData> {
        self.datasets.get_mut(index)
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns whether outliers are shown.
    pub fn show_outliers(&self) -> bool {
        self.show_outliers
    }

    /// Sets whether outliers are shown.
    pub fn set_show_outliers(&mut self, show: bool) {
        self.show_outliers = show;
    }

    /// Returns the orientation.
    pub fn orientation(&self) -> &BoxPlotOrientation {
        &self.orientation
    }

    /// Sets the orientation.
    pub fn set_orientation(&mut self, orientation: BoxPlotOrientation) {
        self.orientation = orientation;
    }

    /// Returns the currently selected dataset index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Sets the selected dataset index.
    pub fn set_selected(&mut self, index: usize) {
        if !self.datasets.is_empty() {
            self.selected = index.min(self.datasets.len() - 1);
        }
    }

    /// Returns the number of datasets.
    pub fn dataset_count(&self) -> usize {
        self.datasets.len()
    }

    /// Returns true if there are no datasets.
    pub fn is_empty(&self) -> bool {
        self.datasets.is_empty()
    }

    /// Adds a dataset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let mut state = BoxPlotState::default();
    /// state.add_dataset(BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0));
    /// assert_eq!(state.dataset_count(), 1);
    /// ```
    pub fn add_dataset(&mut self, dataset: BoxPlotData) {
        self.datasets.push(dataset);
    }

    /// Clears all datasets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let mut state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ]);
    /// state.clear_datasets();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear_datasets(&mut self) {
        self.datasets.clear();
        self.selected = 0;
    }

    /// Computes the global minimum value across all datasets (including outliers
    /// if show_outliers is enabled).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0),
    ///     BoxPlotData::new("B", 8.0, 15.0, 25.0, 35.0, 50.0),
    /// ]);
    /// assert_eq!(state.global_min(), 5.0);
    /// ```
    pub fn global_min(&self) -> f64 {
        self.datasets
            .iter()
            .map(|d| {
                if self.show_outliers {
                    d.overall_min()
                } else {
                    d.min()
                }
            })
            .reduce(f64::min)
            .unwrap_or(0.0)
    }

    /// Computes the global maximum value across all datasets (including outliers
    /// if show_outliers is enabled).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0),
    ///     BoxPlotData::new("B", 8.0, 15.0, 25.0, 35.0, 50.0),
    /// ]);
    /// assert_eq!(state.global_max(), 50.0);
    /// ```
    pub fn global_max(&self) -> f64 {
        self.datasets
            .iter()
            .map(|d| {
                if self.show_outliers {
                    d.overall_max()
                } else {
                    d.max()
                }
            })
            .reduce(f64::max)
            .unwrap_or(0.0)
    }

    // ---- Focus / Disabled ----

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: BoxPlotMessage) -> Option<()> {
        BoxPlot::update(self, msg)
    }
}

/// A box plot component for statistical distribution visualization.
///
/// Displays box-and-whisker plots showing the five-number summary (min, Q1,
/// median, Q3, max) and optional outliers. Multiple datasets can be displayed
/// side-by-side for comparison.
///
/// # Key Bindings
///
/// - `Left` / `h` -- Select previous dataset
/// - `Right` / `l` -- Select next dataset
/// - `o` -- Toggle outlier display
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, BoxPlot, BoxPlotState, BoxPlotData};
///
/// let state = BoxPlotState::new(vec![
///     BoxPlotData::new("Service A", 10.0, 20.0, 30.0, 40.0, 50.0),
///     BoxPlotData::new("Service B", 15.0, 25.0, 35.0, 45.0, 55.0),
/// ])
/// .with_title("Latency Comparison");
/// assert_eq!(state.datasets().len(), 2);
/// ```
pub struct BoxPlot(PhantomData<()>);

impl Component for BoxPlot {
    type State = BoxPlotState;
    type Message = BoxPlotMessage;
    type Output = ();

    fn init() -> Self::State {
        BoxPlotState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            KeyCode::Right | KeyCode::Char('l') => Some(BoxPlotMessage::NextDataset),
            KeyCode::Left | KeyCode::Char('h') => Some(BoxPlotMessage::PrevDataset),
            KeyCode::Char('o') => Some(BoxPlotMessage::ToggleOutliers),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            BoxPlotMessage::NextDataset => {
                if !state.datasets.is_empty() {
                    state.selected = (state.selected + 1) % state.datasets.len();
                }
            }
            BoxPlotMessage::PrevDataset => {
                if !state.datasets.is_empty() {
                    state.selected = if state.selected == 0 {
                        state.datasets.len() - 1
                    } else {
                        state.selected - 1
                    };
                }
            }
            BoxPlotMessage::ToggleOutliers => {
                state.show_outliers = !state.show_outliers;
            }
            BoxPlotMessage::SetDatasets(datasets) => {
                state.datasets = datasets;
                state.selected = 0;
            }
            BoxPlotMessage::AddDataset(dataset) => {
                state.datasets.push(dataset);
            }
            BoxPlotMessage::ClearDatasets => {
                state.datasets.clear();
                state.selected = 0;
            }
            BoxPlotMessage::SetOrientation(orientation) => {
                state.orientation = orientation;
            }
        }
        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if area.height < 3 || area.width < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::container("box_plot")
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

        if inner.height == 0 || inner.width == 0 || state.datasets.is_empty() {
            return;
        }

        match state.orientation {
            BoxPlotOrientation::Vertical => {
                render::render_vertical(state, frame, inner, theme, ctx.focused, ctx.disabled);
            }
            BoxPlotOrientation::Horizontal => {
                render::render_horizontal(state, frame, inner, theme, ctx.focused, ctx.disabled);
            }
        }
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
