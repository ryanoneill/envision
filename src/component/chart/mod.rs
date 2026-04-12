//! Chart components for data visualization.
//!
//! Provides line charts (braille rendering with shared axes), bar charts
//! (horizontal/vertical), area charts, and scatter plots with data series,
//! labels, colors, threshold lines, logarithmic scaling, smart tick labels,
//! LTTB downsampling, and auto-scaling axes.
//! State is stored in [`ChartState`] and updated via [`ChartMessage`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Chart, ChartState, ChartMessage, DataSeries, ChartKind,
//! };
//! use ratatui::style::Color;
//!
//! let series = DataSeries::new("Temperature", vec![20.0, 22.0, 25.0, 23.0])
//!     .with_color(Color::Red);
//! let mut state = ChartState::line(vec![series]);
//! assert_eq!(state.series().len(), 1);
//! assert_eq!(state.kind(), &ChartKind::Line);
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, KeyCode};

mod annotations;
pub(crate) mod downsample;
mod error_bands;
pub(crate) mod format;
mod grid;
mod render;
pub(crate) mod scale;
mod series;
mod state;
pub(crate) mod ticks;

pub use annotations::ChartAnnotation;
pub use grid::ChartGrid;
pub use scale::Scale;
pub use series::{DEFAULT_PALETTE, chart_palette_color};

/// Default color palette for auto-assigning colors to multi-series charts.
/// A named data series with values and styling.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DataSeries {
    /// The series label.
    label: String,
    /// The data values (Y-axis values).
    values: Vec<f64>,
    /// The display color.
    color: Color,
    /// Optional explicit X-axis values. When present, these are used instead of
    /// sequential indices (0, 1, 2, ...). Useful for ROC curves, scatter plots
    /// with non-uniform spacing, and parametric curves.
    x_values: Option<Vec<f64>>,
    upper_bound: Option<Vec<f64>>,
    lower_bound: Option<Vec<f64>>,
}

// DataSeries methods are in series.rs

/// The bar rendering mode for bar charts.
///
/// Controls how multiple series are displayed in bar charts:
/// - `Single`: Only the active series is shown (default, backwards-compatible).
/// - `Grouped`: All series are shown side-by-side at each position.
/// - `Stacked`: All series are stacked vertically at each position.
///
/// # Example
///
/// ```rust
/// use envision::component::BarMode;
///
/// let mode = BarMode::default();
/// assert_eq!(mode, BarMode::Single);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum BarMode {
    /// Render only the active series (default).
    #[default]
    Single,
    /// Render all series side-by-side at each position.
    Grouped,
    /// Stack all series vertically at each position.
    Stacked,
}

/// The kind of chart to display.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ChartKind {
    /// A line chart (braille markers with shared axes).
    Line,
    /// A vertical bar chart.
    BarVertical,
    /// A horizontal bar chart.
    BarHorizontal,
    /// A filled line chart (area chart) using shared axes.
    Area,
    /// A scatter plot with individual data points.
    Scatter,
}

/// A horizontal threshold/reference line rendered on area and scatter charts.
///
/// Threshold lines are drawn as horizontal lines spanning the full chart width
/// at a specified y-value, useful for marking targets, SLOs, or limits.
///
/// # Example
///
/// ```rust
/// use envision::component::ThresholdLine;
/// use ratatui::style::Color;
///
/// let threshold = ThresholdLine::new(95.0, "SLO Target", Color::Yellow);
/// assert_eq!(threshold.value, 95.0);
/// assert_eq!(threshold.label, "SLO Target");
/// assert_eq!(threshold.color, Color::Yellow);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ThresholdLine {
    /// The y-value for this threshold.
    pub value: f64,
    /// Label for the threshold (e.g., "SLO target").
    pub label: String,
    /// Color for the threshold line.
    pub color: Color,
}

impl ThresholdLine {
    /// Creates a new threshold line.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ThresholdLine;
    /// use ratatui::style::Color;
    ///
    /// let t = ThresholdLine::new(100.0, "Max", Color::Red);
    /// assert_eq!(t.value, 100.0);
    /// assert_eq!(t.label, "Max");
    /// ```
    pub fn new(value: f64, label: impl Into<String>, color: Color) -> Self {
        Self {
            value,
            label: label.into(),
            color,
        }
    }
}

/// A vertical reference line rendered on line, area, and scatter charts.
///
/// Vertical lines are drawn as vertical lines spanning the full chart height
/// at a specified x-value, useful for marking events, transitions, or epochs.
///
/// # Example
///
/// ```rust
/// use envision::component::VerticalLine;
/// use ratatui::style::Color;
///
/// let vline = VerticalLine::new(10000.0, "Grokking", Color::Yellow);
/// assert_eq!(vline.x_value, 10000.0);
/// assert_eq!(vline.label, "Grokking");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct VerticalLine {
    /// The x-value for this vertical line.
    pub x_value: f64,
    /// Label for the vertical line.
    pub label: String,
    /// Color for the vertical line.
    pub color: Color,
}

impl VerticalLine {
    /// Creates a new vertical reference line.
    pub fn new(x_value: f64, label: impl Into<String>, color: Color) -> Self {
        Self {
            x_value,
            label: label.into(),
            color,
        }
    }
}

/// Messages that can be sent to a Chart.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ChartMessage {
    /// Cycle to the next series (for multi-series line charts).
    NextSeries,
    /// Cycle to the previous series.
    PrevSeries,
    /// Set threshold lines, replacing any existing ones.
    SetThresholds(Vec<ThresholdLine>),
    /// Add a single threshold line.
    AddThreshold(ThresholdLine),
    /// Set the manual Y-axis range. `None` values fall back to auto-scaling.
    SetYRange(Option<f64>, Option<f64>),
    /// Set the Y-axis scale (Linear, Log10, or SymLog).
    SetYScale(Scale),
    /// Set vertical reference lines, replacing any existing ones.
    SetVerticalLines(Vec<VerticalLine>),
    /// Add a single vertical reference line.
    AddVerticalLine(VerticalLine),
    /// Move the crosshair cursor left.
    CursorLeft,
    /// Move the crosshair cursor right.
    CursorRight,
    /// Move the crosshair cursor to the start.
    CursorHome,
    /// Move the crosshair cursor to the end.
    CursorEnd,
    /// Toggle the crosshair cursor visibility.
    ToggleCrosshair,
    /// Toggle grid line visibility.
    ToggleGrid,
}

/// Output messages from a Chart.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ChartOutput {
    /// The active series changed.
    ActiveSeriesChanged(usize),
    /// The crosshair cursor moved to a new position.
    CursorMoved(usize),
    /// The crosshair was toggled on or off.
    CrosshairToggled(bool),
    /// Grid lines were toggled on or off.
    GridToggled(bool),
}

/// State for a Chart component.
///
/// Contains the data series, chart kind, and display options.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ChartState {
    /// The data series to display.
    pub(crate) series: Vec<DataSeries>,
    /// The chart kind.
    pub(crate) kind: ChartKind,
    /// Index of the active (highlighted) series.
    pub(crate) active_series: usize,
    /// Optional title.
    pub(crate) title: Option<String>,
    /// X-axis label.
    pub(crate) x_label: Option<String>,
    /// Y-axis label.
    pub(crate) y_label: Option<String>,
    /// Whether to show the legend.
    pub(crate) show_legend: bool,
    /// Maximum data points to display (for line charts).
    pub(crate) max_display_points: usize,
    /// Bar width for bar charts.
    pub(crate) bar_width: u16,
    /// Bar gap for bar charts.
    pub(crate) bar_gap: u16,
    /// Horizontal threshold/reference lines.
    pub(crate) thresholds: Vec<ThresholdLine>,
    /// Manual Y-axis minimum (None = auto from data).
    pub(crate) y_min: Option<f64>,
    /// Manual Y-axis maximum (None = auto from data).
    pub(crate) y_max: Option<f64>,
    /// Y-axis scale transformation.
    pub(crate) y_scale: Scale,
    /// Vertical reference lines.
    pub(crate) vertical_lines: Vec<VerticalLine>,
    /// Cursor position (data index) for crosshair mode.
    pub(crate) cursor_position: Option<usize>,
    /// Whether to show the crosshair cursor.
    pub(crate) show_crosshair: bool,
    /// Whether to show grid lines at tick positions.
    pub(crate) show_grid: bool,
    /// Category labels for bar chart x-axis (e.g., ["Q1", "Q2", "Q3"]).
    pub(crate) categories: Vec<String>,
    /// Bar rendering mode (Single, Grouped, or Stacked).
    pub(crate) bar_mode: BarMode,
    /// Optional string labels for the X-axis of line, area, and scatter charts.
    /// When present, these replace the numeric tick labels on the X-axis.
    /// Useful for displaying dates, timestamps, or durations without a datetime dependency.
    pub(crate) x_labels: Option<Vec<String>>,
    /// Text annotations at specific data coordinates.
    pub(crate) annotations: Vec<ChartAnnotation>,
}

impl Default for ChartState {
    fn default() -> Self {
        Self {
            series: Vec::new(),
            kind: ChartKind::Line,
            active_series: 0,
            title: None,
            x_label: None,
            y_label: None,
            show_legend: true,
            max_display_points: 500,
            bar_width: 3,
            bar_gap: 1,
            thresholds: Vec::new(),
            y_min: None,
            y_max: None,
            y_scale: Scale::default(),
            vertical_lines: Vec::new(),
            cursor_position: None,
            show_crosshair: false,
            show_grid: false,
            categories: Vec::new(),
            bar_mode: BarMode::default(),
            x_labels: None,
            annotations: Vec::new(),
        }
    }
}

impl ChartState {
    /// Applies default palette colors to series that have the default Cyan color.
    fn apply_palette_colors(mut series: Vec<DataSeries>) -> Vec<DataSeries> {
        for (i, s) in series.iter_mut().enumerate() {
            if s.color() == Color::Cyan {
                s.set_color(DEFAULT_PALETTE[i % DEFAULT_PALETTE.len()]);
            }
        }
        series
    }

    /// Creates a line chart state with the given series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![
    ///     DataSeries::new("Series A", vec![1.0, 2.0, 3.0]),
    /// ]);
    /// assert_eq!(state.series().len(), 1);
    /// ```
    pub fn line(series: Vec<DataSeries>) -> Self {
        Self {
            series: Self::apply_palette_colors(series),
            kind: ChartKind::Line,
            ..Default::default()
        }
    }

    /// Creates a vertical bar chart state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartKind, ChartState, DataSeries};
    ///
    /// let state = ChartState::bar_vertical(vec![
    ///     DataSeries::new("Sales", vec![10.0, 20.0, 30.0]),
    /// ]);
    /// assert_eq!(state.kind(), &ChartKind::BarVertical);
    /// ```
    pub fn bar_vertical(series: Vec<DataSeries>) -> Self {
        Self {
            series: Self::apply_palette_colors(series),
            kind: ChartKind::BarVertical,
            ..Default::default()
        }
    }

    /// Creates a horizontal bar chart state.
    pub fn bar_horizontal(series: Vec<DataSeries>) -> Self {
        Self {
            series: Self::apply_palette_colors(series),
            kind: ChartKind::BarHorizontal,
            ..Default::default()
        }
    }

    /// Creates an area chart state with the given series.
    ///
    /// Area charts render as filled line charts using shared axes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartKind, ChartState, DataSeries};
    ///
    /// let state = ChartState::area(vec![
    ///     DataSeries::new("CPU", vec![45.0, 52.0, 48.0]),
    /// ]);
    /// assert_eq!(state.kind(), &ChartKind::Area);
    /// ```
    pub fn area(series: Vec<DataSeries>) -> Self {
        Self {
            series: Self::apply_palette_colors(series),
            kind: ChartKind::Area,
            ..Default::default()
        }
    }

    /// Creates a scatter plot state with the given series.
    ///
    /// Scatter plots render individual data points without connecting lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartKind, ChartState, DataSeries};
    ///
    /// let state = ChartState::scatter(vec![
    ///     DataSeries::new("Points", vec![10.0, 25.0, 15.0, 30.0]),
    /// ]);
    /// assert_eq!(state.kind(), &ChartKind::Scatter);
    /// ```
    pub fn scatter(series: Vec<DataSeries>) -> Self {
        Self {
            series: Self::apply_palette_colors(series),
            kind: ChartKind::Scatter,
            ..Default::default()
        }
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_title("CPU Usage");
    /// assert_eq!(state.title(), Some("CPU Usage"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the X-axis label (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_x_label("Time");
    /// assert_eq!(state.x_label(), Some("Time"));
    /// ```
    pub fn with_x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = Some(label.into());
        self
    }

    /// Sets the Y-axis label (builder pattern).
    pub fn with_y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Sets whether to show the legend (builder pattern).
    pub fn with_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    /// Sets the maximum display points for line charts (builder pattern).
    pub fn with_max_display_points(mut self, max: usize) -> Self {
        self.max_display_points = max;
        self
    }

    /// Sets the bar width (builder pattern).
    pub fn with_bar_width(mut self, width: u16) -> Self {
        self.bar_width = width.max(1);
        self
    }

    /// Sets the bar gap (builder pattern).
    pub fn with_bar_gap(mut self, gap: u16) -> Self {
        self.bar_gap = gap;
        self
    }

    /// Sets the bar rendering mode (builder pattern).
    ///
    /// Controls how multiple series are displayed in bar charts:
    /// - [`BarMode::Single`]: Only the active series is shown (default).
    /// - [`BarMode::Grouped`]: All series are shown side-by-side at each position.
    /// - [`BarMode::Stacked`]: All series are stacked vertically at each position.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BarMode, ChartState, DataSeries};
    ///
    /// let state = ChartState::bar_vertical(vec![
    ///     DataSeries::new("Q1", vec![10.0, 20.0]),
    ///     DataSeries::new("Q2", vec![15.0, 25.0]),
    /// ])
    /// .with_bar_mode(BarMode::Grouped);
    /// assert_eq!(state.bar_mode(), &BarMode::Grouped);
    /// ```
    pub fn with_bar_mode(mut self, mode: BarMode) -> Self {
        self.bar_mode = mode;
        self
    }

    /// Sets the category labels for bar chart x-axis (builder pattern).
    ///
    /// When set, these labels replace numeric indices on the x-axis of bar charts.
    /// If fewer categories are provided than data points, remaining bars fall back
    /// to numeric labels. Extra categories beyond the data length are ignored.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::bar_vertical(vec![
    ///     DataSeries::new("Importance", vec![0.85, 0.72, 0.64, 0.51]),
    /// ])
    /// .with_categories(vec!["Income", "Education", "Age", "Hours/Week"]);
    /// assert_eq!(state.categories(), &["Income", "Education", "Age", "Hours/Week"]);
    /// ```
    pub fn with_categories(mut self, categories: Vec<impl Into<String>>) -> Self {
        self.categories = categories.into_iter().map(Into::into).collect();
        self
    }

    /// Adds a threshold line (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_threshold(95.0, "SLO", Color::Yellow);
    /// assert_eq!(state.thresholds().len(), 1);
    /// ```
    pub fn with_threshold(mut self, value: f64, label: impl Into<String>, color: Color) -> Self {
        self.thresholds
            .push(ThresholdLine::new(value, label, color));
        self
    }

    /// Sets the manual Y-axis range (builder pattern).
    ///
    /// Values outside this range will be clipped by the chart widget.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_y_range(0.0, 100.0);
    /// assert_eq!(state.y_min(), Some(0.0));
    /// assert_eq!(state.y_max(), Some(100.0));
    /// ```
    pub fn with_y_range(mut self, min: f64, max: f64) -> Self {
        self.y_min = Some(min);
        self.y_max = Some(max);
        self
    }

    /// Sets the Y-axis scale (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, Scale};
    ///
    /// let state = ChartState::line(vec![])
    ///     .with_y_scale(Scale::Log10);
    /// assert_eq!(state.y_scale(), &Scale::Log10);
    /// ```
    pub fn with_y_scale(mut self, scale: Scale) -> Self {
        self.y_scale = scale;
        self
    }

    /// Adds a vertical reference line (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 60.0, 70.0])])
    ///     .with_vertical_line(1.0, "Deploy", Color::Yellow);
    /// assert_eq!(state.vertical_lines().len(), 1);
    /// ```
    pub fn with_vertical_line(
        mut self,
        x_value: f64,
        label: impl Into<String>,
        color: Color,
    ) -> Self {
        self.vertical_lines
            .push(VerticalLine::new(x_value, label, color));
        self
    }

    /// Sets whether to show grid lines at tick positions (builder pattern).
    pub fn with_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Sets custom string labels for the X-axis (builder pattern).
    ///
    /// When set, these labels replace the numeric tick labels on the X-axis
    /// of line, area, and scatter charts. The caller is responsible for
    /// formatting the labels (e.g., formatting timestamps as strings).
    ///
    /// Labels are spaced evenly across the X-axis width. If there are more
    /// labels than can fit, a subset is displayed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let series = DataSeries::new("Requests", vec![100.0, 250.0, 180.0, 320.0, 90.0]);
    /// let state = ChartState::line(vec![series])
    ///     .with_x_labels(vec!["00:00", "06:00", "12:00", "18:00", "24:00"])
    ///     .with_title("Request Rate (24h)");
    /// assert_eq!(state.x_labels().unwrap(), &["00:00", "06:00", "12:00", "18:00", "24:00"]);
    /// ```
    pub fn with_x_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.x_labels = Some(labels.into_iter().map(Into::into).collect());
        self
    }

    /// Adds a text annotation at a data coordinate (builder pattern).
    ///
    /// Annotations are rendered as text labels near the specified (x, y)
    /// position in the chart's data space. Useful for labeling notable
    /// data points such as peaks, anomalies, or events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let state = ChartState::line(vec![
    ///     DataSeries::new("CPU", vec![50.0, 90.0, 60.0]),
    /// ])
    /// .with_annotation(1.0, 90.0, "Peak", Color::Yellow);
    /// assert_eq!(state.annotations().len(), 1);
    /// assert_eq!(state.annotations()[0].label, "Peak");
    /// ```
    pub fn with_annotation(
        mut self,
        x: f64,
        y: f64,
        label: impl Into<String>,
        color: Color,
    ) -> Self {
        self.annotations
            .push(ChartAnnotation::new(x, y, label, color));
        self
    }
}

/// A chart component for data visualization.
///
/// Supports line charts (braille rendering with shared axes), vertical bar
/// charts, horizontal bar charts, area charts (filled line), and scatter plots
/// with multiple data series, threshold lines, logarithmic scaling, smart tick
/// labels, LTTB downsampling, and manual Y-axis scaling.
///
/// # Key Bindings
///
/// - `Tab` — Cycle to next series
/// - `BackTab` — Cycle to previous series
/// - `Left` / `h` — Move crosshair cursor left
/// - `Right` / `l` — Move crosshair cursor right
/// - `Home` — Move crosshair cursor to start
/// - `End` — Move crosshair cursor to end
/// - `c` — Toggle crosshair cursor visibility
/// - `g` — Toggle grid line visibility
pub struct Chart(PhantomData<()>);

impl Component for Chart {
    type State = ChartState;
    type Message = ChartMessage;
    type Output = ChartOutput;

    fn init() -> Self::State {
        ChartState::default()
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
            KeyCode::Tab => Some(ChartMessage::NextSeries),
            KeyCode::BackTab => Some(ChartMessage::PrevSeries),
            KeyCode::Left | KeyCode::Char('h') => Some(ChartMessage::CursorLeft),
            KeyCode::Right | KeyCode::Char('l') => Some(ChartMessage::CursorRight),
            KeyCode::Home => Some(ChartMessage::CursorHome),
            KeyCode::End => Some(ChartMessage::CursorEnd),
            KeyCode::Char('c') => Some(ChartMessage::ToggleCrosshair),
            KeyCode::Char('g') => Some(ChartMessage::ToggleGrid),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ChartMessage::SetThresholds(thresholds) => {
                state.thresholds = thresholds;
                None
            }
            ChartMessage::AddThreshold(threshold) => {
                state.thresholds.push(threshold);
                None
            }
            ChartMessage::SetYRange(min, max) => {
                state.y_min = min;
                state.y_max = max;
                None
            }
            ChartMessage::SetYScale(scale) => {
                state.y_scale = scale;
                None
            }
            ChartMessage::SetVerticalLines(lines) => {
                state.vertical_lines = lines;
                None
            }
            ChartMessage::AddVerticalLine(line) => {
                state.vertical_lines.push(line);
                None
            }
            ChartMessage::ToggleCrosshair => {
                state.show_crosshair = !state.show_crosshair;
                if state.show_crosshair && state.cursor_position.is_none() {
                    state.cursor_position = Some(0);
                }
                Some(ChartOutput::CrosshairToggled(state.show_crosshair))
            }
            ChartMessage::ToggleGrid => {
                state.show_grid = !state.show_grid;
                Some(ChartOutput::GridToggled(state.show_grid))
            }
            ChartMessage::CursorLeft
            | ChartMessage::CursorRight
            | ChartMessage::CursorHome
            | ChartMessage::CursorEnd => {
                let max_idx = state
                    .series
                    .iter()
                    .map(|s| s.values().len())
                    .max()
                    .unwrap_or(1)
                    .saturating_sub(1);

                if max_idx == 0 {
                    return None;
                }

                let current = state.cursor_position.unwrap_or(0);

                let new_pos = match msg {
                    ChartMessage::CursorLeft => current.saturating_sub(1),
                    ChartMessage::CursorRight => (current + 1).min(max_idx),
                    ChartMessage::CursorHome => 0,
                    ChartMessage::CursorEnd => max_idx,
                    _ => unreachable!(),
                };

                if new_pos != current || state.cursor_position.is_none() {
                    state.cursor_position = Some(new_pos);
                    if !state.show_crosshair {
                        state.show_crosshair = true;
                    }
                    Some(ChartOutput::CursorMoved(new_pos))
                } else {
                    None
                }
            }
            ChartMessage::NextSeries | ChartMessage::PrevSeries => {
                if state.series.is_empty() {
                    return None;
                }

                let len = state.series.len();

                match msg {
                    ChartMessage::NextSeries => {
                        state.active_series = (state.active_series + 1) % len;
                        Some(ChartOutput::ActiveSeriesChanged(state.active_series))
                    }
                    ChartMessage::PrevSeries => {
                        state.active_series = if state.active_series == 0 {
                            len - 1
                        } else {
                            state.active_series - 1
                        };
                        Some(ChartOutput::ActiveSeriesChanged(state.active_series))
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if ctx.area.height < 3 || ctx.area.width < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::container("chart")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

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

        if inner.height == 0 || inner.width == 0 || state.series.is_empty() {
            return;
        }

        // Reserve space for title padding, legend, and axis labels
        let title_padding = if state.title.is_some() { 1u16 } else { 0 };

        let legend_entry_count =
            state.series.len() + state.thresholds.len() + state.vertical_lines.len();
        let legend_height = if state.show_legend && legend_entry_count > 1 {
            1u16
        } else {
            0
        };

        let x_label_height = if state.x_label.is_some() { 1u16 } else { 0 };

        let has_extras = title_padding + legend_height + x_label_height > 0;
        let chart_area = if has_extras {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(title_padding),
                    Constraint::Min(1),
                    Constraint::Length(legend_height),
                    Constraint::Length(x_label_height),
                ])
                .split(inner);

            // Render legend
            if legend_height > 0 {
                render::render_legend(state, ctx.frame, chunks[2]);
            }

            // Render x-axis label
            if x_label_height > 0 {
                if let Some(ref label) = state.x_label {
                    let p = Paragraph::new(label.as_str())
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::DarkGray));
                    ctx.frame.render_widget(p, chunks[3]);
                }
            }

            chunks[1]
        } else {
            inner
        };

        match state.kind {
            ChartKind::Line | ChartKind::Area | ChartKind::Scatter => {
                render::render_shared_axis_chart(
                    state,
                    ctx.frame,
                    chart_area,
                    ctx.theme,
                    ctx.focused,
                    ctx.disabled,
                );

                // Render crosshair value readout overlay
                if state.show_crosshair {
                    if let Some(pos) = state.cursor_position {
                        render::render_crosshair_readout(state, ctx.frame, chart_area, pos);
                    }
                }
            }
            ChartKind::BarVertical => render::render_bar_chart(
                state,
                ctx.frame,
                chart_area,
                ctx.theme,
                false,
                ctx.focused,
                ctx.disabled,
            ),
            ChartKind::BarHorizontal => render::render_bar_chart(
                state,
                ctx.frame,
                chart_area,
                ctx.theme,
                true,
                ctx.focused,
                ctx.disabled,
            ),
        }
    }
}

#[cfg(test)]
mod annotation_tests;
#[cfg(test)]
mod area_fill_tests;
#[cfg(test)]
mod enhancement_tests;
#[cfg(test)]
mod error_band_tests;
#[cfg(test)]
mod render_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod x_labels_tests;
