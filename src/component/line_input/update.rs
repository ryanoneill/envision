//! Message handling for [`LineInput`].
//!
//! This module contains the [`update`] function that processes [`LineInputMessage`]
//! values and mutates [`LineInputState`] accordingly.

use crate::undo::{EditKind, UndoStack};

use super::LineInputState;
use super::chunking::{self, cursor_to_visual};
use super::editing;
use super::types::{LineInputMessage, LineInputOutput};

/// Processes a [`LineInputMessage`] and mutates the given state.
///
/// Returns `Some(output)` when the message produces observable output
/// (e.g., a value change or a submission), and `None` otherwise.
pub(super) fn update(state: &mut LineInputState, msg: LineInputMessage) -> Option<LineInputOutput> {
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

        LineInputMessage::DeleteToEnd => {
            if state.cursor >= state.buffer.len() {
                return None;
            }
            let snapshot = state.snapshot();
            state.undo_stack.save(snapshot, EditKind::Delete);
            state.buffer.truncate(state.cursor);
            state.clear_selection();
            state.history.exit_browse();
            Some(LineInputOutput::Changed(state.buffer.clone()))
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
            let (new_buffer, new_cursor) = editing::insert_str(&state.buffer, state.cursor, &text);
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
