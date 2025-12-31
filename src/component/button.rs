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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        let style = if state.disabled {
            Style::default().fg(Color::DarkGray)
        } else if state.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let border_style = if state.focused && !state.disabled {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
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
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = ButtonState::new("Click me");
        assert_eq!(state.label(), "Click me");
        assert!(!state.is_disabled());
        assert!(!Button::is_focused(&state));
    }

    #[test]
    fn test_default() {
        let state = ButtonState::default();
        assert_eq!(state.label(), "");
        assert!(!state.is_disabled());
        assert!(!Button::is_focused(&state));
    }

    #[test]
    fn test_label_accessors() {
        let mut state = ButtonState::new("Original");
        assert_eq!(state.label(), "Original");

        state.set_label("Updated");
        assert_eq!(state.label(), "Updated");
    }

    #[test]
    fn test_disabled_accessors() {
        let mut state = ButtonState::new("Test");
        assert!(!state.is_disabled());

        state.set_disabled(true);
        assert!(state.is_disabled());

        state.set_disabled(false);
        assert!(!state.is_disabled());
    }

    #[test]
    fn test_press_enabled() {
        let mut state = ButtonState::new("Submit");

        let output = Button::update(&mut state, ButtonMessage::Press);
        assert_eq!(output, Some(ButtonOutput::Pressed));
    }

    #[test]
    fn test_press_disabled() {
        let mut state = ButtonState::new("Submit");
        state.set_disabled(true);

        let output = Button::update(&mut state, ButtonMessage::Press);
        assert_eq!(output, None);
    }

    #[test]
    fn test_focusable() {
        let mut state = ButtonState::new("Test");

        assert!(!Button::is_focused(&state));

        Button::set_focused(&mut state, true);
        assert!(Button::is_focused(&state));

        Button::blur(&mut state);
        assert!(!Button::is_focused(&state));

        Button::focus(&mut state);
        assert!(Button::is_focused(&state));
    }

    #[test]
    fn test_init() {
        let state = Button::init();
        assert_eq!(state.label(), "");
        assert!(!state.is_disabled());
        assert!(!Button::is_focused(&state));
    }

    #[test]
    fn test_clone() {
        let state = ButtonState::new("Clone me");
        let cloned = state.clone();
        assert_eq!(cloned.label(), "Clone me");
    }

    #[test]
    fn test_view() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = ButtonState::new("Click");

        let backend = CaptureBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Button::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Click"));
    }

    #[test]
    fn test_view_focused() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = ButtonState::new("Focused");
        Button::set_focused(&mut state, true);

        let backend = CaptureBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Button::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Focused"));
    }

    #[test]
    fn test_view_disabled() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = ButtonState::new("Disabled");
        state.set_disabled(true);

        let backend = CaptureBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Button::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Disabled"));
    }
}
