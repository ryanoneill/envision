//! A component for displaying scrolling status messages.
//!
//! `StatusLog` provides a scrolling list of status messages with severity levels,
//! commonly used to display application status, progress updates, or log entries.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{StatusLog, StatusLogState, StatusLogLevel, Component};
//!
//! let mut state = StatusLogState::new();
//!
//! // Add messages with convenience methods
//! state.info("Starting process...");
//! state.success("Process completed");
//! state.warning("Low disk space");
//! state.error("Connection failed");
//!
//! // Messages are displayed newest first
//! assert_eq!(state.len(), 4);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use super::{Component, Focusable};
use crate::theme::Theme;

/// Severity level for status log entries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatusLogLevel {
    /// Informational message.
    #[default]
    Info,
    /// Success message.
    Success,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
}

impl StatusLogLevel {
    /// Returns the color associated with this level.
    pub fn color(&self) -> Color {
        match self {
            StatusLogLevel::Info => Color::Cyan,
            StatusLogLevel::Success => Color::Green,
            StatusLogLevel::Warning => Color::Yellow,
            StatusLogLevel::Error => Color::Red,
        }
    }

    /// Returns the prefix symbol for this level.
    pub fn prefix(&self) -> &'static str {
        match self {
            StatusLogLevel::Info => "ℹ",
            StatusLogLevel::Success => "✓",
            StatusLogLevel::Warning => "⚠",
            StatusLogLevel::Error => "✗",
        }
    }
}

/// A single status log entry.
#[derive(Clone, Debug)]
pub struct StatusLogEntry {
    /// Unique identifier.
    id: u64,
    /// The message content.
    message: String,
    /// Severity level.
    level: StatusLogLevel,
    /// Optional timestamp string.
    timestamp: Option<String>,
}

impl StatusLogEntry {
    /// Creates a new status log entry.
    pub fn new(id: u64, message: impl Into<String>, level: StatusLogLevel) -> Self {
        Self {
            id,
            message: message.into(),
            level,
            timestamp: None,
        }
    }

    /// Creates a new entry with a timestamp.
    pub fn with_timestamp(
        id: u64,
        message: impl Into<String>,
        level: StatusLogLevel,
        timestamp: impl Into<String>,
    ) -> Self {
        Self {
            id,
            message: message.into(),
            level,
            timestamp: Some(timestamp.into()),
        }
    }

    /// Returns the entry ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the level.
    pub fn level(&self) -> StatusLogLevel {
        self.level
    }

    /// Returns the timestamp if set.
    pub fn timestamp(&self) -> Option<&str> {
        self.timestamp.as_deref()
    }
}

/// Messages that can be sent to a StatusLog component.
#[derive(Clone, Debug, PartialEq)]
pub enum StatusLogMessage {
    /// Add a new log entry.
    Push {
        /// The message content.
        message: String,
        /// Severity level.
        level: StatusLogLevel,
        /// Optional timestamp.
        timestamp: Option<String>,
    },
    /// Clear all entries.
    Clear,
    /// Remove a specific entry by ID.
    Remove(u64),
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll to the top (newest).
    ScrollToTop,
    /// Scroll to the bottom (oldest visible).
    ScrollToBottom,
}

/// Output messages from a StatusLog component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusLogOutput {
    /// An entry was added (returns ID).
    Added(u64),
    /// An entry was removed.
    Removed(u64),
    /// All entries were cleared.
    Cleared,
    /// An old entry was evicted due to max_entries limit.
    Evicted(u64),
}

/// State for a StatusLog component.
///
/// Contains log entries and display configuration.
///
/// # Example
///
/// ```rust
/// use envision::component::StatusLogState;
///
/// let mut state = StatusLogState::new()
///     .with_max_entries(100)
///     .with_timestamps(true);
///
/// state.info("Application started");
/// ```
#[derive(Clone, Debug)]
pub struct StatusLogState {
    /// All log entries (stored in insertion order, displayed newest first).
    entries: Vec<StatusLogEntry>,
    /// Counter for generating unique IDs.
    next_id: u64,
    /// Maximum number of entries to keep.
    max_entries: usize,
    /// Whether to show timestamps.
    show_timestamps: bool,
    /// Scroll offset for viewing older entries.
    scroll_offset: usize,
    /// Whether the component is focused.
    focused: bool,
    /// Title for the block.
    title: Option<String>,
}

impl Default for StatusLogState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
            max_entries: 50,
            show_timestamps: false,
            scroll_offset: 0,
            focused: false,
            title: None,
        }
    }
}

impl StatusLogState {
    /// Creates a new empty StatusLog state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let state = StatusLogState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of entries to keep.
    ///
    /// When this limit is exceeded, the oldest entries are evicted.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let state = StatusLogState::new().with_max_entries(100);
    /// assert_eq!(state.max_entries(), 100);
    /// ```
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Sets whether to show timestamps.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let state = StatusLogState::new().with_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn with_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    /// Sets the title for the log block.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Adds an info-level message.
    ///
    /// # Returns
    ///
    /// The ID of the new entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// let id = state.info("Processing...");
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn info(&mut self, message: impl Into<String>) -> u64 {
        self.push(message, StatusLogLevel::Info, None)
    }

    /// Adds a success-level message.
    pub fn success(&mut self, message: impl Into<String>) -> u64 {
        self.push(message, StatusLogLevel::Success, None)
    }

    /// Adds a warning-level message.
    pub fn warning(&mut self, message: impl Into<String>) -> u64 {
        self.push(message, StatusLogLevel::Warning, None)
    }

    /// Adds an error-level message.
    pub fn error(&mut self, message: impl Into<String>) -> u64 {
        self.push(message, StatusLogLevel::Error, None)
    }

    /// Adds an info-level message with timestamp.
    pub fn info_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push(message, StatusLogLevel::Info, Some(timestamp.into()))
    }

    /// Adds a success-level message with timestamp.
    pub fn success_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push(message, StatusLogLevel::Success, Some(timestamp.into()))
    }

    /// Adds a warning-level message with timestamp.
    pub fn warning_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push(message, StatusLogLevel::Warning, Some(timestamp.into()))
    }

    /// Adds an error-level message with timestamp.
    pub fn error_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push(message, StatusLogLevel::Error, Some(timestamp.into()))
    }

    /// Internal method to push an entry.
    fn push(
        &mut self,
        message: impl Into<String>,
        level: StatusLogLevel,
        timestamp: Option<String>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let entry = if let Some(ts) = timestamp {
            StatusLogEntry::with_timestamp(id, message, level, ts)
        } else {
            StatusLogEntry::new(id, message, level)
        };

        self.entries.push(entry);
        id
    }

    /// Enforces max_entries limit and returns evicted ID if any.
    fn enforce_limit(&mut self) -> Option<u64> {
        if self.entries.len() > self.max_entries {
            let evicted = self.entries.remove(0);
            Some(evicted.id)
        } else {
            None
        }
    }

    /// Returns all entries.
    pub fn entries(&self) -> &[StatusLogEntry] {
        &self.entries
    }

    /// Returns entries in display order (newest first).
    pub fn entries_newest_first(&self) -> impl Iterator<Item = &StatusLogEntry> {
        self.entries.iter().rev()
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the maximum number of entries.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Sets the maximum number of entries.
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
    }

    /// Returns whether timestamps are shown.
    pub fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    /// Sets whether to show timestamps.
    pub fn set_show_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }

    /// Returns the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the scroll offset.
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.entries.len().saturating_sub(1));
    }

    /// Removes an entry by ID.
    pub fn remove(&mut self, id: u64) -> bool {
        let len_before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < len_before
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.scroll_offset = 0;
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }
}

/// A component for displaying scrolling status messages.
///
/// `StatusLog` displays messages with severity levels (Info, Success, Warning, Error),
/// with the newest messages shown first.
///
/// # Visual Format
///
/// ```text
/// ┌─Status─────────────────┐
/// │ ✗ Connection failed    │
/// │ ⚠ Low disk space       │
/// │ ✓ Process completed    │
/// │ ℹ Starting process...  │
/// └────────────────────────┘
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{StatusLog, StatusLogState, StatusLogMessage, StatusLogLevel, Component};
///
/// let mut state = StatusLogState::new();
///
/// // Add via convenience methods
/// state.info("Starting...");
///
/// // Or via update
/// StatusLog::update(&mut state, StatusLogMessage::Push {
///     message: "Done!".to_string(),
///     level: StatusLogLevel::Success,
///     timestamp: None,
/// });
/// ```
pub struct StatusLog;

impl Component for StatusLog {
    type State = StatusLogState;
    type Message = StatusLogMessage;
    type Output = StatusLogOutput;

    fn init() -> Self::State {
        StatusLogState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            StatusLogMessage::Push {
                message,
                level,
                timestamp,
            } => {
                let id = state.push(message, level, timestamp);
                if let Some(evicted_id) = state.enforce_limit() {
                    // Return evicted output if we hit the limit
                    return Some(StatusLogOutput::Evicted(evicted_id));
                }
                Some(StatusLogOutput::Added(id))
            }
            StatusLogMessage::Clear => {
                if state.entries.is_empty() {
                    None
                } else {
                    state.clear();
                    Some(StatusLogOutput::Cleared)
                }
            }
            StatusLogMessage::Remove(id) => {
                if state.remove(id) {
                    Some(StatusLogOutput::Removed(id))
                } else {
                    None
                }
            }
            StatusLogMessage::ScrollUp => {
                if state.scroll_offset > 0 {
                    state.scroll_offset -= 1;
                }
                None
            }
            StatusLogMessage::ScrollDown => {
                if state.scroll_offset < state.entries.len().saturating_sub(1) {
                    state.scroll_offset += 1;
                }
                None
            }
            StatusLogMessage::ScrollToTop => {
                state.scroll_offset = 0;
                None
            }
            StatusLogMessage::ScrollToBottom => {
                state.scroll_offset = state.entries.len().saturating_sub(1);
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let block = if let Some(title) = &state.title {
            Block::default().borders(Borders::ALL).title(title.as_str())
        } else {
            Block::default().borders(Borders::ALL)
        };

        let inner = block.inner(area);

        // Build list items (newest first, with scroll offset)
        let items: Vec<ListItem> = state
            .entries_newest_first()
            .skip(state.scroll_offset)
            .take(inner.height as usize)
            .map(|entry| {
                let prefix = entry.level.prefix();
                let style = match entry.level {
                    StatusLogLevel::Info => theme.info_style(),
                    StatusLogLevel::Success => theme.success_style(),
                    StatusLogLevel::Warning => theme.warning_style(),
                    StatusLogLevel::Error => theme.error_style(),
                };

                let content = if state.show_timestamps {
                    if let Some(ts) = &entry.timestamp {
                        format!("{} [{}] {}", prefix, ts, entry.message)
                    } else {
                        format!("{} {}", prefix, entry.message)
                    }
                } else {
                    format!("{} {}", prefix, entry.message)
                };

                ListItem::new(content).style(style)
            })
            .collect();

        frame.render_widget(block, area);

        if !items.is_empty() {
            let list = List::new(items);
            frame.render_widget(list, inner);
        }
    }
}

impl Focusable for StatusLog {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
