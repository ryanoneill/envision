//! A progress indicator component for displaying completion status.
//!
//! `ProgressBar` provides a visual progress indicator that shows completion
//! from 0% to 100%. This is a **display-only** component that does not
//! receive keyboard focus.
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

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};

use super::Component;
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
}

/// Output messages from a ProgressBar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressBarOutput {
    /// Progress reached 100%.
    Completed,
}

/// State for a ProgressBar component.
#[derive(Clone, Debug, Default)]
pub struct ProgressBarState {
    /// The current progress value (0.0 to 1.0).
    progress: f32,
    /// Optional label to display.
    label: Option<String>,
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
            label: None,
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
            progress: 0.0,
            label: Some(label.into()),
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let label = match &state.label {
            Some(l) => format!("{} {}%", l, state.percentage()),
            None => format!("{}%", state.percentage()),
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(theme.progress_filled_style())
            .percent(state.percentage())
            .label(label);

        frame.render_widget(gauge, area);
    }
}

#[cfg(test)]
mod tests;
