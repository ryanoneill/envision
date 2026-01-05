//! A text input field component with cursor navigation and editing.
//!
//! `InputField` provides a single-line text input with cursor movement,
//! text insertion, deletion, and selection.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Focusable, InputField, InputFieldState, InputMessage};
//!
//! // Create an input field
//! let mut state = InputField::init();
//!
//! // Type some text
//! InputField::update(&mut state, InputMessage::Insert('H'));
//! InputField::update(&mut state, InputMessage::Insert('i'));
//!
//! assert_eq!(state.value(), "Hi");
//! assert_eq!(state.cursor_position(), 2);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, Focusable};
use crate::theme::Theme;

/// Messages that can be sent to an InputField.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputMessage {
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
pub enum InputOutput {
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
    type Message = InputMessage;
    type Output = InputOutput;

    fn init() -> Self::State {
        InputFieldState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            InputMessage::Insert(c) => {
                state.insert(c);
                Some(InputOutput::Changed(state.value.clone()))
            }
            InputMessage::Backspace => {
                if state.backspace() {
                    Some(InputOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputMessage::Delete => {
                if state.delete() {
                    Some(InputOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputMessage::Left => {
                state.move_left();
                None
            }
            InputMessage::Right => {
                state.move_right();
                None
            }
            InputMessage::Home => {
                state.cursor = 0;
                None
            }
            InputMessage::End => {
                state.cursor = state.value.len();
                None
            }
            InputMessage::WordLeft => {
                state.move_word_left();
                None
            }
            InputMessage::WordRight => {
                state.move_word_right();
                None
            }
            InputMessage::DeleteWordBack => {
                if state.delete_word_back() {
                    Some(InputOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputMessage::DeleteWordForward => {
                if state.delete_word_forward() {
                    Some(InputOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputMessage::Clear => {
                if !state.value.is_empty() {
                    state.value.clear();
                    state.cursor = 0;
                    Some(InputOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputMessage::SetValue(value) => {
                if state.value != value {
                    state.set_value(value);
                    Some(InputOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputMessage::Submit => Some(InputOutput::Submitted(state.value.clone())),
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
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let state = InputField::init();
        assert!(state.is_empty());
        assert_eq!(state.value(), "");
        assert_eq!(state.cursor_position(), 0);
    }

    #[test]
    fn test_with_value() {
        let state = InputFieldState::with_value("hello");
        assert_eq!(state.value(), "hello");
        assert_eq!(state.cursor_position(), 5);
    }

    #[test]
    fn test_with_placeholder() {
        let state = InputFieldState::with_placeholder("Enter text...");
        assert_eq!(state.placeholder(), "Enter text...");
        assert!(state.is_empty());
    }

    #[test]
    fn test_insert_char() {
        let mut state = InputField::init();

        let output = InputField::update(&mut state, InputMessage::Insert('a'));
        assert_eq!(state.value(), "a");
        assert_eq!(state.cursor_position(), 1);
        assert_eq!(output, Some(InputOutput::Changed("a".to_string())));

        InputField::update(&mut state, InputMessage::Insert('b'));
        assert_eq!(state.value(), "ab");
        assert_eq!(state.cursor_position(), 2);
    }

    #[test]
    fn test_insert_unicode() {
        let mut state = InputField::init();

        InputField::update(&mut state, InputMessage::Insert('日'));
        InputField::update(&mut state, InputMessage::Insert('本'));
        assert_eq!(state.value(), "日本");
        assert_eq!(state.cursor_position(), 2);
        assert_eq!(state.len(), 2);
    }

    #[test]
    fn test_backspace() {
        let mut state = InputFieldState::with_value("abc");

        let output = InputField::update(&mut state, InputMessage::Backspace);
        assert_eq!(state.value(), "ab");
        assert_eq!(output, Some(InputOutput::Changed("ab".to_string())));

        InputField::update(&mut state, InputMessage::Backspace);
        InputField::update(&mut state, InputMessage::Backspace);
        assert_eq!(state.value(), "");

        // Backspace on empty should return None
        let output = InputField::update(&mut state, InputMessage::Backspace);
        assert_eq!(output, None);
    }

    #[test]
    fn test_delete() {
        let mut state = InputFieldState::with_value("abc");
        state.set_cursor(0);

        let output = InputField::update(&mut state, InputMessage::Delete);
        assert_eq!(state.value(), "bc");
        assert_eq!(output, Some(InputOutput::Changed("bc".to_string())));

        // Move to end and delete should return None
        state.cursor = state.value.len();
        let output = InputField::update(&mut state, InputMessage::Delete);
        assert_eq!(output, None);
    }

    #[test]
    fn test_cursor_movement() {
        let mut state = InputFieldState::with_value("hello");

        InputField::update(&mut state, InputMessage::Left);
        assert_eq!(state.cursor_position(), 4);

        InputField::update(&mut state, InputMessage::Left);
        assert_eq!(state.cursor_position(), 3);

        InputField::update(&mut state, InputMessage::Right);
        assert_eq!(state.cursor_position(), 4);

        InputField::update(&mut state, InputMessage::Home);
        assert_eq!(state.cursor_position(), 0);

        InputField::update(&mut state, InputMessage::End);
        assert_eq!(state.cursor_position(), 5);
    }

    #[test]
    fn test_cursor_bounds() {
        let mut state = InputFieldState::with_value("hi");

        // Can't go left past beginning
        state.set_cursor(0);
        InputField::update(&mut state, InputMessage::Left);
        assert_eq!(state.cursor_position(), 0);

        // Can't go right past end
        state.set_cursor(10); // Over the length
        assert_eq!(state.cursor_position(), 2); // Clamped
        InputField::update(&mut state, InputMessage::Right);
        assert_eq!(state.cursor_position(), 2);
    }

    #[test]
    fn test_word_navigation() {
        let mut state = InputFieldState::with_value("hello world test");

        // Start at end
        InputField::update(&mut state, InputMessage::WordLeft);
        assert_eq!(state.cursor_position(), 12); // Start of "test"

        InputField::update(&mut state, InputMessage::WordLeft);
        assert_eq!(state.cursor_position(), 6); // Start of "world"

        InputField::update(&mut state, InputMessage::WordLeft);
        assert_eq!(state.cursor_position(), 0); // Start of "hello"

        InputField::update(&mut state, InputMessage::WordRight);
        assert_eq!(state.cursor_position(), 6); // After "hello "

        InputField::update(&mut state, InputMessage::WordRight);
        assert_eq!(state.cursor_position(), 12); // After "world "
    }

    #[test]
    fn test_delete_word_back() {
        let mut state = InputFieldState::with_value("hello world");

        let output = InputField::update(&mut state, InputMessage::DeleteWordBack);
        assert_eq!(state.value(), "hello ");
        assert_eq!(output, Some(InputOutput::Changed("hello ".to_string())));

        InputField::update(&mut state, InputMessage::DeleteWordBack);
        assert_eq!(state.value(), "");

        // Delete word back on empty
        let output = InputField::update(&mut state, InputMessage::DeleteWordBack);
        assert_eq!(output, None);
    }

    #[test]
    fn test_delete_word_forward() {
        let mut state = InputFieldState::with_value("hello world");
        state.set_cursor(0);

        let output = InputField::update(&mut state, InputMessage::DeleteWordForward);
        assert_eq!(state.value(), "world");
        assert_eq!(output, Some(InputOutput::Changed("world".to_string())));

        // Cursor at end
        state.cursor = state.value.len();
        let output = InputField::update(&mut state, InputMessage::DeleteWordForward);
        assert_eq!(output, None);
    }

    #[test]
    fn test_clear() {
        let mut state = InputFieldState::with_value("hello");

        let output = InputField::update(&mut state, InputMessage::Clear);
        assert_eq!(state.value(), "");
        assert_eq!(state.cursor_position(), 0);
        assert_eq!(output, Some(InputOutput::Changed("".to_string())));

        // Clear empty should return None
        let output = InputField::update(&mut state, InputMessage::Clear);
        assert_eq!(output, None);
    }

    #[test]
    fn test_set_value() {
        let mut state = InputField::init();

        let output =
            InputField::update(&mut state, InputMessage::SetValue("new value".to_string()));
        assert_eq!(state.value(), "new value");
        assert_eq!(state.cursor_position(), 9);
        assert_eq!(output, Some(InputOutput::Changed("new value".to_string())));

        // Setting same value returns None
        let output =
            InputField::update(&mut state, InputMessage::SetValue("new value".to_string()));
        assert_eq!(output, None);
    }

    #[test]
    fn test_submit() {
        let mut state = InputFieldState::with_value("submitted text");

        let output = InputField::update(&mut state, InputMessage::Submit);
        assert_eq!(
            output,
            Some(InputOutput::Submitted("submitted text".to_string()))
        );
        // Value should remain unchanged
        assert_eq!(state.value(), "submitted text");
    }

    #[test]
    fn test_insert_at_cursor() {
        let mut state = InputFieldState::with_value("helo");
        state.set_cursor(3);

        InputField::update(&mut state, InputMessage::Insert('l'));
        assert_eq!(state.value(), "hello");
        assert_eq!(state.cursor_position(), 4);
    }

    #[test]
    fn test_focusable() {
        let mut state = InputField::init();

        assert!(!InputField::is_focused(&state));

        InputField::set_focused(&mut state, true);
        assert!(InputField::is_focused(&state));

        InputField::blur(&mut state);
        assert!(!InputField::is_focused(&state));
    }

    #[test]
    fn test_len() {
        let state = InputFieldState::with_value("hello");
        assert_eq!(state.len(), 5);

        let state = InputFieldState::with_value("日本語");
        assert_eq!(state.len(), 3);
    }

    #[test]
    fn test_view() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = InputFieldState::with_value("Hello");
        state.focused = true;
        let theme = Theme::default();

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                InputField::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_view_placeholder() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = InputField::init();
        state.set_placeholder("Enter text...");
        let theme = Theme::default();

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                InputField::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Enter text..."));
    }
}
