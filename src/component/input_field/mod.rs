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
use unicode_width::UnicodeWidthStr;

use super::{Component, Focusable};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;
use crate::undo::{EditKind, UndoStack};

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

/// A snapshot of InputField state for undo/redo.
#[derive(Debug, Clone)]
struct InputSnapshot {
    value: String,
    cursor: usize,
}

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
    /// Extend selection left by one character.
    SelectLeft,
    /// Extend selection right by one character.
    SelectRight,
    /// Extend selection to the beginning of the input.
    SelectHome,
    /// Extend selection to the end of the input.
    SelectEnd,
    /// Extend selection left by one word.
    SelectWordLeft,
    /// Extend selection right by one word.
    SelectWordRight,
    /// Select all text.
    SelectAll,
    /// Copy the selected text to the internal clipboard.
    Copy,
    /// Cut the selected text to the internal clipboard.
    Cut,
    /// Paste text at the cursor position, replacing any selection.
    Paste(String),
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
    /// Undo the last edit.
    Undo,
    /// Redo the last undone edit.
    Redo,
}

/// Output messages from an InputField.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputFieldOutput {
    /// The value was submitted (e.g., Enter pressed).
    Submitted(String),
    /// The value changed.
    Changed(String),
    /// Text was copied to the internal clipboard.
    Copied(String),
}

/// State for an InputField component.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct InputFieldState {
    /// The current text value.
    value: String,
    /// Cursor position (byte offset into value).
    cursor: usize,
    /// Whether the input is focused.
    focused: bool,
    /// Whether the input is disabled.
    disabled: bool,
    /// Placeholder text shown when empty.
    placeholder: String,
    /// Selection anchor (byte offset). When `Some`, text is selected from
    /// the anchor to the cursor. The anchor stays fixed while the cursor moves.
    selection_anchor: Option<usize>,
    /// Internal clipboard buffer for copy/cut/paste operations.
    clipboard: String,
    /// Undo/redo history stack.
    #[cfg_attr(feature = "serialization", serde(skip))]
    undo_stack: UndoStack<InputSnapshot>,
}

impl InputFieldState {
    /// Creates a new empty input field state.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = InputFieldState::new();
    /// assert_eq!(state.value(), "");
    /// assert_eq!(state.cursor_position(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new state with the given initial value.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = InputFieldState::with_value("hello");
    /// assert_eq!(state.value(), "hello");
    /// assert_eq!(state.cursor_position(), 5);
    /// ```
    pub fn with_value(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.len();
        Self {
            value,
            cursor,
            focused: false,
            disabled: false,
            placeholder: String::new(),
            selection_anchor: None,
            clipboard: String::new(),
            undo_stack: UndoStack::default(),
        }
    }

    /// Creates a new state with placeholder text.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = InputFieldState::with_placeholder("Enter name...");
    /// assert_eq!(state.placeholder(), "Enter name...");
    /// assert_eq!(state.value(), "");
    /// ```
    pub fn with_placeholder(placeholder: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            focused: false,
            disabled: false,
            placeholder: placeholder.into(),
            selection_anchor: None,
            clipboard: String::new(),
            undo_stack: UndoStack::default(),
        }
    }

    /// Returns the current value.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = InputFieldState::with_value("hello");
    /// assert_eq!(state.value(), "hello");
    /// ```
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets the value and moves cursor to the end.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut state = InputFieldState::new();
    /// state.set_value("world");
    /// assert_eq!(state.value(), "world");
    /// assert_eq!(state.cursor_position(), 5);
    /// ```
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.len();
        self.selection_anchor = None;
    }

    /// Returns the cursor position (character index).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = InputFieldState::with_value("abc");
    /// assert_eq!(state.cursor_position(), 3);
    /// ```
    pub fn cursor_position(&self) -> usize {
        self.value[..self.cursor].chars().count()
    }

    /// Returns the cursor display position (terminal column width).
    ///
    /// Unlike [`cursor_position()`](Self::cursor_position) which returns the
    /// character count, this returns the display width accounting for
    /// wide characters (emoji, CJK) that occupy 2 terminal columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{InputField, InputFieldState, InputFieldMessage, Component};
    ///
    /// let mut state = InputField::init();
    /// InputField::update(&mut state, InputFieldMessage::Insert('A'));
    /// InputField::update(&mut state, InputFieldMessage::Insert('\u{1F600}')); // emoji
    ///
    /// // Character count is 2 (two characters)
    /// assert_eq!(state.cursor_position(), 2);
    /// // Display width is 3 (A=1 + 😀=2)
    /// assert_eq!(state.cursor_display_position(), 3);
    /// ```
    pub fn cursor_display_position(&self) -> usize {
        self.value[..self.cursor].width()
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

    /// Returns true if there is an active text selection.
    pub fn has_selection(&self) -> bool {
        self.selection_anchor.is_some() && self.selection_anchor != Some(self.cursor)
    }

    /// Returns the selected byte range as `(start, end)` where `start < end`.
    ///
    /// Returns `None` if there is no active selection or the selection is empty.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_anchor.and_then(|anchor| {
            if anchor == self.cursor {
                None
            } else {
                let start = anchor.min(self.cursor);
                let end = anchor.max(self.cursor);
                Some((start, end))
            }
        })
    }

    /// Returns the currently selected text, or `None` if no selection.
    pub fn selected_text(&self) -> Option<&str> {
        self.selection_range()
            .map(|(start, end)| &self.value[start..end])
    }

    /// Returns a reference to the internal clipboard contents.
    pub fn clipboard(&self) -> &str {
        &self.clipboard
    }

    /// Clears the current selection without modifying text.
    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Sets the selection anchor to the current cursor position if not already set.
    fn ensure_selection_anchor(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
    }

    /// Deletes the currently selected text, returning the deleted text.
    ///
    /// Moves the cursor to the start of the selection. Clears the selection.
    /// Returns `None` if no text was selected.
    fn delete_selection(&mut self) -> Option<String> {
        let (start, end) = self.selection_range()?;
        let deleted: String = self.value[start..end].to_string();
        self.value.drain(start..end);
        self.cursor = start;
        self.selection_anchor = None;
        Some(deleted)
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
    fn snapshot(&self) -> InputSnapshot {
        InputSnapshot {
            value: self.value.clone(),
            cursor: self.cursor,
        }
    }

    /// Restores state from a snapshot.
    fn restore(&mut self, snapshot: InputSnapshot) {
        self.value = snapshot.value;
        self.cursor = snapshot.cursor;
        self.clear_selection();
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
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = InputFieldState::new();
    /// assert!(!state.is_focused());
    /// ```
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut state = InputFieldState::new();
    /// state.set_focused(true);
    /// assert!(state.is_focused());
    /// ```
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the input field is disabled.
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
        if state.disabled {
            return None;
        }
        match msg {
            InputFieldMessage::Insert(c) => {
                if c.is_whitespace() {
                    state.undo_stack.break_group();
                }
                let snapshot = state.snapshot();
                state.undo_stack.save(snapshot, EditKind::Insert);
                state.delete_selection();
                state.insert(c);
                if c.is_whitespace() {
                    state.undo_stack.break_group();
                }
                Some(InputFieldOutput::Changed(state.value.clone()))
            }
            InputFieldMessage::Backspace => {
                let snapshot = state.snapshot();
                if state.has_selection() {
                    state.delete_selection();
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else if state.backspace() {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Delete => {
                let snapshot = state.snapshot();
                if state.has_selection() {
                    state.delete_selection();
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else if state.delete() {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Left => {
                if state.has_selection() {
                    // Move cursor to start of selection
                    if let Some((start, _)) = state.selection_range() {
                        state.cursor = start;
                    }
                    state.clear_selection();
                } else {
                    state.move_left();
                }
                None
            }
            InputFieldMessage::Right => {
                if state.has_selection() {
                    // Move cursor to end of selection
                    if let Some((_, end)) = state.selection_range() {
                        state.cursor = end;
                    }
                    state.clear_selection();
                } else {
                    state.move_right();
                }
                None
            }
            InputFieldMessage::Home => {
                state.clear_selection();
                state.cursor = 0;
                None
            }
            InputFieldMessage::End => {
                state.clear_selection();
                state.cursor = state.value.len();
                None
            }
            InputFieldMessage::WordLeft => {
                state.clear_selection();
                state.move_word_left();
                None
            }
            InputFieldMessage::WordRight => {
                state.clear_selection();
                state.move_word_right();
                None
            }
            InputFieldMessage::SelectLeft => {
                state.ensure_selection_anchor();
                state.move_left();
                None
            }
            InputFieldMessage::SelectRight => {
                state.ensure_selection_anchor();
                state.move_right();
                None
            }
            InputFieldMessage::SelectHome => {
                state.ensure_selection_anchor();
                state.cursor = 0;
                None
            }
            InputFieldMessage::SelectEnd => {
                state.ensure_selection_anchor();
                state.cursor = state.value.len();
                None
            }
            InputFieldMessage::SelectWordLeft => {
                state.ensure_selection_anchor();
                state.move_word_left();
                None
            }
            InputFieldMessage::SelectWordRight => {
                state.ensure_selection_anchor();
                state.move_word_right();
                None
            }
            InputFieldMessage::SelectAll => {
                if state.value.is_empty() {
                    return None;
                }
                state.selection_anchor = Some(0);
                state.cursor = state.value.len();
                None
            }
            InputFieldMessage::Copy => {
                if let Some(text) = state.selected_text() {
                    let text = text.to_string();
                    state.clipboard = text.clone();
                    #[cfg(feature = "clipboard")]
                    system_clipboard_set(&text);
                    Some(InputFieldOutput::Copied(text))
                } else {
                    None
                }
            }
            InputFieldMessage::Cut => {
                if let Some(text) = state.selected_text() {
                    let text = text.to_string();
                    let snapshot = state.snapshot();
                    state.clipboard = text.clone();
                    #[cfg(feature = "clipboard")]
                    system_clipboard_set(&text);
                    state.delete_selection();
                    state.undo_stack.save(snapshot, EditKind::Other);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Paste(text) => {
                if text.is_empty() {
                    return None;
                }
                let snapshot = state.snapshot();
                state.undo_stack.save(snapshot, EditKind::Other);
                state.delete_selection();
                // Insert each character at cursor position
                for c in text.chars() {
                    state.insert(c);
                }
                Some(InputFieldOutput::Changed(state.value.clone()))
            }
            InputFieldMessage::DeleteWordBack => {
                let snapshot = state.snapshot();
                if state.has_selection() {
                    state.delete_selection();
                    state.undo_stack.save(snapshot, EditKind::Other);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else if state.delete_word_back() {
                    state.undo_stack.save(snapshot, EditKind::Other);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::DeleteWordForward => {
                let snapshot = state.snapshot();
                if state.has_selection() {
                    state.delete_selection();
                    state.undo_stack.save(snapshot, EditKind::Other);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else if state.delete_word_forward() {
                    state.undo_stack.save(snapshot, EditKind::Other);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Clear => {
                state.clear_selection();
                if !state.value.is_empty() {
                    let snapshot = state.snapshot();
                    state.undo_stack.save(snapshot, EditKind::Other);
                    state.value.clear();
                    state.cursor = 0;
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::SetValue(value) => {
                if state.value != value {
                    let snapshot = state.snapshot();
                    state.undo_stack.save(snapshot, EditKind::Other);
                    state.set_value(value);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Submit => Some(InputFieldOutput::Submitted(state.value.clone())),
            InputFieldMessage::Undo => {
                let snapshot = state.snapshot();
                if let Some(restored) = state.undo_stack.undo(snapshot) {
                    state.restore(restored);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
            InputFieldMessage::Redo => {
                let snapshot = state.snapshot();
                if let Some(restored) = state.undo_stack.redo(snapshot) {
                    state.restore(restored);
                    Some(InputFieldOutput::Changed(state.value.clone()))
                } else {
                    None
                }
            }
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        // Handle paste events from terminal (bracketed paste)
        if let Event::Paste(ref text) = event {
            return Some(InputFieldMessage::Paste(text.clone()));
        }

        if let Some(key) = event.as_key() {
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);
            match key.code {
                // Undo/redo
                KeyCode::Char('z') if ctrl => Some(InputFieldMessage::Undo),
                KeyCode::Char('y') if ctrl => Some(InputFieldMessage::Redo),
                // Clipboard operations
                KeyCode::Char('c') if ctrl => Some(InputFieldMessage::Copy),
                KeyCode::Char('x') if ctrl => Some(InputFieldMessage::Cut),
                KeyCode::Char('v') if ctrl => {
                    // Try system clipboard first, fall back to internal
                    #[cfg(feature = "clipboard")]
                    if let Some(text) = system_clipboard_get() {
                        return Some(InputFieldMessage::Paste(text));
                    }
                    if state.clipboard.is_empty() {
                        None
                    } else {
                        Some(InputFieldMessage::Paste(state.clipboard.clone()))
                    }
                }
                KeyCode::Char('a') if ctrl => Some(InputFieldMessage::SelectAll),
                // Character input (only when no ctrl modifier)
                KeyCode::Char(c) if !ctrl => Some(InputFieldMessage::Insert(c)),
                // Selection movement (shift+key)
                KeyCode::Left if ctrl && shift => Some(InputFieldMessage::SelectWordLeft),
                KeyCode::Right if ctrl && shift => Some(InputFieldMessage::SelectWordRight),
                KeyCode::Left if shift => Some(InputFieldMessage::SelectLeft),
                KeyCode::Right if shift => Some(InputFieldMessage::SelectRight),
                KeyCode::Home if shift => Some(InputFieldMessage::SelectHome),
                KeyCode::End if shift => Some(InputFieldMessage::SelectEnd),
                // Deletion
                KeyCode::Backspace if ctrl => Some(InputFieldMessage::DeleteWordBack),
                KeyCode::Delete if ctrl => Some(InputFieldMessage::DeleteWordForward),
                KeyCode::Backspace => Some(InputFieldMessage::Backspace),
                KeyCode::Delete => Some(InputFieldMessage::Delete),
                // Navigation (clears selection)
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
        let border_style = if state.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        let is_placeholder = state.value.is_empty() && !state.placeholder.is_empty();

        let base_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_style()
        } else if is_placeholder {
            theme.placeholder_style()
        } else {
            theme.normal_style()
        };

        let text = if is_placeholder {
            &state.placeholder
        } else {
            &state.value
        };

        // Build line with selection highlighting
        let line = if let Some((sel_start, sel_end)) = state.selection_range() {
            let selection_style = theme.selection_style();
            let before = &state.value[..sel_start];
            let selected = &state.value[sel_start..sel_end];
            let after = &state.value[sel_end..];
            Line::from(vec![
                Span::styled(before.to_string(), base_style),
                Span::styled(selected.to_string(), selection_style),
                Span::styled(after.to_string(), base_style),
            ])
        } else {
            Line::from(Span::styled(text.to_string(), base_style))
        };

        let paragraph = Paragraph::new(line).block(block);
        frame.render_widget(paragraph, area);

        // Show cursor when focused
        if state.focused && area.width > 2 && area.height > 2 {
            let cursor_x = area.x + 1 + state.cursor_display_position() as u16;
            let cursor_y = area.y + 1;

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
#[cfg(test)]
mod undo_tests;
