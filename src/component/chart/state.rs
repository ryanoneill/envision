//! ChartState accessor methods, setters, computed properties, and instance methods.
//!
//! Extracted from the main chart module to keep file sizes manageable.

use super::{
    Chart, ChartKind, ChartMessage, ChartOutput, ChartState, DataSeries, Scale, ThresholdLine,
    VerticalLine,
};
use crate::component::Component;

impl ChartState {
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

    /// Returns the Y-axis scale.
    pub fn y_scale(&self) -> &Scale {
        &self.y_scale
    }

    /// Sets the Y-axis scale.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, Scale};
    ///
    /// let mut state = ChartState::line(vec![]);
    /// state.set_y_scale(Scale::Log10);
    /// assert_eq!(state.y_scale(), &Scale::Log10);
    /// ```
    pub fn set_y_scale(&mut self, scale: Scale) {
        self.y_scale = scale;
    }

    // ---- Mutation methods ----

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

    /// Returns the vertical reference lines.
    pub fn vertical_lines(&self) -> &[VerticalLine] {
        &self.vertical_lines
    }

    /// Adds a vertical reference line.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries, VerticalLine};
    /// use ratatui::style::Color;
    ///
    /// let mut state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 60.0])]);
    /// state.add_vertical_line(VerticalLine::new(1.0, "Deploy", Color::Yellow));
    /// assert_eq!(state.vertical_lines().len(), 1);
    /// ```
    pub fn add_vertical_line(&mut self, line: VerticalLine) {
        self.vertical_lines.push(line);
    }

    /// Clears all vertical reference lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let mut state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 60.0])])
    ///     .with_vertical_line(1.0, "Deploy", Color::Yellow);
    /// state.clear_vertical_lines();
    /// assert!(state.vertical_lines().is_empty());
    /// ```
    pub fn clear_vertical_lines(&mut self) {
        self.vertical_lines.clear();
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

    // ---- Computed properties ----

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

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: ChartMessage) -> Option<ChartOutput> {
        Chart::update(self, msg)
    }
}
