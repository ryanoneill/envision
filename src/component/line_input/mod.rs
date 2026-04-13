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
mod update;
mod view_helpers;

pub use types::{LineInputMessage, LineInputOutput};

#[cfg(test)]
mod handle_event_tests;
#[cfg(test)]
mod property_tests;
#[cfg(test)]
mod tests;

use crate::component::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::undo::UndoStack;

/// Input keybinding mode for [`LineInput`].
///
/// Controls how certain Ctrl-key combinations are interpreted.
///
/// - **Desktop** (default): Ctrl-A = Select All, Ctrl-C/X/V = clipboard.
///   Standard desktop text editor bindings.
/// - **Readline**: Ctrl-A = Home, Ctrl-E = End, Ctrl-W = Delete Word Left,
///   Ctrl-K = Delete to End. Unix shell / readline-style bindings.
///   Ctrl-C/X/V clipboard operations are NOT available in this mode.
///
/// # Example
///
/// ```rust
/// use envision::component::line_input::InputMode;
/// use envision::component::LineInputState;
///
/// let state = LineInputState::new().with_input_mode(InputMode::Readline);
/// assert_eq!(state.input_mode(), &InputMode::Readline);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum InputMode {
    /// Standard desktop bindings (Ctrl-A = Select All, Ctrl-C/X/V = clipboard).
    Desktop,
    /// Unix readline bindings (Ctrl-A = Home, Ctrl-E = End, Ctrl-W = Delete Word Left).
    Readline,
}

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
    /// Keybinding mode (Desktop or Readline).
    input_mode: InputMode,
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
            input_mode: InputMode::Desktop,
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new().with_max_history(20);
    /// assert_eq!(state.max_history(), 20);
    /// ```
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

    /// Sets the input keybinding mode (builder pattern).
    ///
    /// See [`InputMode`] for the available modes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::line_input::InputMode;
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new().with_input_mode(InputMode::Readline);
    /// assert_eq!(state.input_mode(), &InputMode::Readline);
    /// ```
    pub fn with_input_mode(mut self, mode: InputMode) -> Self {
        self.input_mode = mode;
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::with_value("hello");
    /// assert_eq!(state.cursor_byte_offset(), 5);
    /// ```
    pub fn cursor_byte_offset(&self) -> usize {
        self.cursor
    }

    /// Returns the cursor as a (row, col) visual position.
    ///
    /// Uses the last known display width from the parent layout.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::with_value("hi");
    /// let (row, col) = state.cursor_visual_position();
    /// assert_eq!(row, 0);
    /// assert_eq!(col, 2);
    /// ```
    pub fn cursor_visual_position(&self) -> (usize, usize) {
        cursor_to_visual(&self.buffer, self.cursor, self.last_display_width)
    }

    /// Sets the display width used for visual wrapping.
    ///
    /// The parent layout should call this before event dispatch so that
    /// Up/Down navigation and rendering use the correct width.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let mut state = LineInputState::new();
    /// state.set_display_width(40);
    /// assert_eq!(state.display_width(), 40);
    /// ```
    pub fn set_display_width(&mut self, width: usize) {
        self.last_display_width = width;
    }

    /// Returns the current display width.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new();
    /// assert_eq!(state.display_width(), 80);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let mut state = LineInputState::new();
    /// state.set_max_length(Some(25));
    /// assert_eq!(state.max_length(), Some(25));
    /// ```
    pub fn set_max_length(&mut self, max: Option<usize>) {
        self.max_length = max;
    }

    /// Returns the placeholder text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new().with_placeholder("Search...");
    /// assert_eq!(state.placeholder(), "Search...");
    /// ```
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Sets the placeholder text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let mut state = LineInputState::new();
    /// state.set_placeholder("Enter your name");
    /// assert_eq!(state.placeholder(), "Enter your name");
    /// ```
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Returns the current input keybinding mode.
    pub fn input_mode(&self) -> &InputMode {
        &self.input_mode
    }

    /// Sets the input keybinding mode.
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LineInputState, LineInputMessage};
    ///
    /// let mut state = LineInputState::new();
    /// state.update(LineInputMessage::SetValue("command one".into()));
    /// state.update(LineInputMessage::Submit);
    /// assert_eq!(state.history_entries(), &["command one"]);
    /// ```
    pub fn history_entries(&self) -> &[String] {
        self.history.entries()
    }

    /// Returns the number of history entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LineInputState, LineInputMessage};
    ///
    /// let mut state = LineInputState::new();
    /// assert_eq!(state.history_count(), 0);
    /// state.update(LineInputMessage::SetValue("cmd".into()));
    /// state.update(LineInputMessage::Submit);
    /// assert_eq!(state.history_count(), 1);
    /// ```
    pub fn history_count(&self) -> usize {
        self.history.count()
    }

    /// Returns true if currently browsing history.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LineInputState;
    ///
    /// let state = LineInputState::new();
    /// assert!(!state.is_browsing_history());
    /// ```
    pub fn is_browsing_history(&self) -> bool {
        self.history.is_browsing()
    }

    /// Returns true if there are entries to undo.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LineInputState, LineInputMessage};
    ///
    /// let mut state = LineInputState::new();
    /// assert!(!state.can_undo());
    /// state.update(LineInputMessage::Insert('a'));
    /// assert!(state.can_undo());
    /// ```
    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }

    /// Returns true if there are entries to redo.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LineInputState, LineInputMessage};
    ///
    /// let mut state = LineInputState::new();
    /// state.update(LineInputMessage::Insert('a'));
    /// state.update(LineInputMessage::Undo);
    /// assert!(state.can_redo());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LineInputState, LineInputMessage};
    ///
    /// let mut state = LineInputState::with_value("hello");
    /// state.update(LineInputMessage::SelectAll);
    /// assert_eq!(state.selection_range(), Some((0, 5)));
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LineInputState, LineInputMessage};
    ///
    /// let mut state = LineInputState::with_value("hello");
    /// state.update(LineInputMessage::SelectAll);
    /// assert_eq!(state.selected_text(), Some("hello"));
    /// ```
    pub fn selected_text(&self) -> Option<&str> {
        let (start, end) = self.selection_range()?;
        Some(&self.buffer[start..end])
    }

    /// Returns the internal clipboard contents.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LineInputState, LineInputMessage};
    ///
    /// let mut state = LineInputState::with_value("hello");
    /// state.update(LineInputMessage::SelectAll);
    /// state.update(LineInputMessage::Copy);
    /// assert_eq!(state.clipboard(), "hello");
    /// ```
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
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        // Handle paste events
        if let Event::Paste(text) = event {
            return Some(LineInputMessage::Paste(text.clone()));
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.ctrl();
        let shift = key.modifiers.shift();

        match key.code {
            // Undo/Redo
            Key::Char('z') if ctrl => Some(LineInputMessage::Undo),
            Key::Char('y') if ctrl => Some(LineInputMessage::Redo),

            // Mode-dependent Ctrl bindings
            Key::Char('a') if ctrl => match state.input_mode {
                InputMode::Desktop => Some(LineInputMessage::SelectAll),
                InputMode::Readline => Some(LineInputMessage::Home),
            },
            Key::Char('e') if ctrl => match state.input_mode {
                InputMode::Desktop => Some(LineInputMessage::End),
                InputMode::Readline => Some(LineInputMessage::End),
            },
            Key::Char('w') if ctrl => Some(LineInputMessage::DeleteWordBack),
            Key::Char('k') if ctrl => match state.input_mode {
                InputMode::Desktop => None,
                InputMode::Readline => Some(LineInputMessage::DeleteToEnd),
            },
            Key::Char('u') if ctrl => Some(LineInputMessage::Clear),

            // Clipboard (Desktop mode only; Readline uses Ctrl-A/E/W/K)
            Key::Char('c') if ctrl && state.input_mode == InputMode::Desktop => {
                Some(LineInputMessage::Copy)
            }
            Key::Char('x') if ctrl && state.input_mode == InputMode::Desktop => {
                Some(LineInputMessage::Cut)
            }
            Key::Char('v') if ctrl && state.input_mode == InputMode::Desktop => {
                if !state.clipboard.is_empty() {
                    Some(LineInputMessage::Paste(state.clipboard.clone()))
                } else {
                    None
                }
            }

            // Regular character insertion
            Key::Char(_) if !ctrl => key.raw_char.map(LineInputMessage::Insert),

            // Selection with shift
            Key::Left if ctrl && shift => Some(LineInputMessage::SelectWordLeft),
            Key::Right if ctrl && shift => Some(LineInputMessage::SelectWordRight),
            Key::Left if shift => Some(LineInputMessage::SelectLeft),
            Key::Right if shift => Some(LineInputMessage::SelectRight),
            Key::Home if shift => Some(LineInputMessage::SelectHome),
            Key::End if shift => Some(LineInputMessage::SelectEnd),

            // Word deletion
            Key::Backspace if ctrl => Some(LineInputMessage::DeleteWordBack),
            Key::Delete if ctrl => Some(LineInputMessage::DeleteWordForward),

            // Character deletion
            Key::Backspace => Some(LineInputMessage::Backspace),
            Key::Delete => Some(LineInputMessage::Delete),

            // Word navigation
            Key::Left if ctrl => Some(LineInputMessage::WordLeft),
            Key::Right if ctrl => Some(LineInputMessage::WordRight),

            // Character navigation
            Key::Left => Some(LineInputMessage::Left),
            Key::Right => Some(LineInputMessage::Right),
            Key::Home => Some(LineInputMessage::Home),
            Key::End => Some(LineInputMessage::End),

            // Up/Down: context-dependent
            Key::Up => {
                let (row, _) =
                    cursor_to_visual(&state.buffer, state.cursor, state.last_display_width);
                if row == 0 {
                    Some(LineInputMessage::HistoryPrev)
                } else {
                    Some(LineInputMessage::VisualUp)
                }
            }
            Key::Down => {
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
            Key::Enter => Some(LineInputMessage::Submit),

            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        update::update(state, msg)
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        view_helpers::render(state, ctx);
    }
}
