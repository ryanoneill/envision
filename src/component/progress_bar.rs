//! A progress indicator component for displaying completion status.
//!
//! `ProgressBar` provides a visual progress indicator that shows completion
//! from 0% to 100%. This is a **display-only** component that does not
//! receive keyboard focus.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{ProgressBar, ProgressMessage, ProgressOutput, ProgressBarState, Component};
//!
//! // Create a progress bar
//! let mut state = ProgressBarState::new();
//! assert_eq!(state.progress(), 0.0);
//!
//! // Update progress
//! let output = ProgressBar::update(&mut state, ProgressMessage::SetProgress(0.5));
//! assert_eq!(output, None);
//! assert_eq!(state.percentage(), 50);
//!
//! // Increment progress
//! ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
//! assert_eq!(state.percentage(), 75);
//!
//! // Complete the progress
//! let output = ProgressBar::update(&mut state, ProgressMessage::Complete);
//! assert_eq!(output, Some(ProgressOutput::Completed));
//! assert!(state.is_complete());
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};

use super::Component;

/// Messages that can be sent to a ProgressBar.
#[derive(Clone, Debug, PartialEq)]
pub enum ProgressMessage {
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
pub enum ProgressOutput {
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
/// The component emits `ProgressOutput::Completed` when progress reaches 100%.
///
/// # Example
///
/// ```rust
/// use envision::component::{ProgressBar, ProgressMessage, ProgressOutput, ProgressBarState, Component};
///
/// let mut state = ProgressBarState::with_label("Loading...");
///
/// // Update progress
/// ProgressBar::update(&mut state, ProgressMessage::SetProgress(0.5));
/// assert_eq!(state.percentage(), 50);
///
/// // Complete
/// let output = ProgressBar::update(&mut state, ProgressMessage::Complete);
/// assert_eq!(output, Some(ProgressOutput::Completed));
/// ```
pub struct ProgressBar;

impl Component for ProgressBar {
    type State = ProgressBarState;
    type Message = ProgressMessage;
    type Output = ProgressOutput;

    fn init() -> Self::State {
        ProgressBarState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        let was_complete = state.is_complete();

        match msg {
            ProgressMessage::SetProgress(value) => {
                state.set_progress(value);
            }
            ProgressMessage::Increment(amount) => {
                state.set_progress(state.progress + amount);
            }
            ProgressMessage::Complete => {
                state.progress = 1.0;
            }
            ProgressMessage::Reset => {
                state.progress = 0.0;
                return None;
            }
        }

        // Emit Completed only when transitioning to complete, or on explicit Complete message
        if state.is_complete() && (!was_complete || matches!(msg, ProgressMessage::Complete)) {
            Some(ProgressOutput::Completed)
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        let label = match &state.label {
            Some(l) => format!("{} {}%", l, state.percentage()),
            None => format!("{}%", state.percentage()),
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::Black))
            .percent(state.percentage())
            .label(label);

        frame.render_widget(gauge, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = ProgressBarState::new();
        assert_eq!(state.progress(), 0.0);
        assert_eq!(state.percentage(), 0);
        assert!(!state.is_complete());
        assert!(state.label().is_none());
    }

    #[test]
    fn test_default() {
        let state = ProgressBarState::default();
        assert_eq!(state.progress(), 0.0);
        assert_eq!(state.percentage(), 0);
    }

    #[test]
    fn test_with_progress() {
        let state = ProgressBarState::with_progress(0.5);
        assert_eq!(state.progress(), 0.5);
        assert_eq!(state.percentage(), 50);
    }

    #[test]
    fn test_with_progress_clamps() {
        let state = ProgressBarState::with_progress(1.5);
        assert_eq!(state.progress(), 1.0);

        let state = ProgressBarState::with_progress(-0.5);
        assert_eq!(state.progress(), 0.0);
    }

    #[test]
    fn test_with_label() {
        let state = ProgressBarState::with_label("Loading...");
        assert_eq!(state.label(), Some("Loading..."));
        assert_eq!(state.progress(), 0.0);
    }

    #[test]
    fn test_progress_accessors() {
        let mut state = ProgressBarState::new();

        state.set_progress(0.75);
        assert_eq!(state.progress(), 0.75);
        assert_eq!(state.percentage(), 75);
    }

    #[test]
    fn test_label_accessors() {
        let mut state = ProgressBarState::new();
        assert!(state.label().is_none());

        state.set_label(Some("Test".to_string()));
        assert_eq!(state.label(), Some("Test"));

        state.set_label(None);
        assert!(state.label().is_none());
    }

    #[test]
    fn test_is_complete() {
        let mut state = ProgressBarState::new();
        assert!(!state.is_complete());

        state.set_progress(0.99);
        assert!(!state.is_complete());

        state.set_progress(1.0);
        assert!(state.is_complete());

        state.set_progress(1.5); // Clamped to 1.0
        assert!(state.is_complete());
    }

    #[test]
    fn test_set_progress_clamps() {
        let mut state = ProgressBarState::new();

        state.set_progress(-0.5);
        assert_eq!(state.progress(), 0.0);

        state.set_progress(1.5);
        assert_eq!(state.progress(), 1.0);

        state.set_progress(0.5);
        assert_eq!(state.progress(), 0.5);
    }

    #[test]
    fn test_set_progress_emits_completed() {
        let mut state = ProgressBarState::new();

        // Not complete yet
        let output = ProgressBar::update(&mut state, ProgressMessage::SetProgress(0.5));
        assert_eq!(output, None);

        // Now complete
        let output = ProgressBar::update(&mut state, ProgressMessage::SetProgress(1.0));
        assert_eq!(output, Some(ProgressOutput::Completed));

        // Already complete, no output
        let output = ProgressBar::update(&mut state, ProgressMessage::SetProgress(1.0));
        assert_eq!(output, None);
    }

    #[test]
    fn test_increment() {
        let mut state = ProgressBarState::new();

        ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
        assert_eq!(state.progress(), 0.25);

        ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
        assert_eq!(state.progress(), 0.5);

        ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
        assert_eq!(state.progress(), 0.75);
    }

    #[test]
    fn test_increment_clamps() {
        let mut state = ProgressBarState::with_progress(0.9);

        let output = ProgressBar::update(&mut state, ProgressMessage::Increment(0.5));
        assert_eq!(state.progress(), 1.0);
        assert_eq!(output, Some(ProgressOutput::Completed));
    }

    #[test]
    fn test_complete() {
        let mut state = ProgressBarState::with_progress(0.5);

        let output = ProgressBar::update(&mut state, ProgressMessage::Complete);
        assert_eq!(state.progress(), 1.0);
        assert!(state.is_complete());
        assert_eq!(output, Some(ProgressOutput::Completed));
    }

    #[test]
    fn test_complete_when_already_complete() {
        let mut state = ProgressBarState::with_progress(1.0);

        // Even if already complete, Complete message still emits Completed
        let output = ProgressBar::update(&mut state, ProgressMessage::Complete);
        assert_eq!(output, Some(ProgressOutput::Completed));
    }

    #[test]
    fn test_reset() {
        let mut state = ProgressBarState::with_progress(0.75);

        let output = ProgressBar::update(&mut state, ProgressMessage::Reset);
        assert_eq!(state.progress(), 0.0);
        assert_eq!(output, None);
    }

    #[test]
    fn test_reset_from_complete() {
        let mut state = ProgressBarState::with_progress(1.0);
        assert!(state.is_complete());

        ProgressBar::update(&mut state, ProgressMessage::Reset);
        assert_eq!(state.progress(), 0.0);
        assert!(!state.is_complete());
    }

    #[test]
    fn test_clone() {
        let mut state = ProgressBarState::with_progress(0.5);
        state.set_label(Some("Test".to_string()));

        let cloned = state.clone();
        assert_eq!(cloned.progress(), 0.5);
        assert_eq!(cloned.label(), Some("Test"));
    }

    #[test]
    fn test_init() {
        let state = ProgressBar::init();
        assert_eq!(state.progress(), 0.0);
        assert!(state.label().is_none());
    }

    #[test]
    fn test_percentage_rounding() {
        let mut state = ProgressBarState::new();

        state.set_progress(0.334);
        assert_eq!(state.percentage(), 33);

        state.set_progress(0.335);
        assert_eq!(state.percentage(), 34);

        state.set_progress(0.999);
        assert_eq!(state.percentage(), 100);
    }

    #[test]
    fn test_view_renders() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = ProgressBarState::with_progress(0.5);
        state.set_label(Some("Loading".to_string()));

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                ProgressBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("50%"));
        assert!(output.contains("Loading"));
    }

    #[test]
    fn test_view_without_label() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = ProgressBarState::with_progress(0.75);

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                ProgressBar::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("75%"));
    }

    #[test]
    fn test_full_workflow() {
        let mut state = ProgressBarState::with_label("Downloading");

        // Start
        assert_eq!(state.percentage(), 0);

        // Progress updates
        ProgressBar::update(&mut state, ProgressMessage::SetProgress(0.25));
        assert_eq!(state.percentage(), 25);

        ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
        assert_eq!(state.percentage(), 50);

        ProgressBar::update(&mut state, ProgressMessage::Increment(0.25));
        assert_eq!(state.percentage(), 75);

        // Complete
        let output = ProgressBar::update(&mut state, ProgressMessage::Complete);
        assert_eq!(output, Some(ProgressOutput::Completed));
        assert!(state.is_complete());

        // Reset for reuse
        ProgressBar::update(&mut state, ProgressMessage::Reset);
        assert_eq!(state.percentage(), 0);
        assert!(!state.is_complete());
    }
}
