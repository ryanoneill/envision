//! Chart components for data visualization.
//!
//! Provides line charts (sparkline with labels) and bar charts
//! (horizontal/vertical) with data series, labels, colors, and
//! auto-scaling axes.
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
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph, Sparkline};

use super::Component;
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// A named data series with values and styling.
#[derive(Clone, Debug, PartialEq)]
pub struct DataSeries {
    /// The series label.
    label: String,
    /// The data values.
    values: Vec<f64>,
    /// The display color.
    color: Color,
}

impl DataSeries {
    /// Creates a new data series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("CPU", vec![10.0, 20.0, 30.0]);
    /// assert_eq!(series.label(), "CPU");
    /// assert_eq!(series.values(), &[10.0, 20.0, 30.0]);
    /// ```
    pub fn new(label: impl Into<String>, values: Vec<f64>) -> Self {
        Self {
            label: label.into(),
            values,
            color: Color::Cyan,
        }
    }

    /// Sets the color (builder pattern).
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns the label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the values.
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Returns the color.
    pub fn color(&self) -> Color {
        self.color
    }

    /// Appends a value.
    pub fn push(&mut self, value: f64) {
        self.values.push(value);
    }

    /// Appends a value, removing the oldest if over max length.
    pub fn push_bounded(&mut self, value: f64, max_len: usize) {
        self.values.push(value);
        while self.values.len() > max_len {
            self.values.remove(0);
        }
    }

    /// Returns the minimum value, or 0.0 if empty.
    pub fn min(&self) -> f64 {
        self.values
            .iter()
            .copied()
            .reduce(f64::min)
            .unwrap_or(0.0)
    }

    /// Returns the maximum value, or 0.0 if empty.
    pub fn max(&self) -> f64 {
        self.values
            .iter()
            .copied()
            .reduce(f64::max)
            .unwrap_or(0.0)
    }

    /// Returns the most recent value.
    pub fn last(&self) -> Option<f64> {
        self.values.last().copied()
    }

    /// Returns the number of data points.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if the series has no data points.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Clears all values.
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Sets the color.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

/// The kind of chart to display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChartKind {
    /// A line chart (sparkline-style).
    Line,
    /// A vertical bar chart.
    BarVertical,
    /// A horizontal bar chart.
    BarHorizontal,
}

/// Messages that can be sent to a Chart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChartMessage {
    /// Cycle to the next series (for multi-series line charts).
    NextSeries,
    /// Cycle to the previous series.
    PrevSeries,
}

/// Output messages from a Chart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChartOutput {
    /// The active series changed.
    ActiveSeriesChanged(usize),
}

/// State for a Chart component.
///
/// Contains the data series, chart kind, and display options.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartState {
    /// The data series to display.
    series: Vec<DataSeries>,
    /// The chart kind.
    kind: ChartKind,
    /// Index of the active (highlighted) series.
    active_series: usize,
    /// Optional title.
    title: Option<String>,
    /// X-axis label.
    x_label: Option<String>,
    /// Y-axis label.
    y_label: Option<String>,
    /// Whether to show the legend.
    show_legend: bool,
    /// Maximum data points to display (for line charts).
    max_display_points: usize,
    /// Bar width for bar charts.
    bar_width: u16,
    /// Bar gap for bar charts.
    bar_gap: u16,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
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

    /// Sets the title (builder pattern).
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the X-axis label (builder pattern).
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

    /// Returns the maximum display points.
    pub fn max_display_points(&self) -> usize {
        self.max_display_points
    }

    /// Returns the bar width.
    pub fn bar_width(&self) -> u16 {
        self.bar_width
    }

    /// Returns the bar gap.
    pub fn bar_gap(&self) -> u16 {
        self.bar_gap
    }

    /// Returns the number of series.
    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    /// Returns true if there are no series.
    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }

    /// Adds a series.
    pub fn add_series(&mut self, series: DataSeries) {
        self.series.push(series);
    }

    /// Clears all series.
    pub fn clear_series(&mut self) {
        self.series.clear();
        self.active_series = 0;
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
/// Supports line charts (sparkline-style), vertical bar charts, and
/// horizontal bar charts with multiple data series.
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
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if area.height < 3 || area.width < 3 {
            return;
        }

        let border_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
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
                render_legend(state, frame, chunks[1]);
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
            ChartKind::Line => render_line_chart(state, frame, chart_area, theme),
            ChartKind::BarVertical => render_bar_chart(state, frame, chart_area, theme, false),
            ChartKind::BarHorizontal => render_bar_chart(state, frame, chart_area, theme, true),
        }
    }
}

/// Renders the legend showing series labels and colors.
fn render_legend(state: &ChartState, frame: &mut Frame, area: Rect) {
    let spans: Vec<Span> = state
        .series
        .iter()
        .enumerate()
        .flat_map(|(i, s)| {
            let marker = if i == state.active_series {
                "●"
            } else {
                "○"
            };
            let separator = if i < state.series.len() - 1 {
                "  "
            } else {
                ""
            };
            vec![
                Span::styled(
                    format!("{} {}{}", marker, s.label(), separator),
                    Style::default().fg(s.color()),
                ),
            ]
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Renders a line chart using sparkline.
fn render_line_chart(
    state: &ChartState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    if state.series.is_empty() {
        return;
    }

    // Show y-axis labels on the left
    let y_label_width = if state.y_label.is_some() { 8u16 } else { 0 };

    let (y_area, chart_area) = if y_label_width > 0 {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(y_label_width), Constraint::Min(1)])
            .split(area);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, area)
    };

    // Render y-axis min/max labels
    if let Some(y_area) = y_area {
        let global_max = state.global_max();
        let global_min = state.global_min();
        let max_text = format!("{:.1}", global_max);
        let min_text = format!("{:.1}", global_min);

        if y_area.height >= 2 {
            let p_max = Paragraph::new(max_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Right);
            frame.render_widget(
                p_max,
                Rect::new(y_area.x, y_area.y, y_area.width, 1),
            );

            let p_min = Paragraph::new(min_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Right);
            frame.render_widget(
                p_min,
                Rect::new(
                    y_area.x,
                    y_area.y + y_area.height - 1,
                    y_area.width,
                    1,
                ),
            );
        }
    }

    // For multi-series, stack sparklines vertically
    if state.series.len() == 1 || chart_area.height < 2 {
        // Single series: full area sparkline
        let series = &state.series[state.active_series];
        let data = series_to_sparkline_data(series, state.max_display_points);
        let style = if state.disabled {
            theme.disabled_style()
        } else {
            Style::default().fg(series.color())
        };
        let sparkline = Sparkline::default().data(&data).style(style);
        frame.render_widget(sparkline, chart_area);
    } else {
        // Multi-series: divide height
        let count = state.series.len() as u16;
        let constraints: Vec<Constraint> = (0..count)
            .map(|_| Constraint::Ratio(1, count as u32))
            .collect();

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(chart_area);

        for (i, series) in state.series.iter().enumerate() {
            if let Some(sparkline_area) = areas.get(i) {
                let data = series_to_sparkline_data(series, state.max_display_points);
                let style = if state.disabled {
                    theme.disabled_style()
                } else if i == state.active_series {
                    Style::default()
                        .fg(series.color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(series.color())
                };
                let sparkline = Sparkline::default().data(&data).style(style);
                frame.render_widget(sparkline, *sparkline_area);
            }
        }
    }
}

/// Converts a data series to sparkline-compatible u64 data.
fn series_to_sparkline_data(series: &DataSeries, max_points: usize) -> Vec<u64> {
    let values = if series.values.len() > max_points {
        &series.values[series.values.len() - max_points..]
    } else {
        &series.values
    };

    if values.is_empty() {
        return Vec::new();
    }

    let min = values.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max = values.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let range = max - min;

    if range == 0.0 {
        return values.iter().map(|_| 50).collect();
    }

    values
        .iter()
        .map(|v| ((v - min) / range * 100.0) as u64)
        .collect()
}

/// Renders a bar chart.
fn render_bar_chart(
    state: &ChartState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    horizontal: bool,
) {
    if state.series.is_empty() {
        return;
    }

    // For bar charts, use the first series (or active series)
    let series = &state.series[state.active_series];
    if series.is_empty() {
        return;
    }

    let style = if state.disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(series.color())
    };

    // Create bars from the series values
    let bars: Vec<Bar> = series
        .values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let label = format!("{}", i + 1);
            Bar::default()
                .value(v.max(0.0) as u64)
                .label(Line::from(label))
                .style(style)
        })
        .collect();

    let group = BarGroup::default().bars(&bars);

    let mut bar_chart = BarChart::default()
        .data(group)
        .bar_width(state.bar_width)
        .bar_gap(state.bar_gap)
        .bar_style(style);

    if horizontal {
        bar_chart = bar_chart.direction(Direction::Horizontal);
    }

    frame.render_widget(bar_chart, area);
}

#[cfg(test)]
mod tests;
