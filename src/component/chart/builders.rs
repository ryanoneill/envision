//! ChartState constructors and builder methods.
//!
//! Extracted from the main chart module to keep file sizes manageable.

use ratatui::style::Color;

use super::series::DEFAULT_PALETTE;
use super::{
    BarMode, ChartAnnotation, ChartKind, ChartState, DataSeries, Scale, ThresholdLine, VerticalLine,
};

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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartKind, ChartState, DataSeries};
    ///
    /// let state = ChartState::bar_horizontal(vec![
    ///     DataSeries::new("Memory", vec![512.0, 768.0, 1024.0]),
    /// ]);
    /// assert_eq!(state.kind(), &ChartKind::BarHorizontal);
    /// ```
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
    pub fn with_y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Sets whether to show the legend (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_show_legend(false);
    /// assert!(!state.show_legend());
    /// ```
    pub fn with_show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    /// Sets the maximum display points for line charts (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("Temp", vec![20.0, 22.0])])
    ///     .with_max_display_points(200);
    /// assert_eq!(state.max_display_points(), 200);
    /// ```
    pub fn with_max_display_points(mut self, max: usize) -> Self {
        self.max_display_points = max;
        self
    }

    /// Sets the bar width (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::bar_vertical(vec![DataSeries::new("Sales", vec![10.0])])
    ///     .with_bar_width(5);
    /// assert_eq!(state.bar_width(), 5);
    /// ```
    pub fn with_bar_width(mut self, width: u16) -> Self {
        self.bar_width = width.max(1);
        self
    }

    /// Sets the bar gap (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::bar_vertical(vec![DataSeries::new("Sales", vec![10.0])])
    ///     .with_bar_gap(2);
    /// assert_eq!(state.bar_gap(), 2);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChartState, DataSeries};
    ///
    /// let state = ChartState::line(vec![DataSeries::new("CPU", vec![50.0])])
    ///     .with_grid(true);
    /// assert!(state.show_grid());
    /// ```
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
