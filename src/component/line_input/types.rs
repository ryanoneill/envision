//! Types for the line input component.

/// Messages for the [`super::LineInput`] component.
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
    /// Delete from the cursor to the end of the buffer.
    DeleteToEnd,
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

/// Output events from the [`super::LineInput`] component.
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
