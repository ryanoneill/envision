//! Supporting types for the AlertPanel component.
//!
//! Defines [`AlertState`], [`AlertThreshold`], and [`AlertMetric`] -- the
//! building blocks used by [`AlertPanelState`](super::AlertPanelState).

/// Alert state for a metric.
///
/// Represents the severity level based on threshold evaluation.
///
/// # Example
///
/// ```rust
/// use envision::component::AlertState;
///
/// let state = AlertState::default();
/// assert_eq!(state, AlertState::Ok);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum AlertState {
    /// Value is within acceptable range.
    #[default]
    Ok,
    /// Value has crossed the warning threshold.
    Warning,
    /// Value has crossed the critical threshold.
    Critical,
    /// State cannot be determined.
    Unknown,
}

impl std::fmt::Display for AlertState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertState::Ok => write!(f, "OK"),
            AlertState::Warning => write!(f, "WARN"),
            AlertState::Critical => write!(f, "CRIT"),
            AlertState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Configuration for alert thresholds.
///
/// Defines the boundaries between OK, Warning, and Critical states.
/// Values below `warning` are OK, values at or above `warning` but
/// below `critical` are Warning, and values at or above `critical`
/// are Critical.
///
/// # Example
///
/// ```rust
/// use envision::component::AlertThreshold;
///
/// let threshold = AlertThreshold::new(70.0, 90.0);
/// assert_eq!(threshold.warning, 70.0);
/// assert_eq!(threshold.critical, 90.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct AlertThreshold {
    /// Value at or above which Warning triggers.
    pub warning: f64,
    /// Value at or above which Critical triggers.
    pub critical: f64,
}

impl AlertThreshold {
    /// Creates a new threshold configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertThreshold;
    ///
    /// let t = AlertThreshold::new(75.0, 95.0);
    /// assert_eq!(t.warning, 75.0);
    /// assert_eq!(t.critical, 95.0);
    /// ```
    pub fn new(warning: f64, critical: f64) -> Self {
        Self { warning, critical }
    }
}

/// A single alert metric with threshold-based state evaluation.
///
/// Each metric has an identifier, display name, current value, optional
/// units, threshold configuration, and history tracking for sparkline
/// display.
///
/// # Example
///
/// ```rust
/// use envision::component::{AlertMetric, AlertThreshold, AlertState};
///
/// let metric = AlertMetric::new("cpu", "CPU Usage", AlertThreshold::new(70.0, 90.0))
///     .with_units("%")
///     .with_value(45.0);
///
/// assert_eq!(metric.id(), "cpu");
/// assert_eq!(metric.name(), "CPU Usage");
/// assert_eq!(metric.state(), &AlertState::Ok);
/// assert_eq!(metric.display_value(), "45.0%");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct AlertMetric {
    /// Unique identifier.
    pub(super) id: String,
    /// Display name.
    name: String,
    /// Current value.
    value: f64,
    /// Units (e.g., "ms", "%", "MB").
    units: Option<String>,
    /// Threshold configuration.
    threshold: AlertThreshold,
    /// Current computed alert state.
    pub(super) state: AlertState,
    /// History of recent values for sparkline.
    history: Vec<f64>,
    /// Maximum history points to retain.
    max_history: usize,
}

impl AlertMetric {
    /// Creates a new alert metric with the given id, name, and threshold.
    ///
    /// The initial value is 0.0 and state is computed as OK (assuming
    /// thresholds are above 0).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold, AlertState};
    ///
    /// let metric = AlertMetric::new("disk", "Disk I/O", AlertThreshold::new(100.0, 200.0));
    /// assert_eq!(metric.id(), "disk");
    /// assert_eq!(metric.name(), "Disk I/O");
    /// assert_eq!(metric.value(), 0.0);
    /// assert_eq!(metric.state(), &AlertState::Ok);
    /// ```
    pub fn new(id: impl Into<String>, name: impl Into<String>, threshold: AlertThreshold) -> Self {
        let mut metric = Self {
            id: id.into(),
            name: name.into(),
            value: 0.0,
            units: None,
            threshold,
            state: AlertState::Ok,
            history: Vec::new(),
            max_history: 20,
        };
        metric.state = metric.compute_state();
        metric
    }

    /// Sets the units string (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///     .with_units("%");
    /// assert_eq!(metric.display_value(), "0.0%");
    /// ```
    pub fn with_units(mut self, units: impl Into<String>) -> Self {
        self.units = Some(units.into());
        self
    }

    /// Sets the initial value and recomputes state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold, AlertState};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///     .with_value(85.0);
    /// assert_eq!(metric.state(), &AlertState::Warning);
    /// ```
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self.state = self.compute_state();
        self
    }

    /// Sets the maximum history size (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///     .with_max_history(50);
    /// assert_eq!(metric.max_history(), 50);
    /// ```
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Updates the value, pushes to history, and recomputes alert state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold, AlertState};
    ///
    /// let mut metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0));
    /// metric.update_value(75.0);
    /// assert_eq!(metric.value(), 75.0);
    /// assert_eq!(metric.state(), &AlertState::Warning);
    /// assert_eq!(metric.history().len(), 1);
    /// ```
    pub fn update_value(&mut self, value: f64) {
        self.value = value;
        self.history.push(value);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        self.state = self.compute_state();
    }

    /// Computes the alert state from the current value and thresholds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold, AlertState};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///     .with_value(95.0);
    /// assert_eq!(metric.compute_state(), AlertState::Critical);
    /// ```
    pub fn compute_state(&self) -> AlertState {
        if self.value >= self.threshold.critical {
            AlertState::Critical
        } else if self.value >= self.threshold.warning {
            AlertState::Warning
        } else {
            AlertState::Ok
        }
    }

    /// Formats the display string with value and optional units.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///     .with_units("%")
    ///     .with_value(45.2);
    /// assert_eq!(metric.display_value(), "45.2%");
    ///
    /// let metric2 = AlertMetric::new("req", "Requests", AlertThreshold::new(1000.0, 5000.0))
    ///     .with_value(123.0);
    /// assert_eq!(metric2.display_value(), "123.0");
    /// ```
    pub fn display_value(&self) -> String {
        match &self.units {
            Some(units) => format!("{:.1}{}", self.value, units),
            None => format!("{:.1}", self.value),
        }
    }

    /// Returns the unique identifier.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0));
    /// assert_eq!(metric.id(), "cpu");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the display name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU Usage", AlertThreshold::new(70.0, 90.0));
    /// assert_eq!(metric.name(), "CPU Usage");
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the current value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///     .with_value(55.0);
    /// assert_eq!(metric.value(), 55.0);
    /// ```
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns the units, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///     .with_units("ms");
    /// assert_eq!(metric.units(), Some("ms"));
    /// ```
    pub fn units(&self) -> Option<&str> {
        self.units.as_deref()
    }

    /// Returns the threshold configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let threshold = AlertThreshold::new(70.0, 90.0);
    /// let metric = AlertMetric::new("cpu", "CPU", threshold.clone());
    /// assert_eq!(metric.threshold(), &threshold);
    /// ```
    pub fn threshold(&self) -> &AlertThreshold {
        &self.threshold
    }

    /// Returns the current alert state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold, AlertState};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///     .with_value(75.0);
    /// assert_eq!(metric.state(), &AlertState::Warning);
    /// ```
    pub fn state(&self) -> &AlertState {
        &self.state
    }

    /// Returns the history of recent values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let mut metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0));
    /// metric.update_value(10.0);
    /// metric.update_value(20.0);
    /// assert_eq!(metric.history(), &[10.0, 20.0]);
    /// ```
    pub fn history(&self) -> &[f64] {
        &self.history
    }

    /// Returns the maximum history size.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertMetric, AlertThreshold};
    ///
    /// let metric = AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0));
    /// assert_eq!(metric.max_history(), 20);
    /// ```
    pub fn max_history(&self) -> usize {
        self.max_history
    }
}
