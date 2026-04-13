//! DataSeries implementation methods.
//!
//! Extracted from the main chart module to keep file sizes manageable.
//! Contains the builder, accessor, and mutation methods for [`DataSeries`],
//! and the 20-color Tableau palette for categorical data visualization.

use ratatui::style::Color;

use super::DataSeries;

/// A 20-color categorical palette based on Tableau 20 / D3's categorical scale.
///
/// The first 10 colors are saturated variants optimized for dark terminal
/// backgrounds. The second 10 are lighter tints of each, giving 20 visually
/// distinct series before any color repeats.
///
/// Colors cycle when the series index exceeds 20 (i.e., index % 20).
///
/// # Example
///
/// ```rust
/// use envision::component::chart_palette_color;
/// use ratatui::style::Color;
///
/// // First color is Tableau blue
/// assert_eq!(chart_palette_color(0), Color::Rgb(31, 119, 180));
/// // Wraps around after 20
/// assert_eq!(chart_palette_color(20), Color::Rgb(31, 119, 180));
/// ```
pub const DEFAULT_PALETTE: &[Color] = &[
    Color::Rgb(31, 119, 180),  // blue
    Color::Rgb(255, 127, 14),  // orange
    Color::Rgb(44, 160, 44),   // green
    Color::Rgb(214, 39, 40),   // red
    Color::Rgb(148, 103, 189), // purple
    Color::Rgb(140, 86, 75),   // brown
    Color::Rgb(227, 119, 194), // pink
    Color::Rgb(127, 127, 127), // gray
    Color::Rgb(188, 189, 34),  // olive
    Color::Rgb(23, 190, 207),  // teal
    // Lighter variants for 11-20
    Color::Rgb(174, 199, 232), // light blue
    Color::Rgb(255, 187, 120), // light orange
    Color::Rgb(152, 223, 138), // light green
    Color::Rgb(255, 152, 150), // light red
    Color::Rgb(197, 176, 213), // light purple
    Color::Rgb(196, 156, 148), // light brown
    Color::Rgb(247, 182, 210), // light pink
    Color::Rgb(199, 199, 199), // light gray
    Color::Rgb(219, 219, 141), // light olive
    Color::Rgb(158, 218, 229), // light teal
];

/// Returns the palette color at the given index, wrapping around after 20.
///
/// This is useful for assigning distinct colors to chart series automatically.
///
/// # Example
///
/// ```rust
/// use envision::component::chart_palette_color;
/// use ratatui::style::Color;
///
/// let color = chart_palette_color(3);
/// assert_eq!(color, Color::Rgb(214, 39, 40)); // red
/// ```
pub fn chart_palette_color(index: usize) -> Color {
    DEFAULT_PALETTE[index % DEFAULT_PALETTE.len()]
}

impl DataSeries {
    /// Creates a new data series.
    ///
    /// The default color is the first color from the Tableau 20 palette (blue).
    /// Use [`with_color`](DataSeries::with_color) to override, or
    /// [`chart_palette_color`](crate::component::chart_palette_color) to assign
    /// distinct colors by series index.
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
            color: DEFAULT_PALETTE[0],
            x_values: None,
            upper_bound: None,
            lower_bound: None,
        }
    }

    /// Creates a new data series with explicit X-Y pairs.
    ///
    /// This is a convenience constructor for data that has explicit X coordinates
    /// rather than using sequential indices. Useful for ROC curves, scatter plots
    /// with non-uniform X spacing, and parametric curves.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// // ROC curve: FPR on X-axis, TPR on Y-axis
    /// let roc = DataSeries::xy(
    ///     "Classifier",
    ///     vec![0.0, 0.1, 0.3, 0.5, 1.0],  // FPR (X)
    ///     vec![0.0, 0.5, 0.8, 0.9, 1.0],  // TPR (Y)
    /// );
    /// assert_eq!(roc.x_values(), Some([0.0, 0.1, 0.3, 0.5, 1.0].as_slice()));
    /// assert_eq!(roc.values(), &[0.0, 0.5, 0.8, 0.9, 1.0]);
    /// ```
    pub fn xy(label: impl Into<String>, x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            label: label.into(),
            values: y,
            color: DEFAULT_PALETTE[0],
            x_values: Some(x),
            upper_bound: None,
            lower_bound: None,
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

    /// Sets explicit X-axis values (builder pattern).
    ///
    /// When present, these X values are used instead of sequential indices
    /// (0, 1, 2, ...) when plotting the series. This enables non-uniform
    /// X spacing for ROC curves, scatter plots, and parametric curves.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("Curve", vec![0.0, 0.5, 0.9, 1.0])
    ///     .with_x_values(vec![0.0, 0.2, 0.6, 1.0]);
    /// assert_eq!(series.x_values(), Some([0.0, 0.2, 0.6, 1.0].as_slice()));
    /// ```
    pub fn with_x_values(mut self, x: Vec<f64>) -> Self {
        self.x_values = Some(x);
        self
    }

    /// Sets the upper bound values for error bands (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("CPU", vec![50.0, 60.0])
    ///     .with_upper_bound(vec![55.0, 65.0]);
    /// assert_eq!(series.upper_bound(), Some([55.0, 65.0].as_slice()));
    /// ```
    pub fn with_upper_bound(mut self, upper: Vec<f64>) -> Self {
        self.upper_bound = Some(upper);
        self
    }

    /// Sets the lower bound values for error bands (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("CPU", vec![50.0, 60.0])
    ///     .with_lower_bound(vec![45.0, 55.0]);
    /// assert_eq!(series.lower_bound(), Some([45.0, 55.0].as_slice()));
    /// ```
    pub fn with_lower_bound(mut self, lower: Vec<f64>) -> Self {
        self.lower_bound = Some(lower);
        self
    }

    /// Sets both upper and lower bound values for error bands (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("CPU", vec![50.0])
    ///     .with_bounds(vec![45.0], vec![55.0]);
    /// assert!(series.lower_bound().is_some());
    /// assert!(series.upper_bound().is_some());
    /// ```
    pub fn with_bounds(mut self, lower: Vec<f64>, upper: Vec<f64>) -> Self {
        self.lower_bound = Some(lower);
        self.upper_bound = Some(upper);
        self
    }

    /// Returns the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("Temperature", vec![20.0, 22.0]);
    /// assert_eq!(series.label(), "Temperature");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("CPU", vec![10.0, 20.0, 30.0]);
    /// assert_eq!(series.values(), &[10.0, 20.0, 30.0]);
    /// ```
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Returns the color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    /// use ratatui::style::Color;
    ///
    /// let series = DataSeries::new("CPU", vec![50.0]).with_color(Color::Green);
    /// assert_eq!(series.color(), Color::Green);
    /// ```
    pub fn color(&self) -> Color {
        self.color
    }

    /// Returns the explicit X-axis values, if set.
    ///
    /// When `Some`, these values are used as X coordinates instead of sequential
    /// indices. When `None`, the series uses implicit indices (0, 1, 2, ...).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let implicit = DataSeries::new("A", vec![1.0, 2.0]);
    /// assert_eq!(implicit.x_values(), None);
    ///
    /// let explicit = DataSeries::xy("B", vec![0.0, 0.5], vec![1.0, 2.0]);
    /// assert_eq!(explicit.x_values(), Some([0.0, 0.5].as_slice()));
    /// ```
    pub fn x_values(&self) -> Option<&[f64]> {
        self.x_values.as_deref()
    }

    /// Returns the upper bound values, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("CPU", vec![50.0]).with_upper_bound(vec![55.0]);
    /// assert_eq!(series.upper_bound(), Some([55.0].as_slice()));
    /// assert_eq!(DataSeries::new("A", vec![1.0]).upper_bound(), None);
    /// ```
    pub fn upper_bound(&self) -> Option<&[f64]> {
        self.upper_bound.as_deref()
    }

    /// Returns the lower bound values, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("CPU", vec![50.0]).with_lower_bound(vec![45.0]);
    /// assert_eq!(series.lower_bound(), Some([45.0].as_slice()));
    /// assert_eq!(DataSeries::new("A", vec![1.0]).lower_bound(), None);
    /// ```
    pub fn lower_bound(&self) -> Option<&[f64]> {
        self.lower_bound.as_deref()
    }

    /// Appends a value.
    ///
    /// Note: This appends only to the Y-values. It does not affect `x_values`.
    /// For series with explicit X coordinates, manage `x_values` separately
    /// via [`set_x_values`](DataSeries::set_x_values).
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("Temp", vec![10.0, 20.0, 30.0]);
    /// assert_eq!(series.last(), Some(30.0));
    /// assert_eq!(DataSeries::new("Empty", vec![]).last(), None);
    /// ```
    pub fn last(&self) -> Option<f64> {
        self.values.last().copied()
    }

    /// Returns the number of data points.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let series = DataSeries::new("CPU", vec![10.0, 20.0, 30.0]);
    /// assert_eq!(series.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns true if the series has no data points.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// assert!(DataSeries::new("Empty", vec![]).is_empty());
    /// assert!(!DataSeries::new("CPU", vec![50.0]).is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Clears all values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let mut series = DataSeries::new("CPU", vec![10.0, 20.0]);
    /// series.clear();
    /// assert!(series.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Sets the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let mut series = DataSeries::new("Old", vec![1.0]);
    /// series.set_label("New");
    /// assert_eq!(series.label(), "New");
    /// ```
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Sets the color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    /// use ratatui::style::Color;
    ///
    /// let mut series = DataSeries::new("CPU", vec![50.0]);
    /// series.set_color(Color::Red);
    /// assert_eq!(series.color(), Color::Red);
    /// ```
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Sets the explicit X-axis values.
    ///
    /// Pass `Some(vec)` to set explicit X coordinates, or `None` to revert
    /// to using sequential indices (0, 1, 2, ...).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let mut series = DataSeries::new("Curve", vec![0.0, 0.5, 1.0]);
    /// assert_eq!(series.x_values(), None);
    ///
    /// series.set_x_values(Some(vec![0.0, 0.3, 1.0]));
    /// assert_eq!(series.x_values(), Some([0.0, 0.3, 1.0].as_slice()));
    ///
    /// series.set_x_values(None);
    /// assert_eq!(series.x_values(), None);
    /// ```
    pub fn set_x_values(&mut self, x: Option<Vec<f64>>) {
        self.x_values = x;
    }

    /// Sets the upper bound values for error bands.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let mut series = DataSeries::new("CPU", vec![50.0]);
    /// series.set_upper_bound(Some(vec![55.0]));
    /// assert_eq!(series.upper_bound(), Some([55.0].as_slice()));
    /// series.set_upper_bound(None);
    /// assert_eq!(series.upper_bound(), None);
    /// ```
    pub fn set_upper_bound(&mut self, upper: Option<Vec<f64>>) {
        self.upper_bound = upper;
    }

    /// Sets the lower bound values for error bands.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DataSeries;
    ///
    /// let mut series = DataSeries::new("CPU", vec![50.0]);
    /// series.set_lower_bound(Some(vec![45.0]));
    /// assert_eq!(series.lower_bound(), Some([45.0].as_slice()));
    /// series.set_lower_bound(None);
    /// assert_eq!(series.lower_bound(), None);
    /// ```
    pub fn set_lower_bound(&mut self, lower: Option<Vec<f64>>) {
        self.lower_bound = lower;
    }
}
