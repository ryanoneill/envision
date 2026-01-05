//! A toast notification component for temporary messages.
//!
//! `Toast` provides non-modal notifications that appear as a vertical stack,
//! with severity levels and auto-dismiss support.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Toast, ToastMessage, ToastState, ToastLevel, Component};
//!
//! // Create toast state with 3 second default duration
//! let mut state = ToastState::with_duration(3000);
//!
//! // Add toasts using convenience methods
//! state.info("Information message");
//! state.success("Operation completed!");
//! state.warning("Low disk space");
//! state.error("Connection failed");
//!
//! // Or via the Push message
//! Toast::update(&mut state, ToastMessage::Push {
//!     message: "Custom toast".into(),
//!     level: ToastLevel::Info,
//!     duration_ms: Some(5000),
//! });
//!
//! // Tick to advance time (call periodically from your app)
//! Toast::update(&mut state, ToastMessage::Tick(100));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::Component;
use crate::theme::Theme;

/// Default maximum number of visible toasts.
const DEFAULT_MAX_VISIBLE: usize = 5;

/// Severity level for toast notifications.
///
/// Each level has a distinct color for visual differentiation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastLevel {
    /// General information (blue).
    #[default]
    Info,
    /// Successful operation (green).
    Success,
    /// Warning message (yellow).
    Warning,
    /// Error message (red).
    Error,
}

/// A single toast notification.
///
/// Each toast has a unique ID, message, severity level, and optional
/// remaining duration for auto-dismiss.
#[derive(Clone, Debug)]
pub struct ToastItem {
    /// Unique identifier for this toast.
    id: u64,
    /// The toast message.
    message: String,
    /// Severity level.
    level: ToastLevel,
    /// Remaining duration in milliseconds (None = persistent).
    remaining_ms: Option<u64>,
}

impl ToastItem {
    /// Returns the toast's unique identifier.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the toast message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the severity level.
    pub fn level(&self) -> ToastLevel {
        self.level
    }

    /// Returns true if this toast is persistent (no auto-dismiss).
    pub fn is_persistent(&self) -> bool {
        self.remaining_ms.is_none()
    }

    /// Returns the remaining duration in milliseconds, if any.
    pub fn remaining_ms(&self) -> Option<u64> {
        self.remaining_ms
    }
}

/// Messages that can be sent to a Toast component.
#[derive(Clone, Debug, PartialEq)]
pub enum ToastMessage {
    /// Add a new toast with optional auto-dismiss duration.
    Push {
        /// The message to display.
        message: String,
        /// Severity level.
        level: ToastLevel,
        /// Duration in milliseconds (None = persistent).
        duration_ms: Option<u64>,
    },
    /// Dismiss a specific toast by ID.
    Dismiss(u64),
    /// Dismiss all toasts.
    Clear,
    /// Advance time by the given milliseconds (for auto-dismiss).
    Tick(u64),
}

/// Output messages from a Toast component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToastOutput {
    /// A toast was added (returns ID).
    Added(u64),
    /// A toast was dismissed by user.
    Dismissed(u64),
    /// A toast expired (auto-dismissed).
    Expired(u64),
    /// All toasts were cleared.
    Cleared,
}

/// State for a Toast component.
///
/// Manages a collection of toast notifications with support for
/// auto-dismiss, manual dismiss, and configurable limits.
#[derive(Clone, Debug)]
pub struct ToastState {
    /// Active toasts.
    toasts: Vec<ToastItem>,
    /// Counter for generating unique IDs.
    next_id: u64,
    /// Default duration for new toasts (ms).
    default_duration_ms: Option<u64>,
    /// Maximum number of visible toasts.
    max_visible: usize,
}

impl Default for ToastState {
    fn default() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
            default_duration_ms: None,
            max_visible: DEFAULT_MAX_VISIBLE,
        }
    }
}

impl ToastState {
    /// Creates a new toast state with default settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ToastState;
    ///
    /// let state = ToastState::new();
    /// assert!(state.is_empty());
    /// assert_eq!(state.default_duration(), None);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a toast state with a default duration for new toasts.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ToastState;
    ///
    /// let state = ToastState::with_duration(3000);
    /// assert_eq!(state.default_duration(), Some(3000));
    /// ```
    pub fn with_duration(duration_ms: u64) -> Self {
        Self {
            default_duration_ms: Some(duration_ms),
            ..Self::default()
        }
    }

    /// Creates a toast state with a custom maximum visible toasts.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ToastState;
    ///
    /// let state = ToastState::with_max_visible(3);
    /// assert_eq!(state.max_visible(), 3);
    /// ```
    pub fn with_max_visible(max: usize) -> Self {
        Self {
            max_visible: max,
            ..Self::default()
        }
    }

    /// Returns all active toasts.
    pub fn toasts(&self) -> &[ToastItem] {
        &self.toasts
    }

    /// Returns the number of active toasts.
    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    /// Returns true if there are no active toasts.
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    /// Returns the default duration for new toasts.
    pub fn default_duration(&self) -> Option<u64> {
        self.default_duration_ms
    }

    /// Returns the maximum number of visible toasts.
    pub fn max_visible(&self) -> usize {
        self.max_visible
    }

    /// Sets the default duration for new toasts.
    pub fn set_default_duration(&mut self, duration_ms: Option<u64>) {
        self.default_duration_ms = duration_ms;
    }

    /// Sets the maximum number of visible toasts.
    pub fn set_max_visible(&mut self, max: usize) {
        self.max_visible = max;
    }

    /// Internal method to add a toast.
    fn push(&mut self, message: String, level: ToastLevel, duration_ms: Option<u64>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        // Use provided duration, or fall back to default
        let remaining_ms = match duration_ms {
            Some(d) => Some(d),
            None => self.default_duration_ms,
        };

        self.toasts.push(ToastItem {
            id,
            message,
            level,
            remaining_ms,
        });

        id
    }

    /// Adds an info toast and returns its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ToastState, ToastLevel};
    ///
    /// let mut state = ToastState::new();
    /// let id = state.info("Information message");
    /// assert_eq!(state.toasts()[0].level(), ToastLevel::Info);
    /// ```
    pub fn info(&mut self, message: impl Into<String>) -> u64 {
        self.push(message.into(), ToastLevel::Info, None)
    }

    /// Adds a success toast and returns its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ToastState, ToastLevel};
    ///
    /// let mut state = ToastState::new();
    /// let id = state.success("Operation completed!");
    /// assert_eq!(state.toasts()[0].level(), ToastLevel::Success);
    /// ```
    pub fn success(&mut self, message: impl Into<String>) -> u64 {
        self.push(message.into(), ToastLevel::Success, None)
    }

    /// Adds a warning toast and returns its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ToastState, ToastLevel};
    ///
    /// let mut state = ToastState::new();
    /// let id = state.warning("Low disk space");
    /// assert_eq!(state.toasts()[0].level(), ToastLevel::Warning);
    /// ```
    pub fn warning(&mut self, message: impl Into<String>) -> u64 {
        self.push(message.into(), ToastLevel::Warning, None)
    }

    /// Adds an error toast and returns its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ToastState, ToastLevel};
    ///
    /// let mut state = ToastState::new();
    /// let id = state.error("Connection failed");
    /// assert_eq!(state.toasts()[0].level(), ToastLevel::Error);
    /// ```
    pub fn error(&mut self, message: impl Into<String>) -> u64 {
        self.push(message.into(), ToastLevel::Error, None)
    }
}

/// A toast notification component.
///
/// `Toast` displays temporary notification messages in a vertical stack.
/// Toasts can have different severity levels and auto-dismiss after a
/// configurable duration.
///
/// # Timer Integration
///
/// The component uses a `Tick` message to track time. Your application
/// should send periodic `Tick(elapsed_ms)` messages (e.g., every 100ms)
/// to drive auto-dismiss functionality.
///
/// # Visual Format
///
/// Toasts render in the bottom-right corner, stacking upward:
/// ```text
///                                    ┌──────────────────────────────────┐
///                                    │ ✓ Operation completed!           │
///                                    └──────────────────────────────────┘
///                                    ┌──────────────────────────────────┐
///                                    │ ℹ Processing your request...     │
///                                    └──────────────────────────────────┘
/// ```
///
/// # Severity Levels
///
/// - `Info` - Blue border, ℹ prefix
/// - `Success` - Green border, ✓ prefix
/// - `Warning` - Yellow border, ⚠ prefix
/// - `Error` - Red border, ✗ prefix
///
/// # Example
///
/// ```rust
/// use envision::component::{Toast, ToastMessage, ToastOutput, ToastState, Component};
///
/// let mut state = ToastState::with_duration(3000);
///
/// // Add a success toast
/// let id = state.success("File saved!");
///
/// // Tick to advance time
/// let output = Toast::update(&mut state, ToastMessage::Tick(3000));
/// assert_eq!(output, Some(ToastOutput::Expired(id)));
/// assert!(state.is_empty());
/// ```
pub struct Toast;

impl Component for Toast {
    type State = ToastState;
    type Message = ToastMessage;
    type Output = ToastOutput;

    fn init() -> Self::State {
        ToastState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ToastMessage::Push {
                message,
                level,
                duration_ms,
            } => {
                let id = state.push(message, level, duration_ms);
                Some(ToastOutput::Added(id))
            }
            ToastMessage::Dismiss(id) => {
                let len_before = state.toasts.len();
                state.toasts.retain(|t| t.id != id);
                if state.toasts.len() < len_before {
                    Some(ToastOutput::Dismissed(id))
                } else {
                    None
                }
            }
            ToastMessage::Clear => {
                if state.toasts.is_empty() {
                    None
                } else {
                    state.toasts.clear();
                    Some(ToastOutput::Cleared)
                }
            }
            ToastMessage::Tick(elapsed_ms) => {
                let mut expired_ids = Vec::new();

                for toast in &mut state.toasts {
                    if let Some(remaining) = toast.remaining_ms.as_mut() {
                        if *remaining <= elapsed_ms {
                            expired_ids.push(toast.id);
                        } else {
                            *remaining -= elapsed_ms;
                        }
                    }
                }

                // Remove expired toasts
                state.toasts.retain(|t| !expired_ids.contains(&t.id));

                // Return first expired ID (if any)
                expired_ids.first().copied().map(ToastOutput::Expired)
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if state.toasts.is_empty() {
            return;
        }

        // Calculate toast dimensions
        let toast_width = 40.min(area.width);
        let toast_height = 3;
        let visible_count = state.toasts.len().min(state.max_visible);

        // Render from bottom-right corner, stacking upward
        // Newest toasts appear at the bottom
        for (i, toast) in state.toasts.iter().rev().take(visible_count).enumerate() {
            let y = area.bottom().saturating_sub((i as u16 + 1) * toast_height);
            let x = area.right().saturating_sub(toast_width);

            if y < area.y {
                break; // Don't render above the area
            }

            let toast_area = Rect::new(x, y, toast_width, toast_height.min(area.bottom() - y));

            let (border_style, prefix) = match toast.level {
                ToastLevel::Info => (theme.info_style(), "i"),
                ToastLevel::Success => (theme.success_style(), "+"),
                ToastLevel::Warning => (theme.warning_style(), "!"),
                ToastLevel::Error => (theme.error_style(), "x"),
            };

            // Clear the area for overlay effect
            frame.render_widget(Clear, toast_area);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            let text = format!("[{}] {}", prefix, toast.message);
            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

            frame.render_widget(paragraph, toast_area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    // ========================================
    // ToastLevel Tests
    // ========================================

    #[test]
    fn test_toast_level_default() {
        let level = ToastLevel::default();
        assert_eq!(level, ToastLevel::Info);
    }

    #[test]
    fn test_toast_level_clone() {
        let level = ToastLevel::Success;
        let cloned = level;
        assert_eq!(cloned, ToastLevel::Success);
    }

    #[test]
    fn test_toast_level_eq() {
        assert_eq!(ToastLevel::Info, ToastLevel::Info);
        assert_ne!(ToastLevel::Info, ToastLevel::Error);
        assert_eq!(ToastLevel::Warning, ToastLevel::Warning);
    }

    // ========================================
    // ToastItem Tests
    // ========================================

    #[test]
    fn test_toast_item_accessors() {
        let mut state = ToastState::new();
        state.push("Test message".into(), ToastLevel::Success, Some(1000));

        let toast = &state.toasts()[0];
        assert_eq!(toast.id(), 0);
        assert_eq!(toast.message(), "Test message");
        assert_eq!(toast.level(), ToastLevel::Success);
        assert_eq!(toast.remaining_ms(), Some(1000));
    }

    #[test]
    fn test_toast_item_is_persistent() {
        let mut state = ToastState::new();
        state.push("Persistent".into(), ToastLevel::Info, None);
        state.push("Timed".into(), ToastLevel::Info, Some(1000));

        assert!(state.toasts()[0].is_persistent());
        assert!(!state.toasts()[1].is_persistent());
    }

    #[test]
    fn test_toast_item_clone() {
        let mut state = ToastState::new();
        state.push("Test".into(), ToastLevel::Info, Some(1000));

        let toast = state.toasts()[0].clone();
        assert_eq!(toast.message(), "Test");
    }

    // ========================================
    // State Creation Tests
    // ========================================

    #[test]
    fn test_new() {
        let state = ToastState::new();
        assert!(state.is_empty());
        assert_eq!(state.default_duration(), None);
        assert_eq!(state.max_visible(), DEFAULT_MAX_VISIBLE);
    }

    #[test]
    fn test_with_duration() {
        let state = ToastState::with_duration(3000);
        assert_eq!(state.default_duration(), Some(3000));
    }

    #[test]
    fn test_with_max_visible() {
        let state = ToastState::with_max_visible(3);
        assert_eq!(state.max_visible(), 3);
    }

    #[test]
    fn test_default() {
        let state = ToastState::default();
        assert!(state.is_empty());
        assert_eq!(state.default_duration(), None);
    }

    // ========================================
    // Accessor Tests
    // ========================================

    #[test]
    fn test_toasts() {
        let mut state = ToastState::new();
        state.info("One");
        state.info("Two");

        assert_eq!(state.toasts().len(), 2);
        assert_eq!(state.toasts()[0].message(), "One");
        assert_eq!(state.toasts()[1].message(), "Two");
    }

    #[test]
    fn test_len() {
        let mut state = ToastState::new();
        assert_eq!(state.len(), 0);

        state.info("Test");
        assert_eq!(state.len(), 1);

        state.info("Test 2");
        assert_eq!(state.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut state = ToastState::new();
        assert!(state.is_empty());

        state.info("Test");
        assert!(!state.is_empty());
    }

    #[test]
    fn test_default_duration() {
        let state = ToastState::new();
        assert_eq!(state.default_duration(), None);

        let state = ToastState::with_duration(5000);
        assert_eq!(state.default_duration(), Some(5000));
    }

    #[test]
    fn test_max_visible() {
        let state = ToastState::new();
        assert_eq!(state.max_visible(), DEFAULT_MAX_VISIBLE);

        let state = ToastState::with_max_visible(10);
        assert_eq!(state.max_visible(), 10);
    }

    // ========================================
    // Convenience Method Tests
    // ========================================

    #[test]
    fn test_info() {
        let mut state = ToastState::new();
        let id = state.info("Info message");

        assert_eq!(state.len(), 1);
        assert_eq!(state.toasts()[0].id(), id);
        assert_eq!(state.toasts()[0].level(), ToastLevel::Info);
        assert_eq!(state.toasts()[0].message(), "Info message");
    }

    #[test]
    fn test_success() {
        let mut state = ToastState::new();
        let id = state.success("Success message");

        assert_eq!(state.toasts()[0].id(), id);
        assert_eq!(state.toasts()[0].level(), ToastLevel::Success);
    }

    #[test]
    fn test_warning() {
        let mut state = ToastState::new();
        let id = state.warning("Warning message");

        assert_eq!(state.toasts()[0].id(), id);
        assert_eq!(state.toasts()[0].level(), ToastLevel::Warning);
    }

    #[test]
    fn test_error() {
        let mut state = ToastState::new();
        let id = state.error("Error message");

        assert_eq!(state.toasts()[0].id(), id);
        assert_eq!(state.toasts()[0].level(), ToastLevel::Error);
    }

    #[test]
    fn test_convenience_returns_id() {
        let mut state = ToastState::new();
        let id1 = state.info("One");
        let id2 = state.info("Two");
        let id3 = state.info("Three");

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
    }

    #[test]
    fn test_convenience_uses_default_duration() {
        let mut state = ToastState::with_duration(3000);
        state.info("Test");

        assert_eq!(state.toasts()[0].remaining_ms(), Some(3000));
    }

    // ========================================
    // Push Message Tests
    // ========================================

    #[test]
    fn test_push() {
        let mut state = ToastState::new();

        Toast::update(
            &mut state,
            ToastMessage::Push {
                message: "Test".into(),
                level: ToastLevel::Success,
                duration_ms: Some(5000),
            },
        );

        assert_eq!(state.len(), 1);
        assert_eq!(state.toasts()[0].message(), "Test");
        assert_eq!(state.toasts()[0].level(), ToastLevel::Success);
    }

    #[test]
    fn test_push_returns_added() {
        let mut state = ToastState::new();

        let output = Toast::update(
            &mut state,
            ToastMessage::Push {
                message: "Test".into(),
                level: ToastLevel::Info,
                duration_ms: None,
            },
        );

        assert_eq!(output, Some(ToastOutput::Added(0)));
    }

    #[test]
    fn test_push_increments_id() {
        let mut state = ToastState::new();

        let out1 = Toast::update(
            &mut state,
            ToastMessage::Push {
                message: "One".into(),
                level: ToastLevel::Info,
                duration_ms: None,
            },
        );
        let out2 = Toast::update(
            &mut state,
            ToastMessage::Push {
                message: "Two".into(),
                level: ToastLevel::Info,
                duration_ms: None,
            },
        );

        assert_eq!(out1, Some(ToastOutput::Added(0)));
        assert_eq!(out2, Some(ToastOutput::Added(1)));
    }

    #[test]
    fn test_push_custom_duration() {
        let mut state = ToastState::with_duration(3000);

        Toast::update(
            &mut state,
            ToastMessage::Push {
                message: "Custom".into(),
                level: ToastLevel::Info,
                duration_ms: Some(10000),
            },
        );

        assert_eq!(state.toasts()[0].remaining_ms(), Some(10000));
    }

    #[test]
    fn test_push_persistent() {
        let mut state = ToastState::new();

        Toast::update(
            &mut state,
            ToastMessage::Push {
                message: "Persistent".into(),
                level: ToastLevel::Info,
                duration_ms: None,
            },
        );

        assert!(state.toasts()[0].is_persistent());
    }

    // ========================================
    // Dismiss Message Tests
    // ========================================

    #[test]
    fn test_dismiss() {
        let mut state = ToastState::new();
        let id = state.info("Test");

        Toast::update(&mut state, ToastMessage::Dismiss(id));

        assert!(state.is_empty());
    }

    #[test]
    fn test_dismiss_returns_dismissed() {
        let mut state = ToastState::new();
        let id = state.info("Test");

        let output = Toast::update(&mut state, ToastMessage::Dismiss(id));

        assert_eq!(output, Some(ToastOutput::Dismissed(id)));
    }

    #[test]
    fn test_dismiss_nonexistent() {
        let mut state = ToastState::new();
        state.info("Test");

        let output = Toast::update(&mut state, ToastMessage::Dismiss(999));

        assert_eq!(output, None);
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_dismiss_preserves_others() {
        let mut state = ToastState::new();
        let id1 = state.info("One");
        let _id2 = state.info("Two");
        let id3 = state.info("Three");

        Toast::update(&mut state, ToastMessage::Dismiss(id1));

        assert_eq!(state.len(), 2);
        assert_eq!(state.toasts()[0].message(), "Two");
        assert_eq!(state.toasts()[1].id(), id3);
    }

    // ========================================
    // Clear Message Tests
    // ========================================

    #[test]
    fn test_clear() {
        let mut state = ToastState::new();
        state.info("One");
        state.info("Two");
        state.info("Three");

        Toast::update(&mut state, ToastMessage::Clear);

        assert!(state.is_empty());
    }

    #[test]
    fn test_clear_returns_cleared() {
        let mut state = ToastState::new();
        state.info("Test");

        let output = Toast::update(&mut state, ToastMessage::Clear);

        assert_eq!(output, Some(ToastOutput::Cleared));
    }

    #[test]
    fn test_clear_empty() {
        let mut state = ToastState::new();

        let output = Toast::update(&mut state, ToastMessage::Clear);

        assert_eq!(output, None);
    }

    // ========================================
    // Tick Message Tests
    // ========================================

    #[test]
    fn test_tick_decrements() {
        let mut state = ToastState::with_duration(3000);
        state.info("Test");

        Toast::update(&mut state, ToastMessage::Tick(1000));

        assert_eq!(state.toasts()[0].remaining_ms(), Some(2000));
    }

    #[test]
    fn test_tick_expires() {
        let mut state = ToastState::with_duration(1000);
        state.info("Test");

        Toast::update(&mut state, ToastMessage::Tick(1000));

        assert!(state.is_empty());
    }

    #[test]
    fn test_tick_returns_expired() {
        let mut state = ToastState::with_duration(1000);
        let id = state.info("Test");

        let output = Toast::update(&mut state, ToastMessage::Tick(1000));

        assert_eq!(output, Some(ToastOutput::Expired(id)));
    }

    #[test]
    fn test_tick_persistent() {
        let mut state = ToastState::new();
        state.info("Persistent");

        Toast::update(&mut state, ToastMessage::Tick(10000));

        // Persistent toast should not be affected
        assert_eq!(state.len(), 1);
        assert!(state.toasts()[0].is_persistent());
    }

    #[test]
    fn test_tick_multiple_expire() {
        let mut state = ToastState::with_duration(1000);
        state.info("One");
        state.info("Two");

        let output = Toast::update(&mut state, ToastMessage::Tick(1000));

        // Both should expire, but we only return the first
        assert!(state.is_empty());
        assert!(matches!(output, Some(ToastOutput::Expired(_))));
    }

    #[test]
    fn test_tick_no_expire() {
        let mut state = ToastState::with_duration(3000);
        state.info("Test");

        let output = Toast::update(&mut state, ToastMessage::Tick(100));

        assert_eq!(output, None);
        assert_eq!(state.len(), 1);
    }

    // ========================================
    // View Tests
    // ========================================

    #[test]
    fn test_view_empty() {
        let state = ToastState::new();

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Toast::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        // Should render nothing
        let output = terminal.backend().to_string();
        assert!(output.trim().is_empty());
    }

    #[test]
    fn test_view_single() {
        let mut state = ToastState::new();
        state.info("Hello, world!");

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Toast::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Hello, world!"));
    }

    #[test]
    fn test_view_multiple() {
        let mut state = ToastState::new();
        state.info("Message 1");
        state.success("Message 2");
        state.error("Message 3");

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Toast::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Message 1"));
        assert!(output.contains("Message 2"));
        assert!(output.contains("Message 3"));
    }

    #[test]
    fn test_view_max_visible() {
        let mut state = ToastState::with_max_visible(2);
        state.info("Message 1");
        state.info("Message 2");
        state.info("Message 3");

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Toast::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        // Only newest 2 should be visible (Message 2 and Message 3)
        let output = terminal.backend().to_string();
        // Note: reversed order for rendering - newest at bottom
        assert!(output.contains("Message 3"));
        assert!(output.contains("Message 2"));
    }

    #[test]
    fn test_view_info_style() {
        let mut state = ToastState::new();
        state.info("Info message");

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Toast::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("[i]"));
    }

    #[test]
    fn test_view_success_style() {
        let mut state = ToastState::new();
        state.success("Success message");

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Toast::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("[+]"));
    }

    #[test]
    fn test_view_warning_style() {
        let mut state = ToastState::new();
        state.warning("Warning message");

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Toast::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("[!]"));
    }

    #[test]
    fn test_view_error_style() {
        let mut state = ToastState::new();
        state.error("Error message");

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Toast::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("[x]"));
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_clone() {
        let mut state = ToastState::with_duration(3000);
        state.info("Test");
        state.success("Test 2");

        let cloned = state.clone();
        assert_eq!(cloned.len(), 2);
        assert_eq!(cloned.default_duration(), Some(3000));
    }

    #[test]
    fn test_init() {
        let state = Toast::init();
        assert!(state.is_empty());
        assert_eq!(state.default_duration(), None);
    }

    #[test]
    fn test_full_workflow() {
        let mut state = ToastState::with_duration(3000);

        // Add some toasts
        let id1 = state.success("File saved!");
        let id2 = state.info("Processing...");

        assert_eq!(state.len(), 2);

        // Tick some time
        Toast::update(&mut state, ToastMessage::Tick(1000));
        assert_eq!(state.toasts()[0].remaining_ms(), Some(2000));

        // Dismiss one
        Toast::update(&mut state, ToastMessage::Dismiss(id1));
        assert_eq!(state.len(), 1);
        assert_eq!(state.toasts()[0].id(), id2);

        // Tick until expire
        let output = Toast::update(&mut state, ToastMessage::Tick(2000));
        assert_eq!(output, Some(ToastOutput::Expired(id2)));
        assert!(state.is_empty());
    }

    #[test]
    fn test_mixed_durations() {
        let mut state = ToastState::new();

        // Add persistent toast
        let persistent_id = state.info("Persistent");

        // Add timed toast via message
        Toast::update(
            &mut state,
            ToastMessage::Push {
                message: "Timed".into(),
                level: ToastLevel::Warning,
                duration_ms: Some(1000),
            },
        );

        assert_eq!(state.len(), 2);

        // Tick past timed duration
        Toast::update(&mut state, ToastMessage::Tick(1000));

        // Only persistent should remain
        assert_eq!(state.len(), 1);
        assert_eq!(state.toasts()[0].id(), persistent_id);
        assert!(state.toasts()[0].is_persistent());
    }

    #[test]
    fn test_set_default_duration() {
        let mut state = ToastState::new();
        assert_eq!(state.default_duration(), None);

        state.set_default_duration(Some(5000));
        assert_eq!(state.default_duration(), Some(5000));

        state.info("Test");
        assert_eq!(state.toasts()[0].remaining_ms(), Some(5000));
    }

    #[test]
    fn test_set_max_visible() {
        let mut state = ToastState::new();
        assert_eq!(state.max_visible(), DEFAULT_MAX_VISIBLE);

        state.set_max_visible(3);
        assert_eq!(state.max_visible(), 3);
    }
}
