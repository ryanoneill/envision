//! A multi-line text editing component with cursor navigation.
//!
//! [`TextArea`] provides multi-line text input with cursor movement,
//! text insertion, deletion, and line operations. State is stored in
//! [`TextAreaState`], updated via [`TextAreaMessage`], and produces
//! [`TextAreaOutput`].
//!
//!
//! See also [`InputField`](super::InputField) for single-line input,
//! and [`LineInput`](super::LineInput) for a wrapping single-line input.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, TextArea, TextAreaState, TextAreaMessage};
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

use ratatui::widgets::{Block, Borders, Paragraph};
use unicode_width::UnicodeWidthStr;

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::undo::UndoStack;

#[cfg(feature = "clipboard")]
use crate::clipboard::system_clipboard_get;

mod cursor;
mod search;
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

    // Search
    /// Start search mode.
    StartSearch,
    /// Set the search query and recompute matches.
    SetSearchQuery(String),
    /// Jump to next search match.
    NextMatch,
    /// Jump to previous search match.
    PrevMatch,
    /// Clear search and exit search mode.
    ClearSearch,

    // Display
    /// Toggle line number display.
    ToggleLineNumbers,
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
    /// Whether to show line numbers.
    show_line_numbers: bool,
    /// Current search query (None = not searching).
    search_query: Option<String>,
    /// List of search matches as (line, byte_col) pairs.
    search_matches: Vec<(usize, usize)>,
    /// Index of the current match within search_matches.
    current_match: usize,
}

impl Default for TextAreaState {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            placeholder: String::new(),
            selection_anchor: None,
            clipboard: String::new(),
            undo_stack: UndoStack::default(),
            show_line_numbers: false,
            search_query: None,
            search_matches: Vec::new(),
            current_match: 0,
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
    /// Sets the content and places cursor at the end (builder pattern).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TextAreaState::new().with_value("Hello\nWorld");
    /// assert_eq!(state.value(), "Hello\nWorld");
    /// assert_eq!(state.line_count(), 2);
    /// ```
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.lines = if value.is_empty() {
            vec![String::new()]
        } else {
            // Use split('\n') instead of lines() to preserve trailing newlines
            value.split('\n').map(String::from).collect()
        };

        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = self.lines.last().map(|l| l.len()).unwrap_or(0);
        self
    }

    /// Sets the placeholder text (builder pattern).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TextAreaState::new().with_placeholder("Enter text...");
    /// assert_eq!(state.placeholder(), "Enter text...");
    /// assert!(state.is_empty());
    /// ```
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Returns the full text content (lines joined with \n).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TextAreaState::new().with_value("line1\nline2");
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaMessage, TextAreaState, Component};
    ///
    /// let mut state = TextAreaState::new().with_value("Line 1\nLine 2");
    /// assert_eq!(state.cursor_row(), 1);
    /// ```
    pub fn cursor_row(&self) -> usize {
        self.cursor_row
    }

    /// Returns the cursor column (byte offset).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaMessage, TextAreaState, Component};
    ///
    /// let state = TextAreaState::new().with_value("Hello");
    /// assert_eq!(state.cursor_col(), 5);
    /// ```
    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }

    /// Returns the number of lines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::new().with_value("a\nb\nc");
    /// assert_eq!(state.line_count(), 3);
    /// ```
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns a specific line by index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::new().with_value("first\nsecond\nthird");
    /// assert_eq!(state.line(0), Some("first"));
    /// assert_eq!(state.line(1), Some("second"));
    /// assert_eq!(state.line(99), None);
    /// ```
    pub fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }

    /// Returns the current line (at cursor row).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::new().with_value("first\nsecond");
    /// assert_eq!(state.current_line(), "second");
    /// ```
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
    /// assert!(!TextAreaState::new().with_value("hi").is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    /// Returns the placeholder text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::new().with_placeholder("Type here...");
    /// assert_eq!(state.placeholder(), "Type here...");
    /// ```
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Sets the placeholder text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let mut state = TextAreaState::new();
    /// state.set_placeholder("Enter your text...");
    /// assert_eq!(state.placeholder(), "Enter your text...");
    /// ```
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Returns the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the cursor position (row, char_column).
    ///
    /// Both row and column are clamped to valid ranges.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let mut state = TextAreaState::new().with_value("Hello\nWorld");
    /// state.set_cursor_position(0, 3);
    /// assert_eq!(state.cursor_position(), (0, 3));
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaMessage, TextAreaState, Component};
    ///
    /// let mut state = TextAreaState::new();
    /// assert!(!state.can_undo());
    /// TextArea::update(&mut state, TextAreaMessage::Insert('a'));
    /// assert!(state.can_undo());
    /// ```
    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    /// Returns true if there are edits that can be redone.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TextArea, TextAreaMessage, TextAreaState, Component};
    ///
    /// let mut state = TextAreaState::new();
    /// TextArea::update(&mut state, TextAreaMessage::Insert('a'));
    /// TextArea::update(&mut state, TextAreaMessage::Undo);
    /// assert!(state.can_redo());
    /// ```
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

    /// Sets whether line numbers are shown (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::new().with_line_numbers(true);
    /// assert!(state.show_line_numbers());
    /// ```
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    /// Returns whether line numbers are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let state = TextAreaState::new().with_line_numbers(true);
    /// assert!(state.show_line_numbers());
    /// ```
    pub fn show_line_numbers(&self) -> bool {
        self.show_line_numbers
    }

    /// Sets whether line numbers are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TextAreaState;
    ///
    /// let mut state = TextAreaState::new();
    /// state.set_show_line_numbers(true);
    /// assert!(state.show_line_numbers());
    /// ```
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
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

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }
        if let Event::Paste(text) = event {
            return Some(TextAreaMessage::Paste(text.clone()));
        }
        if let Some(key) = event.as_key() {
            let ctrl = key.modifiers.ctrl();
            let shift = key.modifiers.shift();
            match key.code {
                // Undo/redo
                Key::Char('z') if ctrl => Some(TextAreaMessage::Undo),
                Key::Char('y') if ctrl => Some(TextAreaMessage::Redo),
                // Clipboard
                Key::Char('c') if ctrl => Some(TextAreaMessage::Copy),
                Key::Char('x') if ctrl => Some(TextAreaMessage::Cut),
                Key::Char('v') if ctrl => {
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
                Key::Char('a') if ctrl => Some(TextAreaMessage::SelectAll),
                Key::Char(_) if !ctrl => key.raw_char.map(TextAreaMessage::Insert),
                Key::Enter => Some(TextAreaMessage::NewLine),
                // Selection movement
                Key::Left if ctrl && shift => Some(TextAreaMessage::SelectWordLeft),
                Key::Right if ctrl && shift => Some(TextAreaMessage::SelectWordRight),
                Key::Left if shift => Some(TextAreaMessage::SelectLeft),
                Key::Right if shift => Some(TextAreaMessage::SelectRight),
                Key::Up if shift => Some(TextAreaMessage::SelectUp),
                Key::Down if shift => Some(TextAreaMessage::SelectDown),
                Key::Home if shift => Some(TextAreaMessage::SelectHome),
                Key::End if shift => Some(TextAreaMessage::SelectEnd),
                // Deletion
                Key::Backspace if ctrl => Some(TextAreaMessage::DeleteLine),
                Key::Backspace => Some(TextAreaMessage::Backspace),
                Key::Delete => Some(TextAreaMessage::Delete),
                // Navigation
                Key::Left if ctrl => Some(TextAreaMessage::WordLeft),
                Key::Right if ctrl => Some(TextAreaMessage::WordRight),
                Key::Left => Some(TextAreaMessage::Left),
                Key::Right => Some(TextAreaMessage::Right),
                Key::Up => Some(TextAreaMessage::Up),
                Key::Down => Some(TextAreaMessage::Down),
                Key::Home if ctrl => Some(TextAreaMessage::TextStart),
                Key::End if ctrl => Some(TextAreaMessage::TextEnd),
                Key::Home => Some(TextAreaMessage::Home),
                Key::End => Some(TextAreaMessage::End),
                Key::Char('k') if ctrl => Some(TextAreaMessage::DeleteToEnd),
                Key::Char('u') if ctrl => Some(TextAreaMessage::DeleteToStart),
                _ => None,
            }
        } else {
            None
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        state.apply_update(msg)
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        crate::annotation::with_registry(|reg| {
            let first_line = state.lines.first().map_or("", |l| l.as_str());
            reg.register(
                ctx.area,
                crate::annotation::Annotation::text_area("text_area")
                    .with_value(first_line)
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let inner_height = ctx.area.height.saturating_sub(2) as usize; // Account for borders

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

        let style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else if ctx.focused {
            ctx.theme.focused_style()
        } else if state.is_empty() && !state.placeholder.is_empty() {
            ctx.theme.placeholder_style()
        } else {
            ctx.theme.normal_style()
        };

        let border_style = if ctx.focused && !ctx.disabled {
            ctx.theme.focused_border_style()
        } else {
            ctx.theme.border_style()
        };

        let paragraph = Paragraph::new(display_text).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        ctx.frame.render_widget(paragraph, ctx.area);

        // Show cursor when focused
        if ctx.focused && ctx.area.width > 2 && ctx.area.height > 2 {
            let cursor_row_in_view = state.cursor_row.saturating_sub(scroll);
            let (_, display_col) = state.cursor_display_position();

            let cursor_x = ctx.area.x + 1 + display_col as u16;
            let cursor_y = ctx.area.y + 1 + cursor_row_in_view as u16;

            // Only show cursor if it's within the visible ctx.area
            if cursor_x < ctx.area.x + ctx.area.width - 1
                && cursor_y < ctx.area.y + ctx.area.height - 1
                && cursor_row_in_view < inner_height
            {
                ctx.frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod undo_tests;
