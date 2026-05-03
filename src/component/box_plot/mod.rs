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

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};

mod render;
mod state;

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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("P99 Latency", 1.0, 2.0, 3.0, 4.0, 5.0);
    /// assert_eq!(data.label(), "P99 Latency");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the minimum value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Latency", 5.0, 15.0, 25.0, 35.0, 45.0);
    /// assert_eq!(data.min(), 5.0);
    /// ```
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Returns the first quartile (Q1).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Latency", 5.0, 15.0, 25.0, 35.0, 45.0);
    /// assert_eq!(data.q1(), 15.0);
    /// ```
    pub fn q1(&self) -> f64 {
        self.q1
    }

    /// Returns the median.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Latency", 5.0, 15.0, 25.0, 35.0, 45.0);
    /// assert_eq!(data.median(), 25.0);
    /// ```
    pub fn median(&self) -> f64 {
        self.median
    }

    /// Returns the third quartile (Q3).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Latency", 5.0, 15.0, 25.0, 35.0, 45.0);
    /// assert_eq!(data.q3(), 35.0);
    /// ```
    pub fn q3(&self) -> f64 {
        self.q3
    }

    /// Returns the maximum value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Latency", 5.0, 15.0, 25.0, 35.0, 45.0);
    /// assert_eq!(data.max(), 45.0);
    /// ```
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Returns the outliers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let data = BoxPlotData::new("Latency", 5.0, 15.0, 25.0, 35.0, 45.0)
    ///     .with_outliers(vec![100.0]);
    /// assert_eq!(data.outliers(), &[100.0]);
    /// ```
    pub fn outliers(&self) -> &[f64] {
        &self.outliers
    }

    /// Returns the color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    /// use ratatui::style::Color;
    ///
    /// let data = BoxPlotData::new("CPU", 1.0, 2.0, 3.0, 4.0, 5.0)
    ///     .with_color(Color::Blue);
    /// assert_eq!(data.color(), Color::Blue);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let mut data = BoxPlotData::new("Old Label", 1.0, 2.0, 3.0, 4.0, 5.0);
    /// data.set_label("New Label");
    /// assert_eq!(data.label(), "New Label");
    /// ```
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Sets the color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    /// use ratatui::style::Color;
    ///
    /// let mut data = BoxPlotData::new("CPU", 1.0, 2.0, 3.0, 4.0, 5.0);
    /// data.set_color(Color::Green);
    /// assert_eq!(data.color(), Color::Green);
    /// ```
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Sets the outliers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let mut data = BoxPlotData::new("Latency", 1.0, 2.0, 3.0, 4.0, 5.0);
    /// data.set_outliers(vec![0.1, 8.0]);
    /// assert_eq!(data.outliers(), &[0.1, 8.0]);
    /// ```
    pub fn set_outliers(&mut self, outliers: Vec<f64>) {
        self.outliers = outliers;
    }

    /// Adds a single outlier value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotData;
    ///
    /// let mut data = BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0);
    /// data.add_outlier(9.5);
    /// assert_eq!(data.outliers(), &[9.5]);
    /// ```
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

// BoxPlotState impl is in state.rs

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
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            Key::Right | Key::Char('l') => Some(BoxPlotMessage::NextDataset),
            Key::Left | Key::Char('h') => Some(BoxPlotMessage::PrevDataset),
            Key::Char('o') => Some(BoxPlotMessage::ToggleOutliers),
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

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if ctx.area.height < 3 || ctx.area.width < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::container("box_plot")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let inner = if ctx.chrome_owned {
            ctx.area
        } else {
            let border_style = if ctx.disabled {
                ctx.theme.disabled_style()
            } else if ctx.focused {
                ctx.theme.focused_border_style()
            } else {
                ctx.theme.border_style()
            };

            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            if let Some(ref title) = state.title {
                block = block.title(title.as_str());
            }

            let inner = block.inner(ctx.area);
            ctx.frame.render_widget(block, ctx.area);
            inner
        };

        if inner.height == 0 || inner.width == 0 || state.datasets.is_empty() {
            return;
        }

        match state.orientation {
            BoxPlotOrientation::Vertical => {
                render::render_vertical(
                    state,
                    ctx.frame,
                    inner,
                    ctx.theme,
                    ctx.focused,
                    ctx.disabled,
                );
            }
            BoxPlotOrientation::Horizontal => {
                render::render_horizontal(
                    state,
                    ctx.frame,
                    inner,
                    ctx.theme,
                    ctx.focused,
                    ctx.disabled,
                );
            }
        }
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
