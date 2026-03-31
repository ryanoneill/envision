//! A numeric input component with validation, increment/decrement, and optional bounds.
//!
//! [`NumberInput`] provides a focused numeric entry field that supports
//! Up/Down (or k/j) for increment/decrement, Enter to switch into text-edit
//! mode, and optional min/max clamping.
//!
//! State is stored in [`NumberInputState`], updated via [`NumberInputMessage`],
//! and produces [`NumberInputOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! See also [`Slider`](super::Slider) for range selection with a visual track,
//! and [`InputField`](super::InputField) for general text input.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{NumberInput, NumberInputMessage, NumberInputOutput, NumberInputState, Component};
//!
//! // Create a number input starting at 42
//! let mut state = NumberInputState::new(42.0);
//! assert_eq!(state.value(), 42.0);
//!
//! // Increment the value
//! let output = NumberInput::update(&mut state, NumberInputMessage::Increment);
//! assert_eq!(output, Some(NumberInputOutput::ValueChanged(43.0)));
//! assert_eq!(state.value(), 43.0);
//!
//! // Set value directly
//! let output = NumberInput::update(&mut state, NumberInputMessage::SetValue(100.0));
//! assert_eq!(output, Some(NumberInputOutput::ValueChanged(100.0)));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, Disableable, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Messages that can be sent to a NumberInput.
#[derive(Clone, Debug, PartialEq)]
pub enum NumberInputMessage {
    /// Increase value by one step.
    Increment,
    /// Decrease value by one step.
    Decrement,
    /// Set value directly (clamped to bounds).
    SetValue(f64),
    /// Enter text edit mode.
    StartEdit,
    /// Parse edit buffer and apply the value.
    ConfirmEdit,
    /// Discard edit buffer and exit edit mode.
    CancelEdit,
    /// Append a character to the edit buffer.
    EditChar(char),
    /// Delete the last character from the edit buffer.
    EditBackspace,
}

/// Output messages from a NumberInput.
#[derive(Clone, Debug, PartialEq)]
pub enum NumberInputOutput {
    /// The numeric value changed. Contains the new value.
    ValueChanged(f64),
    /// Entered text edit mode.
    EditStarted,
    /// Edit confirmed with a new value.
    EditConfirmed(f64),
    /// Edit was cancelled.
    EditCancelled,
}

/// State for a NumberInput component.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct NumberInputState {
    /// The current numeric value.
    value: f64,
    /// Optional minimum bound.
    min: Option<f64>,
    /// Optional maximum bound.
    max: Option<f64>,
    /// Increment/decrement step size.
    step: f64,
    /// Decimal places to display.
    precision: usize,
    /// Optional label.
    label: Option<String>,
    /// Placeholder text shown when empty in edit mode.
    placeholder: Option<String>,
    /// Whether currently in text edit mode.
    editing: bool,
    /// Text buffer used during edit mode.
    edit_buffer: String,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for NumberInputState {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: None,
            max: None,
            step: 1.0,
            precision: 0,
            label: None,
            placeholder: None,
            editing: false,
            edit_buffer: String::new(),
            focused: false,
            disabled: false,
        }
    }
}

impl NumberInputState {
    /// Creates a new number input with the given initial value.
    ///
    /// Defaults to step 1.0 and precision 0 (integer display).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(42.0);
    /// assert_eq!(state.value(), 42.0);
    /// assert_eq!(state.format_value(), "42");
    /// ```
    pub fn new(value: f64) -> Self {
        Self {
            value,
            ..Self::default()
        }
    }

    /// Creates a new number input configured for integer values.
    ///
    /// Convenience constructor that sets precision to 0 and step to 1.0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::integer(42);
    /// assert_eq!(state.value(), 42.0);
    /// assert_eq!(state.format_value(), "42");
    /// ```
    pub fn integer(value: i64) -> Self {
        Self {
            value: value as f64,
            step: 1.0,
            precision: 0,
            ..Self::default()
        }
    }

    /// Sets the minimum bound (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(5.0).with_min(0.0);
    /// ```
    pub fn with_min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self.value = self.clamp(self.value);
        self
    }

    /// Sets the maximum bound (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(5.0).with_max(10.0);
    /// ```
    pub fn with_max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self.value = self.clamp(self.value);
        self
    }

    /// Sets both minimum and maximum bounds (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(5.0).with_range(0.0, 10.0);
    /// ```
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self.value = self.clamp(self.value);
        self
    }

    /// Sets the step size (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(0.0).with_step(0.5);
    /// ```
    pub fn with_step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Sets the decimal precision (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(3.75).with_precision(2);
    /// assert_eq!(state.format_value(), "3.75");
    /// ```
    pub fn with_precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Sets the label (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(0.0).with_label("Quantity");
    /// assert_eq!(state.label(), Some("Quantity"));
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the placeholder text (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(0.0).with_placeholder("Enter value...");
    /// ```
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(0.0).with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the current numeric value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(42.0);
    /// assert_eq!(state.value(), 42.0);
    /// ```
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Sets the current value, clamping to any configured bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let mut state = NumberInputState::new(0.0).with_range(0.0, 100.0);
    /// state.set_value(50.0);
    /// assert_eq!(state.value(), 50.0);
    ///
    /// state.set_value(200.0);
    /// assert_eq!(state.value(), 100.0);
    /// ```
    pub fn set_value(&mut self, value: f64) {
        self.value = self.clamp(value);
    }

    /// Returns true if the component is in text edit mode.
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Returns the current edit buffer contents.
    pub fn edit_buffer(&self) -> &str {
        &self.edit_buffer
    }

    /// Returns the label, if set.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the placeholder, if set.
    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    /// Sets the placeholder text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let mut state = NumberInputState::new(0.0);
    /// state.set_placeholder("Enter a number...");
    /// assert_eq!(state.placeholder(), Some("Enter a number..."));
    /// ```
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = Some(placeholder.into());
    }

    /// Returns the step size.
    pub fn step(&self) -> f64 {
        self.step
    }

    /// Returns the precision (decimal places).
    pub fn precision(&self) -> usize {
        self.precision
    }

    /// Returns the minimum bound, if set.
    pub fn min(&self) -> Option<f64> {
        self.min
    }

    /// Returns the maximum bound, if set.
    pub fn max(&self) -> Option<f64> {
        self.max
    }

    /// Formats the current value according to the configured precision.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::NumberInputState;
    ///
    /// let state = NumberInputState::new(42.0);
    /// assert_eq!(state.format_value(), "42");
    ///
    /// let state = NumberInputState::new(3.75).with_precision(2);
    /// assert_eq!(state.format_value(), "3.75");
    /// ```
    pub fn format_value(&self) -> String {
        format!("{:.prec$}", self.value, prec = self.precision)
    }

    /// Returns true if the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Maps an input event to a number input message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{NumberInputMessage, NumberInputState};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = NumberInputState::new(0.0);
    /// state.set_focused(true);
    /// let event = Event::key(KeyCode::Up);
    /// assert_eq!(state.handle_event(&event), Some(NumberInputMessage::Increment));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<NumberInputMessage> {
        NumberInput::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<NumberInputOutput> {
        NumberInput::dispatch_event(self, event)
    }

    /// Updates the number input state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{NumberInputMessage, NumberInputOutput, NumberInputState};
    ///
    /// let mut state = NumberInputState::new(10.0);
    /// let output = state.update(NumberInputMessage::Increment);
    /// assert_eq!(output, Some(NumberInputOutput::ValueChanged(11.0)));
    /// assert_eq!(state.value(), 11.0);
    /// ```
    pub fn update(&mut self, msg: NumberInputMessage) -> Option<NumberInputOutput> {
        NumberInput::update(self, msg)
    }

    /// Clamps a value to the configured bounds.
    fn clamp(&self, value: f64) -> f64 {
        let mut v = value;
        if let Some(min) = self.min {
            if v < min {
                v = min;
            }
        }
        if let Some(max) = self.max {
            if v > max {
                v = max;
            }
        }
        v
    }
}

/// A numeric input component with validation and increment/decrement support.
///
/// `NumberInput` provides a focused entry field for numeric values. It supports:
///
/// - **Increment/Decrement** via Up/Down arrow keys (or k/j)
/// - **Direct text editing** by pressing Enter to switch into edit mode
/// - **Validation** with optional min/max bounds
/// - **Configurable precision** for integer or floating-point display
///
/// # Keyboard Controls
///
/// Normal mode (not editing):
/// - Up / k: increment by step
/// - Down / j: decrement by step
/// - Enter: enter text edit mode
///
/// Edit mode:
/// - 0-9, '.', '-': append to edit buffer
/// - Backspace: delete last character
/// - Enter: confirm edit (parse and apply)
/// - Escape: cancel edit
///
/// # Visual Format
///
/// Normal mode:
/// ```text
/// ┌──────────────────┐
/// │ Label:       42  │
/// └──────────────────┘
/// ```
///
/// Edit mode:
/// ```text
/// ┌──────────────────┐
/// │ Label:      42_  │
/// └──────────────────┘
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{NumberInput, NumberInputMessage, NumberInputOutput, NumberInputState, Component, Focusable};
///
/// let mut state = NumberInputState::new(50.0)
///     .with_range(0.0, 100.0)
///     .with_step(5.0)
///     .with_label("Volume");
///
/// let output = NumberInput::update(&mut state, NumberInputMessage::Increment);
/// assert_eq!(output, Some(NumberInputOutput::ValueChanged(55.0)));
/// assert_eq!(state.value(), 55.0);
/// ```
pub struct NumberInput;

impl Component for NumberInput {
    type State = NumberInputState;
    type Message = NumberInputMessage;
    type Output = NumberInputOutput;

    fn init() -> Self::State {
        NumberInputState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            NumberInputMessage::Increment => {
                let old = state.value;
                let new = state.clamp(state.value + state.step);
                if (new - old).abs() > f64::EPSILON {
                    state.value = new;
                    Some(NumberInputOutput::ValueChanged(state.value))
                } else {
                    None
                }
            }
            NumberInputMessage::Decrement => {
                let old = state.value;
                let new = state.clamp(state.value - state.step);
                if (new - old).abs() > f64::EPSILON {
                    state.value = new;
                    Some(NumberInputOutput::ValueChanged(state.value))
                } else {
                    None
                }
            }
            NumberInputMessage::SetValue(v) => {
                let old = state.value;
                let new = state.clamp(v);
                if (new - old).abs() > f64::EPSILON {
                    state.value = new;
                    Some(NumberInputOutput::ValueChanged(state.value))
                } else {
                    None
                }
            }
            NumberInputMessage::StartEdit => {
                state.editing = true;
                state.edit_buffer = state.format_value();
                Some(NumberInputOutput::EditStarted)
            }
            NumberInputMessage::ConfirmEdit => {
                state.editing = false;
                if let Ok(parsed) = state.edit_buffer.parse::<f64>() {
                    let new = state.clamp(parsed);
                    state.value = new;
                    state.edit_buffer.clear();
                    Some(NumberInputOutput::EditConfirmed(state.value))
                } else {
                    // Invalid input: revert to current value
                    state.edit_buffer.clear();
                    Some(NumberInputOutput::EditCancelled)
                }
            }
            NumberInputMessage::CancelEdit => {
                state.editing = false;
                state.edit_buffer.clear();
                Some(NumberInputOutput::EditCancelled)
            }
            NumberInputMessage::EditChar(c) => {
                if is_valid_numeric_char(c, &state.edit_buffer) {
                    state.edit_buffer.push(c);
                }
                None
            }
            NumberInputMessage::EditBackspace => {
                state.edit_buffer.pop();
                None
            }
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        if let Some(key) = event.as_key() {
            if state.editing {
                // Edit mode key handling
                match key.code {
                    KeyCode::Enter => Some(NumberInputMessage::ConfirmEdit),
                    KeyCode::Esc => Some(NumberInputMessage::CancelEdit),
                    KeyCode::Backspace => Some(NumberInputMessage::EditBackspace),
                    KeyCode::Char(c) if is_valid_numeric_char(c, &state.edit_buffer) => {
                        Some(NumberInputMessage::EditChar(c))
                    }
                    _ => None,
                }
            } else {
                // Normal mode key handling
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => Some(NumberInputMessage::Increment),
                    KeyCode::Down | KeyCode::Char('j') => Some(NumberInputMessage::Decrement),
                    KeyCode::Enter => Some(NumberInputMessage::StartEdit),
                    _ => None,
                }
            }
        } else {
            None
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let border_style = if state.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        let content_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_style()
        } else {
            theme.normal_style()
        };

        // Build the display text
        let display_text = if state.editing {
            let cursor = "_";
            if state.edit_buffer.is_empty() {
                if let Some(placeholder) = &state.placeholder {
                    placeholder.clone()
                } else {
                    cursor.to_string()
                }
            } else {
                format!("{}{cursor}", state.edit_buffer)
            }
        } else {
            state.format_value()
        };

        // Build the full line with optional label
        let full_text = if let Some(label) = &state.label {
            format!("{label}: {display_text}")
        } else {
            display_text
        };

        let paragraph = Paragraph::new(full_text)
            .style(content_style)
            .block(block)
            .alignment(Alignment::Right);

        // Register annotation
        let value_str = state.format_value();
        let annotation = crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
            "NumberInput".to_string(),
        ))
        .with_id("number_input")
        .with_value(value_str);

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

/// Returns true if the character is valid for numeric input.
///
/// Allows digits, a single decimal point, and a leading minus sign.
fn is_valid_numeric_char(c: char, buffer: &str) -> bool {
    match c {
        '0'..='9' => true,
        '.' => !buffer.contains('.'),
        '-' => buffer.is_empty(),
        _ => false,
    }
}

impl Focusable for NumberInput {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for NumberInput {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;
