//! A multi-line text editing component with cursor navigation.
//!
//! `TextArea` provides multi-line text input with cursor movement,
//! text insertion, deletion, and line operations.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Focusable, TextArea, TextAreaState, TextAreaMessage};
//!
//! // Create a textarea
//! let mut state = TextArea::init();
//!
//! // Type some text
//! TextArea::update(&mut state, TextAreaMessage::Insert('H'));
//! TextArea::update(&mut state, TextAreaMessage::Insert('i'));
//! TextArea::update(&mut state, TextAreaMessage::NewLine);
//! TextArea::update(&mut state, TextAreaMessage::Insert('!'));
//!
//! assert_eq!(state.value(), "Hi\n!");
//! assert_eq!(state.line_count(), 2);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, Focusable};

/// Messages that can be sent to a TextArea.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextAreaMessage {
    // Character editing
    /// Insert a character at the cursor position.
    Insert(char),
    /// Insert a newline, splitting the current line.
    NewLine,
    /// Delete the character before the cursor (backspace).
    Backspace,
    /// Delete the character at the cursor position.
    Delete,

    // Cursor movement
    /// Move cursor left by one character.
    Left,
    /// Move cursor right by one character.
    Right,
    /// Move cursor up by one line.
    Up,
    /// Move cursor down by one line.
    Down,
    /// Move cursor to the beginning of the current line.
    Home,
    /// Move cursor to the end of the current line.
    End,
    /// Move cursor to the beginning of the text.
    TextStart,
    /// Move cursor to the end of the text.
    TextEnd,
    /// Move cursor left by one word.
    WordLeft,
    /// Move cursor right by one word.
    WordRight,

    // Line operations
    /// Delete the entire current line.
    DeleteLine,
    /// Delete from cursor to end of line.
    DeleteToEnd,
    /// Delete from cursor to beginning of line.
    DeleteToStart,

    // Bulk operations
    /// Clear all content.
    Clear,
    /// Set the entire content.
    SetValue(String),
    /// Submit the current value.
    Submit,
}

/// Output messages from a TextArea.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextAreaOutput {
    /// The value was submitted.
    Submitted(String),
    /// The value changed.
    Changed(String),
}

/// State for a TextArea component.
#[derive(Clone, Debug)]
pub struct TextAreaState {
    /// Lines of text.
    lines: Vec<String>,
    /// Cursor row (line index).
    cursor_row: usize,
    /// Cursor column (byte offset within line).
    cursor_col: usize,
    /// First visible line (for scrolling).
    scroll_offset: usize,
    /// Whether the textarea is focused.
    focused: bool,
    /// Placeholder text shown when empty.
    placeholder: String,
}

impl Default for TextAreaState {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            focused: false,
            placeholder: String::new(),
        }
    }
}

impl TextAreaState {
    /// Creates a new empty textarea.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::new();
    /// assert!(state.is_empty());
    /// assert_eq!(state.line_count(), 1);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a textarea with initial content.
    ///
    /// The content is split on newlines into separate lines.
    /// The cursor is placed at the end of the content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::with_value("Hello\nWorld");
    /// assert_eq!(state.line_count(), 2);
    /// assert_eq!(state.line(0), Some("Hello"));
    /// assert_eq!(state.line(1), Some("World"));
    /// ```
    pub fn with_value(value: impl Into<String>) -> Self {
        let value = value.into();
        let lines: Vec<String> = if value.is_empty() {
            vec![String::new()]
        } else {
            // Use split('\n') instead of lines() to preserve trailing newlines
            value.split('\n').map(String::from).collect()
        };

        let cursor_row = lines.len().saturating_sub(1);
        let cursor_col = lines.last().map(|l| l.len()).unwrap_or(0);

        Self {
            lines,
            cursor_row,
            cursor_col,
            scroll_offset: 0,
            focused: false,
            placeholder: String::new(),
        }
    }

    /// Creates a textarea with placeholder text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::with_placeholder("Enter description...");
    /// assert_eq!(state.placeholder(), "Enter description...");
    /// assert!(state.is_empty());
    /// ```
    pub fn with_placeholder(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..Default::default()
        }
    }

    /// Returns the full text content (lines joined with \n).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::with_value("Line 1\nLine 2");
    /// assert_eq!(state.value(), "Line 1\nLine 2");
    /// ```
    pub fn value(&self) -> String {
        self.lines.join("\n")
    }

    /// Sets the content from a string (splits on \n).
    ///
    /// The cursor is moved to the end of the content.
    pub fn set_value(&mut self, value: impl Into<String>) {
        let value = value.into();
        self.lines = if value.is_empty() {
            vec![String::new()]
        } else {
            // Use split('\n') instead of lines() to preserve trailing newlines
            value.split('\n').map(String::from).collect()
        };
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = self.lines[self.cursor_row].len();
        self.scroll_offset = 0;
    }

    /// Returns the cursor position as (row, char_column).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::with_value("Hello\nWorld");
    /// // Cursor is at end of "World" (row 1, char 5)
    /// assert_eq!(state.cursor_position(), (1, 5));
    /// ```
    pub fn cursor_position(&self) -> (usize, usize) {
        let char_col = self.lines[self.cursor_row][..self.cursor_col]
            .chars()
            .count();
        (self.cursor_row, char_col)
    }

    /// Returns the cursor row.
    pub fn cursor_row(&self) -> usize {
        self.cursor_row
    }

    /// Returns the cursor column (byte offset).
    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }

    /// Returns the number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns a specific line by index.
    pub fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }

    /// Returns the current line (at cursor row).
    pub fn current_line(&self) -> &str {
        &self.lines[self.cursor_row]
    }

    /// Returns true if the textarea is empty.
    ///
    /// A textarea is empty if it contains only a single empty line.
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    /// Returns the placeholder text.
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Sets the placeholder text.
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Returns the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the cursor position (row, char_column).
    ///
    /// Both row and column are clamped to valid ranges.
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.lines.len().saturating_sub(1));
        // Convert char position to byte offset
        let line = &self.lines[self.cursor_row];
        let char_count = line.chars().count();
        let clamped_col = col.min(char_count);
        self.cursor_col = line
            .char_indices()
            .nth(clamped_col)
            .map(|(i, _)| i)
            .unwrap_or(line.len());
    }

    /// Ensures the cursor is visible within the viewport.
    pub fn ensure_cursor_visible(&mut self, visible_lines: usize) {
        if visible_lines == 0 {
            return;
        }
        // Scroll up if cursor above viewport
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        }
        // Scroll down if cursor below viewport
        if self.cursor_row >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.cursor_row - visible_lines + 1;
        }
    }

    /// Insert a character at the cursor position.
    fn insert(&mut self, c: char) {
        self.lines[self.cursor_row].insert(self.cursor_col, c);
        self.cursor_col += c.len_utf8();
    }

    /// Insert a newline at the cursor position.
    fn new_line(&mut self) {
        let remainder = self.lines[self.cursor_row].split_off(self.cursor_col);
        self.lines.insert(self.cursor_row + 1, remainder);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    /// Delete the character before the cursor.
    fn backspace(&mut self) -> bool {
        if self.cursor_col > 0 {
            // Find previous character boundary
            let prev_cursor = self.cursor_col;
            self.cursor_col = self.lines[self.cursor_row][..self.cursor_col]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.lines[self.cursor_row].drain(self.cursor_col..prev_cursor);
            true
        } else if self.cursor_row > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current_line);
            true
        } else {
            false
        }
    }

    /// Delete the character at the cursor.
    fn delete(&mut self) -> bool {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            // Find next character boundary
            let next = self.lines[self.cursor_row][self.cursor_col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_col + i)
                .unwrap_or(line_len);
            self.lines[self.cursor_row].drain(self.cursor_col..next);
            true
        } else if self.cursor_row < self.lines.len() - 1 {
            // Join with next line
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
            true
        } else {
            false
        }
    }

    /// Move cursor left by one character.
    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col = self.lines[self.cursor_row][..self.cursor_col]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        } else if self.cursor_row > 0 {
            // Wrap to end of previous line
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// Move cursor right by one character.
    fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.cursor_col = self.lines[self.cursor_row][self.cursor_col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_col + i)
                .unwrap_or(line_len);
        } else if self.cursor_row < self.lines.len() - 1 {
            // Wrap to start of next line
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor up by one line.
    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            // Remember char position, not byte position
            let char_pos = self.lines[self.cursor_row][..self.cursor_col]
                .chars()
                .count();
            self.cursor_row -= 1;
            // Restore to same char position (clamped to line length)
            let line = &self.lines[self.cursor_row];
            let char_count = line.chars().count();
            let target_pos = char_pos.min(char_count);
            self.cursor_col = line
                .char_indices()
                .nth(target_pos)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
        }
    }

    /// Move cursor down by one line.
    fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            // Remember char position, not byte position
            let char_pos = self.lines[self.cursor_row][..self.cursor_col]
                .chars()
                .count();
            self.cursor_row += 1;
            // Restore to same char position (clamped to line length)
            let line = &self.lines[self.cursor_row];
            let char_count = line.chars().count();
            let target_pos = char_pos.min(char_count);
            self.cursor_col = line
                .char_indices()
                .nth(target_pos)
                .map(|(i, _)| i)
                .unwrap_or(line.len());
        }
    }

    /// Move cursor to the start of the previous word.
    fn move_word_left(&mut self) {
        if self.cursor_col == 0 {
            // If at line start, move to previous line end
            if self.cursor_row > 0 {
                self.cursor_row -= 1;
                self.cursor_col = self.lines[self.cursor_row].len();
            }
            return;
        }

        let before = &self.lines[self.cursor_row][..self.cursor_col];
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

        self.cursor_col = chars.get(idx).map(|(i, _)| *i).unwrap_or(0);
    }

    /// Move cursor to the end of the next word.
    fn move_word_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col >= line_len {
            // If at line end, move to next line start
            if self.cursor_row < self.lines.len() - 1 {
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
            return;
        }

        let after = &self.lines[self.cursor_row][self.cursor_col..];
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

        self.cursor_col = chars
            .get(idx)
            .map(|(i, _)| self.cursor_col + *i)
            .unwrap_or(line_len);
    }

    /// Delete the entire current line.
    fn delete_line(&mut self) -> bool {
        if self.lines.len() > 1 {
            self.lines.remove(self.cursor_row);
            if self.cursor_row >= self.lines.len() {
                self.cursor_row = self.lines.len() - 1;
            }
            // Clamp cursor column
            let line_len = self.lines[self.cursor_row].len();
            self.cursor_col = self.cursor_col.min(line_len);
            true
        } else {
            // Single line: just clear it
            if !self.lines[0].is_empty() {
                self.lines[0].clear();
                self.cursor_col = 0;
                true
            } else {
                false
            }
        }
    }

    /// Delete from cursor to end of line.
    fn delete_to_end(&mut self) -> bool {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.lines[self.cursor_row].truncate(self.cursor_col);
            true
        } else {
            false
        }
    }

    /// Delete from cursor to beginning of line.
    fn delete_to_start(&mut self) -> bool {
        if self.cursor_col > 0 {
            self.lines[self.cursor_row].drain(..self.cursor_col);
            self.cursor_col = 0;
            true
        } else {
            false
        }
    }
}

/// A multi-line text editing component.
///
/// This component provides multi-line text input with cursor navigation
/// and editing capabilities.
///
/// # Navigation
///
/// - `Left` / `Right` - Move cursor by one character (wraps at line ends)
/// - `Up` / `Down` - Move cursor by one line
/// - `Home` / `End` - Jump to beginning/end of current line
/// - `TextStart` / `TextEnd` - Jump to beginning/end of entire text
/// - `WordLeft` / `WordRight` - Move by word
///
/// # Editing
///
/// - `Insert(char)` - Insert a character
/// - `NewLine` - Insert a newline (split line)
/// - `Backspace` - Delete before cursor (joins lines at line start)
/// - `Delete` - Delete at cursor (joins lines at line end)
/// - `DeleteLine` - Delete the entire current line
/// - `DeleteToEnd` / `DeleteToStart` - Delete to line boundary
/// - `Clear` - Clear all text
/// - `SetValue(String)` - Replace all text
pub struct TextArea;

impl Component for TextArea {
    type State = TextAreaState;
    type Message = TextAreaMessage;
    type Output = TextAreaOutput;

    fn init() -> Self::State {
        TextAreaState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            TextAreaMessage::Insert(c) => {
                state.insert(c);
                Some(TextAreaOutput::Changed(state.value()))
            }
            TextAreaMessage::NewLine => {
                state.new_line();
                Some(TextAreaOutput::Changed(state.value()))
            }
            TextAreaMessage::Backspace => {
                if state.backspace() {
                    Some(TextAreaOutput::Changed(state.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::Delete => {
                if state.delete() {
                    Some(TextAreaOutput::Changed(state.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::Left => {
                state.move_left();
                None
            }
            TextAreaMessage::Right => {
                state.move_right();
                None
            }
            TextAreaMessage::Up => {
                state.move_up();
                None
            }
            TextAreaMessage::Down => {
                state.move_down();
                None
            }
            TextAreaMessage::Home => {
                state.cursor_col = 0;
                None
            }
            TextAreaMessage::End => {
                state.cursor_col = state.lines[state.cursor_row].len();
                None
            }
            TextAreaMessage::TextStart => {
                state.cursor_row = 0;
                state.cursor_col = 0;
                None
            }
            TextAreaMessage::TextEnd => {
                state.cursor_row = state.lines.len() - 1;
                state.cursor_col = state.lines[state.cursor_row].len();
                None
            }
            TextAreaMessage::WordLeft => {
                state.move_word_left();
                None
            }
            TextAreaMessage::WordRight => {
                state.move_word_right();
                None
            }
            TextAreaMessage::DeleteLine => {
                if state.delete_line() {
                    Some(TextAreaOutput::Changed(state.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::DeleteToEnd => {
                if state.delete_to_end() {
                    Some(TextAreaOutput::Changed(state.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::DeleteToStart => {
                if state.delete_to_start() {
                    Some(TextAreaOutput::Changed(state.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::Clear => {
                if !state.is_empty() {
                    state.lines = vec![String::new()];
                    state.cursor_row = 0;
                    state.cursor_col = 0;
                    state.scroll_offset = 0;
                    Some(TextAreaOutput::Changed(state.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::SetValue(value) => {
                let old_value = state.value();
                if old_value != value {
                    state.set_value(value);
                    Some(TextAreaOutput::Changed(state.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::Submit => Some(TextAreaOutput::Submitted(state.value())),
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        let inner_height = area.height.saturating_sub(2) as usize; // Account for borders

        // Ensure cursor is visible
        let mut scroll = state.scroll_offset;
        if inner_height > 0 {
            if state.cursor_row < scroll {
                scroll = state.cursor_row;
            }
            if state.cursor_row >= scroll + inner_height {
                scroll = state.cursor_row - inner_height + 1;
            }
        }

        // Build display text
        let display_text = if state.is_empty() && !state.placeholder.is_empty() {
            state.placeholder.clone()
        } else {
            state
                .lines
                .iter()
                .skip(scroll)
                .take(inner_height.max(1))
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        };

        let style = if state.focused {
            Style::default().fg(Color::Yellow)
        } else if state.is_empty() && !state.placeholder.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let border_style = if state.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let paragraph = Paragraph::new(display_text).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(paragraph, area);

        // Show cursor when focused
        if state.focused && area.width > 2 && area.height > 2 {
            let cursor_row_in_view = state.cursor_row.saturating_sub(scroll);
            let char_col = state.lines[state.cursor_row][..state.cursor_col]
                .chars()
                .count();

            let cursor_x = area.x + 1 + char_col as u16;
            let cursor_y = area.y + 1 + cursor_row_in_view as u16;

            // Only show cursor if it's within the visible area
            if cursor_x < area.x + area.width - 1
                && cursor_y < area.y + area.height - 1
                && cursor_row_in_view < inner_height
            {
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
}

impl Focusable for TextArea {
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

    // State Tests

    #[test]
    fn test_new() {
        let state = TextAreaState::new();
        assert!(state.is_empty());
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.line(0), Some(""));
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_default() {
        let state = TextAreaState::default();
        assert!(state.is_empty());
        assert_eq!(state.line_count(), 1);
    }

    #[test]
    fn test_with_value() {
        let state = TextAreaState::with_value("Hello\nWorld");
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.line(0), Some("Hello"));
        assert_eq!(state.line(1), Some("World"));
        // Cursor at end of last line
        assert_eq!(state.cursor_position(), (1, 5));
    }

    #[test]
    fn test_with_value_empty() {
        let state = TextAreaState::with_value("");
        assert!(state.is_empty());
        assert_eq!(state.line_count(), 1);
    }

    #[test]
    fn test_with_placeholder() {
        let state = TextAreaState::with_placeholder("Enter text...");
        assert_eq!(state.placeholder(), "Enter text...");
        assert!(state.is_empty());
    }

    // Content Accessors

    #[test]
    fn test_value() {
        let state = TextAreaState::with_value("Line 1\nLine 2\nLine 3");
        assert_eq!(state.value(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_set_value() {
        let mut state = TextAreaState::new();
        state.set_value("New\nContent");
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.line(0), Some("New"));
        assert_eq!(state.line(1), Some("Content"));
        assert_eq!(state.cursor_position(), (1, 7));
    }

    #[test]
    fn test_line() {
        let state = TextAreaState::with_value("a\nb\nc");
        assert_eq!(state.line(0), Some("a"));
        assert_eq!(state.line(1), Some("b"));
        assert_eq!(state.line(2), Some("c"));
        assert_eq!(state.line(3), None);
    }

    #[test]
    fn test_current_line() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(0, 0);
        assert_eq!(state.current_line(), "Hello");
        state.set_cursor(1, 0);
        assert_eq!(state.current_line(), "World");
    }

    #[test]
    fn test_line_count() {
        assert_eq!(TextAreaState::new().line_count(), 1);
        assert_eq!(TextAreaState::with_value("a").line_count(), 1);
        assert_eq!(TextAreaState::with_value("a\nb").line_count(), 2);
        assert_eq!(TextAreaState::with_value("a\nb\nc").line_count(), 3);
    }

    #[test]
    fn test_is_empty() {
        assert!(TextAreaState::new().is_empty());
        assert!(!TextAreaState::with_value("a").is_empty());
        assert!(!TextAreaState::with_value("\n").is_empty()); // Two empty lines
    }

    // Cursor Tests

    #[test]
    fn test_cursor_position() {
        let state = TextAreaState::with_value("Hello\nWorld");
        assert_eq!(state.cursor_position(), (1, 5));
    }

    #[test]
    fn test_set_cursor() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(0, 2);
        assert_eq!(state.cursor_position(), (0, 2));
    }

    #[test]
    fn test_cursor_clamp_row() {
        let mut state = TextAreaState::with_value("Hello");
        state.set_cursor(10, 0); // Row out of bounds
        assert_eq!(state.cursor_row(), 0);
    }

    #[test]
    fn test_cursor_clamp_col() {
        let mut state = TextAreaState::with_value("Hi");
        state.set_cursor(0, 100); // Col out of bounds
        assert_eq!(state.cursor_position(), (0, 2));
    }

    // Character Editing

    #[test]
    fn test_insert() {
        let mut state = TextArea::init();
        let output = TextArea::update(&mut state, TextAreaMessage::Insert('H'));
        assert_eq!(state.value(), "H");
        assert!(matches!(output, Some(TextAreaOutput::Changed(_))));

        TextArea::update(&mut state, TextAreaMessage::Insert('i'));
        assert_eq!(state.value(), "Hi");
    }

    #[test]
    fn test_insert_unicode() {
        let mut state = TextArea::init();
        TextArea::update(&mut state, TextAreaMessage::Insert('日'));
        TextArea::update(&mut state, TextAreaMessage::Insert('本'));
        assert_eq!(state.value(), "日本");
        assert_eq!(state.cursor_position(), (0, 2));
    }

    #[test]
    fn test_newline() {
        let mut state = TextAreaState::with_value("Hello");
        state.set_cursor(0, 2);
        TextArea::update(&mut state, TextAreaMessage::NewLine);
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.line(0), Some("He"));
        assert_eq!(state.line(1), Some("llo"));
        assert_eq!(state.cursor_position(), (1, 0));
    }

    #[test]
    fn test_newline_at_start() {
        let mut state = TextAreaState::with_value("Hello");
        state.set_cursor(0, 0);
        TextArea::update(&mut state, TextAreaMessage::NewLine);
        assert_eq!(state.line(0), Some(""));
        assert_eq!(state.line(1), Some("Hello"));
    }

    #[test]
    fn test_newline_at_end() {
        let mut state = TextAreaState::with_value("Hello");
        TextArea::update(&mut state, TextAreaMessage::NewLine);
        assert_eq!(state.line(0), Some("Hello"));
        assert_eq!(state.line(1), Some(""));
    }

    #[test]
    fn test_backspace() {
        let mut state = TextAreaState::with_value("Hello");
        let output = TextArea::update(&mut state, TextAreaMessage::Backspace);
        assert_eq!(state.value(), "Hell");
        assert!(matches!(output, Some(TextAreaOutput::Changed(_))));
    }

    #[test]
    fn test_backspace_join_lines() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(1, 0); // Start of second line
        TextArea::update(&mut state, TextAreaMessage::Backspace);
        assert_eq!(state.value(), "HelloWorld");
        assert_eq!(state.cursor_position(), (0, 5));
    }

    #[test]
    fn test_backspace_first_line_start() {
        let mut state = TextAreaState::with_value("Hello");
        state.set_cursor(0, 0);
        let output = TextArea::update(&mut state, TextAreaMessage::Backspace);
        assert_eq!(output, None);
        assert_eq!(state.value(), "Hello");
    }

    #[test]
    fn test_delete() {
        let mut state = TextAreaState::with_value("Hello");
        state.set_cursor(0, 0);
        let output = TextArea::update(&mut state, TextAreaMessage::Delete);
        assert_eq!(state.value(), "ello");
        assert!(matches!(output, Some(TextAreaOutput::Changed(_))));
    }

    #[test]
    fn test_delete_join_lines() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(0, 5); // End of first line
        TextArea::update(&mut state, TextAreaMessage::Delete);
        assert_eq!(state.value(), "HelloWorld");
    }

    #[test]
    fn test_delete_last_line_end() {
        let mut state = TextAreaState::with_value("Hello");
        // Cursor is already at end
        let output = TextArea::update(&mut state, TextAreaMessage::Delete);
        assert_eq!(output, None);
    }

    // Navigation

    #[test]
    fn test_left() {
        let mut state = TextAreaState::with_value("Hello");
        TextArea::update(&mut state, TextAreaMessage::Left);
        assert_eq!(state.cursor_position(), (0, 4));
    }

    #[test]
    fn test_left_wrap() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(1, 0);
        TextArea::update(&mut state, TextAreaMessage::Left);
        assert_eq!(state.cursor_position(), (0, 5)); // End of first line
    }

    #[test]
    fn test_left_at_start() {
        let mut state = TextAreaState::with_value("Hello");
        state.set_cursor(0, 0);
        TextArea::update(&mut state, TextAreaMessage::Left);
        assert_eq!(state.cursor_position(), (0, 0)); // Stays at start
    }

    #[test]
    fn test_right() {
        let mut state = TextAreaState::with_value("Hello");
        state.set_cursor(0, 0);
        TextArea::update(&mut state, TextAreaMessage::Right);
        assert_eq!(state.cursor_position(), (0, 1));
    }

    #[test]
    fn test_right_wrap() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(0, 5); // End of first line
        TextArea::update(&mut state, TextAreaMessage::Right);
        assert_eq!(state.cursor_position(), (1, 0)); // Start of second line
    }

    #[test]
    fn test_right_at_end() {
        let mut state = TextAreaState::with_value("Hello");
        // Already at end
        TextArea::update(&mut state, TextAreaMessage::Right);
        assert_eq!(state.cursor_position(), (0, 5)); // Stays at end
    }

    #[test]
    fn test_up() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        TextArea::update(&mut state, TextAreaMessage::Up);
        assert_eq!(state.cursor_position(), (0, 5));
    }

    #[test]
    fn test_up_clamps_column() {
        let mut state = TextAreaState::with_value("Hi\nHello");
        state.set_cursor(1, 5); // End of "Hello"
        TextArea::update(&mut state, TextAreaMessage::Up);
        assert_eq!(state.cursor_position(), (0, 2)); // Clamped to "Hi" length
    }

    #[test]
    fn test_up_at_first_line() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(0, 2);
        TextArea::update(&mut state, TextAreaMessage::Up);
        assert_eq!(state.cursor_position(), (0, 2)); // Stays on first line
    }

    #[test]
    fn test_down() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(0, 2);
        TextArea::update(&mut state, TextAreaMessage::Down);
        assert_eq!(state.cursor_position(), (1, 2));
    }

    #[test]
    fn test_down_clamps_column() {
        let mut state = TextAreaState::with_value("Hello\nHi");
        state.set_cursor(0, 5); // End of "Hello"
        TextArea::update(&mut state, TextAreaMessage::Down);
        assert_eq!(state.cursor_position(), (1, 2)); // Clamped to "Hi" length
    }

    #[test]
    fn test_down_at_last_line() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        // Already on last line
        TextArea::update(&mut state, TextAreaMessage::Down);
        assert_eq!(state.cursor_row(), 1); // Stays on last line
    }

    #[test]
    fn test_home() {
        let mut state = TextAreaState::with_value("Hello");
        TextArea::update(&mut state, TextAreaMessage::Home);
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_end() {
        let mut state = TextAreaState::with_value("Hello");
        state.set_cursor(0, 0);
        TextArea::update(&mut state, TextAreaMessage::End);
        assert_eq!(state.cursor_position(), (0, 5));
    }

    #[test]
    fn test_text_start() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        TextArea::update(&mut state, TextAreaMessage::TextStart);
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_text_end() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(0, 0);
        TextArea::update(&mut state, TextAreaMessage::TextEnd);
        assert_eq!(state.cursor_position(), (1, 5));
    }

    #[test]
    fn test_word_left() {
        let mut state = TextAreaState::with_value("hello world");
        TextArea::update(&mut state, TextAreaMessage::WordLeft);
        assert_eq!(state.cursor_position(), (0, 6)); // Start of "world"
    }

    #[test]
    fn test_word_right() {
        let mut state = TextAreaState::with_value("hello world");
        state.set_cursor(0, 0);
        TextArea::update(&mut state, TextAreaMessage::WordRight);
        assert_eq!(state.cursor_position(), (0, 6)); // After "hello "
    }

    // Line Operations

    #[test]
    fn test_delete_line() {
        let mut state = TextAreaState::with_value("Line 1\nLine 2\nLine 3");
        state.set_cursor(1, 0);
        TextArea::update(&mut state, TextAreaMessage::DeleteLine);
        assert_eq!(state.line_count(), 2);
        assert_eq!(state.value(), "Line 1\nLine 3");
    }

    #[test]
    fn test_delete_line_single() {
        let mut state = TextAreaState::with_value("Hello");
        TextArea::update(&mut state, TextAreaMessage::DeleteLine);
        assert!(state.is_empty());
        assert_eq!(state.line_count(), 1);
    }

    #[test]
    fn test_delete_to_end() {
        let mut state = TextAreaState::with_value("Hello World");
        state.set_cursor(0, 5);
        TextArea::update(&mut state, TextAreaMessage::DeleteToEnd);
        assert_eq!(state.value(), "Hello");
    }

    #[test]
    fn test_delete_to_start() {
        let mut state = TextAreaState::with_value("Hello World");
        state.set_cursor(0, 6);
        TextArea::update(&mut state, TextAreaMessage::DeleteToStart);
        assert_eq!(state.value(), "World");
        assert_eq!(state.cursor_position(), (0, 0));
    }

    // Bulk Operations

    #[test]
    fn test_clear() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        let output = TextArea::update(&mut state, TextAreaMessage::Clear);
        assert!(state.is_empty());
        assert!(matches!(output, Some(TextAreaOutput::Changed(_))));
    }

    #[test]
    fn test_clear_empty() {
        let mut state = TextArea::init();
        let output = TextArea::update(&mut state, TextAreaMessage::Clear);
        assert_eq!(output, None);
    }

    #[test]
    fn test_set_value_message() {
        let mut state = TextArea::init();
        let output = TextArea::update(
            &mut state,
            TextAreaMessage::SetValue("New\nValue".to_string()),
        );
        assert_eq!(state.value(), "New\nValue");
        assert!(matches!(output, Some(TextAreaOutput::Changed(_))));
    }

    #[test]
    fn test_set_value_same() {
        let mut state = TextAreaState::with_value("Same");
        let output = TextArea::update(&mut state, TextAreaMessage::SetValue("Same".to_string()));
        assert_eq!(output, None);
    }

    #[test]
    fn test_submit() {
        let mut state = TextAreaState::with_value("My content");
        let output = TextArea::update(&mut state, TextAreaMessage::Submit);
        assert_eq!(
            output,
            Some(TextAreaOutput::Submitted("My content".to_string()))
        );
    }

    // Scroll Tests

    #[test]
    fn test_scroll_offset() {
        let state = TextAreaState::new();
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_ensure_cursor_visible_down() {
        let mut state = TextAreaState::with_value("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        state.set_cursor(9, 0); // Last line
        state.ensure_cursor_visible(5);
        assert!(state.scroll_offset > 0);
        assert!(state.cursor_row >= state.scroll_offset);
        assert!(state.cursor_row < state.scroll_offset + 5);
    }

    #[test]
    fn test_ensure_cursor_visible_up() {
        let mut state = TextAreaState::with_value("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        state.scroll_offset = 5;
        state.set_cursor(2, 0);
        state.ensure_cursor_visible(5);
        assert_eq!(state.scroll_offset, 2);
    }

    // Focus Tests

    #[test]
    fn test_focusable() {
        let mut state = TextArea::init();
        assert!(!TextArea::is_focused(&state));

        TextArea::set_focused(&mut state, true);
        assert!(TextArea::is_focused(&state));

        TextArea::blur(&mut state);
        assert!(!TextArea::is_focused(&state));
    }

    #[test]
    fn test_view_focused() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = TextAreaState::with_value("Hello");
        state.focused = true;

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TextArea::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_view_unfocused() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = TextAreaState::with_value("Hello");

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TextArea::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_view_placeholder() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = TextAreaState::with_placeholder("Enter text...");

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TextArea::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Enter text..."));
    }

    // Integration

    #[test]
    fn test_view_renders() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = TextAreaState::with_value("Line 1\nLine 2\nLine 3");

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TextArea::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    #[test]
    fn test_full_workflow() {
        let mut state = TextAreaState::new();
        TextArea::set_focused(&mut state, true);

        // Type "Hello"
        TextArea::update(&mut state, TextAreaMessage::Insert('H'));
        TextArea::update(&mut state, TextAreaMessage::Insert('e'));
        TextArea::update(&mut state, TextAreaMessage::Insert('l'));
        TextArea::update(&mut state, TextAreaMessage::Insert('l'));
        TextArea::update(&mut state, TextAreaMessage::Insert('o'));

        // New line
        TextArea::update(&mut state, TextAreaMessage::NewLine);

        // Type "World"
        TextArea::update(&mut state, TextAreaMessage::Insert('W'));
        TextArea::update(&mut state, TextAreaMessage::Insert('o'));
        TextArea::update(&mut state, TextAreaMessage::Insert('r'));
        TextArea::update(&mut state, TextAreaMessage::Insert('l'));
        TextArea::update(&mut state, TextAreaMessage::Insert('d'));

        assert_eq!(state.value(), "Hello\nWorld");
        assert_eq!(state.line_count(), 2);

        // Navigate up
        TextArea::update(&mut state, TextAreaMessage::Up);
        assert_eq!(state.cursor_position(), (0, 5));

        // Go to start of line
        TextArea::update(&mut state, TextAreaMessage::Home);
        assert_eq!(state.cursor_position(), (0, 0));

        // Delete line
        TextArea::update(&mut state, TextAreaMessage::DeleteLine);
        assert_eq!(state.value(), "World");

        // Clear
        TextArea::update(&mut state, TextAreaMessage::Clear);
        assert!(state.is_empty());
    }

    #[test]
    fn test_clone() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(1, 3);
        state.focused = true;

        let cloned = state.clone();
        assert_eq!(cloned.value(), "Hello\nWorld");
        assert_eq!(cloned.cursor_position(), (1, 3));
        assert!(cloned.focused);
    }

    #[test]
    fn test_init() {
        let state = TextArea::init();
        assert!(state.is_empty());
        assert!(!state.focused);
    }

    #[test]
    fn test_set_value_empty_string() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_value("");
        assert!(state.is_empty());
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.cursor_position(), (0, 0));
    }

    #[test]
    fn test_set_placeholder_method() {
        let mut state = TextAreaState::new();
        state.set_placeholder("Type here...");
        assert_eq!(state.placeholder(), "Type here...");
    }

    #[test]
    fn test_cursor_col_accessor() {
        let state = TextAreaState::with_value("Hello");
        assert_eq!(state.cursor_col(), 5);
    }

    #[test]
    fn test_word_left_at_line_start() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(1, 0); // Start of "World"
        TextArea::update(&mut state, TextAreaMessage::WordLeft);
        // Should wrap to end of previous line
        assert_eq!(state.cursor_position(), (0, 5));
    }

    #[test]
    fn test_word_left_skip_whitespace() {
        let mut state = TextAreaState::with_value("hello   world");
        state.set_cursor(0, 8); // In the middle of spaces
        TextArea::update(&mut state, TextAreaMessage::WordLeft);
        assert!(state.cursor_col() < 8);
    }

    #[test]
    fn test_word_right_at_line_end() {
        let mut state = TextAreaState::with_value("Hello\nWorld");
        state.set_cursor(0, 5); // End of "Hello"
        TextArea::update(&mut state, TextAreaMessage::WordRight);
        // Should wrap to start of next line
        assert_eq!(state.cursor_position(), (1, 0));
    }

    #[test]
    fn test_word_right_skip_word() {
        let mut state = TextAreaState::with_value("abc def");
        state.set_cursor(0, 0);
        TextArea::update(&mut state, TextAreaMessage::WordRight);
        // Should skip past "abc " to start of "def"
        assert_eq!(state.cursor_position(), (0, 4));
    }

    #[test]
    fn test_delete_line_last_line() {
        let mut state = TextAreaState::with_value("Line 1\nLine 2");
        state.set_cursor(1, 3); // On last line
        TextArea::update(&mut state, TextAreaMessage::DeleteLine);
        // Should adjust cursor_row when deleting the last line
        assert_eq!(state.line_count(), 1);
        assert_eq!(state.cursor_row(), 0);
    }

    #[test]
    fn test_delete_line_single_empty() {
        let mut state = TextArea::init();
        // Single empty line - should return None
        let output = TextArea::update(&mut state, TextAreaMessage::DeleteLine);
        assert_eq!(output, None);
    }

    #[test]
    fn test_delete_to_end_at_end() {
        let mut state = TextAreaState::with_value("Hello");
        // Cursor already at end
        let output = TextArea::update(&mut state, TextAreaMessage::DeleteToEnd);
        assert_eq!(output, None);
    }

    #[test]
    fn test_delete_to_start_at_start() {
        let mut state = TextAreaState::with_value("Hello");
        state.set_cursor(0, 0);
        let output = TextArea::update(&mut state, TextAreaMessage::DeleteToStart);
        assert_eq!(output, None);
    }

    #[test]
    fn test_view_with_scroll() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        // Create a long content that needs scrolling
        let mut state = TextAreaState::with_value(
            "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10",
        );
        state.focused = true;

        let backend = CaptureBackend::new(40, 5); // Small height to trigger scrolling
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TextArea::view(&state, frame, frame.area());
            })
            .unwrap();

        // Should render without panic
    }

    #[test]
    fn test_view_cursor_above_scroll() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = TextAreaState::with_value("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        state.scroll_offset = 5; // Scroll down
        state.set_cursor(2, 0); // Cursor above scroll
        state.focused = true;

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TextArea::view(&state, frame, frame.area());
            })
            .unwrap();

        // Should adjust scroll to show cursor
    }

    #[test]
    fn test_ensure_cursor_visible_zero_lines() {
        let mut state = TextAreaState::with_value("Hello");
        state.ensure_cursor_visible(0);
        // Should not panic or change anything
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn test_text_area_message_debug() {
        let msg = TextAreaMessage::Insert('x');
        let debug = format!("{:?}", msg);
        assert!(debug.contains("Insert"));
    }

    #[test]
    fn test_text_area_message_eq() {
        assert_eq!(TextAreaMessage::Left, TextAreaMessage::Left);
        assert_eq!(TextAreaMessage::Right, TextAreaMessage::Right);
        assert_eq!(TextAreaMessage::Up, TextAreaMessage::Up);
        assert_eq!(TextAreaMessage::Down, TextAreaMessage::Down);
        assert_eq!(TextAreaMessage::Home, TextAreaMessage::Home);
        assert_eq!(TextAreaMessage::End, TextAreaMessage::End);
        assert_eq!(TextAreaMessage::TextStart, TextAreaMessage::TextStart);
        assert_eq!(TextAreaMessage::TextEnd, TextAreaMessage::TextEnd);
        assert_eq!(TextAreaMessage::WordLeft, TextAreaMessage::WordLeft);
        assert_eq!(TextAreaMessage::WordRight, TextAreaMessage::WordRight);
        assert_eq!(TextAreaMessage::Insert('a'), TextAreaMessage::Insert('a'));
        assert_eq!(TextAreaMessage::NewLine, TextAreaMessage::NewLine);
        assert_eq!(TextAreaMessage::Backspace, TextAreaMessage::Backspace);
        assert_eq!(TextAreaMessage::Delete, TextAreaMessage::Delete);
        assert_eq!(TextAreaMessage::DeleteLine, TextAreaMessage::DeleteLine);
        assert_eq!(TextAreaMessage::DeleteToEnd, TextAreaMessage::DeleteToEnd);
        assert_eq!(
            TextAreaMessage::DeleteToStart,
            TextAreaMessage::DeleteToStart
        );
        assert_eq!(TextAreaMessage::Clear, TextAreaMessage::Clear);
        assert_eq!(TextAreaMessage::Submit, TextAreaMessage::Submit);
    }

    #[test]
    fn test_text_area_output_debug() {
        let out = TextAreaOutput::Changed("test".to_string());
        let debug = format!("{:?}", out);
        assert!(debug.contains("Changed"));
    }

    #[test]
    fn test_text_area_output_eq() {
        let out1 = TextAreaOutput::Changed("a".to_string());
        let out2 = TextAreaOutput::Changed("a".to_string());
        assert_eq!(out1, out2);

        let out3 = TextAreaOutput::Submitted("b".to_string());
        let out4 = TextAreaOutput::Submitted("b".to_string());
        assert_eq!(out3, out4);
    }

    #[test]
    fn test_state_debug() {
        let state = TextAreaState::with_value("test");
        let debug = format!("{:?}", state);
        assert!(debug.contains("TextAreaState"));
    }

    #[test]
    fn test_backspace_unicode() {
        let mut state = TextAreaState::with_value("日本");
        TextArea::update(&mut state, TextAreaMessage::Backspace);
        assert_eq!(state.value(), "日");
    }

    #[test]
    fn test_delete_unicode() {
        let mut state = TextAreaState::with_value("日本");
        state.set_cursor(0, 0);
        TextArea::update(&mut state, TextAreaMessage::Delete);
        assert_eq!(state.value(), "本");
    }
}
