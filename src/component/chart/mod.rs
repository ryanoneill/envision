//! Chart components for data visualization.
//!
//! Provides line charts (sparkline with labels), bar charts
//! (horizontal/vertical), area charts, and scatter plots with data series,
//! labels, colors, threshold lines, manual scaling, and auto-scaling axes.
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

use super::{Component, Disableable, ViewContext};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

mod render;
mod series;

/// A named data series with values and styling.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DataSeries {
    /// The series label.
    label: String,
    /// The data values.
    values: Vec<f64>,
    /// The display color.
    color: Color,
}

// DataSeries methods are in series.rs

/// The kind of chart to display.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ChartKind {
    /// A line chart (sparkline-style).
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
}

/// Output messages from a Chart.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
#[non_exhaustive]
pub enum ChartOutput {
    /// The active series changed.
    ActiveSeriesChanged(usize),
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
    /// Whether the component is focused.
    pub(crate) focused: bool,
    /// Whether the component is disabled.
    pub(crate) disabled: bool,
    /// Horizontal threshold/reference lines.
    pub(crate) thresholds: Vec<ThresholdLine>,
    /// Manual Y-axis minimum (None = auto from data).
    pub(crate) y_min: Option<f64>,
    /// Manual Y-axis maximum (None = auto from data).
    pub(crate) y_max: Option<f64>,
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
            max_display_points: 50,
            bar_width: 3,
            bar_gap: 1,
            focused: false,
            disabled: false,
            thresholds: Vec::new(),
            y_min: None,
            y_max: None,
        }
    }
}

impl ChartState {
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
            series,
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
            series,
            kind: ChartKind::BarVertical,
            ..Default::default()
        }
    }

    /// Creates a horizontal bar chart state.
    pub fn bar_horizontal(series: Vec<DataSeries>) -> Self {
        Self {
            series,
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
            series,
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
            series,
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

    /// Sets the disabled state (builder pattern).
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    // ---- Accessors ----

    /// Returns the data series.
    pub fn series(&self) -> &[DataSeries] {
        &self.series
    }

    /// Returns a mutable reference to the series.
    pub fn series_mut(&mut self) -> &mut [DataSeries] {
        &mut self.series
    }

    /// Returns the series at the given index.
    pub fn get_series(&self, index: usize) -> Option<&DataSeries> {
        self.series.get(index)
    }

    /// Returns a mutable reference to the series at the given index.
    pub fn get_series_mut(&mut self, index: usize) -> Option<&mut DataSeries> {
        self.series.get_mut(index)
    }

    /// Returns the chart kind.
    pub fn kind(&self) -> &ChartKind {
        &self.kind
    }

    /// Sets the chart kind.
    pub fn set_kind(&mut self, kind: ChartKind) {
        self.kind = kind;
    }

    /// Returns the active series index.
    pub fn active_series(&self) -> usize {
        self.active_series
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns the X-axis label.
    pub fn x_label(&self) -> Option<&str> {
        self.x_label.as_deref()
    }

    /// Returns the Y-axis label.
    pub fn y_label(&self) -> Option<&str> {
        self.y_label.as_deref()
    }

    /// Returns whether the legend is shown.
    pub fn show_legend(&self) -> bool {
        self.show_legend
    }

    /// Sets whether to show the legend.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let mut state = ChartState::line(vec![]);
    /// state.set_show_legend(true);
    /// assert!(state.show_legend());
    /// ```
    pub fn set_show_legend(&mut self, show: bool) {
        self.show_legend = show;
    }

    /// Returns the maximum display points.
    pub fn max_display_points(&self) -> usize {
        self.max_display_points
    }

    /// Sets the maximum display points for line charts.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let mut state = ChartState::line(vec![]);
    /// state.set_max_display_points(200);
    /// assert_eq!(state.max_display_points(), 200);
    /// ```
    pub fn set_max_display_points(&mut self, max: usize) {
        self.max_display_points = max;
    }

    /// Returns the bar width.
    pub fn bar_width(&self) -> u16 {
        self.bar_width
    }

    /// Sets the bar width.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let mut state = ChartState::bar_vertical(vec![]);
    /// state.set_bar_width(3);
    /// assert_eq!(state.bar_width(), 3);
    /// ```
    pub fn set_bar_width(&mut self, width: u16) {
        self.bar_width = width.max(1);
    }

    /// Returns the bar gap.
    pub fn bar_gap(&self) -> u16 {
        self.bar_gap
    }

    /// Sets the bar gap.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let mut state = ChartState::bar_vertical(vec![]);
    /// state.set_bar_gap(2);
    /// assert_eq!(state.bar_gap(), 2);
    /// ```
    pub fn set_bar_gap(&mut self, gap: u16) {
        self.bar_gap = gap;
    }

    /// Returns the number of series.
    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    /// Returns true if there are no series.
    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }

    /// Returns the threshold lines.
    pub fn thresholds(&self) -> &[ThresholdLine] {
        &self.thresholds
    }

    /// Returns the manual Y-axis minimum, if set.
    pub fn y_min(&self) -> Option<f64> {
        self.y_min
    }

    /// Returns the manual Y-axis maximum, if set.
    pub fn y_max(&self) -> Option<f64> {
        self.y_max
    }

    /// Adds a series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let mut state = ChartState::line(vec![]);
    /// state.add_series(DataSeries::new("CPU", vec![50.0]));
    /// assert_eq!(state.series_count(), 1);
    /// ```
    pub fn add_series(&mut self, series: DataSeries) {
        self.series.push(series);
    }

    /// Clears all series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let mut state = ChartState::line(vec![
    ///     DataSeries::new("A", vec![1.0]),
    /// ]);
    /// state.clear_series();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear_series(&mut self) {
        self.series.clear();
        self.active_series = 0;
    }

    /// Adds a threshold line.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries, ThresholdLine};
    /// use ratatui::style::Color;
    ///
    /// let mut state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])]);
    /// state.add_threshold(ThresholdLine::new(90.0, "Warning", Color::Yellow));
    /// assert_eq!(state.thresholds().len(), 1);
    /// ```
    pub fn add_threshold(&mut self, threshold: ThresholdLine) {
        self.thresholds.push(threshold);
    }

    /// Clears all threshold lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let mut state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_threshold(90.0, "Warning", Color::Yellow);
    /// state.clear_thresholds();
    /// assert!(state.thresholds().is_empty());
    /// ```
    pub fn clear_thresholds(&mut self) {
        self.thresholds.clear();
    }

    /// Sets the manual Y-axis range.
    ///
    /// Pass `None` for either bound to fall back to auto-scaling from data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let mut state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])]);
    /// state.set_y_range(Some(0.0), Some(100.0));
    /// assert_eq!(state.y_min(), Some(0.0));
    /// assert_eq!(state.y_max(), Some(100.0));
    /// ```
    pub fn set_y_range(&mut self, min: Option<f64>, max: Option<f64>) {
        self.y_min = min;
        self.y_max = max;
    }

    /// Computes the global min value across all series.
    pub fn global_min(&self) -> f64 {
        self.series
            .iter()
            .map(|s| s.min())
            .reduce(f64::min)
            .unwrap_or(0.0)
    }

    /// Computes the global max value across all series.
    pub fn global_max(&self) -> f64 {
        self.series
            .iter()
            .map(|s| s.max())
            .reduce(f64::max)
            .unwrap_or(0.0)
    }

    /// Computes the effective minimum for the Y-axis, considering manual override.
    ///
    /// If `y_min` is set, uses that value. Otherwise auto-scales from data,
    /// also considering threshold line values.
    pub fn effective_min(&self) -> f64 {
        self.y_min.unwrap_or_else(|| {
            let data_min = self.global_min();
            let threshold_min = self
                .thresholds
                .iter()
                .map(|t| t.value)
                .reduce(f64::min)
                .unwrap_or(data_min);
            f64::min(data_min, threshold_min)
        })
    }

    /// Computes the effective maximum for the Y-axis, considering manual override.
    ///
    /// If `y_max` is set, uses that value. Otherwise auto-scales from data,
    /// also considering threshold line values.
    pub fn effective_max(&self) -> f64 {
        self.y_max.unwrap_or_else(|| {
            let data_max = self.global_max();
            let threshold_max = self
                .thresholds
                .iter()
                .map(|t| t.value)
                .reduce(f64::max)
                .unwrap_or(data_max);
            f64::max(data_max, threshold_max)
        })
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

    /// Maps an input event to a chart message.
    pub fn handle_event(&self, event: &Event) -> Option<ChartMessage> {
        Chart::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<ChartOutput> {
        Chart::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: ChartMessage) -> Option<ChartOutput> {
        Chart::update(self, msg)
    }
}

/// A chart component for data visualization.
///
/// Supports line charts (sparkline-style), vertical bar charts, horizontal
/// bar charts, area charts (filled line), and scatter plots with multiple
/// data series, threshold lines, and manual Y-axis scaling.
///
/// # Key Bindings
///
/// - `Tab` — Cycle to next series
/// - `BackTab` — Cycle to previous series
pub struct Chart(PhantomData<()>);

impl Component for Chart {
    type State = ChartState;
    type Message = ChartMessage;
    type Output = ChartOutput;

    fn init() -> Self::State {
        ChartState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            KeyCode::Tab => Some(ChartMessage::NextSeries),
            KeyCode::BackTab => Some(ChartMessage::PrevSeries),
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
            ChartMessage::NextSeries | ChartMessage::PrevSeries => {
                if state.disabled || state.series.is_empty() {
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if area.height < 3 || area.width < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::container("chart")
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

        if inner.height == 0 || inner.width == 0 || state.series.is_empty() {
            return;
        }

        // Reserve space for legend and axis labels
        let legend_height = if state.show_legend && state.series.len() > 1 {
            1u16
        } else {
            0
        };

        let x_label_height = if state.x_label.is_some() { 1u16 } else { 0 };

        let chart_area = if legend_height + x_label_height > 0 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(legend_height),
                    Constraint::Length(x_label_height),
                ])
                .split(inner);

            // Render legend
            if legend_height > 0 {
                render::render_legend(state, frame, chunks[1]);
            }

            // Render x-axis label
            if x_label_height > 0 {
                if let Some(ref label) = state.x_label {
                    let p = Paragraph::new(label.as_str())
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(Color::DarkGray));
                    frame.render_widget(p, chunks[2]);
                }
            }

            chunks[0]
        } else {
            inner
        };

        match state.kind {
            ChartKind::Line => render::render_line_chart(
                state,
                frame,
                chart_area,
                theme,
                ctx.focused,
                ctx.disabled,
            ),
            ChartKind::BarVertical => render::render_bar_chart(
                state,
                frame,
                chart_area,
                theme,
                false,
                ctx.focused,
                ctx.disabled,
            ),
            ChartKind::BarHorizontal => render::render_bar_chart(
                state,
                frame,
                chart_area,
                theme,
                true,
                ctx.focused,
                ctx.disabled,
            ),
            ChartKind::Area | ChartKind::Scatter => render::render_shared_axis_chart(
                state,
                frame,
                chart_area,
                theme,
                ctx.focused,
                ctx.disabled,
            ),
        }
    }
}

impl Disableable for Chart {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod enhancement_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
