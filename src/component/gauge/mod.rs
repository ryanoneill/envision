//! A ratio and measurement display component.
//!
//! [`Gauge`] provides a visual fill bar for displaying ratios and measurements
//! such as CPU usage, memory consumption, disk utilization, and other metrics.
//! This is a **display-only** component that does not receive keyboard focus.
//! State is stored in [`GaugeState`], updated via [`GaugeMessage`], and
//! produces [`GaugeOutput`] (unit type `()`).
//!
//! Unlike [`ProgressBar`](super::ProgressBar) which tracks task completion,
//! `Gauge` is designed for showing current measurements against a maximum.
//! It supports configurable threshold zones that change the bar color based
//! on the current value (e.g., green for normal, yellow for warning, red for
//! critical).
//!
//! Two visual variants are supported:
//! - [`GaugeVariant::Full`]: A block-fill gauge with centered label (ratatui `Gauge`)
//! - [`GaugeVariant::Line`]: A compact single-line gauge (ratatui `LineGauge`)
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Gauge, GaugeMessage, GaugeState, GaugeVariant, Component};
//!
//! // Create a gauge for CPU usage
//! let mut state = GaugeState::new(45.0, 100.0)
//!     .with_units("%")
//!     .with_title("CPU Usage");
//! assert_eq!(state.value(), 45.0);
//! assert_eq!(state.max(), 100.0);
//!
//! // Update the value
//! Gauge::update(&mut state, GaugeMessage::SetValue(75.0));
//! assert_eq!(state.value(), 75.0);
//! assert_eq!(state.display_percentage(), 75);
//!
//! // Check threshold color (75% is in the yellow zone by default)
//! assert_eq!(state.current_color(), ratatui::style::Color::Yellow);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge as RatatuiGauge, LineGauge};

use super::{Component, Disableable};
use crate::theme::Theme;

/// The visual variant of the gauge.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum GaugeVariant {
    /// Full block gauge with centered label (ratatui Gauge).
    #[default]
    Full,
    /// Single-line compact gauge (ratatui LineGauge).
    Line,
}

/// A threshold zone with a color and breakpoint.
///
/// Threshold zones define color changes based on the gauge's current
/// percentage. When the gauge percentage is at or above the `above` value,
/// and below any higher threshold, this zone's color is used.
///
/// # Example
///
/// ```rust
/// use envision::component::ThresholdZone;
/// use ratatui::style::Color;
///
/// let zone = ThresholdZone {
///     above: 0.7,
///     color: Color::Yellow,
/// };
/// assert_eq!(zone.above, 0.7);
/// assert_eq!(zone.color, Color::Yellow);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ThresholdZone {
    /// Values at or above this percentage trigger this zone's color.
    pub above: f64,
    /// The color for this zone.
    pub color: Color,
}

/// Messages that can be sent to a Gauge.
#[derive(Clone, Debug, PartialEq)]
pub enum GaugeMessage {
    /// Set the current value.
    SetValue(f64),
    /// Set the maximum value.
    SetMax(f64),
    /// Set the label.
    SetLabel(Option<String>),
    /// Set the units display string.
    SetUnits(Option<String>),
}

/// Output messages from a Gauge.
///
/// The Gauge is display-only so no meaningful output is emitted.
pub type GaugeOutput = ();

/// Returns the default threshold zones: green (0%), yellow (70%), red (90%).
fn default_thresholds() -> Vec<ThresholdZone> {
    vec![
        ThresholdZone {
            above: 0.0,
            color: Color::Green,
        },
        ThresholdZone {
            above: 0.7,
            color: Color::Yellow,
        },
        ThresholdZone {
            above: 0.9,
            color: Color::Red,
        },
    ]
}

/// State for a Gauge component.
///
/// Contains the current value, maximum, display options, and threshold
/// configuration. Use the builder methods to configure the gauge after
/// construction.
///
/// # Example
///
/// ```rust
/// use envision::component::{GaugeState, GaugeVariant, ThresholdZone};
/// use ratatui::style::Color;
///
/// let state = GaugeState::new(512.0, 1024.0)
///     .with_units("MB")
///     .with_title("Memory")
///     .with_variant(GaugeVariant::Line);
///
/// assert_eq!(state.value(), 512.0);
/// assert_eq!(state.max(), 1024.0);
/// assert_eq!(state.display_percentage(), 50);
/// assert_eq!(state.label_text(), "512.0 / 1024.0 MB");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct GaugeState {
    /// The current value.
    value: f64,
    /// The maximum value.
    max: f64,
    /// Optional custom label (overrides the default formatted label).
    label: Option<String>,
    /// Optional units display string (e.g., "MB", "ms", "%").
    units: Option<String>,
    /// The visual variant (Full or Line).
    variant: GaugeVariant,
    /// Threshold zones sorted by `above` ascending.
    thresholds: Vec<ThresholdZone>,
    /// Optional border title.
    title: Option<String>,
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for GaugeState {
    fn default() -> Self {
        Self {
            value: 0.0,
            max: 100.0,
            label: None,
            units: None,
            variant: GaugeVariant::default(),
            thresholds: default_thresholds(),
            title: None,
            disabled: false,
        }
    }
}

impl GaugeState {
    /// Creates a new gauge with the given value and maximum.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GaugeState;
    ///
    /// let state = GaugeState::new(25.0, 100.0);
    /// assert_eq!(state.value(), 25.0);
    /// assert_eq!(state.max(), 100.0);
    /// assert_eq!(state.display_percentage(), 25);
    /// ```
    pub fn new(value: f64, max: f64) -> Self {
        Self {
            value,
            max,
            ..Self::default()
        }
    }

    /// Sets the label using builder pattern.
    ///
    /// When a custom label is set, it replaces the default formatted label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GaugeState;
    ///
    /// let state = GaugeState::new(50.0, 100.0).with_label("Half full");
    /// assert_eq!(state.label_text(), "Half full");
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the units display string using builder pattern.
    ///
    /// Units are appended to the formatted label (e.g., "512.0 / 1024.0 MB").
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GaugeState;
    ///
    /// let state = GaugeState::new(512.0, 1024.0).with_units("MB");
    /// assert_eq!(state.label_text(), "512.0 / 1024.0 MB");
    /// ```
    pub fn with_units(mut self, units: impl Into<String>) -> Self {
        self.units = Some(units.into());
        self
    }

    /// Sets the visual variant using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{GaugeState, GaugeVariant};
    ///
    /// let state = GaugeState::new(50.0, 100.0)
    ///     .with_variant(GaugeVariant::Line);
    /// ```
    pub fn with_variant(mut self, variant: GaugeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets custom threshold zones using builder pattern.
    ///
    /// Thresholds should be sorted by `above` ascending. They are re-sorted
    /// internally to ensure correct behavior.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{GaugeState, ThresholdZone};
    /// use ratatui::style::Color;
    ///
    /// let state = GaugeState::new(30.0, 100.0)
    ///     .with_thresholds(vec![
    ///         ThresholdZone { above: 0.0, color: Color::Blue },
    ///         ThresholdZone { above: 0.5, color: Color::Cyan },
    ///     ]);
    /// assert_eq!(state.current_color(), Color::Blue);
    /// ```
    pub fn with_thresholds(mut self, mut thresholds: Vec<ThresholdZone>) -> Self {
        thresholds.sort_by(|a, b| {
            a.above
                .partial_cmp(&b.above)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.thresholds = thresholds;
        self
    }

    /// Sets the border title using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GaugeState;
    ///
    /// let state = GaugeState::new(50.0, 100.0).with_title("CPU Usage");
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the disabled state using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GaugeState;
    ///
    /// let state = GaugeState::new(50.0, 100.0).with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the current value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Returns the maximum value.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Sets the current value.
    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }

    /// Sets the maximum value.
    pub fn set_max(&mut self, max: f64) {
        self.max = max;
    }

    /// Returns the percentage as a ratio clamped to 0.0..=1.0.
    ///
    /// If `max` is zero or negative, returns 0.0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GaugeState;
    ///
    /// let state = GaugeState::new(75.0, 100.0);
    /// assert!((state.percentage() - 0.75).abs() < f64::EPSILON);
    ///
    /// let state = GaugeState::new(150.0, 100.0);
    /// assert!((state.percentage() - 1.0).abs() < f64::EPSILON);
    /// ```
    pub fn percentage(&self) -> f64 {
        if self.max <= 0.0 {
            return 0.0;
        }
        (self.value / self.max).clamp(0.0, 1.0)
    }

    /// Returns the percentage as a u16 (0-100) for ratatui's `Gauge::percent`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GaugeState;
    ///
    /// let state = GaugeState::new(75.0, 100.0);
    /// assert_eq!(state.display_percentage(), 75);
    ///
    /// let state = GaugeState::new(33.3, 100.0);
    /// assert_eq!(state.display_percentage(), 33);
    /// ```
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn display_percentage(&self) -> u16 {
        (self.percentage() * 100.0) as u16
    }

    /// Returns the color for the current threshold zone.
    ///
    /// Iterates through thresholds (sorted ascending by `above`) and returns
    /// the color of the highest threshold that the current percentage meets.
    /// Falls back to `Color::Green` if no thresholds are defined.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GaugeState;
    /// use ratatui::style::Color;
    ///
    /// // Default thresholds: green < 70%, yellow 70-90%, red >= 90%
    /// let state = GaugeState::new(50.0, 100.0);
    /// assert_eq!(state.current_color(), Color::Green);
    ///
    /// let state = GaugeState::new(80.0, 100.0);
    /// assert_eq!(state.current_color(), Color::Yellow);
    ///
    /// let state = GaugeState::new(95.0, 100.0);
    /// assert_eq!(state.current_color(), Color::Red);
    /// ```
    pub fn current_color(&self) -> Color {
        let pct = self.percentage();
        let mut color = Color::Green;
        for zone in &self.thresholds {
            if pct >= zone.above {
                color = zone.color;
            } else {
                break;
            }
        }
        color
    }

    /// Formats the display label text.
    ///
    /// If a custom label is set, returns it directly. Otherwise formats as
    /// `"{value} / {max} {units}"` when units are present, or
    /// `"{percentage}%"` when no units are set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GaugeState;
    ///
    /// // With units
    /// let state = GaugeState::new(512.0, 1024.0).with_units("MB");
    /// assert_eq!(state.label_text(), "512.0 / 1024.0 MB");
    ///
    /// // Without units
    /// let state = GaugeState::new(75.0, 100.0);
    /// assert_eq!(state.label_text(), "75%");
    ///
    /// // With custom label
    /// let state = GaugeState::new(75.0, 100.0).with_label("Three quarters");
    /// assert_eq!(state.label_text(), "Three quarters");
    /// ```
    pub fn label_text(&self) -> String {
        if let Some(label) = &self.label {
            return label.clone();
        }
        if let Some(units) = &self.units {
            format!("{:.1} / {:.1} {}", self.value, self.max, units)
        } else {
            format!("{}%", self.display_percentage())
        }
    }

    /// Returns true if the gauge is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Updates the gauge state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{GaugeMessage, GaugeState};
    ///
    /// let mut state = GaugeState::new(0.0, 100.0);
    /// state.update(GaugeMessage::SetValue(50.0));
    /// assert_eq!(state.value(), 50.0);
    /// ```
    pub fn update(&mut self, msg: GaugeMessage) -> Option<GaugeOutput> {
        Gauge::update(self, msg)
    }

    /// Maps an event to a gauge message, if applicable.
    ///
    /// Since Gauge is display-only, this always returns `None`.
    pub fn handle_event(&self, event: &crate::input::Event) -> Option<GaugeMessage> {
        Gauge::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// Since Gauge is display-only, this always returns `None`.
    pub fn dispatch_event(&mut self, event: &crate::input::Event) -> Option<GaugeOutput> {
        Gauge::dispatch_event(self, event)
    }
}

/// A ratio and measurement display component.
///
/// `Gauge` displays a visual fill bar for ratios and measurements using
/// ratatui's `Gauge` (full variant) or `LineGauge` (line variant) widgets.
/// This is a display-only component that does not implement `Focusable`.
///
/// Unlike [`ProgressBar`](super::ProgressBar) which tracks task completion
/// with progress from 0% to 100%, `Gauge` shows a current value relative to
/// a maximum, with configurable threshold zones that change the bar color.
///
/// # Visual Variants
///
/// ```text
/// Full variant (GaugeVariant::Full):
/// ┌ CPU Usage ────────────────────────┐
/// │██████████████████░░░░░░░░░░ 75%   │
/// └───────────────────────────────────┘
///
/// Line variant (GaugeVariant::Line):
/// 512.0 / 1024.0 MB ━━━━━━━━━━━━━━╌╌╌╌╌╌╌
/// ```
///
/// # Threshold Zones
///
/// Default thresholds color the gauge green below 70%, yellow from 70-90%,
/// and red at 90% and above. Custom thresholds can be set via
/// [`GaugeState::with_thresholds`].
///
/// # Messages
///
/// - `SetValue(f64)` - Set the current value
/// - `SetMax(f64)` - Set the maximum value
/// - `SetLabel(Option<String>)` - Set a custom label
/// - `SetUnits(Option<String>)` - Set the units display string
///
/// # Example
///
/// ```rust
/// use envision::component::{Gauge, GaugeMessage, GaugeState, Component};
///
/// let mut state = GaugeState::new(45.0, 100.0)
///     .with_units("%")
///     .with_title("CPU");
///
/// Gauge::update(&mut state, GaugeMessage::SetValue(92.0));
/// assert_eq!(state.display_percentage(), 92);
/// assert_eq!(state.current_color(), ratatui::style::Color::Red);
/// ```
pub struct Gauge;

impl Component for Gauge {
    type State = GaugeState;
    type Message = GaugeMessage;
    type Output = GaugeOutput;

    fn init() -> Self::State {
        GaugeState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            GaugeMessage::SetValue(value) => {
                state.value = value;
            }
            GaugeMessage::SetMax(max) => {
                state.max = max;
            }
            GaugeMessage::SetLabel(label) => {
                state.label = label;
            }
            GaugeMessage::SetUnits(units) => {
                state.units = units;
            }
        }
        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let label_text = state.label_text();

        match state.variant {
            GaugeVariant::Full => {
                render_full_gauge(state, frame, area, theme, &label_text);
            }
            GaugeVariant::Line => {
                render_line_gauge(state, frame, area, theme, &label_text);
            }
        }
    }
}

/// Renders the full block gauge variant.
fn render_full_gauge(
    state: &GaugeState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    label_text: &str,
) {
    let color = if state.disabled {
        theme.disabled
    } else {
        state.current_color()
    };

    let block = build_block(state, theme);

    let gauge = RatatuiGauge::default()
        .block(block)
        .percent(state.display_percentage())
        .label(label_text.to_string())
        .gauge_style(Style::default().fg(color).bg(theme.background));

    let annotation =
        crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom("Gauge".into()))
            .with_id("gauge")
            .with_label(label_text)
            .with_value(format!("{}%", state.display_percentage()));
    let annotated = crate::annotation::Annotate::new(gauge, annotation);
    frame.render_widget(annotated, area);
}

/// Renders the line gauge variant.
fn render_line_gauge(
    state: &GaugeState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    label_text: &str,
) {
    let color = if state.disabled {
        theme.disabled
    } else {
        state.current_color()
    };

    let mut gauge = LineGauge::default()
        .ratio(state.percentage())
        .label(label_text.to_string())
        .filled_style(Style::default().fg(color))
        .unfilled_style(theme.disabled_style());

    if let Some(title) = &state.title {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.border_style())
            .title(title.clone());
        gauge = gauge.block(block);
    }

    let annotation =
        crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom("Gauge".into()))
            .with_id("gauge")
            .with_label(label_text)
            .with_value(format!("{}%", state.display_percentage()));
    let annotated = crate::annotation::Annotate::new(gauge, annotation);
    frame.render_widget(annotated, area);
}

/// Builds the block for the full gauge variant.
fn build_block(state: &GaugeState, theme: &Theme) -> Block<'static> {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    if let Some(title) = &state.title {
        block = block.title(title.clone());
    }

    block
}

impl Disableable for Gauge {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod tests;
