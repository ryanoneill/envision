//! A threshold-based alert panel for metric monitoring.
//!
//! [`AlertPanel`] displays a grid of metrics, each with configurable
//! thresholds that determine alert states (OK, Warning, Critical, Unknown).
//! Each metric card shows a state indicator, current value with units, and
//! an optional sparkline history. The title bar summarizes aggregate counts.
//!
//! State is stored in [`AlertPanelState`], updated via [`AlertPanelMessage`],
//! and produces [`AlertPanelOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     AlertPanel, AlertPanelState, AlertMetric, AlertThreshold, AlertState,
//!     Component,
//! };
//!
//! let metrics = vec![
//!     AlertMetric::new("cpu", "CPU Usage", AlertThreshold::new(70.0, 90.0))
//!         .with_units("%")
//!         .with_value(45.0),
//!     AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
//!         .with_units("%")
//!         .with_value(82.0),
//! ];
//!
//! let state = AlertPanelState::new()
//!     .with_metrics(metrics)
//!     .with_columns(2);
//!
//! assert_eq!(state.metrics().len(), 2);
//! assert_eq!(state.ok_count(), 1);
//! assert_eq!(state.warning_count(), 1);
//! ```

mod metric;
mod render;

pub use metric::{AlertMetric, AlertState, AlertThreshold};

use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Messages that can be sent to an AlertPanel.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     AlertPanel, AlertPanelState, AlertPanelMessage, AlertMetric, AlertThreshold,
///     Component,
/// };
///
/// let mut state = AlertPanelState::new().with_metrics(vec![
///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)).with_value(50.0),
/// ]);
/// let output = state.update(AlertPanelMessage::UpdateMetric {
///     id: "cpu".into(),
///     value: 80.0,
/// });
/// assert!(output.is_some());
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum AlertPanelMessage {
    /// Update a metric's value by id.
    UpdateMetric {
        /// The metric identifier.
        id: String,
        /// The new value.
        value: f64,
    },
    /// Add a new metric.
    AddMetric(AlertMetric),
    /// Remove a metric by id.
    RemoveMetric(String),
    /// Replace all metrics.
    SetMetrics(Vec<AlertMetric>),
    /// Select the next metric.
    SelectNext,
    /// Select the previous metric.
    SelectPrev,
    /// Navigate up in the grid.
    SelectUp,
    /// Navigate down in the grid.
    SelectDown,
    /// Set the number of grid columns.
    SetColumns(usize),
    /// Confirm selection of the current metric.
    Select,
}

/// Output messages from an AlertPanel.
///
/// # Example
///
/// ```rust
/// use envision::component::{AlertPanelOutput, AlertState};
///
/// let output = AlertPanelOutput::StateChanged {
///     id: "cpu".into(),
///     old: AlertState::Ok,
///     new_state: AlertState::Warning,
/// };
/// assert!(matches!(output, AlertPanelOutput::StateChanged { .. }));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum AlertPanelOutput {
    /// A metric changed alert state.
    StateChanged {
        /// The metric identifier.
        id: String,
        /// The previous alert state.
        old: AlertState,
        /// The new alert state.
        new_state: AlertState,
    },
    /// A metric was selected (Enter pressed).
    MetricSelected(String),
}

/// State for the AlertPanel component.
///
/// Contains the metrics, layout configuration, and navigation state.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     AlertPanelState, AlertMetric, AlertThreshold,
/// };
///
/// let state = AlertPanelState::new()
///     .with_metrics(vec![
///         AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
///             .with_value(45.0),
///     ])
///     .with_columns(2)
///     .with_title("Alerts");
///
/// assert_eq!(state.metrics().len(), 1);
/// assert_eq!(state.ok_count(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct AlertPanelState {
    /// The alert metrics.
    metrics: Vec<AlertMetric>,
    /// Number of columns in the grid layout.
    columns: usize,
    /// Currently selected metric index.
    selected: Option<usize>,
    /// Optional title.
    title: Option<String>,
    /// Whether to show sparkline history.
    show_sparklines: bool,
    /// Whether to show threshold values.
    show_thresholds: bool,
}

impl Default for AlertPanelState {
    fn default() -> Self {
        Self {
            metrics: Vec::new(),
            columns: 2,
            selected: None,
            title: None,
            show_sparklines: true,
            show_thresholds: false,
        }
    }
}

impl AlertPanelState {
    /// Creates a new empty alert panel state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new();
    /// assert!(state.metrics().is_empty());
    /// assert_eq!(state.columns(), 2);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial metrics (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)),
    /// ]);
    /// assert_eq!(state.metrics().len(), 1);
    /// ```
    pub fn with_metrics(mut self, metrics: Vec<AlertMetric>) -> Self {
        self.selected = if metrics.is_empty() { None } else { Some(0) };
        self.metrics = metrics;
        self
    }

    /// Sets the number of grid columns (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new().with_columns(3);
    /// assert_eq!(state.columns(), 3);
    /// ```
    pub fn with_columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new().with_title("System Alerts");
    /// assert_eq!(state.title(), Some("System Alerts"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show sparklines (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new().with_show_sparklines(false);
    /// assert!(!state.show_sparklines());
    /// ```
    pub fn with_show_sparklines(mut self, show: bool) -> Self {
        self.show_sparklines = show;
        self
    }

    /// Sets whether to show threshold values (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new().with_show_thresholds(true);
    /// assert!(state.show_thresholds());
    /// ```
    pub fn with_show_thresholds(mut self, show: bool) -> Self {
        self.show_thresholds = show;
        self
    }

    // ---- Accessors ----

    /// Returns the metrics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)),
    /// ]);
    /// assert_eq!(state.metrics().len(), 1);
    /// ```
    pub fn metrics(&self) -> &[AlertMetric] {
        &self.metrics
    }

    /// Returns a mutable reference to the alert metrics.
    ///
    /// This is safe because metrics are simple data containers.
    /// Selection state is index-based and unaffected by value mutation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let mut state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///         .with_value(50.0),
    /// ]);
    /// state.metrics_mut()[0].update_value(85.0);
    /// assert_eq!(state.metrics()[0].value(), 85.0);
    /// ```
    pub fn metrics_mut(&mut self) -> &mut Vec<AlertMetric> {
        &mut self.metrics
    }

    /// Returns the number of grid columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new().with_columns(4);
    /// assert_eq!(state.columns(), 4);
    /// ```
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Returns the selected metric index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)),
    /// ]);
    /// assert_eq!(state.selected(), Some(0));
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new().with_title("Alerts");
    /// assert_eq!(state.title(), Some("Alerts"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let mut state = AlertPanelState::new();
    /// state.set_title("System Alerts");
    /// assert_eq!(state.title(), Some("System Alerts"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns whether sparklines are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new();
    /// assert!(state.show_sparklines());
    /// ```
    pub fn show_sparklines(&self) -> bool {
        self.show_sparklines
    }

    /// Returns whether threshold values are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new();
    /// assert!(!state.show_thresholds());
    /// ```
    pub fn show_thresholds(&self) -> bool {
        self.show_thresholds
    }

    /// Sets whether sparklines are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let mut state = AlertPanelState::new();
    /// state.set_show_sparklines(false);
    /// assert!(!state.show_sparklines());
    /// ```
    pub fn set_show_sparklines(&mut self, show: bool) {
        self.show_sparklines = show;
    }

    /// Sets whether threshold values are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let mut state = AlertPanelState::new();
    /// state.set_show_thresholds(true);
    /// assert!(state.show_thresholds());
    /// ```
    pub fn set_show_thresholds(&mut self, show: bool) {
        self.show_thresholds = show;
    }

    /// Adds a metric to the panel.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let mut state = AlertPanelState::new();
    /// state.add_metric(
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    /// );
    /// assert_eq!(state.metrics().len(), 1);
    /// assert_eq!(state.selected(), Some(0));
    /// ```
    pub fn add_metric(&mut self, metric: AlertMetric) {
        self.metrics.push(metric);
        if self.selected.is_none() {
            self.selected = Some(0);
        }
    }

    /// Updates a metric's value by id.
    ///
    /// Returns `Some((old_state, new_state))` if the alert state changed,
    /// or `None` if the state did not change or the metric was not found.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold, AlertState};
    ///
    /// let mut state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///         .with_value(50.0),
    /// ]);
    /// let result = state.update_metric("cpu", 80.0);
    /// assert_eq!(result, Some((AlertState::Ok, AlertState::Warning)));
    /// ```
    pub fn update_metric(&mut self, id: &str, value: f64) -> Option<(AlertState, AlertState)> {
        if let Some(metric) = self.metrics.iter_mut().find(|m| m.id == id) {
            let old_state = metric.state.clone();
            metric.update_value(value);
            let new_state = metric.state.clone();
            if old_state != new_state {
                Some((old_state, new_state))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns a reference to a metric by id.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)),
    /// ]);
    /// assert!(state.metric_by_id("cpu").is_some());
    /// assert!(state.metric_by_id("unknown").is_none());
    /// ```
    pub fn metric_by_id(&self, id: &str) -> Option<&AlertMetric> {
        self.metrics.iter().find(|m| m.id == id)
    }

    /// Returns the count of metrics in OK state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///         .with_value(50.0),
    ///     AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
    ///         .with_value(30.0),
    /// ]);
    /// assert_eq!(state.ok_count(), 2);
    /// ```
    pub fn ok_count(&self) -> usize {
        self.metrics
            .iter()
            .filter(|m| m.state == AlertState::Ok)
            .count()
    }

    /// Returns the count of metrics in Warning state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///         .with_value(80.0),
    /// ]);
    /// assert_eq!(state.warning_count(), 1);
    /// ```
    pub fn warning_count(&self) -> usize {
        self.metrics
            .iter()
            .filter(|m| m.state == AlertState::Warning)
            .count()
    }

    /// Returns the count of metrics in Critical state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
    ///         .with_value(95.0),
    /// ]);
    /// assert_eq!(state.critical_count(), 1);
    /// ```
    pub fn critical_count(&self) -> usize {
        self.metrics
            .iter()
            .filter(|m| m.state == AlertState::Critical)
            .count()
    }

    /// Returns the count of metrics in Unknown state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AlertPanelState;
    ///
    /// let state = AlertPanelState::new();
    /// assert_eq!(state.unknown_count(), 0);
    /// ```
    pub fn unknown_count(&self) -> usize {
        self.metrics
            .iter()
            .filter(|m| m.state == AlertState::Unknown)
            .count()
    }

    /// Returns a reference to the currently selected metric.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AlertPanelState, AlertMetric, AlertThreshold};
    ///
    /// let state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)),
    /// ]);
    /// assert_eq!(state.selected_metric().unwrap().id(), "cpu");
    /// ```
    pub fn selected_metric(&self) -> Option<&AlertMetric> {
        self.metrics.get(self.selected?)
    }

    /// Returns the number of rows in the grid.
    pub fn rows(&self) -> usize {
        if self.metrics.is_empty() {
            0
        } else {
            self.metrics.len().div_ceil(self.columns)
        }
    }

    /// Builds the title string with aggregate state counts.
    pub(crate) fn title_with_counts(&self) -> String {
        let base = self.title.as_deref().unwrap_or("Alerts");
        let ok = self.ok_count();
        let warn = self.warning_count();
        let crit = self.critical_count();
        let unknown = self.unknown_count();

        let mut parts = Vec::new();
        if ok > 0 {
            parts.push(format!("{} OK", ok));
        }
        if warn > 0 {
            parts.push(format!("{} WARN", warn));
        }
        if crit > 0 {
            parts.push(format!("{} CRIT", crit));
        }
        if unknown > 0 {
            parts.push(format!("{} UNKNOWN", unknown));
        }

        if parts.is_empty() {
            base.to_string()
        } else {
            format!("{} ({})", base, parts.join(", "))
        }
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     AlertPanelState, AlertPanelMessage, AlertPanelOutput,
    ///     AlertMetric, AlertThreshold,
    /// };
    ///
    /// let mut state = AlertPanelState::new().with_metrics(vec![
    ///     AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0)),
    /// ]);
    /// let output = state.update(AlertPanelMessage::Select);
    /// assert_eq!(output, Some(AlertPanelOutput::MetricSelected("cpu".into())));
    /// ```
    pub fn update(&mut self, msg: AlertPanelMessage) -> Option<AlertPanelOutput> {
        AlertPanel::update(self, msg)
    }
}

/// A threshold-based alert panel component.
///
/// Displays metrics in a grid layout with visual state indicators,
/// sparkline history, and keyboard navigation.
///
/// # Key Bindings
///
/// - `Left` / `h` -- Move selection left
/// - `Right` / `l` -- Move selection right
/// - `Up` / `k` -- Move selection up
/// - `Down` / `j` -- Move selection down
/// - `Enter` -- Confirm selection
pub struct AlertPanel(PhantomData<()>);

impl Component for AlertPanel {
    type State = AlertPanelState;
    type Message = AlertPanelMessage;
    type Output = AlertPanelOutput;

    fn init() -> Self::State {
        AlertPanelState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => Some(AlertPanelMessage::SelectPrev),
            KeyCode::Right | KeyCode::Char('l') => Some(AlertPanelMessage::SelectNext),
            KeyCode::Up | KeyCode::Char('k') => Some(AlertPanelMessage::SelectUp),
            KeyCode::Down | KeyCode::Char('j') => Some(AlertPanelMessage::SelectDown),
            KeyCode::Enter => Some(AlertPanelMessage::Select),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            AlertPanelMessage::UpdateMetric { id, value } => {
                if let Some(metric) = state.metrics.iter_mut().find(|m| m.id == id) {
                    let old_state = metric.state.clone();
                    metric.update_value(value);
                    let new_state = metric.state.clone();
                    if old_state != new_state {
                        return Some(AlertPanelOutput::StateChanged {
                            id,
                            old: old_state,
                            new_state,
                        });
                    }
                }
                None
            }
            AlertPanelMessage::AddMetric(metric) => {
                state.add_metric(metric);
                None
            }
            AlertPanelMessage::RemoveMetric(id) => {
                state.metrics.retain(|m| m.id != id);
                if state.metrics.is_empty() {
                    state.selected = None;
                } else if let Some(sel) = state.selected {
                    if sel >= state.metrics.len() {
                        state.selected = Some(state.metrics.len() - 1);
                    }
                }
                None
            }
            AlertPanelMessage::SetMetrics(metrics) => {
                state.selected = if metrics.is_empty() { None } else { Some(0) };
                state.metrics = metrics;
                None
            }
            AlertPanelMessage::SelectNext => {
                if state.metrics.is_empty() {
                    return None;
                }
                let current = state.selected.unwrap_or(0);
                let cols = state.columns;
                let current_col = current % cols;
                if current_col < cols - 1 && current + 1 < state.metrics.len() {
                    let new_index = current + 1;
                    state.selected = Some(new_index);
                    Some(AlertPanelOutput::MetricSelected(
                        state.metrics[new_index].id.clone(),
                    ))
                } else {
                    None
                }
            }
            AlertPanelMessage::SelectPrev => {
                if state.metrics.is_empty() {
                    return None;
                }
                let current = state.selected.unwrap_or(0);
                let current_col = current % state.columns;
                if current_col > 0 {
                    let new_index = current - 1;
                    state.selected = Some(new_index);
                    Some(AlertPanelOutput::MetricSelected(
                        state.metrics[new_index].id.clone(),
                    ))
                } else {
                    None
                }
            }
            AlertPanelMessage::SelectUp => {
                if state.metrics.is_empty() {
                    return None;
                }
                let current = state.selected.unwrap_or(0);
                let cols = state.columns;
                let current_row = current / cols;
                if current_row > 0 {
                    let new_index = (current_row - 1) * cols + (current % cols);
                    if new_index < state.metrics.len() {
                        state.selected = Some(new_index);
                        return Some(AlertPanelOutput::MetricSelected(
                            state.metrics[new_index].id.clone(),
                        ));
                    }
                }
                None
            }
            AlertPanelMessage::SelectDown => {
                if state.metrics.is_empty() {
                    return None;
                }
                let current = state.selected.unwrap_or(0);
                let cols = state.columns;
                let new_index = (current / cols + 1) * cols + (current % cols);
                if new_index < state.metrics.len() {
                    state.selected = Some(new_index);
                    Some(AlertPanelOutput::MetricSelected(
                        state.metrics[new_index].id.clone(),
                    ))
                } else {
                    None
                }
            }
            AlertPanelMessage::SetColumns(columns) => {
                state.columns = columns.max(1);
                None
            }
            AlertPanelMessage::Select => state
                .selected_metric()
                .map(|metric| AlertPanelOutput::MetricSelected(metric.id.clone())),
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        render::render_alert_panel(state, frame, area, theme, ctx.focused, ctx.disabled);
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
