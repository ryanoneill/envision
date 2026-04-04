//! A dynamic form component with multiple field types.
//!
//! [`Form`] composes [`InputField`](super::InputField),
//! [`Checkbox`](super::Checkbox), and [`Select`](super::Select) fields into
//! a navigable form. Tab/BackTab moves between fields, and submitting
//! collects all field values. State is stored in [`FormState`], updated via
//! [`FormMessage`], and produces [`FormOutput`]. Fields are defined with
//! [`FormField`] and [`FormFieldKind`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, Form, FormState, FormMessage,
//!     FormOutput, FormField, FormFieldKind, FormValue,
//! };
//!
//! let mut state = FormState::new(vec![
//!     FormField::text("name", "Name"),
//!     FormField::checkbox("agree", "I agree to the terms"),
//!     FormField::select("color", "Favorite color", vec!["Red", "Green", "Blue"]),
//! ]);
//! Form::set_focused(&mut state, true);
//!
//! // Fill in the name field (first field is focused by default)
//! Form::update(&mut state, FormMessage::Input('J'));
//! Form::update(&mut state, FormMessage::Input('o'));
//! Form::update(&mut state, FormMessage::Input('e'));
//! assert_eq!(state.value("name"), Some(FormValue::Text("Joe".into())));
//!
//! // Tab to checkbox and toggle it
//! Form::update(&mut state, FormMessage::FocusNext);
//! Form::update(&mut state, FormMessage::Toggle);
//! assert_eq!(state.value("agree"), Some(FormValue::Bool(true)));
//! ```

pub mod field;

pub use field::{FormField, FormFieldKind, FormValue};

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{
    Checkbox, CheckboxMessage, CheckboxState, Component, Disableable, Focusable, InputField,
    InputFieldMessage, InputFieldState, Select, SelectMessage, SelectState, ViewContext,
};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Internal representation of a field's widget state.
#[derive(Clone, Debug, PartialEq)]
enum FieldState {
    Text(InputFieldState),
    Checkbox(CheckboxState),
    Select(SelectState),
}

/// Messages that can be sent to a Form.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum FormMessage {
    /// Move focus to the next field.
    FocusNext,
    /// Move focus to the previous field.
    FocusPrev,
    /// Type a character into the focused text field.
    Input(char),
    /// Delete the character before the cursor.
    Backspace,
    /// Delete the character after the cursor.
    Delete,
    /// Move cursor left in a text field.
    Left,
    /// Move cursor right in a text field.
    Right,
    /// Move cursor to start of text field.
    Home,
    /// Move cursor to end of text field.
    End,
    /// Toggle a checkbox, open/navigate a select, or submit a text field.
    Toggle,
    /// Move selection up in a select dropdown.
    SelectUp,
    /// Move selection down in a select dropdown.
    SelectDown,
    /// Confirm the current select choice.
    SelectConfirm,
    /// Submit the entire form.
    Submit,
    /// Clear the focused text field.
    Clear,
}

/// Output messages from a Form.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum FormOutput {
    /// The form was submitted. Contains field ID-value pairs.
    Submitted(Vec<(String, FormValue)>),
    /// A field value changed.
    FieldChanged(String, FormValue),
}

/// State for a Form component.
///
/// Contains the field descriptors, their widget states, focus tracking,
/// and overall form state.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct FormState {
    /// Field descriptors (id, label, kind).
    fields: Vec<FormField>,
    /// Widget states corresponding to each field.
    #[cfg_attr(feature = "serialization", serde(skip))]
    states: Vec<FieldState>,
    /// Index of the currently focused field.
    focused_index: usize,
    /// Whether the form itself is focused.
    focused: bool,
    /// Whether the form is disabled.
    disabled: bool,
}

impl FormState {
    /// Creates a new form with the given field descriptors.
    ///
    /// The first field receives initial focus.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("name", "Name"),
    ///     FormField::checkbox("agree", "I agree"),
    /// ]);
    /// assert_eq!(state.field_count(), 2);
    /// assert_eq!(state.focused_field_id(), Some("name"));
    /// ```
    pub fn new(fields: Vec<FormField>) -> Self {
        let states: Vec<FieldState> = fields
            .iter()
            .map(|field| match &field.kind {
                FormFieldKind::Text => FieldState::Text(InputFieldState::new()),
                FormFieldKind::TextWithPlaceholder(p) => {
                    FieldState::Text(InputFieldState::with_placeholder(p))
                }
                FormFieldKind::Checkbox => FieldState::Checkbox(CheckboxState::new(&field.label)),
                FormFieldKind::Select(options) => {
                    FieldState::Select(SelectState::new(options.clone()))
                }
            })
            .collect();

        // Focus the first field
        let mut form = Self {
            fields,
            states,
            focused_index: 0,
            focused: false,
            disabled: false,
        };
        form.sync_field_focus();
        form
    }

    /// Returns the number of fields.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("a", "A"),
    ///     FormField::text("b", "B"),
    /// ]);
    /// assert_eq!(state.field_count(), 2);
    /// ```
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Returns the ID of the currently focused field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("name", "Name"),
    ///     FormField::checkbox("agree", "I agree"),
    /// ]);
    /// assert_eq!(state.focused_field_id(), Some("name"));
    /// ```
    pub fn focused_field_id(&self) -> Option<&str> {
        self.fields.get(self.focused_index).map(|f| f.id.as_str())
    }

    /// Returns the index of the currently focused field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("name", "Name"),
    /// ]);
    /// assert_eq!(state.focused_field_index(), 0);
    /// ```
    pub fn focused_field_index(&self) -> usize {
        self.focused_index
    }

    /// Returns true if the form is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![FormField::text("name", "Name")]);
    /// assert!(!state.is_focused());
    /// ```
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let mut state = FormState::new(vec![FormField::text("name", "Name")]);
    /// state.set_focused(true);
    /// assert!(state.is_focused());
    /// ```
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.sync_field_focus();
    }

    /// Returns true if the form is disabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![FormField::text("name", "Name")]);
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let mut state = FormState::new(vec![FormField::text("name", "Name")]);
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
        for state in &mut self.states {
            match state {
                FieldState::Text(s) => s.set_disabled(disabled),
                FieldState::Checkbox(s) => s.set_disabled(disabled),
                FieldState::Select(s) => s.set_disabled(disabled),
            }
        }
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![FormField::text("name", "Name")])
    ///     .with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.set_disabled(disabled);
        self
    }

    /// Returns the value of a field by its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField, FormValue};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("name", "Name"),
    ///     FormField::checkbox("agree", "I agree"),
    /// ]);
    /// assert_eq!(state.value("name"), Some(FormValue::Text(String::new())));
    /// assert_eq!(state.value("agree"), Some(FormValue::Bool(false)));
    /// assert_eq!(state.value("missing"), None);
    /// ```
    pub fn value(&self, id: &str) -> Option<FormValue> {
        self.fields
            .iter()
            .zip(self.states.iter())
            .find(|(field, _)| field.id == id)
            .map(|(_, state)| Self::extract_value(state))
    }

    /// Returns all field values as ID-value pairs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("name", "Name"),
    ///     FormField::checkbox("agree", "I agree"),
    /// ]);
    /// let values = state.values();
    /// assert_eq!(values.len(), 2);
    /// assert_eq!(values[0].0, "name");
    /// assert_eq!(values[1].0, "agree");
    /// ```
    pub fn values(&self) -> Vec<(String, FormValue)> {
        self.fields
            .iter()
            .zip(self.states.iter())
            .map(|(field, state)| (field.id.clone(), Self::extract_value(state)))
            .collect()
    }

    /// Returns the field descriptors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("name", "Name"),
    /// ]);
    /// assert_eq!(state.fields().len(), 1);
    /// assert_eq!(state.fields()[0].id(), "name");
    /// ```
    pub fn fields(&self) -> &[FormField] {
        &self.fields
    }

    /// Returns the label for a field at the given index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("name", "Full Name"),
    /// ]);
    /// assert_eq!(state.field_label(0), Some("Full Name"));
    /// assert_eq!(state.field_label(99), None);
    /// ```
    pub fn field_label(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(|f| f.label.as_str())
    }

    /// Returns true if the field at the given index is a text field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("name", "Name"),
    ///     FormField::checkbox("agree", "Agree"),
    /// ]);
    /// assert!(state.is_text_field(0));
    /// assert!(!state.is_text_field(1));
    /// ```
    pub fn is_text_field(&self, index: usize) -> bool {
        matches!(self.states.get(index), Some(FieldState::Text(_)))
    }

    /// Returns true if the field at the given index is a checkbox.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::text("name", "Name"),
    ///     FormField::checkbox("agree", "Agree"),
    /// ]);
    /// assert!(!state.is_checkbox_field(0));
    /// assert!(state.is_checkbox_field(1));
    /// ```
    pub fn is_checkbox_field(&self, index: usize) -> bool {
        matches!(self.states.get(index), Some(FieldState::Checkbox(_)))
    }

    /// Returns true if the field at the given index is a select.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField};
    ///
    /// let state = FormState::new(vec![
    ///     FormField::select("color", "Color", vec!["Red", "Blue"]),
    /// ]);
    /// assert!(state.is_select_field(0));
    /// ```
    pub fn is_select_field(&self, index: usize) -> bool {
        matches!(self.states.get(index), Some(FieldState::Select(_)))
    }

    /// Maps an input event to a form message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField, FormMessage};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = FormState::new(vec![FormField::text("name", "Name")]);
    /// state.set_focused(true);
    /// let event = Event::key(KeyCode::Tab);
    /// assert_eq!(state.handle_event(&event), Some(FormMessage::FocusNext));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<FormMessage> {
        Form::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField, FormOutput, FormValue};
    /// use envision::input::Event;
    ///
    /// let mut state = FormState::new(vec![FormField::text("name", "Name")]);
    /// state.set_focused(true);
    /// let event = Event::char('A');
    /// let output = state.dispatch_event(&event);
    /// assert!(matches!(output, Some(FormOutput::FieldChanged(_, FormValue::Text(_)))));
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<FormOutput> {
        Form::dispatch_event(self, event)
    }

    /// Updates the form state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FormState, FormField, FormMessage, FormOutput};
    ///
    /// let mut state = FormState::new(vec![
    ///     FormField::text("name", "Name"),
    /// ]);
    /// let output = state.update(FormMessage::Submit);
    /// assert!(matches!(output, Some(FormOutput::Submitted(_))));
    /// ```
    pub fn update(&mut self, msg: FormMessage) -> Option<FormOutput> {
        Form::update(self, msg)
    }

    /// Extracts the value from a field state.
    fn extract_value(state: &FieldState) -> FormValue {
        match state {
            FieldState::Text(s) => FormValue::Text(s.value().to_string()),
            FieldState::Checkbox(s) => FormValue::Bool(s.is_checked()),
            FieldState::Select(s) => FormValue::Selected(s.selected_item().map(|v| v.to_string())),
        }
    }

    /// Synchronizes focus state to the currently focused field.
    fn sync_field_focus(&mut self) {
        for (i, state) in self.states.iter_mut().enumerate() {
            let should_focus = self.focused && i == self.focused_index;
            match state {
                FieldState::Text(s) => s.set_focused(should_focus),
                FieldState::Checkbox(s) => s.set_focused(should_focus),
                FieldState::Select(s) => s.set_focused(should_focus),
            }
        }
    }

    /// Move focus to the next field, wrapping around.
    fn focus_next(&mut self) {
        if self.fields.is_empty() {
            return;
        }
        self.focused_index = (self.focused_index + 1) % self.fields.len();
        self.sync_field_focus();
    }

    /// Move focus to the previous field, wrapping around.
    fn focus_prev(&mut self) {
        if self.fields.is_empty() {
            return;
        }
        self.focused_index = if self.focused_index == 0 {
            self.fields.len() - 1
        } else {
            self.focused_index - 1
        };
        self.sync_field_focus();
    }
}

/// A dynamic form component with multiple field types.
///
/// `Form` composes text inputs, checkboxes, and select fields into a
/// single navigable component. It manages internal focus between fields
/// and collects values on submission.
///
/// # Navigation
///
/// - `Tab` — Focus next field
/// - `BackTab` (Shift+Tab) — Focus previous field
/// - `Ctrl+Enter` — Submit the form
///
/// # Field-specific keys
///
/// - **Text fields**: Normal typing, Backspace, Delete, Home/End, Left/Right
/// - **Checkbox**: Space or Enter to toggle
/// - **Select**: Enter to open, Up/Down to navigate, Enter to confirm
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, Focusable, Form, FormState, FormMessage,
///     FormOutput, FormField, FormValue,
/// };
///
/// let mut state = FormState::new(vec![
///     FormField::text("username", "Username"),
///     FormField::checkbox("remember", "Remember me"),
///     FormField::select("role", "Role", vec!["User", "Admin"]),
/// ]);
/// Form::set_focused(&mut state, true);
///
/// // Type username
/// Form::update(&mut state, FormMessage::Input('A'));
/// assert_eq!(state.value("username"), Some(FormValue::Text("A".into())));
///
/// // Submit the form
/// let output = Form::update(&mut state, FormMessage::Submit);
/// assert!(matches!(output, Some(FormOutput::Submitted(_))));
/// ```
pub struct Form;

impl Component for Form {
    type State = FormState;
    type Message = FormMessage;
    type Output = FormOutput;

    fn init() -> Self::State {
        FormState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled || state.fields.is_empty() {
            return None;
        }

        if let Some(key) = event.as_key() {
            // Global keys (regardless of field type)
            if key.code == KeyCode::Tab {
                return Some(FormMessage::FocusNext);
            }
            if key.code == KeyCode::BackTab {
                return Some(FormMessage::FocusPrev);
            }

            // Ctrl+Enter submits the form
            if key.code == KeyCode::Enter
                && key.modifiers.contains(crate::input::KeyModifiers::CONTROL)
            {
                return Some(FormMessage::Submit);
            }

            // Field-specific keys
            match &state.states.get(state.focused_index)? {
                FieldState::Text(_) => match key.code {
                    KeyCode::Char(c) => Some(FormMessage::Input(c)),
                    KeyCode::Backspace => Some(FormMessage::Backspace),
                    KeyCode::Delete => Some(FormMessage::Delete),
                    KeyCode::Left => Some(FormMessage::Left),
                    KeyCode::Right => Some(FormMessage::Right),
                    KeyCode::Home => Some(FormMessage::Home),
                    KeyCode::End => Some(FormMessage::End),
                    _ => None,
                },
                FieldState::Checkbox(_) => match key.code {
                    KeyCode::Char(' ') | KeyCode::Enter => Some(FormMessage::Toggle),
                    _ => None,
                },
                FieldState::Select(s) => {
                    if s.is_open() {
                        match key.code {
                            KeyCode::Up | KeyCode::Char('k') => Some(FormMessage::SelectUp),
                            KeyCode::Down | KeyCode::Char('j') => Some(FormMessage::SelectDown),
                            KeyCode::Enter => Some(FormMessage::SelectConfirm),
                            KeyCode::Esc => Some(FormMessage::Toggle),
                            _ => None,
                        }
                    } else {
                        match key.code {
                            KeyCode::Enter | KeyCode::Char(' ') => Some(FormMessage::Toggle),
                            _ => None,
                        }
                    }
                }
            }
        } else {
            None
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled || state.fields.is_empty() {
            return None;
        }

        match msg {
            FormMessage::FocusNext => {
                state.focus_next();
                None
            }
            FormMessage::FocusPrev => {
                state.focus_prev();
                None
            }
            FormMessage::Submit => {
                let values = state.values();
                Some(FormOutput::Submitted(values))
            }
            FormMessage::Input(c) => {
                if let Some(FieldState::Text(ref mut s)) = state.states.get_mut(state.focused_index)
                {
                    InputField::update(s, InputFieldMessage::Insert(c));
                    let id = state.fields[state.focused_index].id.clone();
                    let value = FormValue::Text(s.value().to_string());
                    Some(FormOutput::FieldChanged(id, value))
                } else {
                    None
                }
            }
            FormMessage::Backspace => {
                if let Some(FieldState::Text(ref mut s)) = state.states.get_mut(state.focused_index)
                {
                    InputField::update(s, InputFieldMessage::Backspace);
                    let id = state.fields[state.focused_index].id.clone();
                    let value = FormValue::Text(s.value().to_string());
                    Some(FormOutput::FieldChanged(id, value))
                } else {
                    None
                }
            }
            FormMessage::Delete => {
                if let Some(FieldState::Text(ref mut s)) = state.states.get_mut(state.focused_index)
                {
                    InputField::update(s, InputFieldMessage::Delete);
                    let id = state.fields[state.focused_index].id.clone();
                    let value = FormValue::Text(s.value().to_string());
                    Some(FormOutput::FieldChanged(id, value))
                } else {
                    None
                }
            }
            FormMessage::Left => {
                if let Some(FieldState::Text(ref mut s)) = state.states.get_mut(state.focused_index)
                {
                    InputField::update(s, InputFieldMessage::Left);
                }
                None
            }
            FormMessage::Right => {
                if let Some(FieldState::Text(ref mut s)) = state.states.get_mut(state.focused_index)
                {
                    InputField::update(s, InputFieldMessage::Right);
                }
                None
            }
            FormMessage::Home => {
                if let Some(FieldState::Text(ref mut s)) = state.states.get_mut(state.focused_index)
                {
                    InputField::update(s, InputFieldMessage::Home);
                }
                None
            }
            FormMessage::End => {
                if let Some(FieldState::Text(ref mut s)) = state.states.get_mut(state.focused_index)
                {
                    InputField::update(s, InputFieldMessage::End);
                }
                None
            }
            FormMessage::Clear => {
                if let Some(FieldState::Text(ref mut s)) = state.states.get_mut(state.focused_index)
                {
                    InputField::update(s, InputFieldMessage::Clear);
                    let id = state.fields[state.focused_index].id.clone();
                    Some(FormOutput::FieldChanged(id, FormValue::Text(String::new())))
                } else {
                    None
                }
            }
            FormMessage::Toggle => {
                let field_state = state.states.get_mut(state.focused_index)?;
                let id = state.fields[state.focused_index].id.clone();
                match field_state {
                    FieldState::Checkbox(ref mut s) => {
                        Checkbox::update(s, CheckboxMessage::Toggle);
                        Some(FormOutput::FieldChanged(
                            id,
                            FormValue::Bool(s.is_checked()),
                        ))
                    }
                    FieldState::Select(ref mut s) => {
                        if s.is_open() {
                            Select::update(s, SelectMessage::Close);
                        } else {
                            Select::update(s, SelectMessage::Open);
                        }
                        None
                    }
                    _ => None,
                }
            }
            FormMessage::SelectUp => {
                if let Some(FieldState::Select(ref mut s)) =
                    state.states.get_mut(state.focused_index)
                {
                    Select::update(s, SelectMessage::Up);
                }
                None
            }
            FormMessage::SelectDown => {
                if let Some(FieldState::Select(ref mut s)) =
                    state.states.get_mut(state.focused_index)
                {
                    Select::update(s, SelectMessage::Down);
                }
                None
            }
            FormMessage::SelectConfirm => {
                if let Some(FieldState::Select(ref mut s)) =
                    state.states.get_mut(state.focused_index)
                {
                    Select::update(s, SelectMessage::Confirm);
                    let id = state.fields[state.focused_index].id.clone();
                    let value = FormValue::Selected(s.selected_item().map(|v| v.to_string()));
                    Some(FormOutput::FieldChanged(id, value))
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if state.fields.is_empty() {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.open(
                area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::Form)
                    .with_id("form")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        // Allocate space: each text/select field gets 3 lines (label+border),
        // each checkbox gets 1 line.
        let constraints: Vec<Constraint> = state
            .fields
            .iter()
            .map(|f| match f.kind {
                FormFieldKind::Checkbox => Constraint::Length(1),
                _ => Constraint::Length(3),
            })
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        for (i, ((field, field_state), chunk)) in state
            .fields
            .iter()
            .zip(state.states.iter())
            .zip(chunks.iter())
            .enumerate()
        {
            let is_field_focused = ctx.focused && i == state.focused_index;

            match field_state {
                FieldState::Text(s) => {
                    render_text_field(frame, *chunk, field, s, is_field_focused, theme);
                }
                FieldState::Checkbox(s) => {
                    render_checkbox(frame, *chunk, s, is_field_focused, theme);
                }
                FieldState::Select(s) => {
                    render_select_field(frame, *chunk, field, s, is_field_focused, theme);
                }
            }
        }

        crate::annotation::with_registry(|reg| {
            reg.close();
        });
    }
}

/// Renders a labeled text input field.
fn render_text_field(
    frame: &mut Frame,
    area: Rect,
    field: &FormField,
    state: &InputFieldState,
    is_focused: bool,
    theme: &Theme,
) {
    let border_style = if state.is_disabled() {
        theme.disabled_style()
    } else if is_focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let display_text = if state.value().is_empty() {
        Span::styled(state.placeholder(), theme.disabled_style())
    } else {
        Span::styled(state.value(), theme.normal_style())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            format!(" {} ", field.label),
            theme.normal_style(),
        ));

    let widget = Paragraph::new(Line::from(display_text)).block(block);
    frame.render_widget(widget, area);

    // Show cursor when focused
    if is_focused && !state.is_disabled() {
        let cursor_x = area.x + 1 + state.cursor_display_position() as u16;
        let cursor_y = area.y + 1;
        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
    }
}

/// Renders a checkbox field.
fn render_checkbox(
    frame: &mut Frame,
    area: Rect,
    state: &CheckboxState,
    is_focused: bool,
    theme: &Theme,
) {
    let check = if state.is_checked() { "[x]" } else { "[ ]" };
    let style = if state.is_disabled() {
        theme.disabled_style()
    } else if is_focused {
        theme.focused_style()
    } else {
        theme.normal_style()
    };

    let text = format!("{} {}", check, state.label());
    let widget = Paragraph::new(Span::styled(text, style));
    frame.render_widget(widget, area);
}

/// Renders a labeled select field.
fn render_select_field(
    frame: &mut Frame,
    area: Rect,
    field: &FormField,
    state: &SelectState,
    is_focused: bool,
    theme: &Theme,
) {
    let border_style = if state.is_disabled() {
        theme.disabled_style()
    } else if is_focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let display_text = match state.selected_item() {
        Some(val) => Span::styled(val, theme.normal_style()),
        None => Span::styled(state.placeholder(), theme.disabled_style()),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            format!(" {} ", field.label),
            theme.normal_style(),
        ));

    let widget = Paragraph::new(Line::from(display_text)).block(block);
    frame.render_widget(widget, area);
}

impl Focusable for Form {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
        state.sync_field_focus();
    }
}

impl Disableable for Form {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
