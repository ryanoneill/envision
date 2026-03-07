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

mod chunking;
mod editing;
mod history;

#[cfg(test)]
mod handle_event_tests;
#[cfg(test)]
mod property_tests;
#[cfg(test)]
mod tests;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::component::{Component, Focusable};
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
    /// Whether this component is focused.
    focused: bool,
    /// Whether this component is disabled.
    disabled: bool,
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
            focused: false,
            disabled: false,
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new LineInput state with the given initial value.
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
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the maximum number of history entries (builder pattern).
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.history = History::new(max);
        self
    }

    /// Sets the disabled state (builder pattern).
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the maximum character count (builder pattern).
    ///
    /// When set, insertions and pastes that would exceed this limit are
    /// rejected or truncated. `None` means unlimited.
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    // --- Accessors ---

    /// Returns the current buffer value.
    pub fn value(&self) -> &str {
        &self.buffer
    }

    /// Sets the buffer value, moving the cursor to the end.
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.buffer = value.into();
        self.cursor = self.buffer.len();
        self.clear_selection();
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the number of characters in the buffer.
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

    /// Returns the maximum character length, or `None` if unlimited.
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

    /// Returns true if this component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
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

    /// Returns true if this component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    // --- Instance methods ---

    /// Maps an input event to a message (instance method).
    pub fn handle_event(&self, event: &Event) -> Option<LineInputMessage> {
        LineInput::handle_event(self, event)
    }

    /// Dispatches an event by mapping and updating (instance method).
    pub fn dispatch_event(&mut self, event: &Event) -> Option<LineInputOutput> {
        LineInput::dispatch_event(self, event)
    }

    /// Updates state with a message (instance method).
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

/// Messages for the [`LineInput`] component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineInputMessage {
    // Editing
    /// Insert a character at the cursor.
    Insert(char),
    /// Delete the character before the cursor.
    Backspace,
    /// Delete the character at the cursor.
    Delete,
    /// Delete the word before the cursor.
    DeleteWordBack,
    /// Delete the word after the cursor.
    DeleteWordForward,
    /// Clear the buffer.
    Clear,
    /// Set the buffer to the given value.
    SetValue(String),
    /// Paste text at the cursor (newlines are stripped).
    Paste(String),

    // Movement
    /// Move cursor one character left.
    Left,
    /// Move cursor one character right.
    Right,
    /// Move cursor to the start of the buffer.
    Home,
    /// Move cursor to the end of the buffer.
    End,
    /// Move cursor to the start of the previous word.
    WordLeft,
    /// Move cursor to the start of the next word.
    WordRight,
    /// Move cursor one visual row up.
    VisualUp,
    /// Move cursor one visual row down.
    VisualDown,

    // Selection
    /// Extend selection one character left.
    SelectLeft,
    /// Extend selection one character right.
    SelectRight,
    /// Extend selection to the start of the buffer.
    SelectHome,
    /// Extend selection to the end of the buffer.
    SelectEnd,
    /// Extend selection to the start of the previous word.
    SelectWordLeft,
    /// Extend selection to the start of the next word.
    SelectWordRight,
    /// Select the entire buffer.
    SelectAll,

    // Clipboard
    /// Copy selected text to internal clipboard.
    Copy,
    /// Cut selected text to internal clipboard.
    Cut,

    // History
    /// Move to the previous (older) history entry.
    HistoryPrev,
    /// Move to the next (newer) history entry.
    HistoryNext,

    // Actions
    /// Submit the current buffer contents.
    Submit,
    /// Undo the last edit.
    Undo,
    /// Redo the last undone edit.
    Redo,
}

/// Output events from the [`LineInput`] component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineInputOutput {
    /// The buffer was submitted (Enter pressed). Contains the submitted text.
    /// The buffer is cleared and the text is pushed to history.
    Submitted(String),
    /// The buffer value changed.
    Changed(String),
    /// Text was copied to the internal clipboard.
    Copied(String),
}

impl Component for LineInput {
    type State = LineInputState;
    type Message = LineInputMessage;
    type Output = LineInputOutput;

    fn init() -> Self::State {
        LineInputState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::line_input("line_input")
                    .with_value(state.value())
                    .with_focus(state.focused)
                    .with_disabled(state.disabled),
            );
        });

        let border_style = if state.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        // Inner area (inside borders)
        let inner = block.inner(area);
        if inner.width == 0 || inner.height == 0 {
            frame.render_widget(block, area);
            return;
        }

        let width = inner.width as usize;
        let is_placeholder = state.buffer.is_empty();

        let base_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_style()
        } else if is_placeholder {
            theme.placeholder_style()
        } else {
            theme.normal_style()
        };

        let display_text = if is_placeholder {
            &state.placeholder
        } else {
            &state.buffer
        };

        // Build visual lines from chunks
        let chunks = chunk_buffer(display_text, width);
        let selection_range = if !is_placeholder {
            state.selection_range()
        } else {
            None
        };

        let mut lines: Vec<Line> = Vec::with_capacity(chunks.len());
        for chunk in &chunks {
            let chunk_text = &display_text[chunk.clone()];
            if let Some((sel_start, sel_end)) = selection_range {
                // Compute overlap between chunk range and selection range
                let overlap_start = sel_start.max(chunk.start);
                let overlap_end = sel_end.min(chunk.end);
                if overlap_start < overlap_end {
                    // There is selected text in this chunk
                    let before = &display_text[chunk.start..overlap_start];
                    let selected = &display_text[overlap_start..overlap_end];
                    let after = &display_text[overlap_end..chunk.end];
                    let mut spans = Vec::new();
                    if !before.is_empty() {
                        spans.push(Span::styled(before.to_string(), base_style));
                    }
                    spans.push(Span::styled(selected.to_string(), theme.selection_style()));
                    if !after.is_empty() {
                        spans.push(Span::styled(after.to_string(), base_style));
                    }
                    lines.push(Line::from(spans));
                } else {
                    lines.push(Line::styled(chunk_text.to_string(), base_style));
                }
            } else {
                lines.push(Line::styled(chunk_text.to_string(), base_style));
            }
        }

        let paragraph = Paragraph::new(Text::from(lines)).block(block);
        frame.render_widget(paragraph, area);

        // Set cursor position when focused
        if state.focused && !state.disabled && inner.width > 0 && inner.height > 0 {
            let (cursor_row, cursor_col) = cursor_to_visual(&state.buffer, state.cursor, width);

            let cursor_x = inner.x + cursor_col as u16;
            let cursor_y = inner.y + cursor_row as u16;

            if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
}

impl Focusable for LineInput {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}
