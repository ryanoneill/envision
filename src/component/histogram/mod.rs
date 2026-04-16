//! Histogram component for frequency distribution visualization.
//!
//! Takes raw continuous data, automatically bins it, and displays the
//! frequency distribution as vertical bars using ratatui's `BarChart`
//! widget.
//!
//! # Adaptive Binning
//!
//! By default, the histogram uses a fixed bin count (10). You can choose
//! an adaptive binning method that computes the optimal number of bins
//! based on the data:
//!
//! - [`BinMethod::Fixed`] — a user-specified number of bins (default: 10).
//! - [`BinMethod::Sturges`] — `ceil(log2(n) + 1)`, good for roughly normal data.
//! - [`BinMethod::SquareRoot`] — `ceil(sqrt(n))`, a simple rule of thumb.
//! - [`BinMethod::Scott`] — `ceil(range / (3.49 * std * n^(-1/3)))`, optimal for normal data.
//! - [`BinMethod::FreedmanDiaconis`] — `ceil(range / (2 * IQR * n^(-1/3)))`, robust to outliers.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Histogram, HistogramState};
//!
//! let state = HistogramState::with_data(vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0]);
//! assert_eq!(state.data().len(), 6);
//! assert_eq!(state.bin_count(), 10);
//! ```

mod state;

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders};

use super::{Component, EventContext, RenderContext};
use crate::input::Event;

/// Strategy for computing the number of histogram bins.
///
/// The default is `Fixed(10)`, which uses a static bin count. Adaptive
/// methods compute the bin count from the data at render time so the
/// histogram automatically adjusts as data changes.
///
/// All adaptive methods clamp the result to the range `[1, 200]`.
///
/// # Example
///
/// ```rust
/// use envision::component::{BinMethod, HistogramState};
///
/// let state = HistogramState::with_data(vec![1.0, 2.0, 3.0, 4.0])
///     .with_bin_method(BinMethod::Sturges);
/// assert_eq!(state.bin_method(), &BinMethod::Sturges);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum BinMethod {
    /// A fixed, user-specified number of bins.
    Fixed(usize),
    /// Freedman-Diaconis rule: `width = 2 * IQR * n^(-1/3)`, `bins = ceil(range / width)`.
    ///
    /// Robust to outliers because it uses the interquartile range.
    FreedmanDiaconis,
    /// Sturges' formula: `ceil(log2(n) + 1)`.
    ///
    /// Works well for roughly normal data but can undercount bins for
    /// large datasets.
    Sturges,
    /// Scott's normal reference rule: `width = 3.49 * std * n^(-1/3)`,
    /// `bins = ceil(range / width)`.
    ///
    /// Optimal for data drawn from a normal distribution.
    Scott,
    /// Square-root rule: `ceil(sqrt(n))`.
    ///
    /// A simple rule of thumb used in many applications.
    SquareRoot,
}

impl Default for BinMethod {
    fn default() -> Self {
        BinMethod::Fixed(10)
    }
}

/// The minimum number of bins any adaptive method can produce.
const MIN_BINS: usize = 1;

/// The maximum number of bins any adaptive method can produce.
const MAX_BINS: usize = 200;

impl BinMethod {
    /// Computes the effective bin count for the given data.
    ///
    /// For `Fixed(n)`, the value is returned directly (clamped to at least 1).
    /// For adaptive methods, the algorithm inspects the data and clamps the
    /// result to `[1, 200]`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BinMethod;
    ///
    /// let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
    /// assert_eq!(BinMethod::SquareRoot.compute_bin_count(&data), 10);
    /// assert_eq!(BinMethod::Sturges.compute_bin_count(&data), 8);
    /// ```
    pub fn compute_bin_count(&self, data: &[f64]) -> usize {
        match self {
            BinMethod::Fixed(n) => (*n).max(1),
            BinMethod::Sturges => Self::sturges(data),
            BinMethod::SquareRoot => Self::square_root(data),
            BinMethod::Scott => Self::scott(data),
            BinMethod::FreedmanDiaconis => Self::freedman_diaconis(data),
        }
    }

    fn sturges(data: &[f64]) -> usize {
        if data.is_empty() {
            return MIN_BINS;
        }
        let n = data.len() as f64;
        let bins = (n.log2() + 1.0).ceil() as usize;
        bins.clamp(MIN_BINS, MAX_BINS)
    }

    fn square_root(data: &[f64]) -> usize {
        if data.is_empty() {
            return MIN_BINS;
        }
        let n = data.len() as f64;
        let bins = n.sqrt().ceil() as usize;
        bins.clamp(MIN_BINS, MAX_BINS)
    }

    fn scott(data: &[f64]) -> usize {
        if data.is_empty() {
            return MIN_BINS;
        }
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        let std = variance.sqrt();
        if std < f64::EPSILON {
            return MIN_BINS;
        }
        let min = data.iter().copied().reduce(f64::min).unwrap_or(0.0);
        let max = data.iter().copied().reduce(f64::max).unwrap_or(0.0);
        let range = max - min;
        if range < f64::EPSILON {
            return MIN_BINS;
        }
        let width = 3.49 * std * n.powf(-1.0 / 3.0);
        if width < f64::EPSILON {
            return MIN_BINS;
        }
        let bins = (range / width).ceil() as usize;
        bins.clamp(MIN_BINS, MAX_BINS)
    }

    fn freedman_diaconis(data: &[f64]) -> usize {
        if data.is_empty() {
            return MIN_BINS;
        }
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = sorted.len();
        let q1 = sorted[n / 4];
        let q3 = sorted[3 * n / 4];
        let iqr = q3 - q1;
        if iqr < f64::EPSILON {
            return MIN_BINS;
        }
        let min = sorted[0];
        let max = sorted[n - 1];
        let range = max - min;
        if range < f64::EPSILON {
            return MIN_BINS;
        }
        let width = 2.0 * iqr * (n as f64).powf(-1.0 / 3.0);
        if width < f64::EPSILON {
            return MIN_BINS;
        }
        let bins = (range / width).ceil() as usize;
        bins.clamp(MIN_BINS, MAX_BINS)
    }
}

/// State for a Histogram component.
///
/// Contains raw data points and configuration for binning and display.
///
/// # Example
///
/// ```rust
/// use envision::component::HistogramState;
///
/// let state = HistogramState::with_data(vec![10.0, 20.0, 30.0])
///     .with_bin_count(5)
///     .with_title("Latency Distribution");
/// assert_eq!(state.data().len(), 3);
/// assert_eq!(state.bin_count(), 5);
/// assert_eq!(state.title(), Some("Latency Distribution"));
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct HistogramState {
    /// Raw data points.
    data: Vec<f64>,
    /// Binning strategy (default: Fixed(10)).
    bin_method: BinMethod,
    /// Manual minimum value (None = auto from data).
    min_value: Option<f64>,
    /// Manual maximum value (None = auto from data).
    max_value: Option<f64>,
    /// Optional title.
    title: Option<String>,
    /// X-axis label.
    x_label: Option<String>,
    /// Y-axis label.
    y_label: Option<String>,
    /// Bar color.
    color: Option<Color>,
    /// Whether to show count labels on bars.
    show_counts: bool,
}

/// Messages that can be sent to a Histogram.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum HistogramMessage {
    /// Replace all data points.
    SetData(Vec<f64>),
    /// Add a single data point.
    PushData(f64),
    /// Add multiple data points.
    PushDataBatch(Vec<f64>),
    /// Clear all data.
    Clear,
    /// Change the number of bins (sets bin method to `Fixed`).
    SetBinCount(usize),
    /// Change the binning strategy.
    SetBinMethod(BinMethod),
    /// Set the manual min/max range.
    SetRange(Option<f64>, Option<f64>),
}

/// A histogram component for frequency distribution visualization.
///
/// Takes raw continuous data, automatically bins it, and renders the
/// frequency distribution as vertical bars.
///
/// This is a display-only component. It does not handle keyboard events.
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, Histogram, HistogramState};
///
/// let state = HistogramState::with_data(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5])
///     .with_bin_count(3)
///     .with_title("Value Distribution");
/// let bins = state.compute_bins();
/// assert_eq!(bins.len(), 3);
/// ```
pub struct Histogram(PhantomData<()>);

impl Component for Histogram {
    type State = HistogramState;
    type Message = HistogramMessage;
    type Output = ();

    fn init() -> Self::State {
        HistogramState::default()
    }

    fn handle_event(
        _state: &Self::State,
        _event: &Event,
        _ctx: &EventContext,
    ) -> Option<Self::Message> {
        // Display-only component; no event handling.
        None
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            HistogramMessage::SetData(data) => {
                state.data = data;
            }
            HistogramMessage::PushData(value) => {
                state.data.push(value);
            }
            HistogramMessage::PushDataBatch(values) => {
                state.data.extend(values);
            }
            HistogramMessage::Clear => {
                state.data.clear();
            }
            HistogramMessage::SetBinCount(count) => {
                state.bin_method = BinMethod::Fixed(count.max(1));
            }
            HistogramMessage::SetBinMethod(method) => {
                state.bin_method = method;
            }
            HistogramMessage::SetRange(min, max) => {
                state.min_value = min;
                state.max_value = max;
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
                crate::annotation::Annotation::container("histogram")
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

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Reserve space for axis labels
        let x_label_height = if state.x_label.is_some() { 1u16 } else { 0 };
        let y_label_height = if state.y_label.is_some() { 1u16 } else { 0 };

        let (chart_area, x_label_area, y_label_area) = if x_label_height > 0 || y_label_height > 0 {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(y_label_height),
                    Constraint::Min(1),
                    Constraint::Length(x_label_height),
                ])
                .split(inner);
            (
                chunks[1],
                if x_label_height > 0 {
                    Some(chunks[2])
                } else {
                    None
                },
                if y_label_height > 0 {
                    Some(chunks[0])
                } else {
                    None
                },
            )
        } else {
            (inner, None, None)
        };

        // Render y-axis label above the chart
        if let Some(y_area) = y_label_area {
            if let Some(ref label) = state.y_label {
                let p = ratatui::widgets::Paragraph::new(label.as_str())
                    .alignment(Alignment::Left)
                    .style(Style::default().fg(Color::DarkGray));
                ctx.frame.render_widget(p, y_area);
            }
        }

        // Render x-axis label below the chart
        if let Some(x_area) = x_label_area {
            if let Some(ref label) = state.x_label {
                let p = ratatui::widgets::Paragraph::new(label.as_str())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::DarkGray));
                ctx.frame.render_widget(p, x_area);
            }
        }

        // Compute bins and render bar chart
        let bins = state.compute_bins();
        let max_count = bins.iter().map(|(_, _, c)| *c).max().unwrap_or(0);

        let bar_color = state.color.unwrap_or(Color::Cyan);
        let bar_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else {
            Style::default().fg(bar_color)
        };

        let bars: Vec<Bar> = bins
            .iter()
            .map(|(start, end, count)| {
                let label = format!("{:.0}", (start + end) / 2.0);
                let mut bar = Bar::default()
                    .value(*count as u64)
                    .label(Line::from(label))
                    .style(bar_style);
                if state.show_counts {
                    bar = bar.text_value(format!("{}", count));
                }
                bar
            })
            .collect();

        let bar_group = BarGroup::default().bars(&bars);

        // Calculate bar width based on available space
        let bin_count = bins.len() as u16;
        let bar_width = if bin_count > 0 {
            // Each bar needs bar_width + gap (1). Total = bin_count * (bar_width + 1) - 1
            // Solve for bar_width: bar_width = (available + 1) / bin_count - 1
            let available = chart_area.width;
            let width = (available.saturating_add(1)) / bin_count.max(1);
            width.saturating_sub(1).max(1)
        } else {
            1
        };

        let chart = BarChart::default()
            .data(bar_group)
            .bar_width(bar_width)
            .bar_gap(1)
            .bar_style(bar_style)
            .max(max_count as u64);

        ctx.frame.render_widget(chart, chart_area);
    }
}

#[cfg(test)]
mod tests;
