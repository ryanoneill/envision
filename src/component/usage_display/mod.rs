//! A component for displaying usage metrics.
//!
//! [`UsageDisplay`] provides a display of usage metrics such as CPU, memory,
//! disk, or any arbitrary label-value pair. Metrics can be arranged in
//! horizontal, vertical, or grid layouts with optional borders and icons.
//!
//! This is a **display-only** component that does not receive keyboard focus.
//! State is stored in [`UsageDisplayState`] and updated via
//! [`UsageDisplayMessage`].
//!
//! # Layouts
//!
//! - [`UsageLayout::Horizontal`]: Compact inline format with separator
//! - [`UsageLayout::Vertical`]: Bordered one-per-line display
//! - [`UsageLayout::Grid`]: Bordered N-column grid layout
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     UsageDisplay, UsageDisplayMessage, UsageDisplayState, UsageLayout, UsageMetric, Component,
//! };
//! use ratatui::style::Color;
//!
//! let state = UsageDisplayState::new()
//!     .metric(UsageMetric::new("CPU", "45%").with_color(Color::Green))
//!     .metric(UsageMetric::new("Memory", "3.2 GB").with_color(Color::Yellow))
//!     .metric(UsageMetric::new("Disk", "120 GB").with_color(Color::Cyan));
//!
//! assert_eq!(state.len(), 3);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, RenderContext};
use crate::theme::Theme;

/// Layout style for usage metrics display.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum UsageLayout {
    /// Metrics displayed inline with a separator: `"CPU: 45% | Memory: 3.2 GB"`.
    #[default]
    Horizontal,
    /// Bordered, one metric per line.
    Vertical,
    /// Bordered grid with the given number of columns.
    Grid(usize),
}

/// A single usage metric entry.
///
/// Represents a label-value pair with optional color and icon.
///
/// # Example
///
/// ```rust
/// use envision::component::UsageMetric;
/// use ratatui::style::Color;
///
/// let metric = UsageMetric::new("CPU", "45%")
///     .with_color(Color::Green)
///     .with_icon("*");
///
/// assert_eq!(metric.label(), "CPU");
/// assert_eq!(metric.value(), "45%");
/// assert_eq!(metric.color(), Some(Color::Green));
/// assert_eq!(metric.icon(), Some("*"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct UsageMetric {
    /// The metric label (e.g., "CPU", "Memory").
    label: String,
    /// The metric value (e.g., "45%", "3.2 GB").
    value: String,
    /// Optional color for the value.
    color: Option<Color>,
    /// Optional icon prefix.
    icon: Option<String>,
}

impl UsageMetric {
    /// Creates a new usage metric with a label and value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageMetric;
    ///
    /// let metric = UsageMetric::new("CPU", "45%");
    /// assert_eq!(metric.label(), "CPU");
    /// assert_eq!(metric.value(), "45%");
    /// assert_eq!(metric.color(), None);
    /// assert_eq!(metric.icon(), None);
    /// ```
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            color: None,
            icon: None,
        }
    }

    /// Sets the color for the metric value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageMetric;
    /// use ratatui::style::Color;
    ///
    /// let metric = UsageMetric::new("CPU", "45%").with_color(Color::Green);
    /// assert_eq!(metric.color(), Some(Color::Green));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the icon prefix for the metric.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageMetric;
    ///
    /// let metric = UsageMetric::new("CPU", "45%").with_icon("*");
    /// assert_eq!(metric.icon(), Some("*"));
    /// ```
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Returns the metric label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the metric value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the optional color.
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Returns the optional icon.
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    /// Sets the metric label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageMetric;
    ///
    /// let mut metric = UsageMetric::new("CPU", "45%");
    /// metric.set_label("Processor");
    /// assert_eq!(metric.label(), "Processor");
    /// ```
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Sets the metric value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageMetric;
    ///
    /// let mut metric = UsageMetric::new("CPU", "45%");
    /// metric.set_value("80%");
    /// assert_eq!(metric.value(), "80%");
    /// ```
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }

    /// Sets the optional color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageMetric;
    /// use ratatui::style::Color;
    ///
    /// let mut metric = UsageMetric::new("CPU", "45%");
    /// metric.set_color(Some(Color::Red));
    /// assert_eq!(metric.color(), Some(Color::Red));
    /// ```
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    /// Sets the optional icon.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageMetric;
    ///
    /// let mut metric = UsageMetric::new("CPU", "45%");
    /// metric.set_icon(Some("*".to_string()));
    /// assert_eq!(metric.icon(), Some("*"));
    /// ```
    pub fn set_icon(&mut self, icon: Option<String>) {
        self.icon = icon;
    }
}

/// Messages that can be sent to a UsageDisplay component.
#[derive(Clone, Debug, PartialEq)]
pub enum UsageDisplayMessage {
    /// Set all metrics at once.
    SetMetrics(Vec<UsageMetric>),
    /// Add a single metric.
    AddMetric(UsageMetric),
    /// Remove a metric by label.
    RemoveMetric(String),
    /// Update a metric's value by label.
    UpdateValue {
        /// The label of the metric to update.
        label: String,
        /// The new value.
        value: String,
    },
    /// Update a metric's color by label.
    UpdateColor {
        /// The label of the metric to update.
        label: String,
        /// The new color (None to clear).
        color: Option<Color>,
    },
    /// Set the layout style.
    SetLayout(UsageLayout),
    /// Set the title.
    SetTitle(Option<String>),
    /// Set the separator used in horizontal layout.
    SetSeparator(String),
    /// Clear all metrics.
    Clear,
}

/// State for a UsageDisplay component.
///
/// Contains all metrics and display configuration.
///
/// # Example
///
/// ```rust
/// use envision::component::{UsageDisplayState, UsageLayout, UsageMetric};
///
/// let state = UsageDisplayState::new()
///     .with_layout(UsageLayout::Vertical)
///     .with_title("System")
///     .metric(UsageMetric::new("CPU", "45%"))
///     .metric(UsageMetric::new("Memory", "3.2 GB"));
///
/// assert_eq!(state.len(), 2);
/// assert_eq!(state.layout(), UsageLayout::Vertical);
/// assert_eq!(state.title(), Some("System"));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct UsageDisplayState {
    /// All usage metrics.
    metrics: Vec<UsageMetric>,
    /// Layout style.
    layout: UsageLayout,
    /// Optional title for bordered layouts.
    title: Option<String>,
    /// Separator for horizontal layout.
    separator: String,
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for UsageDisplayState {
    fn default() -> Self {
        Self {
            metrics: Vec::new(),
            layout: UsageLayout::default(),
            title: None,
            separator: " \u{2502} ".to_string(), // " | " (thin vertical box char)
            disabled: false,
        }
    }
}

impl UsageDisplayState {
    /// Creates a new empty UsageDisplay state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let state = UsageDisplayState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new state with the given metrics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let metrics = vec![UsageMetric::new("CPU", "45%")];
    /// let state = UsageDisplayState::with_metrics(metrics);
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn with_metrics(metrics: Vec<UsageMetric>) -> Self {
        Self {
            metrics,
            ..Self::default()
        }
    }

    /// Sets the layout style using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageLayout};
    ///
    /// let state = UsageDisplayState::new()
    ///     .with_layout(UsageLayout::Vertical);
    /// assert_eq!(state.layout(), UsageLayout::Vertical);
    /// ```
    pub fn with_layout(mut self, layout: UsageLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Sets the title using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let state = UsageDisplayState::new()
    ///     .with_title("System Metrics");
    /// assert_eq!(state.title(), Some("System Metrics"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the separator for horizontal layout using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let state = UsageDisplayState::new()
    ///     .with_separator(" | ");
    /// assert_eq!(state.separator(), " | ");
    /// ```
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// Sets the disabled state using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let state = UsageDisplayState::new()
    ///     .with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Adds a metric using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let state = UsageDisplayState::new()
    ///     .metric(UsageMetric::new("CPU", "45%"))
    ///     .metric(UsageMetric::new("Memory", "3.2 GB"));
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn metric(mut self, metric: UsageMetric) -> Self {
        self.metrics.push(metric);
        self
    }

    /// Returns all metrics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let state = UsageDisplayState::new()
    ///     .metric(UsageMetric::new("CPU", "45%"));
    /// assert_eq!(state.metrics().len(), 1);
    /// assert_eq!(state.metrics()[0].label(), "CPU");
    /// ```
    pub fn metrics(&self) -> &[UsageMetric] {
        &self.metrics
    }

    /// Returns the layout style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageLayout};
    ///
    /// let state = UsageDisplayState::new().with_layout(UsageLayout::Grid(2));
    /// assert_eq!(state.layout(), UsageLayout::Grid(2));
    /// ```
    pub fn layout(&self) -> UsageLayout {
        self.layout
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let state = UsageDisplayState::new().with_title("System");
    /// assert_eq!(state.title(), Some("System"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the separator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let state = UsageDisplayState::new().with_separator(" / ");
    /// assert_eq!(state.separator(), " / ");
    /// ```
    pub fn separator(&self) -> &str {
        &self.separator
    }

    /// Returns the number of metrics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let state = UsageDisplayState::new()
    ///     .metric(UsageMetric::new("CPU", "45%"))
    ///     .metric(UsageMetric::new("Memory", "3.2 GB"));
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    /// Returns true if there are no metrics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// assert!(UsageDisplayState::new().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }

    /// Returns true if the component is disabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let state = UsageDisplayState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the metrics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let mut state = UsageDisplayState::new();
    /// state.set_metrics(vec![UsageMetric::new("CPU", "45%")]);
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn set_metrics(&mut self, metrics: Vec<UsageMetric>) {
        self.metrics = metrics;
    }

    /// Adds a metric.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let mut state = UsageDisplayState::new();
    /// state.add_metric(UsageMetric::new("Disk", "120 GB"));
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn add_metric(&mut self, metric: UsageMetric) {
        self.metrics.push(metric);
    }

    /// Removes a metric by label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let mut state = UsageDisplayState::new()
    ///     .metric(UsageMetric::new("CPU", "45%"))
    ///     .metric(UsageMetric::new("Memory", "3.2 GB"));
    /// state.remove_metric("CPU");
    /// assert_eq!(state.len(), 1);
    /// assert_eq!(state.metrics()[0].label(), "Memory");
    /// ```
    pub fn remove_metric(&mut self, label: &str) {
        self.metrics.retain(|m| m.label != label);
    }

    /// Updates a metric's value by label.
    ///
    /// Returns true if the metric was found and updated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let mut state = UsageDisplayState::new()
    ///     .metric(UsageMetric::new("CPU", "45%"));
    /// assert!(state.update_value("CPU", "80%"));
    /// assert_eq!(state.find("CPU").unwrap().value(), "80%");
    /// ```
    pub fn update_value(&mut self, label: &str, value: impl Into<String>) -> bool {
        if let Some(metric) = self.metrics.iter_mut().find(|m| m.label == label) {
            metric.value = value.into();
            true
        } else {
            false
        }
    }

    /// Updates a metric's color by label.
    ///
    /// Returns true if the metric was found and updated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    /// use ratatui::style::Color;
    ///
    /// let mut state = UsageDisplayState::new()
    ///     .metric(UsageMetric::new("CPU", "45%"));
    /// assert!(state.update_color("CPU", Some(Color::Red)));
    /// assert_eq!(state.find("CPU").unwrap().color(), Some(Color::Red));
    /// ```
    pub fn update_color(&mut self, label: &str, color: Option<Color>) -> bool {
        if let Some(metric) = self.metrics.iter_mut().find(|m| m.label == label) {
            metric.color = color;
            true
        } else {
            false
        }
    }

    /// Sets the layout.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageLayout};
    ///
    /// let mut state = UsageDisplayState::new();
    /// state.set_layout(UsageLayout::Vertical);
    /// assert_eq!(state.layout(), UsageLayout::Vertical);
    /// ```
    pub fn set_layout(&mut self, layout: UsageLayout) {
        self.layout = layout;
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let mut state = UsageDisplayState::new();
    /// state.set_title(Some("Metrics".to_string()));
    /// assert_eq!(state.title(), Some("Metrics"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Sets the separator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let mut state = UsageDisplayState::new();
    /// state.set_separator(" / ");
    /// assert_eq!(state.separator(), " / ");
    /// ```
    pub fn set_separator(&mut self, separator: impl Into<String>) {
        self.separator = separator.into();
    }

    /// Sets the disabled state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::UsageDisplayState;
    ///
    /// let mut state = UsageDisplayState::new();
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Clears all metrics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let mut state = UsageDisplayState::new()
    ///     .metric(UsageMetric::new("CPU", "45%"));
    /// state.clear();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.metrics.clear();
    }

    /// Finds a metric by label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let state = UsageDisplayState::new()
    ///     .metric(UsageMetric::new("CPU", "45%"));
    /// assert_eq!(state.find("CPU").unwrap().value(), "45%");
    /// assert!(state.find("Disk").is_none());
    /// ```
    pub fn find(&self, label: &str) -> Option<&UsageMetric> {
        self.metrics.iter().find(|m| m.label == label)
    }

    /// Finds a mutable metric by label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{UsageDisplayState, UsageMetric};
    ///
    /// let mut state = UsageDisplayState::new()
    ///     .metric(UsageMetric::new("CPU", "45%"));
    /// if let Some(metric) = state.find_mut("CPU") {
    ///     metric.set_value("80%");
    /// }
    /// assert_eq!(state.find("CPU").unwrap().value(), "80%");
    /// ```
    pub fn find_mut(&mut self, label: &str) -> Option<&mut UsageMetric> {
        self.metrics.iter_mut().find(|m| m.label == label)
    }
}

/// A component for displaying usage metrics.
///
/// `UsageDisplay` renders a collection of label-value metrics in various
/// layouts. This is a display-only component; it does not receive focus.
///
/// # Layouts
///
/// **Horizontal** (compact, no borders):
/// ```text
/// CPU: 45% | Memory: 3.2 GB | Disk: 120 GB
/// ```
///
/// **Vertical** (bordered, one per line):
/// ```text
/// +-- System --------+
/// | CPU: 45%         |
/// | Memory: 3.2 GB   |
/// | Disk: 120 GB     |
/// +------------------+
/// ```
///
/// **Grid(2)** (bordered, 2 columns):
/// ```text
/// +-- System ---------+
/// | CPU: 45%  | Mem: 3.2 GB |
/// | Disk: 120 |             |
/// +---------------------------+
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     UsageDisplay, UsageDisplayMessage, UsageDisplayState, UsageMetric, Component,
/// };
///
/// let mut state = UsageDisplayState::new()
///     .metric(UsageMetric::new("CPU", "45%"))
///     .metric(UsageMetric::new("Memory", "3.2 GB"));
///
/// // Update a metric value
/// UsageDisplay::update(
///     &mut state,
///     UsageDisplayMessage::UpdateValue {
///         label: "CPU".to_string(),
///         value: "80%".to_string(),
///     },
/// );
///
/// assert_eq!(state.find("CPU").unwrap().value(), "80%");
/// ```
pub struct UsageDisplay;

impl UsageDisplay {
    /// Renders a single metric as a sequence of spans.
    fn metric_spans(metric: &UsageMetric, theme: &Theme) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        if let Some(icon) = &metric.icon {
            spans.push(Span::styled(format!("{} ", icon), theme.normal_style()));
        }

        spans.push(Span::styled(
            format!("{}: ", metric.label),
            theme.normal_style(),
        ));

        let value_style = if let Some(color) = metric.color {
            Style::default().fg(color)
        } else {
            theme.normal_style()
        };
        spans.push(Span::styled(metric.value.clone(), value_style));

        spans
    }

    /// Renders horizontal layout.
    fn view_horizontal(state: &UsageDisplayState, frame: &mut Frame, area: Rect, theme: &Theme) {
        let mut spans = Vec::new();
        for (i, metric) in state.metrics.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(
                    state.separator.clone(),
                    theme.disabled_style(),
                ));
            }
            spans.extend(Self::metric_spans(metric, theme));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);

        let annotation = crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
            "UsageDisplay".to_string(),
        ))
        .with_id("usage_display")
        .with_meta("metric_count", state.metrics.len().to_string())
        .with_meta("layout", "horizontal".to_string());
        let annotated = crate::annotation::Annotate::new(paragraph, annotation);
        frame.render_widget(annotated, area);
    }

    /// Renders vertical layout.
    fn view_vertical(state: &UsageDisplayState, frame: &mut Frame, area: Rect, theme: &Theme) {
        let mut block = Block::default().borders(Borders::ALL);
        if let Some(title) = &state.title {
            block = block.title(format!(" {} ", title));
        }
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let lines: Vec<Line<'static>> = state
            .metrics
            .iter()
            .map(|metric| Line::from(Self::metric_spans(metric, theme)))
            .collect();

        let paragraph = Paragraph::new(lines);

        let annotation = crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
            "UsageDisplay".to_string(),
        ))
        .with_id("usage_display")
        .with_meta("metric_count", state.metrics.len().to_string())
        .with_meta("layout", "vertical".to_string());
        let annotated = crate::annotation::Annotate::new(paragraph, annotation);
        frame.render_widget(annotated, inner);
    }

    /// Renders grid layout.
    fn view_grid(
        state: &UsageDisplayState,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        columns: usize,
    ) {
        let columns = columns.max(1);

        let mut block = Block::default().borders(Borders::ALL);
        if let Some(title) = &state.title {
            block = block.title(format!(" {} ", title));
        }
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if state.metrics.is_empty() || inner.width == 0 || inner.height == 0 {
            return;
        }

        let col_width = inner.width / columns as u16;
        if col_width == 0 {
            return;
        }

        let rows: Vec<&[UsageMetric]> = state.metrics.chunks(columns).collect();

        let mut lines: Vec<Line<'static>> = Vec::new();
        for row in &rows {
            let mut spans: Vec<Span<'static>> = Vec::new();
            for (col_idx, metric) in row.iter().enumerate() {
                let metric_spans = Self::metric_spans(metric, theme);
                let metric_text_len: usize = metric_spans.iter().map(|s| s.content.len()).sum();
                spans.extend(metric_spans);

                // Pad to column width for all but last column
                if col_idx < columns - 1 {
                    let padding = (col_width as usize).saturating_sub(metric_text_len);
                    if padding > 0 {
                        spans.push(Span::raw(" ".repeat(padding)));
                    }
                }
            }
            lines.push(Line::from(spans));
        }

        let paragraph = Paragraph::new(lines);

        let annotation = crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
            "UsageDisplay".to_string(),
        ))
        .with_id("usage_display")
        .with_meta("metric_count", state.metrics.len().to_string())
        .with_meta("layout", format!("grid({})", columns));
        let annotated = crate::annotation::Annotate::new(paragraph, annotation);
        frame.render_widget(annotated, inner);
    }
}

impl Component for UsageDisplay {
    type State = UsageDisplayState;
    type Message = UsageDisplayMessage;
    type Output = ();

    fn init() -> Self::State {
        UsageDisplayState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            UsageDisplayMessage::SetMetrics(metrics) => {
                state.metrics = metrics;
            }
            UsageDisplayMessage::AddMetric(metric) => {
                state.metrics.push(metric);
            }
            UsageDisplayMessage::RemoveMetric(label) => {
                state.metrics.retain(|m| m.label != label);
            }
            UsageDisplayMessage::UpdateValue { label, value } => {
                if let Some(metric) = state.metrics.iter_mut().find(|m| m.label == label) {
                    metric.value = value;
                }
            }
            UsageDisplayMessage::UpdateColor { label, color } => {
                if let Some(metric) = state.metrics.iter_mut().find(|m| m.label == label) {
                    metric.color = color;
                }
            }
            UsageDisplayMessage::SetLayout(layout) => {
                state.layout = layout;
            }
            UsageDisplayMessage::SetTitle(title) => {
                state.title = title;
            }
            UsageDisplayMessage::SetSeparator(separator) => {
                state.separator = separator;
            }
            UsageDisplayMessage::Clear => {
                state.metrics.clear();
            }
        }
        None // Display-only, no output
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if state.metrics.is_empty() || ctx.area.width == 0 || ctx.area.height == 0 {
            return;
        }

        match state.layout {
            UsageLayout::Horizontal => Self::view_horizontal(state, ctx.frame, ctx.area, ctx.theme),
            UsageLayout::Vertical => Self::view_vertical(state, ctx.frame, ctx.area, ctx.theme),
            UsageLayout::Grid(cols) => Self::view_grid(state, ctx.frame, ctx.area, ctx.theme, cols),
        }
    }
}

#[cfg(test)]
mod tests;
