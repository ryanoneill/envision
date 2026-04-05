//! Single-line text input with visual wrapping across multiple rows.
//!
//! [`LineInput`] provides a single-buffer text input that wraps visually
//! across multiple rows by character-level chunking (not word-wrap). Ideal
//! for chat input boxes, command palettes, and other contexts where the user
//! types potentially long single-line input.
//!
//! # Features
//!
//! - Character-level visual wrapping with CJK/emoji support
//! - Shell-style history with Up/Down recall
//! - Undo/redo with edit grouping
//! - Selection, copy, cut, paste
//! - Word-level navigation and deletion
//!
//! State is stored in [`LineInputState`], updated via [`LineInputMessage`],
//! and produces [`LineInputOutput`].
//!
//!
//! See also [`InputField`](super::InputField) for a simpler single-line input,
//! and [`TextArea`](super::TextArea) for multi-line editing.

mod chunking;
mod editing;
mod history;
mod types;
mod view_helpers;

pub use types::{LineInputMessage, LineInputOutput};

#[cfg(test)]
mod handle_event_tests;
#[cfg(test)]
mod property_tests;
#[cfg(test)]
mod tests;

use ratatui::prelude::*;

use crate::component::{Component, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;
use crate::undo::{EditKind, UndoStack};

use chunking::{chunk_buffer, cursor_to_visual};
use history::History;

/// Snapshot of LineInput state for undo/redo.
#[derive(Debug, Clone)]
struct LineInputSnapshot {
    buffer: String,
    cursor: usize,
}

/// A single-line text input that wraps visually across multiple rows.
///
/// All text lives in a single `String` buffer with no embedded newlines.
/// The buffer is split into visual rows of at most `width` display columns
/// using character-level chunking. Wide characters (CJK, emoji) that don't
/// fit at the end of a row are bumped to the next row.
pub struct LineInput;

/// State for the [`LineInput`] component.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct LineInputState {
    /// The text buffer (single line, no newlines).
    buffer: String,
    /// Cursor position as a byte offset into `buffer`.
    cursor: usize,
    /// Placeholder text shown when the buffer is empty.
    placeholder: String,
    /// Selection anchor (byte offset); `None` if no selection.
    selection_anchor: Option<usize>,
    /// Internal clipboard.
    clipboard: String,
    /// Maximum number of characters allowed, or `None` for unlimited.
    max_length: Option<usize>,
    /// Command history.
    #[cfg_attr(feature = "serialization", serde(skip))]
    history: History,
    /// Undo/redo stack.
    #[cfg_attr(feature = "serialization", serde(skip))]
    undo_stack: UndoStack<LineInputSnapshot>,
    /// Last known display width from the parent layout.
    #[cfg_attr(feature = "serialization", serde(skip))]
    last_display_width: usize,
}

impl Default for LineInputState {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            placeholder: String::new(),
            selection_anchor: None,
            clipboard: String::new(),
            max_length: None,
            history: History::default(),
            undo_stack: UndoStack::default(),
            last_display_width: 80,
        }
    }
}

impl LineInputState {
    /// Creates a new empty LineInput state.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new();
    /// assert!(state.is_empty());
    /// assert_eq!(state.value(), "");
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new LineInput state with the given initial value.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::with_value("hello");
    /// assert_eq!(state.value(), "hello");
    /// assert_eq!(state.len(), 5);
    /// ```
    pub fn with_value(value: impl Into<String>) -> Self {
        let buffer: String = value.into();
        let cursor = buffer.len();
        Self {
            buffer,
            cursor,
            ..Self::default()
        }
    }

    /// Sets the placeholder text (builder pattern).
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new().with_placeholder("Type here...");
    /// assert_eq!(state.placeholder(), "Type here...");
    /// ```
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the maximum number of history entries (builder pattern).
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.history = History::new(max);
        self
    }

    /// Sets the maximum character count (builder pattern).
    ///
    /// When set, insertions and pastes that would exceed this limit are
    /// rejected or truncated. `None` means unlimited.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new().with_max_length(100);
    /// assert_eq!(state.max_length(), Some(100));
    /// ```
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    // --- Accessors ---

    /// Returns the current buffer value.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::with_value("hello world");
    /// assert_eq!(state.value(), "hello world");
    /// ```
    pub fn value(&self) -> &str {
        &self.buffer
    }

    /// Sets the buffer value, moving the cursor to the end.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let mut state = LineInputState::new();
    /// state.set_value("new text");
    /// assert_eq!(state.value(), "new text");
    /// ```
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.buffer = value.into();
        self.cursor = self.buffer.len();
        self.clear_selection();
    }

    /// Returns true if the buffer is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let empty = LineInputState::new();
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = LineInputState::with_value("x");
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the number of characters in the buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::with_value("café");
    /// assert_eq!(state.len(), 4); // chars, not bytes
    /// ```
    pub fn len(&self) -> usize {
        self.buffer.chars().count()
    }

    /// Returns the cursor byte offset.
    pub fn cursor_byte_offset(&self) -> usize {
        self.cursor
    }

    /// Returns the cursor as a (row, col) visual position.
    ///
    /// Uses the last known display width from the parent layout.
    pub fn cursor_visual_position(&self) -> (usize, usize) {
        cursor_to_visual(&self.buffer, self.cursor, self.last_display_width)
    }

    /// Sets the display width used for visual wrapping.
    ///
    /// The parent layout should call this before event dispatch so that
    /// Up/Down navigation and rendering use the correct width.
    pub fn set_display_width(&mut self, width: usize) {
        self.last_display_width = width;
    }

    /// Returns the current display width.
    pub fn display_width(&self) -> usize {
        self.last_display_width
    }

    /// Returns the number of visual rows the buffer would occupy at the
    /// given display width.
    ///
    /// This is useful for dynamic-height layouts where a parent container
    /// needs to allocate the correct number of rows for the input.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::with_value("Hello, world!");
    /// assert_eq!(state.visual_rows_at_width(20), 1);
    /// assert_eq!(state.visual_rows_at_width(7), 2);
    /// assert_eq!(state.visual_rows_at_width(5), 3);
    /// ```
    pub fn visual_rows_at_width(&self, width: usize) -> usize {
        chunk_buffer(&self.buffer, width).len()
    }

    /// Returns the maximum character length, or `None` if unlimited.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new();
    /// assert_eq!(state.max_length(), None);
    ///
    /// let capped = LineInputState::new().with_max_length(50);
    /// assert_eq!(capped.max_length(), Some(50));
    /// ```
    pub fn max_length(&self) -> Option<usize> {
        self.max_length
    }

    /// Sets the maximum character length. `None` means unlimited.
    ///
    /// Does not truncate existing content -- only constrains future edits.
    pub fn set_max_length(&mut self, max: Option<usize>) {
        self.max_length = max;
    }

    /// Returns the placeholder text.
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Sets the placeholder text.
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Returns the maximum number of history entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new().with_max_history(50);
    /// assert_eq!(state.max_history(), 50);
    /// ```
    pub fn max_history(&self) -> usize {
        self.history.max_entries()
    }

    /// Sets the maximum number of history entries.
    ///
    /// If the current count exceeds the new maximum, oldest entries are removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let mut state = LineInputState::new();
    /// state.set_max_history(50);
    /// assert_eq!(state.max_history(), 50);
    /// ```
    pub fn set_max_history(&mut self, max: usize) {
        self.history.set_max_entries(max);
    }

    /// Returns the history entries.
    pub fn history_entries(&self) -> &[String] {
        self.history.entries()
    }

    /// Returns the number of history entries.
    pub fn history_count(&self) -> usize {
        self.history.count()
    }

    /// Returns true if currently browsing history.
    pub fn is_browsing_history(&self) -> bool {
        self.history.is_browsing()
    }

    /// Returns true if there are entries to undo.
    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    /// Returns true if there are entries to redo.
    pub fn can_redo(&self) -> bool {
        self.undo_stack.can_redo()
    }

    /// Returns true if there is an active selection.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new();
    /// assert!(!state.has_selection());
    /// ```
    pub fn has_selection(&self) -> bool {
        self.selection_anchor.is_some()
    }

    /// Returns the selected byte range `(start, end)`, or `None`.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        let start = anchor.min(self.cursor);
        let end = anchor.max(self.cursor);
        if start == end {
            None
        } else {
            Some((start, end))
        }
    }

    /// Returns the selected text, or `None` if no selection.
    pub fn selected_text(&self) -> Option<&str> {
        let (start, end) = self.selection_range()?;
        Some(&self.buffer[start..end])
    }

    /// Returns the internal clipboard contents.
    pub fn clipboard(&self) -> &str {
        &self.clipboard
    }

    // --- Instance methods ---

    /// Updates state with a message (instance method).
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::{LineInputState, LineInputMessage, LineInputOutput};
    ///
    /// let mut state = LineInputState::new();
    /// state.update(LineInputMessage::Insert('h'));
    /// state.update(LineInputMessage::Insert('i'));
    /// assert_eq!(state.value(), "hi");
    ///
    /// let output = state.update(LineInputMessage::Submit);
    /// assert_eq!(output, Some(LineInputOutput::Submitted("hi".into())));
    /// ```
    pub fn update(&mut self, msg: LineInputMessage) -> Option<LineInputOutput> {
        LineInput::update(self, msg)
    }

    // --- Private helpers ---

    /// Returns the number of characters that can still be inserted before reaching max_length.
    fn remaining_capacity(&self) -> usize {
        match self.max_length {
            Some(max) => max.saturating_sub(self.buffer.chars().count()),
            None => usize::MAX,
        }
    }

    fn snapshot(&self) -> LineInputSnapshot {
        LineInputSnapshot {
            buffer: self.buffer.clone(),
            cursor: self.cursor,
        }
    }

    fn restore(&mut self, snapshot: LineInputSnapshot) {
        self.buffer = snapshot.buffer;
        self.cursor = snapshot.cursor;
        self.clear_selection();
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn ensure_selection_anchor(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
    }

    /// Deletes the selected text, returning it. Clears the selection.
    fn delete_selection(&mut self) -> Option<String> {
        let (start, end) = self.selection_range()?;
        let deleted = self.buffer[start..end].to_string();
        let mut new_buffer = String::with_capacity(self.buffer.len() - (end - start));
        new_buffer.push_str(&self.buffer[..start]);
        new_buffer.push_str(&self.buffer[end..]);
        self.buffer = new_buffer;
        self.cursor = start;
        self.clear_selection();
        Some(deleted)
    }
}

impl Component for LineInput {
    type State = LineInputState;
    type Message = LineInputMessage;
    type Output = LineInputOutput;

    fn init() -> Self::State {
        LineInputState::default()
    }

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        // Handle paste events
        if let Event::Paste(ref text) = event {
            return Some(LineInputMessage::Paste(text.clone()));
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            // Undo/Redo
            KeyCode::Char('z') if ctrl => Some(LineInputMessage::Undo),
            KeyCode::Char('y') if ctrl => Some(LineInputMessage::Redo),

            // Clipboard
            KeyCode::Char('c') if ctrl => Some(LineInputMessage::Copy),
            KeyCode::Char('x') if ctrl => Some(LineInputMessage::Cut),
            KeyCode::Char('v') if ctrl => {
                // Use internal clipboard
                if !state.clipboard.is_empty() {
                    Some(LineInputMessage::Paste(state.clipboard.clone()))
                } else {
                    None
                }
            }

            // Select all
            KeyCode::Char('a') if ctrl => Some(LineInputMessage::SelectAll),

            // Clear
            KeyCode::Char('u') if ctrl => Some(LineInputMessage::Clear),

            // Regular character insertion
            KeyCode::Char(c) if !ctrl => Some(LineInputMessage::Insert(c)),

            // Selection with shift
            KeyCode::Left if ctrl && shift => Some(LineInputMessage::SelectWordLeft),
            KeyCode::Right if ctrl && shift => Some(LineInputMessage::SelectWordRight),
            KeyCode::Left if shift => Some(LineInputMessage::SelectLeft),
            KeyCode::Right if shift => Some(LineInputMessage::SelectRight),
            KeyCode::Home if shift => Some(LineInputMessage::SelectHome),
            KeyCode::End if shift => Some(LineInputMessage::SelectEnd),

            // Word deletion
            KeyCode::Backspace if ctrl => Some(LineInputMessage::DeleteWordBack),
            KeyCode::Delete if ctrl => Some(LineInputMessage::DeleteWordForward),

            // Character deletion
            KeyCode::Backspace => Some(LineInputMessage::Backspace),
            KeyCode::Delete => Some(LineInputMessage::Delete),

            // Word navigation
            KeyCode::Left if ctrl => Some(LineInputMessage::WordLeft),
            KeyCode::Right if ctrl => Some(LineInputMessage::WordRight),

            // Character navigation
            KeyCode::Left => Some(LineInputMessage::Left),
            KeyCode::Right => Some(LineInputMessage::Right),
            KeyCode::Home => Some(LineInputMessage::Home),
            KeyCode::End => Some(LineInputMessage::End),

            // Up/Down: context-dependent
            KeyCode::Up => {
                let (row, _) =
                    cursor_to_visual(&state.buffer, state.cursor, state.last_display_width);
                if row == 0 {
                    Some(LineInputMessage::HistoryPrev)
                } else {
                    Some(LineInputMessage::VisualUp)
                }
            }
            KeyCode::Down => {
                let (row, _) =
                    cursor_to_visual(&state.buffer, state.cursor, state.last_display_width);
                let chunks = chunk_buffer(&state.buffer, state.last_display_width);
                let last_row = chunks.len().saturating_sub(1);
                // Also handle phantom trailing row
                let buffer_on_phantom = row >= chunks.len();
                if row >= last_row || buffer_on_phantom {
                    Some(LineInputMessage::HistoryNext)
                } else {
                    Some(LineInputMessage::VisualDown)
                }
            }

            // Submit
            KeyCode::Enter => Some(LineInputMessage::Submit),

            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            LineInputMessage::Insert(c) => {
                // Check max_length before inserting.
                // If there is a selection, account for the characters that will be removed.
                let selection_chars = state
                    .selection_range()
                    .map(|(s, e)| state.buffer[s..e].chars().count())
                    .unwrap_or(0);
                let effective_len = state.buffer.chars().count() - selection_chars;
                if let Some(max) = state.max_length {
                    if effective_len >= max {
                        return None;
                    }
                }
                if c.is_whitespace() {
                    state.undo_stack.break_group();
                }
                let snapshot = state.snapshot();
                state.undo_stack.save(snapshot, EditKind::Insert);
                state.delete_selection();
                let (new_buffer, new_cursor) = editing::insert_char(&state.buffer, state.cursor, c);
                state.buffer = new_buffer;
                state.cursor = new_cursor;
                if c.is_whitespace() {
                    state.undo_stack.break_group();
                }
                state.history.exit_browse();
                Some(LineInputOutput::Changed(state.buffer.clone()))
            }

            LineInputMessage::Backspace => {
                let snapshot = state.snapshot();
                if state.has_selection() {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    state.delete_selection();
                    state.history.exit_browse();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else if let Some((new_buffer, new_cursor)) =
                    editing::backspace(&state.buffer, state.cursor)
                {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    state.buffer = new_buffer;
                    state.cursor = new_cursor;
                    state.history.exit_browse();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else {
                    None
                }
            }

            LineInputMessage::Delete => {
                let snapshot = state.snapshot();
                if state.has_selection() {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    state.delete_selection();
                    state.history.exit_browse();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else if let Some(new_buffer) = editing::delete_at(&state.buffer, state.cursor) {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    state.buffer = new_buffer;
                    state.history.exit_browse();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else {
                    None
                }
            }

            LineInputMessage::DeleteWordBack => {
                let snapshot = state.snapshot();
                if state.has_selection() {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    state.delete_selection();
                    state.history.exit_browse();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else if let Some((new_buffer, new_cursor)) =
                    editing::delete_word_back(&state.buffer, state.cursor)
                {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    state.buffer = new_buffer;
                    state.cursor = new_cursor;
                    state.history.exit_browse();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else {
                    None
                }
            }

            LineInputMessage::DeleteWordForward => {
                let snapshot = state.snapshot();
                if state.has_selection() {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    state.delete_selection();
                    state.history.exit_browse();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else if let Some(new_buffer) =
                    editing::delete_word_forward(&state.buffer, state.cursor)
                {
                    state.undo_stack.save(snapshot, EditKind::Delete);
                    state.buffer = new_buffer;
                    state.history.exit_browse();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else {
                    None
                }
            }

            LineInputMessage::Clear => {
                if state.buffer.is_empty() {
                    return None;
                }
                let snapshot = state.snapshot();
                state.undo_stack.save(snapshot, EditKind::Other);
                state.buffer.clear();
                state.cursor = 0;
                state.clear_selection();
                state.history.exit_browse();
                Some(LineInputOutput::Changed(state.buffer.clone()))
            }

            LineInputMessage::SetValue(value) => {
                let snapshot = state.snapshot();
                state.undo_stack.save(snapshot, EditKind::Other);
                // Truncate value to max_length
                let value = if let Some(max) = state.max_length {
                    if value.chars().count() > max {
                        value.chars().take(max).collect()
                    } else {
                        value
                    }
                } else {
                    value
                };
                state.buffer = value;
                state.cursor = state.buffer.len();
                state.clear_selection();
                state.history.exit_browse();
                Some(LineInputOutput::Changed(state.buffer.clone()))
            }

            LineInputMessage::Paste(text) => {
                let snapshot = state.snapshot();
                state.undo_stack.save(snapshot, EditKind::Other);
                state.delete_selection();
                // Truncate paste content to remaining capacity
                let remaining = state.remaining_capacity();
                if remaining == 0 {
                    return None;
                }
                let text = if remaining < usize::MAX {
                    text.chars().take(remaining).collect()
                } else {
                    text
                };
                let (new_buffer, new_cursor) =
                    editing::insert_str(&state.buffer, state.cursor, &text);
                state.buffer = new_buffer;
                state.cursor = new_cursor;
                state.history.exit_browse();
                Some(LineInputOutput::Changed(state.buffer.clone()))
            }

            // Movement — no output
            LineInputMessage::Left => {
                state.clear_selection();
                state.cursor = editing::move_left(&state.buffer, state.cursor);
                None
            }

            LineInputMessage::Right => {
                state.clear_selection();
                state.cursor = editing::move_right(&state.buffer, state.cursor);
                None
            }

            LineInputMessage::Home => {
                state.clear_selection();
                state.cursor = 0;
                None
            }

            LineInputMessage::End => {
                state.clear_selection();
                state.cursor = state.buffer.len();
                None
            }

            LineInputMessage::WordLeft => {
                state.clear_selection();
                state.cursor = editing::move_word_left(&state.buffer, state.cursor);
                None
            }

            LineInputMessage::WordRight => {
                state.clear_selection();
                state.cursor = editing::move_word_right(&state.buffer, state.cursor);
                None
            }

            LineInputMessage::VisualUp => {
                state.clear_selection();
                let width = state.last_display_width;
                let (row, col) = cursor_to_visual(&state.buffer, state.cursor, width);
                if row > 0 {
                    state.cursor = chunking::visual_to_cursor(&state.buffer, row - 1, col, width);
                }
                None
            }

            LineInputMessage::VisualDown => {
                state.clear_selection();
                let width = state.last_display_width;
                let (row, col) = cursor_to_visual(&state.buffer, state.cursor, width);
                state.cursor = chunking::visual_to_cursor(&state.buffer, row + 1, col, width);
                None
            }

            // Selection — no output
            LineInputMessage::SelectLeft => {
                state.ensure_selection_anchor();
                state.cursor = editing::move_left(&state.buffer, state.cursor);
                None
            }

            LineInputMessage::SelectRight => {
                state.ensure_selection_anchor();
                state.cursor = editing::move_right(&state.buffer, state.cursor);
                None
            }

            LineInputMessage::SelectHome => {
                state.ensure_selection_anchor();
                state.cursor = 0;
                None
            }

            LineInputMessage::SelectEnd => {
                state.ensure_selection_anchor();
                state.cursor = state.buffer.len();
                None
            }

            LineInputMessage::SelectWordLeft => {
                state.ensure_selection_anchor();
                state.cursor = editing::move_word_left(&state.buffer, state.cursor);
                None
            }

            LineInputMessage::SelectWordRight => {
                state.ensure_selection_anchor();
                state.cursor = editing::move_word_right(&state.buffer, state.cursor);
                None
            }

            LineInputMessage::SelectAll => {
                if state.buffer.is_empty() {
                    return None;
                }
                state.selection_anchor = Some(0);
                state.cursor = state.buffer.len();
                None
            }

            // Clipboard
            LineInputMessage::Copy => {
                if let Some(text) = state.selected_text() {
                    let text = text.to_string();
                    state.clipboard = text.clone();
                    Some(LineInputOutput::Copied(text))
                } else {
                    None
                }
            }

            LineInputMessage::Cut => {
                if let Some(text) = state.selected_text() {
                    let text = text.to_string();
                    state.clipboard = text.clone();
                    let snapshot = state.snapshot();
                    state.undo_stack.save(snapshot, EditKind::Other);
                    state.delete_selection();
                    state.history.exit_browse();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else {
                    None
                }
            }

            // History
            LineInputMessage::HistoryPrev => {
                if let Some(entry) = state.history.prev(&state.buffer) {
                    state.buffer = entry.to_string();
                    state.cursor = state.buffer.len();
                    state.clear_selection();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else {
                    None
                }
            }

            LineInputMessage::HistoryNext => {
                if let Some(entry) = state.history.next() {
                    state.buffer = entry;
                    state.cursor = state.buffer.len();
                    state.clear_selection();
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else {
                    None
                }
            }

            // Submit
            LineInputMessage::Submit => {
                let value = state.buffer.clone();
                state.history.push(value.clone());
                state.buffer.clear();
                state.cursor = 0;
                state.clear_selection();
                state.undo_stack = UndoStack::default();
                Some(LineInputOutput::Submitted(value))
            }

            // Undo/Redo
            LineInputMessage::Undo => {
                let current = state.snapshot();
                if let Some(restored) = state.undo_stack.undo(current) {
                    state.restore(restored);
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else {
                    None
                }
            }

            LineInputMessage::Redo => {
                let current = state.snapshot();
                if let Some(restored) = state.undo_stack.redo(current) {
                    state.restore(restored);
                    Some(LineInputOutput::Changed(state.buffer.clone()))
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, _ctx: &ViewContext) {
        view_helpers::render(state, frame, area, theme);
    }
}
