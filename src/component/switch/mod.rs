//! An on/off toggle switch component with keyboard activation.
//!
//! [`Switch`] provides a boolean on/off input that can be toggled via keyboard
//! (Enter or Space) when focused. Visually distinct from [`Checkbox`](super::Checkbox),
//! it displays a sliding toggle indicator rather than a checkmark.
//!
//! State is stored in [`SwitchState`], updated via [`SwitchMessage`], and
//! produces [`SwitchOutput`].
//!
//! Implements [`Focusable`], [`Disableable`], and [`Toggleable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Switch, SwitchMessage, SwitchOutput, SwitchState, Component, Focusable};
//!
//! // Create an off switch
//! let mut state = SwitchState::new();
//! assert!(!state.is_on());
//!
//! // Toggle it on
//! let output = Switch::update(&mut state, SwitchMessage::Toggle);
//! assert_eq!(output, Some(SwitchOutput::On));
//! assert!(state.is_on());
//!
//! // Toggle it off
//! let output = Switch::update(&mut state, SwitchMessage::Toggle);
//! assert_eq!(output, Some(SwitchOutput::Off));
//! assert!(!state.is_on());
//!
//! // Disabled switches don't toggle
//! state.set_disabled(true);
//! let output = Switch::update(&mut state, SwitchMessage::Toggle);
//! assert_eq!(output, None);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, Disableable, Focusable, Toggleable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Messages that can be sent to a Switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitchMessage {
    /// Toggle the switch state.
    Toggle,
    /// Set the switch to a specific on/off state.
    SetOn(bool),
    /// Set the switch label.
    SetLabel(Option<String>),
}

/// Output messages from a Switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitchOutput {
    /// The switch was toggled. Contains the new on/off value.
    Toggled(bool),
    /// The switch was turned on.
    On,
    /// The switch was turned off.
    Off,
}

/// State for a Switch component.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SwitchState {
    /// Whether the switch is on.
    on: bool,
    /// Optional label displayed next to the switch.
    label: Option<String>,
    /// Text shown when the switch is on.
    on_label: String,
    /// Text shown when the switch is off.
    off_label: String,
    /// Whether the switch is focused.
    focused: bool,
    /// Whether the switch is disabled.
    disabled: bool,
}

impl Default for SwitchState {
    fn default() -> Self {
        Self {
            on: false,
            label: None,
            on_label: "ON".to_string(),
            off_label: "OFF".to_string(),
            focused: false,
            disabled: false,
        }
    }
}

impl SwitchState {
    /// Creates a new switch in the off state with no label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let state = SwitchState::new();
    /// assert!(!state.is_on());
    /// assert!(state.label().is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial on/off state using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let state = SwitchState::new().with_on(true);
    /// assert!(state.is_on());
    /// ```
    pub fn with_on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// Sets the label using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let state = SwitchState::new().with_label("Dark Mode");
    /// assert_eq!(state.label(), Some("Dark Mode"));
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the text shown when the switch is on using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let state = SwitchState::new().with_on_label("YES");
    /// ```
    pub fn with_on_label(mut self, on_label: impl Into<String>) -> Self {
        self.on_label = on_label.into();
        self
    }

    /// Sets the text shown when the switch is off using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let state = SwitchState::new().with_off_label("NO");
    /// ```
    pub fn with_off_label(mut self, off_label: impl Into<String>) -> Self {
        self.off_label = off_label.into();
        self
    }

    /// Sets the disabled state using builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let state = SwitchState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns true if the switch is on.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let state = SwitchState::new().with_on(true);
    /// assert!(state.is_on());
    /// ```
    pub fn is_on(&self) -> bool {
        self.on
    }

    /// Sets the on/off state directly.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let mut state = SwitchState::new();
    /// state.set_on(true);
    /// assert!(state.is_on());
    /// ```
    pub fn set_on(&mut self, on: bool) {
        self.on = on;
    }

    /// Toggles the switch between on and off.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let mut state = SwitchState::new();
    /// assert!(!state.is_on());
    /// state.toggle();
    /// assert!(state.is_on());
    /// state.toggle();
    /// assert!(!state.is_on());
    /// ```
    pub fn toggle(&mut self) {
        self.on = !self.on;
    }

    /// Returns the optional label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let state = SwitchState::new().with_label("Notifications");
    /// assert_eq!(state.label(), Some("Notifications"));
    ///
    /// let state = SwitchState::new();
    /// assert_eq!(state.label(), None);
    /// ```
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Sets the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SwitchState;
    ///
    /// let mut state = SwitchState::new();
    /// state.set_label(Some("Wi-Fi".to_string()));
    /// assert_eq!(state.label(), Some("Wi-Fi"));
    /// state.set_label(None);
    /// assert_eq!(state.label(), None);
    /// ```
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    /// Returns true if the switch is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// Disabled switches do not respond to toggle events.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Returns true if the switch is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Maps an input event to a switch message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SwitchMessage, SwitchState};
    /// use envision::input::Event;
    ///
    /// let mut state = SwitchState::new();
    /// state.set_focused(true);
    /// let event = Event::key(envision::input::KeyCode::Enter);
    /// assert_eq!(state.handle_event(&event), Some(SwitchMessage::Toggle));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<SwitchMessage> {
        Switch::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<SwitchOutput> {
        Switch::dispatch_event(self, event)
    }

    /// Updates the switch state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SwitchMessage, SwitchOutput, SwitchState};
    ///
    /// let mut state = SwitchState::new();
    /// let output = state.update(SwitchMessage::Toggle);
    /// assert_eq!(output, Some(SwitchOutput::On));
    /// assert!(state.is_on());
    /// ```
    pub fn update(&mut self, msg: SwitchMessage) -> Option<SwitchOutput> {
        Switch::update(self, msg)
    }
}

/// An on/off toggle switch component.
///
/// This component provides a boolean on/off input that can be toggled via
/// keyboard when focused. The switch emits [`SwitchOutput::Toggled`],
/// [`SwitchOutput::On`], or [`SwitchOutput::Off`] messages when toggled.
///
/// # Keyboard Activation
///
/// When focused, Enter or Space toggles the switch state.
///
/// # Visual States
///
/// - **Off**: `( ) OFF`
/// - **On**: `(*) ON` with success/green coloring
/// - **Focused**: Yellow/focused text
/// - **Disabled**: Dark gray text, doesn't respond to toggle
/// - **With label**: `Label  (*) ON`
///
/// # Example
///
/// ```rust
/// use envision::component::{Switch, SwitchMessage, SwitchOutput, SwitchState, Component};
///
/// let mut state = SwitchState::new();
///
/// // Toggle the switch on
/// let output = Switch::update(&mut state, SwitchMessage::Toggle);
/// assert_eq!(output, Some(SwitchOutput::On));
/// assert!(state.is_on());
/// ```
pub struct Switch;

impl Component for Switch {
    type State = SwitchState;
    type Message = SwitchMessage;
    type Output = SwitchOutput;

    fn init() -> Self::State {
        SwitchState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            SwitchMessage::Toggle => {
                if state.disabled {
                    None
                } else {
                    state.on = !state.on;
                    if state.on {
                        Some(SwitchOutput::On)
                    } else {
                        Some(SwitchOutput::Off)
                    }
                }
            }
            SwitchMessage::SetOn(on) => {
                if state.disabled {
                    return None;
                }
                let changed = state.on != on;
                state.on = on;
                if changed {
                    Some(SwitchOutput::Toggled(on))
                } else {
                    None
                }
            }
            SwitchMessage::SetLabel(label) => {
                state.label = label;
                None
            }
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => Some(SwitchMessage::Toggle),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let indicator = if state.on {
            format!("(*) {}", state.on_label)
        } else {
            format!("( ) {}", state.off_label)
        };

        let text = if let Some(label) = &state.label {
            format!("{}  {}", label, indicator)
        } else {
            indicator
        };

        let style = if state.disabled {
            theme.disabled_style()
        } else if state.on {
            if state.focused {
                theme.focused_style()
            } else {
                theme.success_style()
            }
        } else if state.focused {
            theme.focused_style()
        } else {
            theme.normal_style()
        };

        let paragraph = Paragraph::new(text).style(style);

        let annotation = crate::annotation::Annotation::switch("switch").with_selected(state.on);
        let annotation = if let Some(label) = &state.label {
            annotation.with_label(label.as_str())
        } else {
            annotation
        };
        let annotated = crate::annotation::Annotate::new(paragraph, annotation)
            .focused(state.focused)
            .disabled(state.disabled);
        frame.render_widget(annotated, area);
    }
}

impl Focusable for Switch {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for Switch {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

impl Toggleable for Switch {
    fn is_visible(state: &Self::State) -> bool {
        state.on
    }

    fn set_visible(state: &mut Self::State, visible: bool) {
        state.on = visible;
    }
}

#[cfg(test)]
mod tests;
