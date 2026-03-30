//! DataSeries implementation methods.
//!
//! Extracted from the main chart module to keep file sizes manageable.
//! Contains the builder, accessor, and mutation methods for [`DataSeries`].

use ratatui::style::Color;

use super::DataSeries;

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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    /// use ratatui::style::Color;
    ///
    /// let series = DataSeries::new("CPU", vec![1.0, 2.0])
    ///     .with_color(Color::Red);
    /// assert_eq!(series.color(), Color::Red);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let mut series = DataSeries::new("Temp", vec![20.0]);
    /// series.push(25.0);
    /// assert_eq!(series.len(), 2);
    /// assert_eq!(series.values(), &[20.0, 25.0]);
    /// ```
    pub fn push(&mut self, value: f64) {
        self.values.push(value);
    }

    /// Appends a value, removing the oldest if over max length.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let mut series = DataSeries::new("Temp", vec![1.0, 2.0, 3.0]);
    /// series.push_bounded(4.0, 3);
    /// assert_eq!(series.values(), &[2.0, 3.0, 4.0]);
    /// ```
    pub fn push_bounded(&mut self, value: f64, max_len: usize) {
        self.values.push(value);
        while self.values.len() > max_len {
            self.values.remove(0);
        }
    }

    /// Returns the minimum value, or 0.0 if empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("Temp", vec![15.0, 22.0, 8.0]);
    /// assert_eq!(series.min(), 8.0);
    /// ```
    pub fn min(&self) -> f64 {
        self.values.iter().copied().reduce(f64::min).unwrap_or(0.0)
    }

    /// Returns the maximum value, or 0.0 if empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("Temp", vec![15.0, 22.0, 8.0]);
    /// assert_eq!(series.max(), 22.0);
    /// ```
    pub fn max(&self) -> f64 {
        self.values.iter().copied().reduce(f64::max).unwrap_or(0.0)
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
