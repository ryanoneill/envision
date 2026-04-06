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

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders};

use super::{Component, ViewContext};
use crate::input::Event;
use crate::theme::Theme;

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

impl HistogramState {
    /// Creates an empty histogram state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new();
    /// assert!(state.data().is_empty());
    /// assert_eq!(state.bin_count(), 10);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a histogram state with initial data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::with_data(vec![1.0, 2.0, 3.0]);
    /// assert_eq!(state.data().len(), 3);
    /// ```
    pub fn with_data(data: Vec<f64>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    /// Sets the number of bins (builder pattern).
    ///
    /// A bin count of 0 is treated as 1.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_bin_count(20);
    /// assert_eq!(state.bin_count(), 20);
    /// ```
    pub fn with_bin_count(mut self, count: usize) -> Self {
        self.bin_method = BinMethod::Fixed(count.max(1));
        self
    }

    /// Sets the binning strategy (builder pattern).
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
    pub fn with_bin_method(mut self, method: BinMethod) -> Self {
        self.bin_method = method;
        self
    }

    /// Sets the manual range (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_range(0.0, 100.0);
    /// assert_eq!(state.effective_min(), 0.0);
    /// assert_eq!(state.effective_max(), 100.0);
    /// ```
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min_value = Some(min);
        self.max_value = Some(max);
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_title("Response Times");
    /// assert_eq!(state.title(), Some("Response Times"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the x-axis label (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_x_label("Latency (ms)");
    /// assert_eq!(state.x_label(), Some("Latency (ms)"));
    /// ```
    pub fn with_x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = Some(label.into());
        self
    }

    /// Sets the y-axis label (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_y_label("Frequency");
    /// assert_eq!(state.y_label(), Some("Frequency"));
    /// ```
    pub fn with_y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Sets the bar color (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    /// use ratatui::style::Color;
    ///
    /// let state = HistogramState::new().with_color(Color::Cyan);
    /// assert_eq!(state.color(), Some(Color::Cyan));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets whether to show count labels on bars (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_show_counts(true);
    /// assert!(state.show_counts());
    /// ```
    pub fn with_show_counts(mut self, show: bool) -> Self {
        self.show_counts = show;
        self
    }

    // ---- Accessors ----

    /// Returns the raw data points.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::with_data(vec![1.0, 2.0, 3.0]);
    /// assert_eq!(state.data(), &[1.0, 2.0, 3.0]);
    /// ```
    pub fn data(&self) -> &[f64] {
        &self.data
    }

    /// Returns a mutable reference to the raw data points.
    ///
    /// This is safe because the histogram has no derived indices or
    /// filter state; bins are recomputed on each render.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let mut state = HistogramState::with_data(vec![1.0, 2.0, 3.0]);
    /// state.data_mut().push(4.0);
    /// assert_eq!(state.data().len(), 4);
    /// ```
    pub fn data_mut(&mut self) -> &mut Vec<f64> {
        &mut self.data
    }

    /// Adds a single data point.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let mut state = HistogramState::new();
    /// state.push(42.0);
    /// assert_eq!(state.data(), &[42.0]);
    /// ```
    pub fn push(&mut self, value: f64) {
        self.data.push(value);
    }

    /// Adds multiple data points.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let mut state = HistogramState::new();
    /// state.push_batch(&[1.0, 2.0, 3.0]);
    /// assert_eq!(state.data().len(), 3);
    /// ```
    pub fn push_batch(&mut self, values: &[f64]) {
        self.data.extend_from_slice(values);
    }

    /// Clears all data points.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let mut state = HistogramState::with_data(vec![1.0, 2.0]);
    /// state.clear();
    /// assert!(state.data().is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns the effective number of bins.
    pub fn bin_count(&self) -> usize {
        self.bin_method.compute_bin_count(&self.data)
    }

    /// Sets the number of bins (convenience, sets `Fixed` method).
    pub fn set_bin_count(&mut self, count: usize) {
        self.bin_method = BinMethod::Fixed(count.max(1));
    }

    /// Returns the current binning method.
    pub fn bin_method(&self) -> &BinMethod {
        &self.bin_method
    }

    /// Sets the binning method.
    pub fn set_bin_method(&mut self, method: BinMethod) {
        self.bin_method = method;
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let mut state = HistogramState::new();
    /// state.set_title("Response Times");
    /// assert_eq!(state.title(), Some("Response Times"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns the x-axis label.
    pub fn x_label(&self) -> Option<&str> {
        self.x_label.as_deref()
    }

    /// Returns the y-axis label.
    pub fn y_label(&self) -> Option<&str> {
        self.y_label.as_deref()
    }

    /// Returns the bar color.
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Sets the bar color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    /// use ratatui::style::Color;
    ///
    /// let mut state = HistogramState::new();
    /// state.set_color(Some(Color::Blue));
    /// assert_eq!(state.color(), Some(Color::Blue));
    /// ```
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    /// Returns whether count labels are shown on bars.
    pub fn show_counts(&self) -> bool {
        self.show_counts
    }

    /// Sets whether count labels are shown on bars.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let mut state = HistogramState::new();
    /// state.set_show_counts(true);
    /// assert!(state.show_counts());
    /// ```
    pub fn set_show_counts(&mut self, show: bool) {
        self.show_counts = show;
    }

    /// Returns the effective minimum value.
    ///
    /// Uses the manual minimum if set, otherwise auto-computes from data.
    /// Returns 0.0 for empty data with no manual minimum.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::with_data(vec![5.0, 10.0, 15.0]);
    /// assert_eq!(state.effective_min(), 5.0);
    ///
    /// let state = HistogramState::with_data(vec![5.0, 10.0]).with_range(0.0, 20.0);
    /// assert_eq!(state.effective_min(), 0.0);
    /// ```
    pub fn effective_min(&self) -> f64 {
        self.min_value
            .unwrap_or_else(|| self.data.iter().copied().reduce(f64::min).unwrap_or(0.0))
    }

    /// Returns the effective maximum value.
    ///
    /// Uses the manual maximum if set, otherwise auto-computes from data.
    /// Returns 0.0 for empty data with no manual maximum.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::with_data(vec![5.0, 10.0, 15.0]);
    /// assert_eq!(state.effective_max(), 15.0);
    ///
    /// let state = HistogramState::with_data(vec![5.0, 10.0]).with_range(0.0, 20.0);
    /// assert_eq!(state.effective_max(), 20.0);
    /// ```
    pub fn effective_max(&self) -> f64 {
        self.max_value
            .unwrap_or_else(|| self.data.iter().copied().reduce(f64::max).unwrap_or(0.0))
    }

    /// Computes the bin edges and frequency counts.
    ///
    /// Returns a vector of `(bin_start, bin_end, count)` tuples, one for each
    /// bin. Bins are evenly spaced from `effective_min()` to `effective_max()`.
    ///
    /// When all data has the same value (range is zero), a single bin is
    /// created spanning `[value - 0.5, value + 0.5)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::with_data(vec![1.0, 2.0, 3.0, 4.0, 5.0])
    ///     .with_bin_count(5)
    ///     .with_range(1.0, 5.0);
    /// let bins = state.compute_bins();
    /// assert_eq!(bins.len(), 5);
    /// // Each bin should have a count
    /// let total: usize = bins.iter().map(|(_, _, c)| c).sum();
    /// assert_eq!(total, 5);
    /// ```
    pub fn compute_bins(&self) -> Vec<(f64, f64, usize)> {
        let bin_count = self.bin_count().max(1);

        if self.data.is_empty() {
            let min = self.effective_min();
            let max = self.effective_max();

            if (max - min).abs() < f64::EPSILON {
                // Zero-range: create bins around the single value
                return vec![(min - 0.5, min + 0.5, 0); bin_count];
            }

            let bin_width = (max - min) / bin_count as f64;
            return (0..bin_count)
                .map(|i| {
                    let start = min + i as f64 * bin_width;
                    let end = min + (i + 1) as f64 * bin_width;
                    (start, end, 0)
                })
                .collect();
        }

        let min = self.effective_min();
        let max = self.effective_max();

        // Handle zero range (all values are the same)
        if (max - min).abs() < f64::EPSILON {
            return vec![(min - 0.5, min + 0.5, self.data.len()); 1];
        }

        let bin_width = (max - min) / bin_count as f64;

        let mut counts = vec![0usize; bin_count];

        for &value in &self.data {
            let bin_index = ((value - min) / bin_width).floor() as usize;
            // Clamp to valid range; the max value falls into the last bin
            let bin_index = bin_index.min(bin_count - 1);
            counts[bin_index] += 1;
        }

        (0..bin_count)
            .map(|i| {
                let start = min + i as f64 * bin_width;
                let end = min + (i + 1) as f64 * bin_width;
                (start, end, counts[i])
            })
            .collect()
    }

    // ---- Focus / Disabled ----

    // ---- Instance methods ----

    /// Maps an input event to a histogram message.
    pub fn handle_event(&self, event: &Event) -> Option<HistogramMessage> {
        Histogram::handle_event(self, event, &ViewContext::default())
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<()> {
        Histogram::dispatch_event(self, event, &ViewContext::default())
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: HistogramMessage) -> Option<()> {
        Histogram::update(self, msg)
    }
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
        _ctx: &ViewContext,
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if area.height < 3 || area.width < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::container("histogram")
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
                frame.render_widget(p, y_area);
            }
        }

        // Render x-axis label below the chart
        if let Some(x_area) = x_label_area {
            if let Some(ref label) = state.x_label {
                let p = ratatui::widgets::Paragraph::new(label.as_str())
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::DarkGray));
                frame.render_widget(p, x_area);
            }
        }

        // Compute bins and render bar chart
        let bins = state.compute_bins();
        let max_count = bins.iter().map(|(_, _, c)| *c).max().unwrap_or(0);

        let bar_color = state.color.unwrap_or(Color::Cyan);
        let bar_style = if ctx.disabled {
            theme.disabled_style()
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

        frame.render_widget(chart, chart_area);
    }
}

#[cfg(test)]
mod tests;
