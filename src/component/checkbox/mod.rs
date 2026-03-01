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
use crate::input::{Event, KeyCode};
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

    /// Returns true if the checkbox is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Maps an input event to a checkbox message.
    pub fn handle_event(&self, event: &Event) -> Option<CheckboxMessage> {
        Checkbox::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<CheckboxOutput> {
        Checkbox::dispatch_event(self, event)
    }

    /// Updates the checkbox state with a message, returning any output.
    pub fn update(&mut self, msg: CheckboxMessage) -> Option<CheckboxOutput> {
        Checkbox::update(self, msg)
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

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => Some(CheckboxMessage::Toggle),
                _ => None,
            }
        } else {
            None
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
mod tests;
