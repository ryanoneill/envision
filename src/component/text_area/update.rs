/// Update logic for TextAreaState.
///
/// The `Component::update` match arms are extracted here to keep
/// the main module under the 1000-line limit.
use crate::undo::EditKind;

use super::{TextAreaMessage, TextAreaOutput, TextAreaState};

#[cfg(feature = "clipboard")]
use super::system_clipboard_set;

impl TextAreaState {
    /// Applies a message to the textarea state, returning any output.
    ///
    /// This is the core update logic, extracted from the `Component` impl
    /// for file size management.
    pub(super) fn apply_update(&mut self, msg: TextAreaMessage) -> Option<TextAreaOutput> {
        if self.disabled {
            return None;
        }

        match msg {
            // Editing (replaces selection if active)
            TextAreaMessage::Insert(c) => {
                if c.is_whitespace() {
                    self.undo_stack.break_group();
                }
                let snapshot = self.snapshot();
                self.undo_stack.save(snapshot, EditKind::Insert);
                self.delete_selection();
                self.insert(c);
                if c.is_whitespace() {
                    self.undo_stack.break_group();
                }
                Some(TextAreaOutput::Changed(self.value()))
            }
            TextAreaMessage::NewLine => {
                let snapshot = self.snapshot();
                self.undo_stack.save(snapshot, EditKind::Other);
                self.delete_selection();
                self.new_line();
                Some(TextAreaOutput::Changed(self.value()))
            }
            TextAreaMessage::Backspace => {
                let snapshot = self.snapshot();
                if self.has_selection() {
                    self.delete_selection();
                    self.undo_stack.save(snapshot, EditKind::Delete);
                    Some(TextAreaOutput::Changed(self.value()))
                } else if self.backspace() {
                    self.undo_stack.save(snapshot, EditKind::Delete);
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::Delete => {
                let snapshot = self.snapshot();
                if self.has_selection() {
                    self.delete_selection();
                    self.undo_stack.save(snapshot, EditKind::Delete);
                    Some(TextAreaOutput::Changed(self.value()))
                } else if self.delete() {
                    self.undo_stack.save(snapshot, EditKind::Delete);
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
            // Navigation (clears selection)
            TextAreaMessage::Left => {
                if self.has_selection() {
                    if let Some((start, _)) = self.selection_positions() {
                        self.cursor_row = start.0;
                        self.cursor_col = start.1;
                    }
                    self.clear_selection();
                } else {
                    self.move_left();
                }
                None
            }
            TextAreaMessage::Right => {
                if self.has_selection() {
                    if let Some((_, end)) = self.selection_positions() {
                        self.cursor_row = end.0;
                        self.cursor_col = end.1;
                    }
                    self.clear_selection();
                } else {
                    self.move_right();
                }
                None
            }
            TextAreaMessage::Up => {
                self.clear_selection();
                self.move_up();
                None
            }
            TextAreaMessage::Down => {
                self.clear_selection();
                self.move_down();
                None
            }
            TextAreaMessage::Home => {
                self.clear_selection();
                self.cursor_col = 0;
                None
            }
            TextAreaMessage::End => {
                self.clear_selection();
                self.cursor_col = self.lines[self.cursor_row].len();
                None
            }
            TextAreaMessage::TextStart => {
                self.clear_selection();
                self.cursor_row = 0;
                self.cursor_col = 0;
                None
            }
            TextAreaMessage::TextEnd => {
                self.clear_selection();
                self.cursor_row = self.lines.len() - 1;
                self.cursor_col = self.lines[self.cursor_row].len();
                None
            }
            TextAreaMessage::WordLeft => {
                self.clear_selection();
                self.move_word_left();
                None
            }
            TextAreaMessage::WordRight => {
                self.clear_selection();
                self.move_word_right();
                None
            }
            // Selection movement
            TextAreaMessage::SelectLeft => {
                self.ensure_selection_anchor();
                self.move_left();
                None
            }
            TextAreaMessage::SelectRight => {
                self.ensure_selection_anchor();
                self.move_right();
                None
            }
            TextAreaMessage::SelectUp => {
                self.ensure_selection_anchor();
                self.move_up();
                None
            }
            TextAreaMessage::SelectDown => {
                self.ensure_selection_anchor();
                self.move_down();
                None
            }
            TextAreaMessage::SelectHome => {
                self.ensure_selection_anchor();
                self.cursor_col = 0;
                None
            }
            TextAreaMessage::SelectEnd => {
                self.ensure_selection_anchor();
                self.cursor_col = self.lines[self.cursor_row].len();
                None
            }
            TextAreaMessage::SelectWordLeft => {
                self.ensure_selection_anchor();
                self.move_word_left();
                None
            }
            TextAreaMessage::SelectWordRight => {
                self.ensure_selection_anchor();
                self.move_word_right();
                None
            }
            TextAreaMessage::SelectAll => {
                if self.is_empty() {
                    return None;
                }
                self.selection_anchor = Some((0, 0));
                let last = self.lines.len() - 1;
                self.cursor_row = last;
                self.cursor_col = self.lines[last].len();
                None
            }
            // Clipboard
            TextAreaMessage::Copy => {
                if let Some(text) = self.selected_text() {
                    self.clipboard = text.clone();
                    #[cfg(feature = "clipboard")]
                    system_clipboard_set(&text);
                    Some(TextAreaOutput::Copied(text))
                } else {
                    None
                }
            }
            TextAreaMessage::Cut => {
                if let Some(text) = self.selected_text() {
                    let snapshot = self.snapshot();
                    self.clipboard = text.clone();
                    #[cfg(feature = "clipboard")]
                    system_clipboard_set(&text);
                    self.delete_selection();
                    self.undo_stack.save(snapshot, EditKind::Other);
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::Paste(text) => {
                if text.is_empty() {
                    return None;
                }
                let snapshot = self.snapshot();
                self.undo_stack.save(snapshot, EditKind::Other);
                self.delete_selection();
                for c in text.chars() {
                    if c == '\n' {
                        self.new_line();
                    } else {
                        self.insert(c);
                    }
                }
                Some(TextAreaOutput::Changed(self.value()))
            }
            // Line operations
            TextAreaMessage::DeleteLine => {
                self.clear_selection();
                let snapshot = self.snapshot();
                if self.delete_line() {
                    self.undo_stack.save(snapshot, EditKind::Other);
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::DeleteToEnd => {
                let snapshot = self.snapshot();
                if self.has_selection() {
                    self.delete_selection();
                    self.undo_stack.save(snapshot, EditKind::Other);
                    Some(TextAreaOutput::Changed(self.value()))
                } else if self.delete_to_end() {
                    self.undo_stack.save(snapshot, EditKind::Other);
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::DeleteToStart => {
                let snapshot = self.snapshot();
                if self.has_selection() {
                    self.delete_selection();
                    self.undo_stack.save(snapshot, EditKind::Other);
                    Some(TextAreaOutput::Changed(self.value()))
                } else if self.delete_to_start() {
                    self.undo_stack.save(snapshot, EditKind::Other);
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::Clear => {
                self.clear_selection();
                if !self.is_empty() {
                    let snapshot = self.snapshot();
                    self.undo_stack.save(snapshot, EditKind::Other);
                    self.lines = vec![String::new()];
                    self.cursor_row = 0;
                    self.cursor_col = 0;
                    self.scroll_offset = 0;
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::SetValue(value) => {
                let old_value = self.value();
                if old_value != value {
                    let snapshot = self.snapshot();
                    self.undo_stack.save(snapshot, EditKind::Other);
                    self.set_value(value);
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::Submit => Some(TextAreaOutput::Submitted(self.value())),
            TextAreaMessage::Undo => {
                let snapshot = self.snapshot();
                if let Some(restored) = self.undo_stack.undo(snapshot) {
                    self.restore(restored);
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
            TextAreaMessage::Redo => {
                let snapshot = self.snapshot();
                if let Some(restored) = self.undo_stack.redo(snapshot) {
                    self.restore(restored);
                    Some(TextAreaOutput::Changed(self.value()))
                } else {
                    None
                }
            }
        }
    }
}
