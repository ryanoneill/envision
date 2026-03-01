//! A text input field component with cursor navigation and editing.
//!
//! `InputField` provides a single-line text input with cursor movement,
//! text insertion, deletion, and selection.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Focusable, InputField, InputFieldState, InputFieldMessage};
//!
//! // Create an input field
//! let mut state = InputField::init();
//!
//! // Type some text
//! InputField::update(&mut state, InputFieldMessage::Insert('H'));
//! InputField::update(&mut state, InputFieldMessage::Insert('i'));
//!
//! assert_eq!(state.value(), "Hi");
//! assert_eq!(state.cursor_position(), 2);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, Focusable};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

/// Messages that can be sent to an InputField.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputFieldMessage {
    /// Insert a character at the cursor position.
    Insert(char),
    /// Delete the character before the cursor (backspace).
    Backspace,
    /// Delete the character at the cursor position.
    Delete,
    /// Move cursor left by one character.
    Left,
    /// Move cursor right by one character.
    Right,
    /// Move cursor to the beginning of the input.
    Home,
    /// Move cursor to the end of the input.
    End,
    /// Move cursor left by one word.
    WordLeft,
    /// Move cursor right by one word.
    WordRight,
    /// Delete from cursor to beginning of word.
    DeleteWordBack,
    /// Delete from cursor to end of word.
    DeleteWordForward,
    /// Clear the entire input.
    Clear,
    /// Set the entire input value.
    SetValue(String),
    /// Submit the current value.
    Submit,
}

/// Output messages from an InputField.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputFieldOutput {
    /// The value was submitted (e.g., Enter pressed).
    Submitted(String),
    /// The value changed.
    Changed(String),
}

/// State for an InputField component.
#[derive(Clone, Debug, Default)]
pub struct InputFieldState {
    /// The current text value.
    value: String,
    /// Cursor position (byte offset into value).
    cursor: usize,
    /// Whether the input is focused.
    focused: bool,
    /// Placeholder text shown when empty.
    placeholder: String,
}

impl InputFieldState {
    /// Creates a new empty input field state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new state with the given initial value.
    pub fn with_value(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.len();
        Self {
            value,
            cursor,
            focused: false,
            placeholder: String::new(),
        }
    }

    /// Creates a new state with placeholder text.
    pub fn with_placeholder(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..Default::default()
        }
    }

    /// Returns the current value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets the value and moves cursor to the end.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.len();
    }

    /// Returns the cursor position (character index).
    pub fn cursor_position(&self) -> usize {
        self.value[..self.cursor].chars().count()
    }

    /// Returns the cursor byte offset.
    pub fn cursor_byte_offset(&self) -> usize {
        self.cursor
    }

    /// Sets the placeholder text.
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Returns the placeholder text.
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Returns true if the input is empty.
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Returns the number of characters in the input.
    pub fn len(&self) -> usize {
        self.value.chars().count()
    }

    /// Moves cursor to the given character position.
    pub fn set_cursor(&mut self, char_pos: usize) {
        let char_count = self.value.chars().count();
        let clamped = char_pos.min(char_count);
        self.cursor = self
            .value
            .char_indices()
            .nth(clamped)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len());
    }

    /// Move cursor left by one character.
    fn move_left(&mut self) {
        if self.cursor > 0 {
            // Find the previous character boundary
            self.cursor = self.value[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right by one character.
    fn move_right(&mut self) {
        if self.cursor < self.value.len() {
            // Find the next character boundary
            self.cursor = self.value[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.value.len());
        }
    }

    /// Move cursor to the start of the previous word.
    fn move_word_left(&mut self) {
        if self.cursor == 0 {
            return;
        }

        let before = &self.value[..self.cursor];
        let chars: Vec<(usize, char)> = before.char_indices().collect();

        // Skip trailing whitespace
        let mut idx = chars.len() - 1;
        while idx > 0 && chars[idx].1.is_whitespace() {
            idx -= 1;
        }

        // Skip word characters
        while idx > 0 && !chars[idx - 1].1.is_whitespace() {
            idx -= 1;
        }

        self.cursor = chars.get(idx).map(|(i, _)| *i).unwrap_or(0);
    }

    /// Move cursor to the end of the next word.
    fn move_word_right(&mut self) {
        if self.cursor >= self.value.len() {
            return;
        }

        let after = &self.value[self.cursor..];
        let chars: Vec<(usize, char)> = after.char_indices().collect();

        // Skip leading non-whitespace
        let mut idx = 0;
        while idx < chars.len() && !chars[idx].1.is_whitespace() {
            idx += 1;
        }

        // Skip whitespace
        while idx < chars.len() && chars[idx].1.is_whitespace() {
            idx += 1;
        }

        self.cursor = chars
            .get(idx)
            .map(|(i, _)| self.cursor + *i)
            .unwrap_or(self.value.len());
    }

    /// Insert a character at the cursor position.
    fn insert(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete the character before the cursor.
    fn backspace(&mut self) -> bool {
        if self.cursor > 0 {
            let prev_cursor = self.cursor;
            self.move_left();
            self.value.drain(self.cursor..prev_cursor);
            true
        } else {
            false
        }
    }

    /// Delete the character at the cursor.
    fn delete(&mut self) -> bool {
        if self.cursor < self.value.len() {
            let next = self.value[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.value.len());
            self.value.drain(self.cursor..next);
            true
        } else {
            false
        }
    }

    /// Delete from cursor back to start of word.
    fn delete_word_back(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        let end = self.cursor;
        self.move_word_left();
        self.value.drain(self.cursor..end);
        true
    }

    /// Delete from cursor forward to end of word.
    fn delete_word_forward(&mut self) -> bool {
        if self.cursor >= self.value.len() {
            return false;
        }

        let start = self.cursor;
        self.move_word_right();
        let end = self.cursor;
        self.cursor = start;
        self.value.drain(start..end);
        true
    }

    /// Returns true if the input field is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Maps an input event to an input field message.
    pub fn handle_event(&self, event: &Event) -> Option<InputFieldMessage> {
        InputField::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<InputFieldOutput> {
        InputField::dispatch_event(self, event)
    }

    /// Updates the input field state with a message, returning any output.
    pub fn update(&mut self, msg: InputFieldMessage) -> Option<InputFieldOutput> {
        InputField::update(self, msg)
    }
}

/// A text input field component.
///
/// This component provides a single-line text input with cursor navigation
/// and editing capabilities.
///
/// # Navigation
///
/// - `Left` / `Right` - Move cursor by one character
/// - `Home` / `End` - Jump to beginning/end
/// - `WordLeft` / `WordRight` - Move by word
///
/// # Editing
///
/// - `Insert(char)` - Insert a character
/// - `Backspace` - Delete before cursor
/// - `Delete` - Delete at cursor
/// - `DeleteWordBack` / `DeleteWordForward` - Delete by word
/// - `Clear` - Clear all text
/// - `SetValue(String)` - Replace all text
pub struct InputField;

impl Component for InputField {
    type State = InputFieldState;
    type Message = InputFieldMessage;
    type Output = InputFieldOutput;

    fn init() -> Self::State {
        InputFieldState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            InputFieldMessage::Insert(c) => {
                state.insert(c);
                Some(InputFieldOutput::Changed(state.value.clone()))
            }
            InputFieldMessage::Backspace => {
                if state.backspace() {
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Delete => {
                if state.delete() {
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Left => {
                state.move_left();
                None
            }
            InputFieldMessage::Right => {
                state.move_right();
                None
            }
            InputFieldMessage::Home => {
                state.cursor = 0;
                None
            }
            InputFieldMessage::End => {
                state.cursor = state.value.len();
                None
            }
            InputFieldMessage::WordLeft => {
                state.move_word_left();
                None
            }
            InputFieldMessage::WordRight => {
                state.move_word_right();
                None
            }
            InputFieldMessage::DeleteWordBack => {
                if state.delete_word_back() {
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::DeleteWordForward => {
                if state.delete_word_forward() {
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Clear => {
                if !state.value.is_empty() {
                    state.value.clear();
                    state.cursor = 0;
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::SetValue(value) => {
                if state.value != value {
                    state.set_value(value);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Submit => Some(InputFieldOutput::Submitted(state.value.clone())),
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused {
            return None;
        }
        if let Some(key) = event.as_key() {
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Char(c) if !ctrl => Some(InputFieldMessage::Insert(c)),
                KeyCode::Backspace if ctrl => Some(InputFieldMessage::DeleteWordBack),
                KeyCode::Delete if ctrl => Some(InputFieldMessage::DeleteWordForward),
                KeyCode::Backspace => Some(InputFieldMessage::Backspace),
                KeyCode::Delete => Some(InputFieldMessage::Delete),
                KeyCode::Left if ctrl => Some(InputFieldMessage::WordLeft),
                KeyCode::Right if ctrl => Some(InputFieldMessage::WordRight),
                KeyCode::Left => Some(InputFieldMessage::Left),
                KeyCode::Right => Some(InputFieldMessage::Right),
                KeyCode::Home => Some(InputFieldMessage::Home),
                KeyCode::End => Some(InputFieldMessage::End),
                KeyCode::Enter => Some(InputFieldMessage::Submit),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let display_text = if state.value.is_empty() && !state.placeholder.is_empty() {
            state.placeholder.clone()
        } else {
            state.value.clone()
        };

        let style = if state.focused {
            theme.focused_style()
        } else if state.value.is_empty() && !state.placeholder.is_empty() {
            theme.placeholder_style()
        } else {
            theme.normal_style()
        };

        let border_style = if state.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let paragraph = Paragraph::new(display_text).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(paragraph, area);

        // Show cursor when focused
        if state.focused && area.width > 2 && area.height > 2 {
            // Calculate cursor x position (accounting for border)
            let cursor_x = area.x + 1 + state.cursor_position() as u16;
            let cursor_y = area.y + 1;

            // Only show cursor if it's within the visible area
            if cursor_x < area.x + area.width - 1 {
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
}

impl Focusable for InputField {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
