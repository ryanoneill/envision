//! ChartState accessor methods, setters, computed properties, and instance methods.
//!
//! Extracted from the main chart module to keep file sizes manageable.

use super::{
    BarMode, Chart, ChartAnnotation, ChartKind, ChartMessage, ChartOutput, ChartState, DataSeries,
    Scale, ThresholdLine, VerticalLine,
};
use crate::component::Component;

impl ChartState {
    // ---- Accessors ----

    /// Returns the data series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])]);
    /// assert_eq!(state.series().len(), 1);
    /// ```
    pub fn series(&self) -> &[DataSeries] {
        &self.series
    }

    /// Returns a mutable reference to the series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let mut state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])]);
    /// assert_eq!(state.series_mut().len(), 1);
    /// ```
    pub fn series_mut(&mut self) -> &mut [DataSeries] {
        &mut self.series
    }

    /// Returns the series at the given index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])]);
    /// assert!(state.get_series(0).is_some());
    /// assert!(state.get_series(1).is_none());
    /// ```
    pub fn get_series(&self, index: usize) -> Option<&DataSeries> {
        self.series.get(index)
    }

    /// Returns a mutable reference to the series at the given index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let mut state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])]);
    /// assert!(state.get_series_mut(0).is_some());
    /// assert!(state.get_series_mut(1).is_none());
    /// ```
    pub fn get_series_mut(&mut self, index: usize) -> Option<&mut DataSeries> {
        self.series.get_mut(index)
    }

    /// Returns the chart kind.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartKind, ChartState};
    ///
    /// let state = ChartState::line(vec![]);
    /// assert_eq!(state.kind(), &ChartKind::Line);
    /// ```
    pub fn kind(&self) -> &ChartKind {
        &self.kind
    }

    /// Sets the chart kind.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartKind, ChartState};
    ///
    /// let mut state = ChartState::line(vec![]);
    /// state.set_kind(ChartKind::BarVertical);
    /// assert_eq!(state.kind(), &ChartKind::BarVertical);
    /// ```
    pub fn set_kind(&mut self, kind: ChartKind) {
        self.kind = kind;
    }

    /// Returns the active series index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])]);
    /// assert_eq!(state.active_series(), 0);
    /// ```
    pub fn active_series(&self) -> usize {
        self.active_series
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![]).with_title("My Chart");
    /// assert_eq!(state.title(), Some("My Chart"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let mut state = ChartState::line(vec![]);
    /// state.set_title(Some("Updated".to_string()));
    /// assert_eq!(state.title(), Some("Updated"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns the X-axis label.
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
    pub fn x_label(&self) -> Option<&str> {
        self.x_label.as_deref()
    }

    /// Returns the Y-axis label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_y_label("Percent");
    /// assert_eq!(state.y_label(), Some("Percent"));
    /// ```
    pub fn y_label(&self) -> Option<&str> {
        self.y_label.as_deref()
    }

    /// Returns whether the legend is shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let state = ChartState::line(vec![]);
    /// assert!(state.show_legend());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let state = ChartState::line(vec![]).with_max_display_points(100);
    /// assert_eq!(state.max_display_points(), 100);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let state = ChartState::bar_vertical(vec![]).with_bar_width(4);
    /// assert_eq!(state.bar_width(), 4);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let state = ChartState::bar_vertical(vec![]).with_bar_gap(2);
    /// assert_eq!(state.bar_gap(), 2);
    /// ```
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

    /// Returns the bar rendering mode.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BarMode, ChartState, DataSeries};
    ///
    /// let state = ChartState::bar_vertical(vec![
    ///     DataSeries::new("Sales", vec![10.0, 20.0]),
    /// ])
    /// .with_bar_mode(BarMode::Grouped);
    /// assert_eq!(state.bar_mode(), &BarMode::Grouped);
    /// ```
    pub fn bar_mode(&self) -> &BarMode {
        &self.bar_mode
    }

    /// Sets the bar rendering mode.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BarMode, ChartState};
    ///
    /// let mut state = ChartState::bar_vertical(vec![]);
    /// state.set_bar_mode(BarMode::Stacked);
    /// assert_eq!(state.bar_mode(), &BarMode::Stacked);
    /// ```
    pub fn set_bar_mode(&mut self, mode: BarMode) {
        self.bar_mode = mode;
    }

    /// Returns the category labels for bar chart x-axis.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::bar_vertical(vec![
    ///     DataSeries::new("Sales", vec![10.0, 20.0, 30.0]),
    /// ])
    /// .with_categories(vec!["Q1", "Q2", "Q3"]);
    /// assert_eq!(state.categories(), &["Q1", "Q2", "Q3"]);
    /// ```
    pub fn categories(&self) -> &[String] {
        &self.categories
    }

    /// Sets the category labels for bar chart x-axis.
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
    /// let mut state = ChartState::bar_vertical(vec![
    ///     DataSeries::new("Sales", vec![10.0, 20.0, 30.0]),
    /// ]);
    /// state.set_categories(vec!["Q1", "Q2", "Q3"]);
    /// assert_eq!(state.categories(), &["Q1", "Q2", "Q3"]);
    /// ```
    pub fn set_categories(&mut self, categories: Vec<impl Into<String>>) {
        self.categories = categories.into_iter().map(Into::into).collect();
    }

    /// Returns the custom X-axis labels, if set.
    ///
    /// When present, these labels replace the numeric tick labels on the X-axis
    /// of line, area, and scatter charts.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 60.0])])
    ///     .with_x_labels(vec!["09:00", "10:00"]);
    /// assert_eq!(state.x_labels().unwrap(), &["09:00", "10:00"]);
    /// ```
    pub fn x_labels(&self) -> Option<&[String]> {
        self.x_labels.as_deref()
    }

    /// Sets custom string labels for the X-axis.
    ///
    /// When set to `Some`, these labels replace the numeric tick labels on the
    /// X-axis of line, area, and scatter charts. Pass `None` to revert to
    /// numeric tick labels.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let mut state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 60.0, 70.0])]);
    /// state.set_x_labels(Some(vec!["Mon", "Tue", "Wed"]));
    /// assert_eq!(state.x_labels().unwrap(), &["Mon", "Tue", "Wed"]);
    ///
    /// state.set_x_labels(None::<Vec<String>>);
    /// assert!(state.x_labels().is_none());
    /// ```
    pub fn set_x_labels(&mut self, labels: Option<Vec<impl Into<String>>>) {
        self.x_labels = labels.map(|v| v.into_iter().map(Into::into).collect());
    }

    /// Returns the number of series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![
    ///     DataSeries::new("A", vec![1.0]),
    ///     DataSeries::new("B", vec![2.0]),
    /// ]);
    /// assert_eq!(state.series_count(), 2);
    /// ```
    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    /// Returns true if there are no series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let state = ChartState::line(vec![]);
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }

    /// Returns the threshold lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_threshold(90.0, "SLO", Color::Yellow);
    /// assert_eq!(state.thresholds().len(), 1);
    /// ```
    pub fn thresholds(&self) -> &[ThresholdLine] {
        &self.thresholds
    }

    /// Returns the manual Y-axis minimum, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_y_range(0.0, 100.0);
    /// assert_eq!(state.y_min(), Some(0.0));
    /// ```
    pub fn y_min(&self) -> Option<f64> {
        self.y_min
    }

    /// Returns the manual Y-axis maximum, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_y_range(0.0, 100.0);
    /// assert_eq!(state.y_max(), Some(100.0));
    /// ```
    pub fn y_max(&self) -> Option<f64> {
        self.y_max
    }

    /// Returns the Y-axis scale.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, Scale};
    ///
    /// let state = ChartState::line(vec![]).with_y_scale(Scale::Log10);
    /// assert_eq!(state.y_scale(), &Scale::Log10);
    /// ```
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
    /// If the series color is the default (Cyan) and there are already series
    /// with Cyan, a color is automatically assigned from the default palette
    /// based on the series index.
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
    pub fn add_series(&mut self, mut series: DataSeries) {
        if series.color() == ratatui::style::Color::Cyan {
            let idx = self.series.len();
            let palette = super::DEFAULT_PALETTE;
            series.set_color(palette[idx % palette.len()]);
        }
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

    /// Returns the cursor position (data index), if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let state = ChartState::line(vec![]);
    /// assert!(state.cursor_position().is_none());
    /// ```
    pub fn cursor_position(&self) -> Option<usize> {
        self.cursor_position
    }

    /// Sets the cursor position.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let mut state = ChartState::line(vec![]);
    /// state.set_cursor_position(Some(3));
    /// assert_eq!(state.cursor_position(), Some(3));
    /// ```
    pub fn set_cursor_position(&mut self, position: Option<usize>) {
        self.cursor_position = position;
    }

    /// Returns whether the crosshair is visible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let state = ChartState::line(vec![]);
    /// assert!(!state.show_crosshair());
    /// ```
    pub fn show_crosshair(&self) -> bool {
        self.show_crosshair
    }

    /// Sets crosshair visibility.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let mut state = ChartState::line(vec![]);
    /// state.set_show_crosshair(true);
    /// assert!(state.show_crosshair());
    /// ```
    pub fn set_show_crosshair(&mut self, show: bool) {
        self.show_crosshair = show;
    }

    /// Returns whether grid lines are visible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let state = ChartState::line(vec![]).with_grid(true);
    /// assert!(state.show_grid());
    /// ```
    pub fn show_grid(&self) -> bool {
        self.show_grid
    }

    /// Sets grid line visibility.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChartState;
    ///
    /// let mut state = ChartState::line(vec![]);
    /// state.set_show_grid(true);
    /// assert!(state.show_grid());
    /// ```
    pub fn set_show_grid(&mut self, show: bool) {
        self.show_grid = show;
    }

    /// Returns the vertical reference lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 60.0])])
    ///     .with_vertical_line(1.0, "Deploy", Color::Yellow);
    /// assert_eq!(state.vertical_lines().len(), 1);
    /// ```
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

    /// Returns the annotations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartAnnotation, ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 90.0])])
    ///     .with_annotation(1.0, 90.0, "Peak", Color::Yellow);
    /// assert_eq!(state.annotations().len(), 1);
    /// assert_eq!(state.annotations()[0].label, "Peak");
    /// ```
    pub fn annotations(&self) -> &[ChartAnnotation] {
        &self.annotations
    }

    /// Adds an annotation at a data coordinate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartAnnotation, ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let mut state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 90.0])]);
    /// state.add_annotation(ChartAnnotation::new(1.0, 90.0, "Peak", Color::Yellow));
    /// assert_eq!(state.annotations().len(), 1);
    /// ```
    pub fn add_annotation(&mut self, annotation: ChartAnnotation) {
        self.annotations.push(annotation);
    }

    /// Clears all annotations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    /// use ratatui::style::Color;
    ///
    /// let mut state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0, 90.0])])
    ///     .with_annotation(1.0, 90.0, "Peak", Color::Yellow);
    /// state.clear_annotations();
    /// assert!(state.annotations().is_empty());
    /// ```
    pub fn clear_annotations(&mut self) {
        self.annotations.clear();
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

    /// Returns the manual Y-axis range as a `(min, max)` tuple.
    ///
    /// Each bound is `None` when auto-scaling from data is used.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let mut state = ChartState::area(vec![DataSeries::new("CPU", vec![50.0])]);
    /// assert_eq!(state.y_range(), (None, None));
    /// state.set_y_range(Some(0.0), Some(100.0));
    /// assert_eq!(state.y_range(), (Some(0.0), Some(100.0)));
    /// ```
    pub fn y_range(&self) -> (Option<f64>, Option<f64>) {
        (self.y_min, self.y_max)
    }

    // ---- Computed properties ----

    /// Computes the global min value across all series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![
    ///     DataSeries::new("A", vec![5.0, 10.0]),
    ///     DataSeries::new("B", vec![2.0, 8.0]),
    /// ]);
    /// assert_eq!(state.global_min(), 2.0);
    /// ```
    pub fn global_min(&self) -> f64 {
        self.series
            .iter()
            .map(|s| s.min())
            .reduce(f64::min)
            .unwrap_or(0.0)
    }

    /// Computes the global max value across all series.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![
    ///     DataSeries::new("A", vec![5.0, 10.0]),
    ///     DataSeries::new("B", vec![2.0, 8.0]),
    /// ]);
    /// assert_eq!(state.global_max(), 10.0);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::area(vec![DataSeries::new("CPU", vec![20.0, 80.0])])
    ///     .with_y_range(0.0, 100.0);
    /// assert_eq!(state.effective_min(), 0.0);
    /// ```
    pub fn effective_min(&self) -> f64 {
        self.y_min.unwrap_or_else(|| {
            let data_min = self.global_min();
            let threshold_min = self
                .thresholds
                .iter()
                .map(|t| t.value)
                .reduce(f64::min)
                .unwrap_or(data_min);
            let bounds_min = self
                .series
                .iter()
                .filter_map(|s| {
                    s.lower_bound()
                        .and_then(|lb| lb.iter().copied().reduce(f64::min))
                })
                .reduce(f64::min)
                .unwrap_or(data_min);
            f64::min(f64::min(data_min, threshold_min), bounds_min)
        })
    }

    /// Computes the effective maximum for the Y-axis, considering manual override.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::area(vec![DataSeries::new("CPU", vec![20.0, 80.0])])
    ///     .with_y_range(0.0, 100.0);
    /// assert_eq!(state.effective_max(), 100.0);
    /// ```
    pub fn effective_max(&self) -> f64 {
        self.y_max.unwrap_or_else(|| {
            let data_max = self.global_max();
            let threshold_max = self
                .thresholds
                .iter()
                .map(|t| t.value)
                .reduce(f64::max)
                .unwrap_or(data_max);
            let bounds_max = self
                .series
                .iter()
                .filter_map(|s| {
                    s.upper_bound()
                        .and_then(|ub| ub.iter().copied().reduce(f64::max))
                })
                .reduce(f64::max)
                .unwrap_or(data_max);
            f64::max(f64::max(data_max, threshold_max), bounds_max)
        })
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, ChartMessage, ChartOutput, DataSeries, Scale};
    ///
    /// let mut state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])]);
    /// let output = state.update(ChartMessage::SetYScale(Scale::Log10));
    /// assert!(output.is_none());
    /// assert_eq!(state.y_scale(), &Scale::Log10);
    /// ```
    pub fn update(&mut self, msg: ChartMessage) -> Option<ChartOutput> {
        Chart::update(self, msg)
    }
}
