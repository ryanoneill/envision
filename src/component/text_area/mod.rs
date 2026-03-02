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
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

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

    // Selection
    /// Extend selection left by one character.
    SelectLeft,
    /// Extend selection right by one character.
    SelectRight,
    /// Extend selection up by one line.
    SelectUp,
    /// Extend selection down by one line.
    SelectDown,
    /// Extend selection to beginning of line.
    SelectHome,
    /// Extend selection to end of line.
    SelectEnd,
    /// Extend selection left by one word.
    SelectWordLeft,
    /// Extend selection right by one word.
    SelectWordRight,
    /// Select all text.
    SelectAll,

    // Clipboard
    /// Copy selected text to internal clipboard.
    Copy,
    /// Cut selected text to internal clipboard.
    Cut,
    /// Paste text at cursor, replacing any selection.
    Paste(String),

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
    /// Text was copied to the internal clipboard.
    Copied(String),
}

/// State for a TextArea component.
#[derive(Clone, Debug, PartialEq)]
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
    /// Whether the textarea is disabled.
    disabled: bool,
    /// Placeholder text shown when empty.
    placeholder: String,
    /// Selection anchor position (row, col_byte). When Some, text is selected
    /// from anchor to cursor.
    selection_anchor: Option<(usize, usize)>,
    /// Internal clipboard buffer for copy/cut/paste.
    clipboard: String,
}

impl Default for TextAreaState {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            focused: false,
            disabled: false,
            placeholder: String::new(),
            selection_anchor: None,
            clipboard: String::new(),
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

    /// Creates a textarea with initial content, split on newlines.
    /// Cursor is placed at the end of the content.
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
            disabled: false,
            placeholder: String::new(),
            selection_anchor: None,
            clipboard: String::new(),
        }
    }

    /// Creates a textarea with placeholder text.
    pub fn with_placeholder(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..Default::default()
        }
    }

    /// Returns the full text content (lines joined with \n).
    pub fn value(&self) -> String {
        self.lines.join("\n")
    }

    /// Sets the content from a string (splits on \n). Cursor moves to end.
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
        self.selection_anchor = None;
    }

    /// Returns the cursor position as (row, char_column).
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

    /// Returns true if there is an active text selection.
    pub fn has_selection(&self) -> bool {
        match self.selection_anchor {
            Some((r, c)) => r != self.cursor_row || c != self.cursor_col,
            None => false,
        }
    }

    /// Returns the ordered selection positions as `((start_row, start_col), (end_row, end_col))`.
    pub fn selection_positions(&self) -> Option<((usize, usize), (usize, usize))> {
        let (ar, ac) = self.selection_anchor?;
        if ar == self.cursor_row && ac == self.cursor_col {
            return None;
        }
        let a = (ar, ac);
        let b = (self.cursor_row, self.cursor_col);
        if a < b { Some((a, b)) } else { Some((b, a)) }
    }

    /// Returns the selected text, or None if no selection.
    pub fn selected_text(&self) -> Option<String> {
        let ((sr, sc), (er, ec)) = self.selection_positions()?;
        if sr == er {
            Some(self.lines[sr][sc..ec].to_string())
        } else {
            let mut result = self.lines[sr][sc..].to_string();
            for row in (sr + 1)..er {
                result.push('\n');
                result.push_str(&self.lines[row]);
            }
            result.push('\n');
            result.push_str(&self.lines[er][..ec]);
            Some(result)
        }
    }

    /// Returns the internal clipboard contents.
    pub fn clipboard(&self) -> &str {
        &self.clipboard
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn ensure_selection_anchor(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some((self.cursor_row, self.cursor_col));
        }
    }

    /// Deletes selected text. Returns deleted text or None.
    fn delete_selection(&mut self) -> Option<String> {
        let ((sr, sc), (er, ec)) = self.selection_positions()?;
        let deleted = self.selected_text()?;
        if sr == er {
            self.lines[sr].drain(sc..ec);
        } else {
            let after = self.lines[er][ec..].to_string();
            self.lines[sr].truncate(sc);
            self.lines[sr].push_str(&after);
            self.lines.drain((sr + 1)..=er);
        }
        self.cursor_row = sr;
        self.cursor_col = sc;
        self.selection_anchor = None;
        Some(deleted)
    }

    /// Returns true if the textarea is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the textarea is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the disabled state using builder pattern.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Maps an input event to a textarea message.
    pub fn handle_event(&self, event: &Event) -> Option<TextAreaMessage> {
        TextArea::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<TextAreaOutput> {
        TextArea::dispatch_event(self, event)
    }

    /// Updates the textarea state with a message, returning any output.
    pub fn update(&mut self, msg: TextAreaMessage) -> Option<TextAreaOutput> {
        TextArea::update(self, msg)
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

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }
        if let Event::Paste(ref text) = event {
            return Some(TextAreaMessage::Paste(text.clone()));
        }
        if let Some(key) = event.as_key() {
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);
            match key.code {
                // Clipboard
                KeyCode::Char('c') if ctrl => Some(TextAreaMessage::Copy),
                KeyCode::Char('x') if ctrl => Some(TextAreaMessage::Cut),
                KeyCode::Char('v') if ctrl => {
                    if state.clipboard.is_empty() { None }
                    else { Some(TextAreaMessage::Paste(state.clipboard.clone())) }
                }
                KeyCode::Char('a') if ctrl => Some(TextAreaMessage::SelectAll),
                KeyCode::Char(c) if !ctrl => Some(TextAreaMessage::Insert(c)),
                KeyCode::Enter => Some(TextAreaMessage::NewLine),
                // Selection movement
                KeyCode::Left if ctrl && shift => Some(TextAreaMessage::SelectWordLeft),
                KeyCode::Right if ctrl && shift => Some(TextAreaMessage::SelectWordRight),
                KeyCode::Left if shift => Some(TextAreaMessage::SelectLeft),
                KeyCode::Right if shift => Some(TextAreaMessage::SelectRight),
                KeyCode::Up if shift => Some(TextAreaMessage::SelectUp),
                KeyCode::Down if shift => Some(TextAreaMessage::SelectDown),
                KeyCode::Home if shift => Some(TextAreaMessage::SelectHome),
                KeyCode::End if shift => Some(TextAreaMessage::SelectEnd),
                // Deletion
                KeyCode::Backspace if ctrl => Some(TextAreaMessage::DeleteLine),
                KeyCode::Backspace => Some(TextAreaMessage::Backspace),
                KeyCode::Delete => Some(TextAreaMessage::Delete),
                // Navigation
                KeyCode::Left if ctrl => Some(TextAreaMessage::WordLeft),
                KeyCode::Right if ctrl => Some(TextAreaMessage::WordRight),
                KeyCode::Left => Some(TextAreaMessage::Left),
                KeyCode::Right => Some(TextAreaMessage::Right),
                KeyCode::Up => Some(TextAreaMessage::Up),
                KeyCode::Down => Some(TextAreaMessage::Down),
                KeyCode::Home if ctrl => Some(TextAreaMessage::TextStart),
                KeyCode::End if ctrl => Some(TextAreaMessage::TextEnd),
                KeyCode::Home => Some(TextAreaMessage::Home),
                KeyCode::End => Some(TextAreaMessage::End),
                KeyCode::Char('k') if ctrl => Some(TextAreaMessage::DeleteToEnd),
                KeyCode::Char('u') if ctrl => Some(TextAreaMessage::DeleteToStart),
                _ => None,
            }
        } else {
            None
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            // Editing (replaces selection if active)
            TextAreaMessage::Insert(c) => {
                state.delete_selection();
                state.insert(c);
                Some(TextAreaOutput::Changed(state.value()))
            }
            TextAreaMessage::NewLine => {
                state.delete_selection();
                state.new_line();
                Some(TextAreaOutput::Changed(state.value()))
            }
            TextAreaMessage::Backspace => {
                if state.has_selection() {
                    state.delete_selection();
                    Some(TextAreaOutput::Changed(state.value()))
                } else if state.backspace() {
                    Some(TextAreaOutput::Changed(state.value()))
                } else { None }
            }
            TextAreaMessage::Delete => {
                if state.has_selection() {
                    state.delete_selection();
                    Some(TextAreaOutput::Changed(state.value()))
                } else if state.delete() {
                    Some(TextAreaOutput::Changed(state.value()))
                } else { None }
            }
            // Navigation (clears selection)
            TextAreaMessage::Left => {
                if state.has_selection() {
                    if let Some((start, _)) = state.selection_positions() {
                        state.cursor_row = start.0;
                        state.cursor_col = start.1;
                    }
                    state.clear_selection();
                } else { state.move_left(); }
                None
            }
            TextAreaMessage::Right => {
                if state.has_selection() {
                    if let Some((_, end)) = state.selection_positions() {
                        state.cursor_row = end.0;
                        state.cursor_col = end.1;
                    }
                    state.clear_selection();
                } else { state.move_right(); }
                None
            }
            TextAreaMessage::Up => { state.clear_selection(); state.move_up(); None }
            TextAreaMessage::Down => { state.clear_selection(); state.move_down(); None }
            TextAreaMessage::Home => { state.clear_selection(); state.cursor_col = 0; None }
            TextAreaMessage::End => {
                state.clear_selection();
                state.cursor_col = state.lines[state.cursor_row].len();
                None
            }
            TextAreaMessage::TextStart => {
                state.clear_selection();
                state.cursor_row = 0;
                state.cursor_col = 0;
                None
            }
            TextAreaMessage::TextEnd => {
                state.clear_selection();
                state.cursor_row = state.lines.len() - 1;
                state.cursor_col = state.lines[state.cursor_row].len();
                None
            }
            TextAreaMessage::WordLeft => { state.clear_selection(); state.move_word_left(); None }
            TextAreaMessage::WordRight => { state.clear_selection(); state.move_word_right(); None }
            // Selection movement
            TextAreaMessage::SelectLeft => { state.ensure_selection_anchor(); state.move_left(); None }
            TextAreaMessage::SelectRight => { state.ensure_selection_anchor(); state.move_right(); None }
            TextAreaMessage::SelectUp => { state.ensure_selection_anchor(); state.move_up(); None }
            TextAreaMessage::SelectDown => { state.ensure_selection_anchor(); state.move_down(); None }
            TextAreaMessage::SelectHome => { state.ensure_selection_anchor(); state.cursor_col = 0; None }
            TextAreaMessage::SelectEnd => {
                state.ensure_selection_anchor();
                state.cursor_col = state.lines[state.cursor_row].len();
                None
            }
            TextAreaMessage::SelectWordLeft => { state.ensure_selection_anchor(); state.move_word_left(); None }
            TextAreaMessage::SelectWordRight => { state.ensure_selection_anchor(); state.move_word_right(); None }
            TextAreaMessage::SelectAll => {
                if state.is_empty() { return None; }
                state.selection_anchor = Some((0, 0));
                let last = state.lines.len() - 1;
                state.cursor_row = last;
                state.cursor_col = state.lines[last].len();
                None
            }
            // Clipboard
            TextAreaMessage::Copy => {
                if let Some(text) = state.selected_text() {
                    state.clipboard = text.clone();
                    Some(TextAreaOutput::Copied(text))
                } else { None }
            }
            TextAreaMessage::Cut => {
                if let Some(text) = state.selected_text() {
                    state.clipboard = text.clone();
                    state.delete_selection();
                    Some(TextAreaOutput::Changed(state.value()))
                } else { None }
            }
            TextAreaMessage::Paste(text) => {
                if text.is_empty() { return None; }
                state.delete_selection();
                for c in text.chars() {
                    if c == '\n' { state.new_line(); } else { state.insert(c); }
                }
                Some(TextAreaOutput::Changed(state.value()))
            }
            // Line operations
            TextAreaMessage::DeleteLine => {
                state.clear_selection();
                if state.delete_line() { Some(TextAreaOutput::Changed(state.value())) } else { None }
            }
            TextAreaMessage::DeleteToEnd => {
                if state.has_selection() {
                    state.delete_selection();
                    Some(TextAreaOutput::Changed(state.value()))
                } else if state.delete_to_end() {
                    Some(TextAreaOutput::Changed(state.value()))
                } else { None }
            }
            TextAreaMessage::DeleteToStart => {
                if state.has_selection() {
                    state.delete_selection();
                    Some(TextAreaOutput::Changed(state.value()))
                } else if state.delete_to_start() {
                    Some(TextAreaOutput::Changed(state.value()))
                } else { None }
            }
            TextAreaMessage::Clear => {
                state.clear_selection();
                if !state.is_empty() {
                    state.lines = vec![String::new()];
                    state.cursor_row = 0;
                    state.cursor_col = 0;
                    state.scroll_offset = 0;
                    Some(TextAreaOutput::Changed(state.value()))
                } else { None }
            }
            TextAreaMessage::SetValue(value) => {
                let old_value = state.value();
                if old_value != value {
                    state.set_value(value);
                    Some(TextAreaOutput::Changed(state.value()))
                } else { None }
            }
            TextAreaMessage::Submit => Some(TextAreaOutput::Submitted(state.value())),
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
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

        let style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_style()
        } else if state.is_empty() && !state.placeholder.is_empty() {
            theme.placeholder_style()
        } else {
            theme.normal_style()
        };

        let border_style = if state.focused && !state.disabled {
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
mod tests;
