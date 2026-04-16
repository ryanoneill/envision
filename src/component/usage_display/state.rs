//! UsageDisplayState constructors, builders, accessors, setters, and instance methods.
//!
//! Extracted from the main usage_display module to keep file sizes manageable.

use ratatui::style::Color;

use super::{UsageDisplayState, UsageLayout, UsageMetric};

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
