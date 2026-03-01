//! A clickable button component with keyboard activation.
//!
//! `Button` provides a simple button that can be activated via keyboard
//! (Enter or Space) when focused.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Button, ButtonMessage, ButtonOutput, ButtonState, Component, Focusable};
//!
//! // Create a button
//! let mut state = ButtonState::new("Submit");
//!
//! // Focus it
//! Button::set_focused(&mut state, true);
//!
//! // Press it
//! let output = Button::update(&mut state, ButtonMessage::Press);
//! assert_eq!(output, Some(ButtonOutput::Pressed));
//!
//! // Disabled buttons don't emit output
//! state.set_disabled(true);
//! let output = Button::update(&mut state, ButtonMessage::Press);
//! assert_eq!(output, None);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Messages that can be sent to a Button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ButtonMessage {
    /// Press/activate the button (typically Enter or Space).
    Press,
}

/// Output messages from a Button.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ButtonOutput {
    /// The button was pressed.
    Pressed,
}

/// State for a Button component.
#[derive(Clone, Debug, Default)]
pub struct ButtonState {
    /// The button label.
    label: String,
    /// Whether the button is focused.
    focused: bool,
    /// Whether the button is disabled.
    disabled: bool,
}

impl ButtonState {
    /// Creates a new button with the given label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ButtonState;
    ///
    /// let state = ButtonState::new("Click me");
    /// assert_eq!(state.label(), "Click me");
    /// ```
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            focused: false,
            disabled: false,
        }
    }

    /// Returns the button label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the button label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Returns true if the button is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// Disabled buttons do not respond to press events.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Returns true if the button is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Maps an input event to a button message.
    pub fn handle_event(&self, event: &Event) -> Option<ButtonMessage> {
        Button::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<ButtonOutput> {
        Button::dispatch_event(self, event)
    }

    /// Updates the button state with a message, returning any output.
    pub fn update(&mut self, msg: ButtonMessage) -> Option<ButtonOutput> {
        Button::update(self, msg)
    }
}

/// A clickable button component.
///
/// This component provides a simple button that can be activated via
/// keyboard when focused. The button emits a [`ButtonOutput::Pressed`]
/// message when activated.
///
/// # Keyboard Activation
///
/// The button itself doesn't handle keyboard events directly. Your
/// application should map Enter/Space keys to [`ButtonMessage::Press`]
/// when the button is focused.
///
/// # Visual States
///
/// - **Normal**: Default styling
/// - **Focused**: Yellow border and text
/// - **Disabled**: Dark gray text, doesn't respond to press
///
/// # Example
///
/// ```rust
/// use envision::component::{Button, ButtonMessage, ButtonOutput, ButtonState, Component};
///
/// let mut state = ButtonState::new("Save");
///
/// // Press the button
/// let output = Button::update(&mut state, ButtonMessage::Press);
/// assert_eq!(output, Some(ButtonOutput::Pressed));
/// ```
pub struct Button;

impl Component for Button {
    type State = ButtonState;
    type Message = ButtonMessage;
    type Output = ButtonOutput;

    fn init() -> Self::State {
        ButtonState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ButtonMessage::Press => {
                if state.disabled {
                    None
                } else {
                    Some(ButtonOutput::Pressed)
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
                KeyCode::Enter | KeyCode::Char(' ') => Some(ButtonMessage::Press),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_style()
        } else {
            theme.normal_style()
        };

        let border_style = if state.focused && !state.disabled {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let paragraph = Paragraph::new(state.label.as_str())
            .style(style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

        frame.render_widget(paragraph, area);
    }
}

impl Focusable for Button {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
