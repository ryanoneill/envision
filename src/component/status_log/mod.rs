//! A component for displaying scrolling status messages.
//!
//! [`StatusLog`] provides a scrolling list of status messages with severity levels,
//! commonly used to display application status, progress updates, or log entries.
//! State is stored in [`StatusLogState`] and updated via [`StatusLogMessage`].
//!
//! See also [`LogViewer`](super::LogViewer) for a searchable log viewer with
//! severity filtering.
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

pub mod entry;

pub use entry::{StatusLogEntry, StatusLogLevel};

use ratatui::widgets::{Block, Borders, List, ListItem};

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};

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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let state = StatusLogState::new().with_title("Events");
    /// assert_eq!(state.title(), Some("Events"));
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusLogState, StatusLogLevel};
    ///
    /// let mut state = StatusLogState::new();
    /// state.success("Build complete");
    /// assert_eq!(state.entries()[0].level(), StatusLogLevel::Success);
    /// ```
    pub fn success(&mut self, message: impl Into<String>) -> u64 {
        self.push(message, StatusLogLevel::Success, None)
    }

    /// Adds a warning-level message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusLogState, StatusLogLevel};
    ///
    /// let mut state = StatusLogState::new();
    /// state.warning("Low memory");
    /// assert_eq!(state.entries()[0].level(), StatusLogLevel::Warning);
    /// ```
    pub fn warning(&mut self, message: impl Into<String>) -> u64 {
        self.push(message, StatusLogLevel::Warning, None)
    }

    /// Adds an error-level message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StatusLogState, StatusLogLevel};
    ///
    /// let mut state = StatusLogState::new();
    /// state.error("Connection failed");
    /// assert_eq!(state.entries()[0].level(), StatusLogLevel::Error);
    /// ```
    pub fn error(&mut self, message: impl Into<String>) -> u64 {
        self.push(message, StatusLogLevel::Error, None)
    }

    /// Adds an info-level message with timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.info_with_timestamp("Starting", "09:00:00");
    /// assert_eq!(state.entries()[0].timestamp(), Some("09:00:00"));
    /// ```
    pub fn info_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push(message, StatusLogLevel::Info, Some(timestamp.into()))
    }

    /// Adds a success-level message with timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.success_with_timestamp("Done", "09:01:00");
    /// assert_eq!(state.entries()[0].message(), "Done");
    /// ```
    pub fn success_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push(message, StatusLogLevel::Success, Some(timestamp.into()))
    }

    /// Adds a warning-level message with timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.warning_with_timestamp("Slow", "09:02:00");
    /// assert_eq!(state.entries()[0].timestamp(), Some("09:02:00"));
    /// ```
    pub fn warning_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push(message, StatusLogLevel::Warning, Some(timestamp.into()))
    }

    /// Adds an error-level message with timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.error_with_timestamp("Crash", "09:03:00");
    /// assert_eq!(state.entries()[0].message(), "Crash");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.info("First");
    /// state.error("Second");
    /// assert_eq!(state.entries().len(), 2);
    /// assert_eq!(state.entries()[0].message(), "First");
    /// ```
    pub fn entries(&self) -> &[StatusLogEntry] {
        &self.entries
    }

    /// Returns entries in display order (newest first).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.info("First");
    /// state.info("Second");
    /// let newest: Vec<_> = state.entries_newest_first().collect();
    /// assert_eq!(newest[0].message(), "Second");
    /// assert_eq!(newest[1].message(), "First");
    /// ```
    pub fn entries_newest_first(&self) -> impl Iterator<Item = &StatusLogEntry> {
        self.entries.iter().rev()
    }

    /// Returns the number of entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// assert_eq!(state.len(), 0);
    /// state.info("Hello");
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if there are no entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let state = StatusLogState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the maximum number of entries.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }

    /// Sets the maximum number of entries.
    ///
    /// If the current count exceeds the new maximum, the oldest entries are
    /// removed to bring the count within the limit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.set_max_entries(10);
    /// assert_eq!(state.max_entries(), 10);
    /// ```
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
        if self.entries.len() > max {
            let excess = self.entries.len() - max;
            self.entries.drain(..excess);
            // Clamp scroll offset after eviction
            if self.scroll_offset >= self.entries.len() {
                self.scroll_offset = self.entries.len().saturating_sub(1);
            }
        }
    }

    /// Returns whether timestamps are shown.
    pub fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    /// Sets whether to show timestamps.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.set_show_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn set_show_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }

    /// Returns the current scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let state = StatusLogState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.info("A");
    /// state.info("B");
    /// state.info("C");
    /// state.set_scroll_offset(1);
    /// assert_eq!(state.scroll_offset(), 1);
    /// ```
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.entries.len().saturating_sub(1));
    }

    /// Removes an entry by ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// let id = state.info("Temporary");
    /// assert_eq!(state.len(), 1);
    /// assert!(state.remove(id));
    /// assert_eq!(state.len(), 0);
    /// assert!(!state.remove(id)); // Already removed
    /// ```
    pub fn remove(&mut self, id: u64) -> bool {
        let len_before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < len_before
    }

    /// Clears all entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.info("A");
    /// state.info("B");
    /// state.clear();
    /// assert!(state.is_empty());
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
        self.scroll_offset = 0;
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let state = StatusLogState::new().with_title("Log");
    /// assert_eq!(state.title(), Some("Log"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StatusLogState;
    ///
    /// let mut state = StatusLogState::new();
    /// state.set_title(Some("Events".to_string()));
    /// assert_eq!(state.title(), Some("Events"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Updates the status log state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     StatusLogState, StatusLogMessage, StatusLogOutput, StatusLogLevel,
    /// };
    ///
    /// let mut state = StatusLogState::new();
    /// let output = state.update(StatusLogMessage::Push {
    ///     message: "Hello".to_string(),
    ///     level: StatusLogLevel::Info,
    ///     timestamp: None,
    /// });
    /// assert!(matches!(output, Some(StatusLogOutput::Added(_))));
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn update(&mut self, msg: StatusLogMessage) -> Option<StatusLogOutput> {
        StatusLog::update(self, msg)
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

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Up | Key::Char('k') => Some(StatusLogMessage::ScrollUp),
                Key::Down | Key::Char('j') => Some(StatusLogMessage::ScrollDown),
                Key::Home => Some(StatusLogMessage::ScrollToTop),
                Key::End => Some(StatusLogMessage::ScrollToBottom),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if ctx.area.width == 0 || ctx.area.height == 0 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::StatusLog)
                    .with_id("status_log")
                    .with_meta("entry_count", state.len().to_string()),
            );
        });

        let block = if let Some(title) = &state.title {
            Block::default().borders(Borders::ALL).title(title.as_str())
        } else {
            Block::default().borders(Borders::ALL)
        };

        let inner = block.inner(ctx.area);

        // Build list items (newest first, with scroll offset)
        let items: Vec<ListItem> = state
            .entries_newest_first()
            .skip(state.scroll_offset)
            .take(inner.height as usize)
            .map(|entry| {
                let prefix = entry.level.prefix();
                let style = if ctx.disabled {
                    ctx.theme.disabled_style()
                } else {
                    match entry.level {
                        StatusLogLevel::Info => ctx.theme.info_style(),
                        StatusLogLevel::Success => ctx.theme.success_style(),
                        StatusLogLevel::Warning => ctx.theme.warning_style(),
                        StatusLogLevel::Error => ctx.theme.error_style(),
                    }
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

        ctx.frame.render_widget(block, ctx.area);

        if !items.is_empty() {
            let list = List::new(items);
            ctx.frame.render_widget(list, inner);
        }
    }
}

#[cfg(test)]
mod tests;
