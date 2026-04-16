//! HistogramState constructors, builders, accessors, setters, and instance methods.
//!
//! Extracted from the main histogram module to keep file sizes manageable.

use ratatui::style::Color;

use super::{BinMethod, EventContext, Histogram, HistogramMessage, HistogramState};
use crate::component::Component;
use crate::input::Event;

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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_bin_count(15);
    /// assert_eq!(state.bin_count(), 15);
    /// ```
    pub fn bin_count(&self) -> usize {
        self.bin_method.compute_bin_count(&self.data)
    }

    /// Sets the number of bins (convenience, sets `Fixed` method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let mut state = HistogramState::new();
    /// state.set_bin_count(25);
    /// assert_eq!(state.bin_count(), 25);
    /// ```
    pub fn set_bin_count(&mut self, count: usize) {
        self.bin_method = BinMethod::Fixed(count.max(1));
    }

    /// Returns the current binning method.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BinMethod, HistogramState};
    ///
    /// let state = HistogramState::new();
    /// assert_eq!(state.bin_method(), &BinMethod::Fixed(10));
    /// ```
    pub fn bin_method(&self) -> &BinMethod {
        &self.bin_method
    }

    /// Sets the binning method.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BinMethod, HistogramState};
    ///
    /// let mut state = HistogramState::new();
    /// state.set_bin_method(BinMethod::Sturges);
    /// assert_eq!(state.bin_method(), &BinMethod::Sturges);
    /// ```
    pub fn set_bin_method(&mut self, method: BinMethod) {
        self.bin_method = method;
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_title("Distribution");
    /// assert_eq!(state.title(), Some("Distribution"));
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_x_label("Value");
    /// assert_eq!(state.x_label(), Some("Value"));
    /// ```
    pub fn x_label(&self) -> Option<&str> {
        self.x_label.as_deref()
    }

    /// Returns the y-axis label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_y_label("Count");
    /// assert_eq!(state.y_label(), Some("Count"));
    /// ```
    pub fn y_label(&self) -> Option<&str> {
        self.y_label.as_deref()
    }

    /// Returns the bar color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    /// use ratatui::style::Color;
    ///
    /// let state = HistogramState::new().with_color(Color::Green);
    /// assert_eq!(state.color(), Some(Color::Green));
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    ///
    /// let state = HistogramState::new().with_show_counts(true);
    /// assert!(state.show_counts());
    /// ```
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
    ///
    /// The histogram is display-only; this always returns `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    /// use envision::input::Event;
    ///
    /// let state = HistogramState::new();
    /// assert!(state.handle_event(&Event::key(envision::input::Key::Enter)).is_none());
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<HistogramMessage> {
        Histogram::handle_event(self, event, &EventContext::default())
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// The histogram is display-only; this always returns `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::HistogramState;
    /// use envision::input::Event;
    ///
    /// let mut state = HistogramState::new();
    /// assert!(state.dispatch_event(&Event::key(envision::input::Key::Enter)).is_none());
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<()> {
        Histogram::dispatch_event(self, event, &EventContext::default())
    }

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{HistogramState, HistogramMessage};
    ///
    /// let mut state = HistogramState::new();
    /// state.update(HistogramMessage::PushData(42.0));
    /// assert_eq!(state.data(), &[42.0]);
    /// ```
    pub fn update(&mut self, msg: HistogramMessage) -> Option<()> {
        Histogram::update(self, msg)
    }
}
