//! A progress indicator component for displaying completion status.
//!
//! [`ProgressBar`] provides a visual progress indicator that shows completion
//! from 0% to 100%. This is a **display-only** component that does not
//! receive keyboard focus. State is stored in [`ProgressBarState`], updated
//! via [`ProgressBarMessage`], and produces [`ProgressBarOutput`].
//!
//! See also [`Spinner`](super::Spinner) for indeterminate progress,
//! and [`MultiProgress`](super::MultiProgress) for tracking multiple tasks.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{ProgressBar, ProgressBarMessage, ProgressBarOutput, ProgressBarState, Component};
//!
//! // Create a progress bar
//! let mut state = ProgressBarState::new();
//! assert_eq!(state.progress(), 0.0);
//!
//! // Update progress
//! let output = ProgressBar::update(&mut state, ProgressBarMessage::SetProgress(0.5));
//! assert_eq!(output, None);
//! assert_eq!(state.percentage(), 50);
//!
//! // Increment progress
//! ProgressBar::update(&mut state, ProgressBarMessage::Increment(0.25));
//! assert_eq!(state.percentage(), 75);
//!
//! // Complete the progress
//! let output = ProgressBar::update(&mut state, ProgressBarMessage::Complete);
//! assert_eq!(output, Some(ProgressBarOutput::Completed));
//! assert!(state.is_complete());
//! ```

use std::time::Duration;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};

use super::{Component, Disableable, ViewContext};
use crate::theme::Theme;

/// Messages that can be sent to a ProgressBar.
#[derive(Clone, Debug, PartialEq)]
pub enum ProgressBarMessage {
    /// Set progress value (0.0 to 1.0).
    SetProgress(f32),
    /// Increment progress by the given amount.
    Increment(f32),
    /// Set progress to complete (1.0).
    Complete,
    /// Reset progress to zero.
    Reset,
    /// Set the estimated time of arrival.
    SetEta(Option<Duration>),
    /// Set the rate text (e.g., "5.2 items/sec").
    SetRateText(Option<String>),
}

/// Output messages from a ProgressBar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressBarOutput {
    /// Progress reached 100%.
    Completed,
}

/// State for a ProgressBar component.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ProgressBarState {
    /// The current progress value (0.0 to 1.0).
    progress: f32,
    /// Optional label to display.
    label: Option<String>,
    /// Whether the component is disabled.
    disabled: bool,
    /// Whether to show percentage in the label.
    show_percentage: bool,
    /// Estimated time remaining in milliseconds.
    eta_millis: Option<u64>,
    /// Rate text (e.g., "5.2 items/sec").
    rate_text: Option<String>,
    /// Whether to show the ETA in the label.
    show_eta: bool,
    /// Whether to show the rate in the label.
    show_rate: bool,
}

impl Default for ProgressBarState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            label: None,
            disabled: false,
            show_percentage: true,
            eta_millis: None,
            rate_text: None,
            show_eta: true,
            show_rate: true,
        }
    }
}

impl ProgressBarState {
    /// Creates a new progress bar at 0%.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let state = ProgressBarState::new();
    /// assert_eq!(state.progress(), 0.0);
    /// assert_eq!(state.percentage(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a progress bar with an initial progress value.
    ///
    /// The value is clamped to the range 0.0..=1.0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let state = ProgressBarState::with_progress(0.5);
    /// assert_eq!(state.percentage(), 50);
    /// ```
    pub fn with_progress(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            ..Self::default()
        }
    }

    /// Creates a progress bar with a label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let state = ProgressBarState::with_label("Downloading...");
    /// assert_eq!(state.label(), Some("Downloading..."));
    /// ```
    pub fn with_label(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..Self::default()
        }
    }

    /// Returns the current progress value (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Returns the progress as a percentage (0 to 100).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn percentage(&self) -> u16 {
        (self.progress * 100.0).round() as u16
    }

    /// Sets the progress value.
    ///
    /// The value is clamped to the range 0.0..=1.0.
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Returns true if the progress is complete (>= 1.0).
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    /// Returns the label if set.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    /// Returns true if the progress bar is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the disabled state using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let state = ProgressBarState::new()
    ///     .with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets whether to show percentage in the label (builder pattern).
    ///
    /// Default is `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let state = ProgressBarState::new().with_show_percentage(false);
    /// assert!(!state.show_percentage());
    /// ```
    pub fn with_show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Sets whether to show ETA in the label (builder pattern).
    ///
    /// Default is `true`. ETA is only shown when `eta_millis` is `Some`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let state = ProgressBarState::new().with_show_eta(false);
    /// assert!(!state.show_eta());
    /// ```
    pub fn with_show_eta(mut self, show: bool) -> Self {
        self.show_eta = show;
        self
    }

    /// Sets whether to show rate text in the label (builder pattern).
    ///
    /// Default is `true`. Rate is only shown when `rate_text` is `Some`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let state = ProgressBarState::new().with_show_rate(false);
    /// assert!(!state.show_rate());
    /// ```
    pub fn with_show_rate(mut self, show: bool) -> Self {
        self.show_rate = show;
        self
    }

    /// Returns whether the percentage is shown in the label.
    pub fn show_percentage(&self) -> bool {
        self.show_percentage
    }

    /// Returns whether the ETA is shown in the label.
    pub fn show_eta(&self) -> bool {
        self.show_eta
    }

    /// Returns whether the rate is shown in the label.
    pub fn show_rate(&self) -> bool {
        self.show_rate
    }

    /// Sets whether the percentage is shown in the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let mut state = ProgressBarState::new();
    /// state.set_show_percentage(true);
    /// assert!(state.show_percentage());
    /// ```
    pub fn set_show_percentage(&mut self, show: bool) {
        self.show_percentage = show;
    }

    /// Sets whether the ETA is shown in the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let mut state = ProgressBarState::new();
    /// state.set_show_eta(true);
    /// assert!(state.show_eta());
    /// ```
    pub fn set_show_eta(&mut self, show: bool) {
        self.show_eta = show;
    }

    /// Sets whether the rate is shown in the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ProgressBarState;
    ///
    /// let mut state = ProgressBarState::new();
    /// state.set_show_rate(true);
    /// assert!(state.show_rate());
    /// ```
    pub fn set_show_rate(&mut self, show: bool) {
        self.show_rate = show;
    }

    /// Returns the ETA as a `Duration`, if set.
    pub fn eta(&self) -> Option<Duration> {
        self.eta_millis.map(Duration::from_millis)
    }

    /// Returns the ETA in milliseconds, if set.
    pub fn eta_millis(&self) -> Option<u64> {
        self.eta_millis
    }

    /// Returns the rate text, if set.
    pub fn rate_text(&self) -> Option<&str> {
        self.rate_text.as_deref()
    }

    /// Sets the ETA.
    pub fn set_eta(&mut self, eta: Option<Duration>) {
        self.eta_millis = eta.map(|d| d.as_millis() as u64);
    }

    /// Sets the rate text.
    pub fn set_rate_text(&mut self, rate_text: Option<String>) {
        self.rate_text = rate_text;
    }
}

/// Formats a duration as a human-readable ETA string.
///
/// # Format
///
/// - Less than 60 seconds: `"45s"`
/// - Less than 1 hour: `"3m 22s"`
/// - 1 hour or more: `"1h 02m"`
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
/// use envision::component::progress_bar::format_eta;
///
/// assert_eq!(format_eta(Duration::from_secs(45)), "45s");
/// assert_eq!(format_eta(Duration::from_secs(202)), "3m 22s");
/// assert_eq!(format_eta(Duration::from_secs(3720)), "1h 02m");
/// ```
pub fn format_eta(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}m {:02}s", mins, secs)
    } else {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        format!("{}h {:02}m", hours, mins)
    }
}

/// A progress indicator component.
///
/// `ProgressBar` displays progress visually using ratatui's `Gauge` widget.
/// This is a display-only component that does not implement `Focusable`.
///
/// # Visual Format
///
/// The progress bar renders as a gauge with optional label:
/// ```text
/// ┌──────────────────────────────┐
/// │████████████░░░░░░░░░░░░ 50%  │
/// └──────────────────────────────┘
/// ```
///
/// # Messages
///
/// - `SetProgress(f32)` - Set progress to a specific value (0.0 to 1.0)
/// - `Increment(f32)` - Add to the current progress
/// - `Complete` - Set progress to 100%
/// - `Reset` - Set progress to 0%
/// - `SetEta(Option<Duration>)` - Set estimated time remaining
/// - `SetRateText(Option<String>)` - Set rate display text
///
/// # Output
///
/// The component emits `ProgressBarOutput::Completed` when progress reaches 100%.
///
/// # Example
///
/// ```rust
/// use envision::component::{ProgressBar, ProgressBarMessage, ProgressBarOutput, ProgressBarState, Component};
///
/// let mut state = ProgressBarState::with_label("Loading...");
///
/// // Update progress
/// ProgressBar::update(&mut state, ProgressBarMessage::SetProgress(0.5));
/// assert_eq!(state.percentage(), 50);
///
/// // Complete
/// let output = ProgressBar::update(&mut state, ProgressBarMessage::Complete);
/// assert_eq!(output, Some(ProgressBarOutput::Completed));
/// ```
pub struct ProgressBar;

impl Component for ProgressBar {
    type State = ProgressBarState;
    type Message = ProgressBarMessage;
    type Output = ProgressBarOutput;

    fn init() -> Self::State {
        ProgressBarState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        let was_complete = state.is_complete();

        match msg {
            ProgressBarMessage::SetProgress(value) => {
                state.set_progress(value);
            }
            ProgressBarMessage::Increment(amount) => {
                state.set_progress(state.progress + amount);
            }
            ProgressBarMessage::Complete => {
                state.progress = 1.0;
            }
            ProgressBarMessage::Reset => {
                state.progress = 0.0;
                state.eta_millis = None;
                state.rate_text = None;
                return None;
            }
            ProgressBarMessage::SetEta(eta) => {
                state.eta_millis = eta.map(|d| d.as_millis() as u64);
                return None;
            }
            ProgressBarMessage::SetRateText(text) => {
                state.rate_text = text;
                return None;
            }
        }

        // Emit Completed only when transitioning to complete, or on explicit Complete message
        if state.is_complete() && (!was_complete || matches!(msg, ProgressBarMessage::Complete)) {
            Some(ProgressBarOutput::Completed)
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, _ctx: &ViewContext) {
        let label = build_label(state);

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(theme.progress_filled_style())
            .percent(state.percentage())
            .label(label.clone());

        let annotation =
            crate::annotation::Annotation::new(crate::annotation::WidgetType::Progress)
                .with_id("progress_bar")
                .with_label(label)
                .with_value(format!("{}%", state.percentage()));
        let annotated = crate::annotation::Annotate::new(gauge, annotation);
        frame.render_widget(annotated, area);
    }
}

impl Disableable for ProgressBar {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

/// Builds the label string from the progress bar state.
fn build_label(state: &ProgressBarState) -> String {
    let mut parts = Vec::new();

    if let Some(l) = &state.label {
        parts.push(l.clone());
    }

    if state.show_percentage {
        parts.push(format!("{}%", state.percentage()));
    }

    if state.show_rate {
        if let Some(rate) = &state.rate_text {
            parts.push(format!("[{}]", rate));
        }
    }

    if state.show_eta {
        if let Some(millis) = state.eta_millis {
            let duration = Duration::from_millis(millis);
            parts.push(format!("ETA: {}", format_eta(duration)));
        }
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests;
