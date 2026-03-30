//! Metric widget types for the dashboard.
//!
//! Contains [`MetricKind`] and [`MetricWidget`], used by the
//! [`MetricsDashboard`](super::MetricsDashboard) component.

/// The kind of metric a widget displays.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum MetricKind {
    /// A numeric counter value.
    Counter {
        /// The current value.
        value: i64,
    },
    /// A gauge value with a known range.
    Gauge {
        /// The current value.
        value: u64,
        /// The maximum value.
        max: u64,
    },
    /// A status indicator (up/down).
    Status {
        /// Whether the status is "up" (healthy).
        up: bool,
    },
    /// A text-based metric.
    Text {
        /// The display text.
        text: String,
    },
}

/// A single metric widget in the dashboard.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MetricWidget {
    /// The display label.
    pub(super) label: String,
    /// The metric kind and value.
    pub(super) kind: MetricKind,
    /// Sparkline history (recent values for trend display).
    pub(super) history: Vec<u64>,
    /// Maximum history length.
    pub(super) max_history: usize,
}

impl MetricWidget {
    /// Creates a counter widget.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::counter("Requests", 42);
    /// assert_eq!(widget.label(), "Requests");
    /// assert_eq!(widget.display_value(), "42");
    /// ```
    pub fn counter(label: impl Into<String>, value: i64) -> Self {
        Self {
            label: label.into(),
            kind: MetricKind::Counter { value },
            history: Vec::new(),
            max_history: 20,
        }
    }

    /// Creates a gauge widget with a maximum value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("CPU %", 75, 100);
    /// assert_eq!(widget.display_value(), "75/100");
    /// ```
    pub fn gauge(label: impl Into<String>, value: u64, max: u64) -> Self {
        Self {
            label: label.into(),
            kind: MetricKind::Gauge { value, max },
            history: Vec::new(),
            max_history: 20,
        }
    }

    /// Creates a status indicator widget.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::status("API", true);
    /// assert_eq!(widget.label(), "API");
    /// assert_eq!(widget.display_value(), "UP");
    /// ```
    pub fn status(label: impl Into<String>, up: bool) -> Self {
        Self {
            label: label.into(),
            kind: MetricKind::Status { up },
            history: Vec::new(),
            max_history: 0,
        }
    }

    /// Creates a text metric widget.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::text("Version", "1.2.3");
    /// assert_eq!(widget.label(), "Version");
    /// assert_eq!(widget.display_value(), "1.2.3");
    /// ```
    pub fn text(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: MetricKind::Text { text: text.into() },
            history: Vec::new(),
            max_history: 0,
        }
    }

    /// Sets the maximum history length for sparkline display (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::counter("Ops", 0).with_max_history(50);
    /// assert_eq!(widget.history().len(), 0); // no values yet
    /// ```
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Returns the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::counter("Requests", 0);
    /// assert_eq!(widget.label(), "Requests");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the metric kind.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MetricWidget, MetricKind};
    ///
    /// let widget = MetricWidget::status("DB", false);
    /// assert!(matches!(widget.kind(), MetricKind::Status { up: false }));
    /// ```
    pub fn kind(&self) -> &MetricKind {
        &self.kind
    }

    /// Returns the sparkline history.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::counter("Ops", 0);
    /// assert!(widget.history().is_empty());
    /// ```
    pub fn history(&self) -> &[u64] {
        &self.history
    }

    /// Returns the display value as a string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// assert_eq!(MetricWidget::counter("A", 42).display_value(), "42");
    /// assert_eq!(MetricWidget::gauge("B", 75, 100).display_value(), "75/100");
    /// assert_eq!(MetricWidget::status("C", true).display_value(), "UP");
    /// assert_eq!(MetricWidget::status("D", false).display_value(), "DOWN");
    /// assert_eq!(MetricWidget::text("E", "ok").display_value(), "ok");
    /// ```
    pub fn display_value(&self) -> String {
        match &self.kind {
            MetricKind::Counter { value } => value.to_string(),
            MetricKind::Gauge { value, max } => format!("{}/{}", value, max),
            MetricKind::Status { up } => {
                if *up {
                    "UP".to_string()
                } else {
                    "DOWN".to_string()
                }
            }
            MetricKind::Text { text } => text.clone(),
        }
    }

    /// Sets the counter value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let mut widget = MetricWidget::counter("Requests", 0);
    /// widget.set_counter_value(100);
    /// assert_eq!(widget.display_value(), "100");
    /// ```
    pub fn set_counter_value(&mut self, value: i64) {
        if let MetricKind::Counter { value: ref mut v } = self.kind {
            *v = value;
            if self.max_history > 0 {
                self.history.push(value.unsigned_abs());
                while self.history.len() > self.max_history {
                    self.history.remove(0);
                }
            }
        }
    }

    /// Sets the gauge value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let mut widget = MetricWidget::gauge("Memory", 0, 1024);
    /// widget.set_gauge_value(512);
    /// assert_eq!(widget.display_value(), "512/1024");
    /// ```
    pub fn set_gauge_value(&mut self, value: u64) {
        if let MetricKind::Gauge {
            value: ref mut v,
            max,
        } = self.kind
        {
            *v = value.min(max);
            if self.max_history > 0 {
                self.history.push(value);
                while self.history.len() > self.max_history {
                    self.history.remove(0);
                }
            }
        }
    }

    /// Sets the status.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let mut widget = MetricWidget::status("API", true);
    /// widget.set_status(false);
    /// assert_eq!(widget.display_value(), "DOWN");
    /// ```
    pub fn set_status(&mut self, up: bool) {
        if let MetricKind::Status { up: ref mut u } = self.kind {
            *u = up;
        }
    }

    /// Sets the text value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let mut widget = MetricWidget::text("Version", "1.0");
    /// widget.set_text("2.0");
    /// assert_eq!(widget.display_value(), "2.0");
    /// ```
    pub fn set_text(&mut self, text: impl Into<String>) {
        if let MetricKind::Text { text: ref mut t } = self.kind {
            *t = text.into();
        }
    }

    /// Increments a counter by the given amount.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let mut widget = MetricWidget::counter("Hits", 10);
    /// widget.increment(5);
    /// assert_eq!(widget.display_value(), "15");
    /// ```
    pub fn increment(&mut self, amount: i64) {
        if let MetricKind::Counter { ref mut value } = self.kind {
            *value += amount;
            if self.max_history > 0 {
                self.history.push(value.unsigned_abs());
                while self.history.len() > self.max_history {
                    self.history.remove(0);
                }
            }
        }
    }

    /// Returns the gauge fill percentage (0.0 to 1.0).
    ///
    /// Returns `None` for non-gauge widgets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("CPU", 75, 100);
    /// assert_eq!(widget.gauge_percentage(), Some(0.75));
    ///
    /// let counter = MetricWidget::counter("Ops", 10);
    /// assert_eq!(counter.gauge_percentage(), None);
    /// ```
    pub fn gauge_percentage(&self) -> Option<f64> {
        match &self.kind {
            MetricKind::Gauge { value, max } if *max > 0 => Some(*value as f64 / *max as f64),
            _ => None,
        }
    }
}
