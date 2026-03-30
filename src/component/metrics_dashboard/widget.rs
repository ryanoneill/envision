//! Metric widget types for the dashboard.
//!
//! Contains [`MetricKind`] and [`MetricWidget`], used by the
//! [`MetricsDashboard`](super::MetricsDashboard) component.

/// The kind of metric a widget displays.
#[derive(Clone, Debug, PartialEq)]
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
pub struct MetricWidget {
    /// The display label.
    pub(super) label: String,
    /// The metric kind and value.
    pub(super) kind: MetricKind,
    /// Sparkline history (recent values for trend display).
    pub(super) history: Vec<u64>,
    /// Maximum history length.
    pub(super) max_history: usize,
    /// Optional warning threshold (0.0 to 1.0) for gauge coloring.
    pub(super) warning_threshold: Option<f64>,
    /// Optional critical threshold (0.0 to 1.0) for gauge coloring.
    pub(super) critical_threshold: Option<f64>,
    /// Optional units string displayed alongside the value.
    pub(super) units: Option<String>,
    /// Optional previous value for delta display.
    pub(super) previous_value: Option<f64>,
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
            warning_threshold: None,
            critical_threshold: None,
            units: None,
            previous_value: None,
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
            warning_threshold: None,
            critical_threshold: None,
            units: None,
            previous_value: None,
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
            warning_threshold: None,
            critical_threshold: None,
            units: None,
            previous_value: None,
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
            warning_threshold: None,
            critical_threshold: None,
            units: None,
            previous_value: None,
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

    /// Sets the warning threshold for gauge coloring (builder pattern).
    ///
    /// The threshold is a fraction from 0.0 to 1.0. When a gauge's fill
    /// percentage reaches this value, the widget displays in the warning color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("CPU", 60, 100)
    ///     .with_warning_threshold(0.6);
    /// assert_eq!(widget.warning_threshold(), Some(0.6));
    /// ```
    pub fn with_warning_threshold(mut self, threshold: f64) -> Self {
        self.warning_threshold = Some(threshold);
        self
    }

    /// Sets the critical threshold for gauge coloring (builder pattern).
    ///
    /// The threshold is a fraction from 0.0 to 1.0. When a gauge's fill
    /// percentage reaches this value, the widget displays in the error color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("CPU", 95, 100)
    ///     .with_critical_threshold(0.95);
    /// assert_eq!(widget.critical_threshold(), Some(0.95));
    /// ```
    pub fn with_critical_threshold(mut self, threshold: f64) -> Self {
        self.critical_threshold = Some(threshold);
        self
    }

    /// Sets the units string displayed alongside the value (builder pattern).
    ///
    /// When set, the display value includes the units, such as "512/1024 MB"
    /// or "75 %".
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("Memory", 512, 1024)
    ///     .with_units("MB");
    /// assert_eq!(widget.display_value(), "512/1024 MB");
    /// ```
    pub fn with_units(mut self, units: impl Into<String>) -> Self {
        self.units = Some(units.into());
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

    /// Returns the warning threshold.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("CPU", 50, 100)
    ///     .with_warning_threshold(0.6);
    /// assert_eq!(widget.warning_threshold(), Some(0.6));
    ///
    /// let plain = MetricWidget::gauge("CPU", 50, 100);
    /// assert_eq!(plain.warning_threshold(), None);
    /// ```
    pub fn warning_threshold(&self) -> Option<f64> {
        self.warning_threshold
    }

    /// Returns the critical threshold.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("CPU", 90, 100)
    ///     .with_critical_threshold(0.85);
    /// assert_eq!(widget.critical_threshold(), Some(0.85));
    ///
    /// let plain = MetricWidget::gauge("CPU", 90, 100);
    /// assert_eq!(plain.critical_threshold(), None);
    /// ```
    pub fn critical_threshold(&self) -> Option<f64> {
        self.critical_threshold
    }

    /// Returns the units string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("Memory", 512, 1024)
    ///     .with_units("MB");
    /// assert_eq!(widget.units(), Some("MB"));
    ///
    /// let plain = MetricWidget::gauge("Memory", 512, 1024);
    /// assert_eq!(plain.units(), None);
    /// ```
    pub fn units(&self) -> Option<&str> {
        self.units.as_deref()
    }

    /// Returns the previous value for delta calculation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let mut widget = MetricWidget::counter("Requests", 100);
    /// widget.set_value_with_previous(150.0, 100.0);
    /// assert_eq!(widget.previous_value(), Some(100.0));
    /// ```
    pub fn previous_value(&self) -> Option<f64> {
        self.previous_value
    }

    /// Returns the display value as a string.
    ///
    /// When units are set, they are appended (e.g. "512/1024 MB").
    /// When a previous value is set, a delta indicator is appended
    /// (e.g. " \u{2191} 5.2%" for an increase).
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
        let base = match &self.kind {
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
        };

        let with_units = if let Some(ref units) = self.units {
            format!("{} {}", base, units)
        } else {
            base
        };

        if let Some(prev) = self.previous_value {
            let current = self.numeric_value();
            let delta = current - prev;
            if delta.abs() < f64::EPSILON {
                with_units
            } else {
                let arrow = if delta > 0.0 { "\u{2191}" } else { "\u{2193}" };
                if prev.abs() > f64::EPSILON {
                    let pct = (delta / prev * 100.0).abs();
                    format!("{} {} {:.1}%", with_units, arrow, pct)
                } else {
                    // When previous is zero, show absolute change.
                    format!("{} {} {:.1}", with_units, arrow, delta.abs())
                }
            }
        } else {
            with_units
        }
    }

    /// Returns the numeric value of this widget as an `f64`.
    ///
    /// Used internally for delta calculations.
    fn numeric_value(&self) -> f64 {
        match &self.kind {
            MetricKind::Counter { value } => *value as f64,
            MetricKind::Gauge { value, .. } => *value as f64,
            MetricKind::Status { up } => {
                if *up {
                    1.0
                } else {
                    0.0
                }
            }
            MetricKind::Text { .. } => 0.0,
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

    /// Sets the current numeric value and records the previous value for delta
    /// display.
    ///
    /// For counter widgets, sets the counter value. For gauge widgets, sets the
    /// gauge value. The previous value is stored for delta calculations in
    /// [`display_value()`](Self::display_value).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let mut widget = MetricWidget::counter("Requests", 100);
    /// widget.set_value_with_previous(150.0, 100.0);
    /// assert_eq!(widget.previous_value(), Some(100.0));
    /// assert!(widget.display_value().contains("\u{2191}"));
    /// ```
    pub fn set_value_with_previous(&mut self, value: f64, previous: f64) {
        self.previous_value = Some(previous);
        match self.kind {
            MetricKind::Counter { .. } => self.set_counter_value(value as i64),
            MetricKind::Gauge { .. } => self.set_gauge_value(value as u64),
            _ => {}
        }
    }

    /// Returns the effective warning threshold for this widget.
    ///
    /// Returns the custom threshold if set, otherwise the default of 0.7.
    pub(super) fn effective_warning_threshold(&self) -> f64 {
        self.warning_threshold.unwrap_or(0.7)
    }

    /// Returns the effective critical threshold for this widget.
    ///
    /// Returns the custom threshold if set, otherwise the default of 0.9.
    pub(super) fn effective_critical_threshold(&self) -> f64 {
        self.critical_threshold.unwrap_or(0.9)
    }

    /// Builds a visual gauge bar string for the given width.
    ///
    /// Uses block characters to show fill percentage with sub-cell resolution.
    /// Returns `None` for non-gauge widgets or when width is zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MetricWidget;
    ///
    /// let widget = MetricWidget::gauge("CPU", 50, 100);
    /// let bar = widget.gauge_bar(10).unwrap();
    /// assert_eq!(bar.chars().count(), 10);
    /// ```
    pub fn gauge_bar(&self, width: usize) -> Option<String> {
        if width == 0 {
            return None;
        }
        let pct = self.gauge_percentage()?;
        let fill = pct * width as f64;
        let full_blocks = fill as usize;
        let fractional = fill - full_blocks as f64;

        // Block characters for sub-cell resolution
        let sub_blocks = [
            ' ', '\u{258F}', '\u{258E}', '\u{258D}', '\u{258C}', '\u{258B}', '\u{258A}',
            '\u{2589}', '\u{2588}',
        ];

        let mut bar = String::with_capacity(width * 4);
        for _ in 0..full_blocks.min(width) {
            bar.push('\u{2588}');
        }
        if full_blocks < width {
            let idx = (fractional * 8.0).round() as usize;
            let idx = idx.min(8);
            bar.push(sub_blocks[idx]);
            for _ in (full_blocks + 1)..width {
                bar.push(' ');
            }
        }

        Some(bar)
    }
}
