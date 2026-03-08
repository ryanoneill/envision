//! A searchable log viewer with severity filtering.
//!
//! [`LogViewer`] composes a [`StatusLog`](super::StatusLog) with an
//! [`InputField`](super::InputField) search bar and severity-level toggle
//! filters. Press `/` to focus the search bar, `Escape` to clear and return
//! to the log, and `1`-`4` to toggle Info/Success/Warning/Error filters.
//! State is stored in [`LogViewerState`], updated via [`LogViewerMessage`],
//! and produces [`LogViewerOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`](super::Disableable).
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, LogViewer, LogViewerState,
//!     LogViewerMessage, LogViewerOutput,
//! };
//!
//! let mut state = LogViewerState::new();
//! state.push_info("Application started");
//! state.push_warning("Disk space low");
//! state.push_error("Connection failed");
//!
//! assert_eq!(state.visible_entries().len(), 3);
//!
//! // Filter to errors only (toggle off Info, Success, Warning)
//! LogViewer::update(&mut state, LogViewerMessage::ToggleInfo);
//! LogViewer::update(&mut state, LogViewerMessage::ToggleSuccess);
//! LogViewer::update(&mut state, LogViewerMessage::ToggleWarning);
//! assert_eq!(state.visible_entries().len(), 1);
//! ```

mod view;

use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{
    Component, Disableable, Focusable, InputFieldMessage, InputFieldState, StatusLogEntry,
    StatusLogLevel,
};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

/// Internal focus target for the log viewer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
enum Focus {
    /// The log list is focused.
    #[default]
    Log,
    /// The search bar is focused.
    Search,
}

/// Messages that can be sent to a LogViewer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogViewerMessage {
    /// Scroll the log up by one line.
    ScrollUp,
    /// Scroll the log down by one line.
    ScrollDown,
    /// Scroll to the top of the log.
    ScrollToTop,
    /// Scroll to the bottom of the log.
    ScrollToBottom,
    /// Focus the search bar.
    FocusSearch,
    /// Return focus to the log (and optionally clear search).
    FocusLog,
    /// Type a character in the search bar.
    SearchInput(char),
    /// Delete character before cursor in search bar.
    SearchBackspace,
    /// Delete character at cursor in search bar.
    SearchDelete,
    /// Move search cursor left.
    SearchLeft,
    /// Move search cursor right.
    SearchRight,
    /// Move search cursor to start.
    SearchHome,
    /// Move search cursor to end.
    SearchEnd,
    /// Clear the search text.
    ClearSearch,
    /// Toggle the Info level filter.
    ToggleInfo,
    /// Toggle the Success level filter.
    ToggleSuccess,
    /// Toggle the Warning level filter.
    ToggleWarning,
    /// Toggle the Error level filter.
    ToggleError,
    /// Add an entry to the log.
    Push {
        /// The message text.
        message: String,
        /// The severity level.
        level: StatusLogLevel,
        /// Optional timestamp.
        timestamp: Option<String>,
    },
    /// Clear all log entries.
    Clear,
    /// Remove a specific entry by ID.
    Remove(u64),
}

/// Output messages from a LogViewer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogViewerOutput {
    /// A log entry was added.
    Added(u64),
    /// A log entry was removed.
    Removed(u64),
    /// All entries were cleared.
    Cleared,
    /// An old entry was evicted due to max_entries limit.
    Evicted(u64),
    /// The search text changed.
    SearchChanged(String),
    /// A filter toggle changed.
    FilterChanged,
}

/// State for a LogViewer component.
///
/// Contains the log entries, search state, and severity filter toggles.
#[derive(Clone, Debug)]
pub struct LogViewerState {
    /// All log entries in insertion order.
    entries: Vec<StatusLogEntry>,
    /// Next entry ID counter.
    next_id: u64,
    /// Maximum number of entries to keep.
    max_entries: usize,
    /// The search input field state.
    search: InputFieldState,
    /// The current search text (cached for filtering).
    search_text: String,
    /// Scroll offset for the visible log.
    scroll_offset: usize,
    /// Severity filter toggles (true = show).
    show_info: bool,
    /// Whether to show success entries.
    show_success: bool,
    /// Whether to show warning entries.
    show_warning: bool,
    /// Whether to show error entries.
    show_error: bool,
    /// Whether to show timestamps.
    show_timestamps: bool,
    /// Optional title for the log block.
    title: Option<String>,
    /// Internal focus state.
    focus: Focus,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for LogViewerState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
            max_entries: 1000,
            search: InputFieldState::new(),
            search_text: String::new(),
            scroll_offset: 0,
            show_info: true,
            show_success: true,
            show_warning: true,
            show_error: true,
            show_timestamps: false,
            title: None,
            focus: Focus::Log,
            focused: false,
            disabled: false,
        }
    }
}

impl PartialEq for LogViewerState {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
            && self.next_id == other.next_id
            && self.max_entries == other.max_entries
            && self.search_text == other.search_text
            && self.scroll_offset == other.scroll_offset
            && self.show_info == other.show_info
            && self.show_success == other.show_success
            && self.show_warning == other.show_warning
            && self.show_error == other.show_error
            && self.show_timestamps == other.show_timestamps
            && self.title == other.title
            && self.focus == other.focus
            && self.focused == other.focused
            && self.disabled == other.disabled
    }
}

impl LogViewerState {
    /// Creates a new empty log viewer state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert!(state.is_empty());
    /// assert_eq!(state.max_entries(), 1000);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of entries (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new().with_max_entries(500);
    /// assert_eq!(state.max_entries(), 500);
    /// ```
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Sets whether to show timestamps (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new().with_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn with_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new().with_title("Application Log");
    /// assert_eq!(state.title(), Some("Application Log"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the disabled state (builder pattern).
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Entry manipulation ----

    /// Adds an info-level entry, returning its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// let id = state.push_info("Server started");
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn push_info(&mut self, message: impl Into<String>) -> u64 {
        self.push_entry(message.into(), StatusLogLevel::Info, None)
    }

    /// Adds a success-level entry, returning its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// let id = state.push_success("Build completed");
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn push_success(&mut self, message: impl Into<String>) -> u64 {
        self.push_entry(message.into(), StatusLogLevel::Success, None)
    }

    /// Adds a warning-level entry, returning its ID.
    pub fn push_warning(&mut self, message: impl Into<String>) -> u64 {
        self.push_entry(message.into(), StatusLogLevel::Warning, None)
    }

    /// Adds an error-level entry, returning its ID.
    pub fn push_error(&mut self, message: impl Into<String>) -> u64 {
        self.push_entry(message.into(), StatusLogLevel::Error, None)
    }

    /// Adds an info-level entry with a timestamp, returning its ID.
    pub fn push_info_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push_entry(message.into(), StatusLogLevel::Info, Some(timestamp.into()))
    }

    /// Adds a success-level entry with a timestamp, returning its ID.
    pub fn push_success_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push_entry(
            message.into(),
            StatusLogLevel::Success,
            Some(timestamp.into()),
        )
    }

    /// Adds a warning-level entry with a timestamp, returning its ID.
    pub fn push_warning_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push_entry(
            message.into(),
            StatusLogLevel::Warning,
            Some(timestamp.into()),
        )
    }

    /// Adds an error-level entry with a timestamp, returning its ID.
    pub fn push_error_with_timestamp(
        &mut self,
        message: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> u64 {
        self.push_entry(
            message.into(),
            StatusLogLevel::Error,
            Some(timestamp.into()),
        )
    }

    /// Internal method to add an entry.
    fn push_entry(
        &mut self,
        message: String,
        level: StatusLogLevel,
        timestamp: Option<String>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let entry = match timestamp {
            Some(ts) => StatusLogEntry::with_timestamp(id, message, level, ts),
            None => StatusLogEntry::new(id, message, level),
        };
        self.entries.push(entry);
        // Evict oldest if over limit
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
        id
    }

    /// Removes an entry by ID. Returns true if the entry was found.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// let id = state.push_info("test");
    /// assert_eq!(state.len(), 1);
    /// assert!(state.remove(id));
    /// assert!(state.is_empty());
    /// ```
    pub fn remove(&mut self, id: u64) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.id() == id) {
            self.entries.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clears all entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.push_info("entry 1");
    /// state.push_info("entry 2");
    /// state.clear();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
        self.scroll_offset = 0;
    }

    // ---- Accessors ----

    /// Returns all entries in insertion order.
    pub fn entries(&self) -> &[StatusLogEntry] {
        &self.entries
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
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// Returns the current search text.
    pub fn search_text(&self) -> &str {
        &self.search_text
    }

    /// Returns the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the scroll offset.
    pub fn set_scroll_offset(&mut self, offset: usize) {
        let max = self.visible_entries().len().saturating_sub(1);
        self.scroll_offset = offset.min(max);
    }

    /// Returns whether timestamps are shown.
    pub fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    /// Sets whether timestamps are shown.
    pub fn set_show_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns true if info entries are shown.
    pub fn show_info(&self) -> bool {
        self.show_info
    }

    /// Returns true if success entries are shown.
    pub fn show_success(&self) -> bool {
        self.show_success
    }

    /// Returns true if warning entries are shown.
    pub fn show_warning(&self) -> bool {
        self.show_warning
    }

    /// Returns true if error entries are shown.
    pub fn show_error(&self) -> bool {
        self.show_error
    }

    /// Sets the info filter.
    pub fn set_show_info(&mut self, show: bool) {
        self.show_info = show;
    }

    /// Sets the success filter.
    pub fn set_show_success(&mut self, show: bool) {
        self.show_success = show;
    }

    /// Sets the warning filter.
    pub fn set_show_warning(&mut self, show: bool) {
        self.show_warning = show;
    }

    /// Sets the error filter.
    pub fn set_show_error(&mut self, show: bool) {
        self.show_error = show;
    }

    /// Returns whether the search bar is focused.
    pub fn is_search_focused(&self) -> bool {
        self.focus == Focus::Search
    }

    /// Returns the current value of the search input field.
    pub fn search_value(&self) -> &str {
        self.search.value()
    }

    /// Returns the cursor display position in the search field.
    pub fn search_cursor_position(&self) -> usize {
        self.search.cursor_display_position()
    }

    // ---- Filtering ----

    /// Returns the entries that match the current filters and search text.
    ///
    /// Entries are returned newest-first.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.push_info("hello world");
    /// state.push_error("connection failed");
    /// state.push_info("hello again");
    ///
    /// assert_eq!(state.visible_entries().len(), 3);
    /// ```
    pub fn visible_entries(&self) -> Vec<&StatusLogEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|entry| self.matches_filters(entry))
            .collect()
    }

    /// Returns true if an entry passes both the level filter and search filter.
    fn matches_filters(&self, entry: &StatusLogEntry) -> bool {
        // Level filter
        let level_ok = match entry.level() {
            StatusLogLevel::Info => self.show_info,
            StatusLogLevel::Success => self.show_success,
            StatusLogLevel::Warning => self.show_warning,
            StatusLogLevel::Error => self.show_error,
        };

        if !level_ok {
            return false;
        }

        // Search text filter (case-insensitive)
        if self.search_text.is_empty() {
            return true;
        }

        let search_lower = self.search_text.to_lowercase();
        entry.message().to_lowercase().contains(&search_lower)
    }

    // ---- Instance methods ----

    /// Returns true if the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Maps an input event to a log viewer message.
    pub fn handle_event(&self, event: &Event) -> Option<LogViewerMessage> {
        LogViewer::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<LogViewerOutput> {
        LogViewer::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: LogViewerMessage) -> Option<LogViewerOutput> {
        LogViewer::update(self, msg)
    }
}

/// A searchable log viewer with severity filtering.
///
/// Composes a log display with a search input field and severity-level
/// toggle filters. The search is case-insensitive and matches against
/// entry messages.
///
/// # Key Bindings (Log Mode)
///
/// - `Up` / `k` — Scroll up
/// - `Down` / `j` — Scroll down
/// - `Home` — Scroll to top (newest)
/// - `End` — Scroll to bottom (oldest)
/// - `/` — Focus search bar
/// - `1` — Toggle Info filter
/// - `2` — Toggle Success filter
/// - `3` — Toggle Warning filter
/// - `4` — Toggle Error filter
///
/// # Key Bindings (Search Mode)
///
/// - `Escape` — Clear search and return to log
/// - `Enter` — Return to log (keep search text)
/// - Standard text editing keys
pub struct LogViewer(PhantomData<()>);

impl Component for LogViewer {
    type State = LogViewerState;
    type Message = LogViewerMessage;
    type Output = LogViewerOutput;

    fn init() -> Self::State {
        LogViewerState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;

        match state.focus {
            Focus::Log => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(LogViewerMessage::ScrollUp),
                KeyCode::Down | KeyCode::Char('j') => Some(LogViewerMessage::ScrollDown),
                KeyCode::Home => Some(LogViewerMessage::ScrollToTop),
                KeyCode::End => Some(LogViewerMessage::ScrollToBottom),
                KeyCode::Char('/') => Some(LogViewerMessage::FocusSearch),
                KeyCode::Char('1') => Some(LogViewerMessage::ToggleInfo),
                KeyCode::Char('2') => Some(LogViewerMessage::ToggleSuccess),
                KeyCode::Char('3') => Some(LogViewerMessage::ToggleWarning),
                KeyCode::Char('4') => Some(LogViewerMessage::ToggleError),
                _ => None,
            },
            Focus::Search => match key.code {
                KeyCode::Esc => Some(LogViewerMessage::ClearSearch),
                KeyCode::Enter => Some(LogViewerMessage::FocusLog),
                KeyCode::Char(c) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        None
                    } else {
                        Some(LogViewerMessage::SearchInput(c))
                    }
                }
                KeyCode::Backspace => Some(LogViewerMessage::SearchBackspace),
                KeyCode::Delete => Some(LogViewerMessage::SearchDelete),
                KeyCode::Left => Some(LogViewerMessage::SearchLeft),
                KeyCode::Right => Some(LogViewerMessage::SearchRight),
                KeyCode::Home => Some(LogViewerMessage::SearchHome),
                KeyCode::End => Some(LogViewerMessage::SearchEnd),
                _ => None,
            },
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            LogViewerMessage::ScrollUp => {
                if state.scroll_offset > 0 {
                    state.scroll_offset -= 1;
                }
                None
            }
            LogViewerMessage::ScrollDown => {
                let max = state.visible_entries().len().saturating_sub(1);
                if state.scroll_offset < max {
                    state.scroll_offset += 1;
                }
                None
            }
            LogViewerMessage::ScrollToTop => {
                state.scroll_offset = 0;
                None
            }
            LogViewerMessage::ScrollToBottom => {
                let max = state.visible_entries().len().saturating_sub(1);
                state.scroll_offset = max;
                None
            }
            LogViewerMessage::FocusSearch => {
                state.focus = Focus::Search;
                state.search.set_focused(true);
                None
            }
            LogViewerMessage::FocusLog => {
                state.focus = Focus::Log;
                state.search.set_focused(false);
                None
            }
            LogViewerMessage::SearchInput(c) => {
                state.search.update(InputFieldMessage::Insert(c));
                state.search_text = state.search.value().to_string();
                state.scroll_offset = 0;
                Some(LogViewerOutput::SearchChanged(state.search_text.clone()))
            }
            LogViewerMessage::SearchBackspace => {
                state.search.update(InputFieldMessage::Backspace);
                state.search_text = state.search.value().to_string();
                state.scroll_offset = 0;
                Some(LogViewerOutput::SearchChanged(state.search_text.clone()))
            }
            LogViewerMessage::SearchDelete => {
                state.search.update(InputFieldMessage::Delete);
                state.search_text = state.search.value().to_string();
                state.scroll_offset = 0;
                Some(LogViewerOutput::SearchChanged(state.search_text.clone()))
            }
            LogViewerMessage::SearchLeft => {
                state.search.update(InputFieldMessage::Left);
                None
            }
            LogViewerMessage::SearchRight => {
                state.search.update(InputFieldMessage::Right);
                None
            }
            LogViewerMessage::SearchHome => {
                state.search.update(InputFieldMessage::Home);
                None
            }
            LogViewerMessage::SearchEnd => {
                state.search.update(InputFieldMessage::End);
                None
            }
            LogViewerMessage::ClearSearch => {
                state.search.update(InputFieldMessage::Clear);
                state.search_text.clear();
                state.scroll_offset = 0;
                state.focus = Focus::Log;
                state.search.set_focused(false);
                Some(LogViewerOutput::SearchChanged(String::new()))
            }
            LogViewerMessage::ToggleInfo => {
                state.show_info = !state.show_info;
                state.scroll_offset = 0;
                Some(LogViewerOutput::FilterChanged)
            }
            LogViewerMessage::ToggleSuccess => {
                state.show_success = !state.show_success;
                state.scroll_offset = 0;
                Some(LogViewerOutput::FilterChanged)
            }
            LogViewerMessage::ToggleWarning => {
                state.show_warning = !state.show_warning;
                state.scroll_offset = 0;
                Some(LogViewerOutput::FilterChanged)
            }
            LogViewerMessage::ToggleError => {
                state.show_error = !state.show_error;
                state.scroll_offset = 0;
                Some(LogViewerOutput::FilterChanged)
            }
            LogViewerMessage::Push {
                message,
                level,
                timestamp,
            } => {
                let id = state.push_entry(message, level, timestamp);
                // Check if eviction happened
                if state.entries.len() > state.max_entries {
                    // Already evicted in push_entry
                }
                Some(LogViewerOutput::Added(id))
            }
            LogViewerMessage::Clear => {
                state.clear();
                Some(LogViewerOutput::Cleared)
            }
            LogViewerMessage::Remove(id) => {
                if state.remove(id) {
                    Some(LogViewerOutput::Removed(id))
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if area.height < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::container("log_viewer")
                    .with_focus(state.focused)
                    .with_disabled(state.disabled),
            );
        });

        // Layout: search bar (1 line) + filter bar (1 line) + log area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(area);

        let search_area = chunks[0];
        let filter_area = chunks[1];
        let log_area = chunks[2];

        // Render search bar
        view::render_search_bar(state, frame, search_area, theme);

        // Render filter bar
        view::render_filter_bar(state, frame, filter_area, theme);

        // Render log entries
        view::render_log(state, frame, log_area, theme);
    }
}

impl Focusable for LogViewer {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for LogViewer {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
