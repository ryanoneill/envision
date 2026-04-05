//! A toggleable checkbox component with keyboard activation.
//!
//! [`Checkbox`] provides a boolean input that can be toggled via keyboard
//! (Enter or Space) when focused. State is stored in [`CheckboxState`],
//! updated via [`CheckboxMessage`], and produces [`CheckboxOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! See also [`Button`](super::Button) for a press-only action.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Checkbox, CheckboxMessage, CheckboxOutput, CheckboxState, Component};
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

use super::{Component, ViewContext};
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
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct CheckboxState {
    /// The checkbox label.
    label: String,
    /// Whether the checkbox is checked.
    checked: bool,
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
        }
    }

    /// Returns the checkbox label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CheckboxState;
    ///
    /// let state = CheckboxState::new("Accept terms");
    /// assert_eq!(state.label(), "Accept terms");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the checkbox label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CheckboxState;
    ///
    /// let mut state = CheckboxState::new("Accept");
    /// state.set_label("I agree to terms");
    /// assert_eq!(state.label(), "I agree to terms");
    /// ```
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Returns true if the checkbox is checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Sets the checked state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CheckboxState;
    ///
    /// let mut state = CheckboxState::new("Opt in");
    /// state.set_checked(true);
    /// assert!(state.is_checked());
    /// ```
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// Sets the checkbox label using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CheckboxState;
    ///
    /// let state = CheckboxState::new("Accept").with_label("Accept all terms");
    /// assert_eq!(state.label(), "Accept all terms");
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the checked state using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CheckboxState;
    ///
    /// let state = CheckboxState::new("Dark mode").with_checked(true);
    /// assert!(state.is_checked());
    /// ```
    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Updates the checkbox state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CheckboxMessage, CheckboxOutput, CheckboxState};
    ///
    /// let mut state = CheckboxState::new("Dark mode");
    /// let output = state.update(CheckboxMessage::Toggle);
    /// assert_eq!(output, Some(CheckboxOutput::Toggled(true)));
    /// assert!(state.is_checked());
    /// ```
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
                state.checked = !state.checked;
                Some(CheckboxOutput::Toggled(state.checked))
            }
        }
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        let check_mark = if state.checked { "x" } else { " " };
        let text = format!("[{}] {}", check_mark, state.label);

        let style = if ctx.disabled {
            theme.disabled_style()
        } else if ctx.focused {
            theme.focused_style()
        } else {
            theme.normal_style()
        };

        let paragraph = Paragraph::new(text).style(style);

        let annotation = crate::annotation::Annotation::checkbox("checkbox")
            .with_label(state.label.as_str())
            .with_selected(state.checked);
        let annotated = crate::annotation::Annotate::new(paragraph, annotation)
            .focused(ctx.focused)
            .disabled(ctx.disabled);
        frame.render_widget(annotated, area);
    }
}

#[cfg(test)]
mod tests;
