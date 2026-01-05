//! A toggleable checkbox component with keyboard activation.
//!
//! `Checkbox` provides a boolean input that can be toggled via keyboard
//! (Enter or Space) when focused.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Checkbox, CheckboxMessage, CheckboxOutput, CheckboxState, Component, Focusable};
//!
//! // Create an unchecked checkbox
//! let mut state = CheckboxState::new("Accept terms");
//!
//! // Toggle it
//! let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
//! assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
//! assert!(state.is_checked());
//!
//! // Toggle again
//! let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
//! assert_eq!(output, Some(CheckboxOutput::Toggled(false)));
//! assert!(!state.is_checked());
//!
//! // Disabled checkboxes don't toggle
//! state.set_disabled(true);
//! let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
//! assert_eq!(output, None);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, Focusable};
use crate::theme::Theme;

/// Messages that can be sent to a Checkbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckboxMessage {
    /// Toggle the checkbox state.
    Toggle,
}

/// Output messages from a Checkbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckboxOutput {
    /// The checkbox was toggled. Contains the new checked state.
    Toggled(bool),
}

/// State for a Checkbox component.
#[derive(Clone, Debug, Default)]
pub struct CheckboxState {
    /// The checkbox label.
    label: String,
    /// Whether the checkbox is checked.
    checked: bool,
    /// Whether the checkbox is focused.
    focused: bool,
    /// Whether the checkbox is disabled.
    disabled: bool,
}

impl CheckboxState {
    /// Creates a new unchecked checkbox with the given label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CheckboxState;
    ///
    /// let state = CheckboxState::new("Enable notifications");
    /// assert_eq!(state.label(), "Enable notifications");
    /// assert!(!state.is_checked());
    /// ```
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: false,
            focused: false,
            disabled: false,
        }
    }

    /// Creates a new checked checkbox with the given label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CheckboxState;
    ///
    /// let state = CheckboxState::checked("Remember me");
    /// assert!(state.is_checked());
    /// ```
    pub fn checked(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: true,
            focused: false,
            disabled: false,
        }
    }

    /// Returns the checkbox label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the checkbox label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Returns true if the checkbox is checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Sets the checked state.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// Returns true if the checkbox is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// Disabled checkboxes do not respond to toggle events.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }
}

/// A toggleable checkbox component.
///
/// This component provides a boolean input that can be toggled via
/// keyboard when focused. The checkbox emits a [`CheckboxOutput::Toggled`]
/// message containing the new state when toggled.
///
/// # Keyboard Activation
///
/// The checkbox itself doesn't handle keyboard events directly. Your
/// application should map Enter/Space keys to [`CheckboxMessage::Toggle`]
/// when the checkbox is focused.
///
/// # Visual States
///
/// - **Unchecked**: `[ ] Label`
/// - **Checked**: `[x] Label`
/// - **Focused**: Yellow text
/// - **Disabled**: Dark gray text, doesn't respond to toggle
///
/// # Example
///
/// ```rust
/// use envision::component::{Checkbox, CheckboxMessage, CheckboxOutput, CheckboxState, Component};
///
/// let mut state = CheckboxState::new("Dark mode");
///
/// // Toggle the checkbox
/// let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
/// assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
/// assert!(state.is_checked());
/// ```
pub struct Checkbox;

impl Component for Checkbox {
    type State = CheckboxState;
    type Message = CheckboxMessage;
    type Output = CheckboxOutput;

    fn init() -> Self::State {
        CheckboxState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            CheckboxMessage::Toggle => {
                if state.disabled {
                    None
                } else {
                    state.checked = !state.checked;
                    Some(CheckboxOutput::Toggled(state.checked))
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let check_mark = if state.checked { "x" } else { " " };
        let text = format!("[{}] {}", check_mark, state.label);

        let style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_style()
        } else {
            theme.normal_style()
        };

        let paragraph = Paragraph::new(text).style(style);
        frame.render_widget(paragraph, area);
    }
}

impl Focusable for Checkbox {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = CheckboxState::new("Test label");
        assert_eq!(state.label(), "Test label");
        assert!(!state.is_checked());
        assert!(!state.is_disabled());
        assert!(!Checkbox::is_focused(&state));
    }

    #[test]
    fn test_checked_constructor() {
        let state = CheckboxState::checked("Checked label");
        assert_eq!(state.label(), "Checked label");
        assert!(state.is_checked());
        assert!(!state.is_disabled());
    }

    #[test]
    fn test_default() {
        let state = CheckboxState::default();
        assert_eq!(state.label(), "");
        assert!(!state.is_checked());
        assert!(!state.is_disabled());
        assert!(!Checkbox::is_focused(&state));
    }

    #[test]
    fn test_label_accessors() {
        let mut state = CheckboxState::new("Original");
        assert_eq!(state.label(), "Original");

        state.set_label("Updated");
        assert_eq!(state.label(), "Updated");
    }

    #[test]
    fn test_checked_accessors() {
        let mut state = CheckboxState::new("Test");
        assert!(!state.is_checked());

        state.set_checked(true);
        assert!(state.is_checked());

        state.set_checked(false);
        assert!(!state.is_checked());
    }

    #[test]
    fn test_disabled_accessors() {
        let mut state = CheckboxState::new("Test");
        assert!(!state.is_disabled());

        state.set_disabled(true);
        assert!(state.is_disabled());

        state.set_disabled(false);
        assert!(!state.is_disabled());
    }

    #[test]
    fn test_toggle_unchecked() {
        let mut state = CheckboxState::new("Test");
        assert!(!state.is_checked());

        let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
        assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
        assert!(state.is_checked());
    }

    #[test]
    fn test_toggle_checked() {
        let mut state = CheckboxState::checked("Test");
        assert!(state.is_checked());

        let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
        assert_eq!(output, Some(CheckboxOutput::Toggled(false)));
        assert!(!state.is_checked());
    }

    #[test]
    fn test_toggle_disabled() {
        let mut state = CheckboxState::new("Test");
        state.set_disabled(true);

        let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
        assert_eq!(output, None);
        assert!(!state.is_checked()); // State unchanged
    }

    #[test]
    fn test_toggle_disabled_when_checked() {
        let mut state = CheckboxState::checked("Test");
        state.set_disabled(true);

        let output = Checkbox::update(&mut state, CheckboxMessage::Toggle);
        assert_eq!(output, None);
        assert!(state.is_checked()); // State unchanged
    }

    #[test]
    fn test_focusable() {
        let mut state = CheckboxState::new("Test");

        assert!(!Checkbox::is_focused(&state));

        Checkbox::set_focused(&mut state, true);
        assert!(Checkbox::is_focused(&state));

        Checkbox::blur(&mut state);
        assert!(!Checkbox::is_focused(&state));

        Checkbox::focus(&mut state);
        assert!(Checkbox::is_focused(&state));
    }

    #[test]
    fn test_init() {
        let state = Checkbox::init();
        assert_eq!(state.label(), "");
        assert!(!state.is_checked());
        assert!(!state.is_disabled());
        assert!(!Checkbox::is_focused(&state));
    }

    #[test]
    fn test_clone() {
        let state = CheckboxState::checked("Clone me");
        let cloned = state.clone();
        assert_eq!(cloned.label(), "Clone me");
        assert!(cloned.is_checked());
    }

    #[test]
    fn test_view_unchecked() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = CheckboxState::new("Unchecked");
        let theme = Theme::default();

        let backend = CaptureBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Checkbox::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("[ ] Unchecked"));
    }

    #[test]
    fn test_view_checked() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = CheckboxState::checked("Checked");
        let theme = Theme::default();

        let backend = CaptureBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Checkbox::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("[x] Checked"));
    }

    #[test]
    fn test_view_focused() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = CheckboxState::new("Focused");
        Checkbox::set_focused(&mut state, true);
        let theme = Theme::default();

        let backend = CaptureBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Checkbox::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("[ ] Focused"));
    }

    #[test]
    fn test_view_disabled() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = CheckboxState::new("Disabled");
        state.set_disabled(true);
        let theme = Theme::default();

        let backend = CaptureBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Checkbox::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("[ ] Disabled"));
    }

    #[test]
    fn test_multiple_toggles() {
        let mut state = CheckboxState::new("Test");

        // Toggle multiple times
        Checkbox::update(&mut state, CheckboxMessage::Toggle);
        assert!(state.is_checked());

        Checkbox::update(&mut state, CheckboxMessage::Toggle);
        assert!(!state.is_checked());

        Checkbox::update(&mut state, CheckboxMessage::Toggle);
        assert!(state.is_checked());
    }
}
