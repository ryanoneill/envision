//! A multi-line text editing component with cursor navigation.
//!
//! [`TextArea`] provides multi-line text input with cursor movement,
//! text insertion, deletion, and line operations. State is stored in
//! [`TextAreaState`], updated via [`TextAreaMessage`], and produces
//! [`TextAreaOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! See also [`InputField`](super::InputField) for single-line input,
//! and [`LineInput`](super::LineInput) for a wrapping single-line input.
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
use unicode_width::UnicodeWidthStr;

use super::{Component, Disableable, Focusable};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;
use crate::undo::UndoStack;

/// Attempt to write text to the system clipboard.
///
/// Errors are silently ignored — this is best-effort. Falls back gracefully
/// in headless environments (CI, SSH) where no clipboard provider exists.
#[cfg(feature = "clipboard")]
fn system_clipboard_set(text: &str) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
}

/// Attempt to read text from the system clipboard.
///
/// Returns `None` if the clipboard is unavailable or doesn't contain text.
#[cfg(feature = "clipboard")]
fn system_clipboard_get() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut cb| cb.get_text().ok())
        .filter(|s| !s.is_empty())
}

mod cursor;
mod selection;
mod update;

/// A snapshot of TextArea state for undo/redo.
#[derive(Debug, Clone)]
struct TextAreaSnapshot {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
}

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
    /// Undo the last edit.
    Undo,
    /// Redo the last undone edit.
    Redo,
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
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
    /// Undo/redo history stack.
    #[cfg_attr(feature = "serialization", serde(skip))]
    undo_stack: UndoStack<TextAreaSnapshot>,
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
            undo_stack: UndoStack::default(),
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
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TextAreaState::with_value("Hello\nWorld");
    /// assert_eq!(state.value(), "Hello\nWorld");
    /// assert_eq!(state.line_count(), 2);
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
            disabled: false,
            placeholder: String::new(),
            selection_anchor: None,
            clipboard: String::new(),
            undo_stack: UndoStack::default(),
        }
    }

    /// Creates a textarea with placeholder text.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TextAreaState::with_placeholder("Enter text...");
    /// assert_eq!(state.placeholder(), "Enter text...");
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
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TextAreaState::with_value("line1\nline2");
    /// assert_eq!(state.value(), "line1\nline2");
    /// ```
    pub fn value(&self) -> String {
        self.lines.join("\n")
    }

    /// Sets the content from a string (splits on \n). Cursor moves to end.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut state = TextAreaState::new();
    /// state.set_value("Hello\nWorld");
    /// assert_eq!(state.value(), "Hello\nWorld");
    /// assert_eq!(state.cursor_position(), (1, 5));
    /// ```
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

    /// Returns the cursor display position as (row, terminal_column_width).
    ///
    /// Unlike [`cursor_position()`](Self::cursor_position) which returns the
    /// character count for the column, this returns the display width
    /// accounting for wide characters (emoji, CJK) that occupy 2 terminal columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaState, TextAreaMessage, Component};
    ///
    /// let mut state = TextArea::init();
    /// TextArea::update(&mut state, TextAreaMessage::Insert('A'));
    /// TextArea::update(&mut state, TextAreaMessage::Insert('\u{1F600}')); // emoji
    ///
    /// // Character count is 2 (two characters)
    /// assert_eq!(state.cursor_position(), (0, 2));
    /// // Display width is 3 (A=1 + 😀=2)
    /// assert_eq!(state.cursor_display_position(), (0, 3));
    /// ```
    pub fn cursor_display_position(&self) -> (usize, usize) {
        let display_col = self.lines[self.cursor_row][..self.cursor_col].width();
        (self.cursor_row, display_col)
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
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// assert!(TextAreaState::new().is_empty());
    /// assert!(!TextAreaState::with_value("hi").is_empty());
    /// ```
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
    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
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

    /// Returns true if there are edits that can be undone.
    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    /// Returns true if there are edits that can be redone.
    pub fn can_redo(&self) -> bool {
        self.undo_stack.can_redo()
    }

    /// Creates a snapshot of the current state for undo.
    fn snapshot(&self) -> TextAreaSnapshot {
        TextAreaSnapshot {
            lines: self.lines.clone(),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
        }
    }

    /// Restores state from a snapshot.
    fn restore(&mut self, snapshot: TextAreaSnapshot) {
        self.lines = snapshot.lines;
        self.cursor_row = snapshot.cursor_row;
        self.cursor_col = snapshot.cursor_col;
        self.clear_selection();
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
                // Undo/redo
                KeyCode::Char('z') if ctrl => Some(TextAreaMessage::Undo),
                KeyCode::Char('y') if ctrl => Some(TextAreaMessage::Redo),
                // Clipboard
                KeyCode::Char('c') if ctrl => Some(TextAreaMessage::Copy),
                KeyCode::Char('x') if ctrl => Some(TextAreaMessage::Cut),
                KeyCode::Char('v') if ctrl => {
                    // Try system clipboard first, fall back to internal
                    #[cfg(feature = "clipboard")]
                    if let Some(text) = system_clipboard_get() {
                        return Some(TextAreaMessage::Paste(text));
                    }
                    if state.clipboard.is_empty() {
                        None
                    } else {
                        Some(TextAreaMessage::Paste(state.clipboard.clone()))
                    }
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
        state.apply_update(msg)
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        crate::annotation::with_registry(|reg| {
            let first_line = state.lines.first().map_or("", |l| l.as_str());
            reg.register(
                area,
                crate::annotation::Annotation::text_area("text_area")
                    .with_value(first_line)
                    .with_focus(state.focused)
                    .with_disabled(state.disabled),
            );
        });

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
            let (_, display_col) = state.cursor_display_position();

            let cursor_x = area.x + 1 + display_col as u16;
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

impl Disableable for TextArea {
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
mod undo_tests;
