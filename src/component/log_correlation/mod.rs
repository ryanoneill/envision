//! Side-by-side time-aligned log streams with independent filtering.
//!
//! [`LogCorrelation`] displays multiple log streams side by side,
//! aligning entries by timestamp so that events occurring at similar
//! times appear on the same row. Each stream can be independently
//! filtered by text or severity level, while scrolling stays
//! synchronized across all streams.
//!
//! State is stored in [`LogCorrelationState`], updated via
//! [`LogCorrelationMessage`], and produces [`LogCorrelationOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, LogCorrelation, LogCorrelationState,
//!     LogCorrelationMessage, LogCorrelationOutput,
//!     LogStream, CorrelationEntry, CorrelationLevel,
//! };
//! use ratatui::prelude::Color;
//!
//! let mut state = LogCorrelationState::new()
//!     .with_streams(vec![
//!         LogStream::new("API Server").with_color(Color::Cyan),
//!         LogStream::new("Database").with_color(Color::Green),
//!     ]);
//!
//! state.push_entry(0, CorrelationEntry::new(1.0, CorrelationLevel::Info, "Request"));
//! state.push_entry(1, CorrelationEntry::new(1.0, CorrelationLevel::Info, "Connected"));
//!
//! assert_eq!(state.stream_count(), 2);
//! assert_eq!(state.streams()[0].entries.len(), 1);
//! ```

mod render;

use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::scroll::ScrollState;

/// Severity level for a correlation log entry.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum CorrelationLevel {
    /// Debug-level message.
    Debug,
    /// Informational message.
    #[default]
    Info,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
}

impl CorrelationLevel {
    /// Returns the display color for this severity level.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CorrelationLevel;
    /// use ratatui::prelude::Color;
    ///
    /// assert_eq!(CorrelationLevel::Error.color(), Color::Red);
    /// assert_eq!(CorrelationLevel::Info.color(), Color::Blue);
    /// ```
    pub fn color(&self) -> Color {
        match self {
            CorrelationLevel::Debug => Color::DarkGray,
            CorrelationLevel::Info => Color::Blue,
            CorrelationLevel::Warning => Color::Yellow,
            CorrelationLevel::Error => Color::Red,
        }
    }

    /// Returns the short label for this severity level.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CorrelationLevel;
    ///
    /// assert_eq!(CorrelationLevel::Warning.label(), "WRN");
    /// assert_eq!(CorrelationLevel::Debug.label(), "DBG");
    /// ```
    pub fn label(&self) -> &'static str {
        match self {
            CorrelationLevel::Debug => "DBG",
            CorrelationLevel::Info => "INF",
            CorrelationLevel::Warning => "WRN",
            CorrelationLevel::Error => "ERR",
        }
    }
}

impl std::fmt::Display for CorrelationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A single log entry in a stream.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct CorrelationEntry {
    /// Timestamp for time-alignment (seconds since an arbitrary epoch).
    pub timestamp: f64,
    /// Severity level.
    pub level: CorrelationLevel,
    /// The log message text.
    pub message: String,
}

impl CorrelationEntry {
    /// Creates a new correlation entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CorrelationEntry, CorrelationLevel};
    ///
    /// let entry = CorrelationEntry::new(1.0, CorrelationLevel::Info, "Request received");
    /// assert_eq!(entry.timestamp, 1.0);
    /// assert_eq!(entry.level, CorrelationLevel::Info);
    /// assert_eq!(entry.message, "Request received");
    /// ```
    pub fn new(timestamp: f64, level: CorrelationLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp,
            level,
            message: message.into(),
        }
    }
}

/// A single log stream (source) in the correlation view.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct LogStream {
    /// Name of this stream (e.g., service name).
    pub name: String,
    /// Color for this stream's header.
    pub color: Color,
    /// All entries in this stream.
    pub entries: Vec<CorrelationEntry>,
    /// Per-stream text filter.
    pub filter: String,
    /// Per-stream minimum severity level filter.
    pub min_level: Option<CorrelationLevel>,
}

impl LogStream {
    /// Creates a new empty log stream with the given name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogStream;
    ///
    /// let stream = LogStream::new("API Server");
    /// assert_eq!(stream.name, "API Server");
    /// assert!(stream.entries.is_empty());
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: Color::White,
            entries: Vec::new(),
            filter: String::new(),
            min_level: None,
        }
    }

    /// Sets the header color (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogStream;
    /// use ratatui::prelude::Color;
    ///
    /// let stream = LogStream::new("API").with_color(Color::Cyan);
    /// assert_eq!(stream.color, Color::Cyan);
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the header color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogStream;
    /// use ratatui::prelude::Color;
    ///
    /// let mut stream = LogStream::new("API");
    /// stream.set_color(Color::Cyan);
    /// assert_eq!(stream.color, Color::Cyan);
    /// ```
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Appends an entry to the stream (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LogStream, CorrelationEntry, CorrelationLevel};
    ///
    /// let stream = LogStream::new("API")
    ///     .with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "Started"));
    /// assert_eq!(stream.entries.len(), 1);
    /// ```
    pub fn with_entry(mut self, entry: CorrelationEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Returns the entries that pass this stream's text and level filters.
    fn filtered_entries(&self) -> Vec<&CorrelationEntry> {
        self.entries
            .iter()
            .filter(|e| self.passes_filter(e))
            .collect()
    }

    /// Returns true if an entry passes both the text and level filters.
    fn passes_filter(&self, entry: &CorrelationEntry) -> bool {
        // Level filter
        if let Some(ref min_level) = self.min_level {
            if entry.level < *min_level {
                return false;
            }
        }

        // Text filter (case-insensitive)
        if self.filter.is_empty() {
            return true;
        }

        let filter_lower = self.filter.to_lowercase();
        entry.message.to_lowercase().contains(&filter_lower)
    }
}

/// Tolerance in seconds for grouping timestamps on the same row.
const TIMESTAMP_TOLERANCE: f64 = 0.1;

/// Messages that can be sent to a LogCorrelation component.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum LogCorrelationMessage {
    /// Add a new log stream.
    AddStream(LogStream),
    /// Replace all streams.
    SetStreams(Vec<LogStream>),
    /// Push an entry to a specific stream.
    PushEntry {
        /// Index of the stream to push to.
        stream: usize,
        /// The entry to add.
        entry: CorrelationEntry,
    },
    /// Clear all streams.
    Clear,
    /// Scroll up by one row.
    ScrollUp,
    /// Scroll down by one row.
    ScrollDown,
    /// Scroll to the first row.
    ScrollToTop,
    /// Scroll to the last row.
    ScrollToBottom,
    /// Set the text filter for a specific stream.
    SetStreamFilter {
        /// Index of the stream.
        stream: usize,
        /// The filter text.
        filter: String,
    },
    /// Set the minimum severity level for a specific stream.
    SetStreamLevelFilter {
        /// Index of the stream.
        stream: usize,
        /// The minimum level (None means show all).
        level: Option<CorrelationLevel>,
    },
    /// Focus the next stream.
    FocusNextStream,
    /// Focus the previous stream.
    FocusPrevStream,
    /// Toggle synchronized scrolling.
    ToggleSyncScroll,
}

/// Output messages from a LogCorrelation component.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum LogCorrelationOutput {
    /// The active stream changed.
    StreamFocused(usize),
    /// An entry was added to a stream.
    EntryAdded {
        /// Index of the stream.
        stream: usize,
    },
}

/// A row in the time-aligned view.
///
/// Each row corresponds to one unique timestamp group.
/// For each stream, the row contains the entries at that timestamp,
/// or an empty slice if the stream has no entries there.
#[derive(Clone, Debug)]
pub(crate) struct AlignedRow {
    /// The representative timestamp for this row.
    pub(crate) timestamp: f64,
    /// For each stream, the indices into the stream's filtered entries.
    pub(crate) stream_entries: Vec<Vec<usize>>,
}

/// State for a LogCorrelation component.
///
/// Contains the log streams, scroll position, active stream index,
/// and configuration for synchronized scrolling.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct LogCorrelationState {
    /// The log streams.
    streams: Vec<LogStream>,
    /// Current scroll position as a timestamp.
    scroll_timestamp: f64,
    /// Which stream is active for filtering.
    active_stream: usize,
    /// Vertical scroll state.
    scroll: ScrollState,
    /// Whether to synchronize scrolling across streams.
    sync_scroll: bool,
    /// Optional title.
    title: Option<String>,
}

impl Default for LogCorrelationState {
    fn default() -> Self {
        Self {
            streams: Vec::new(),
            scroll_timestamp: 0.0,
            active_stream: 0,
            scroll: ScrollState::default(),
            sync_scroll: true,
            title: None,
        }
    }
}

impl PartialEq for LogCorrelationState {
    fn eq(&self, other: &Self) -> bool {
        self.streams == other.streams
            && (self.scroll_timestamp - other.scroll_timestamp).abs() < f64::EPSILON
            && self.active_stream == other.active_stream
            && self.scroll == other.scroll
            && self.sync_scroll == other.sync_scroll
            && self.title == other.title
    }
}

impl LogCorrelationState {
    /// Creates a new empty log correlation state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogCorrelationState;
    ///
    /// let state = LogCorrelationState::new();
    /// assert_eq!(state.stream_count(), 0);
    /// assert!(state.sync_scroll());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial streams (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LogCorrelationState, LogStream};
    ///
    /// let state = LogCorrelationState::new()
    ///     .with_streams(vec![
    ///         LogStream::new("API"),
    ///         LogStream::new("DB"),
    ///     ]);
    /// assert_eq!(state.stream_count(), 2);
    /// ```
    pub fn with_streams(mut self, streams: Vec<LogStream>) -> Self {
        self.streams = streams;
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogCorrelationState;
    ///
    /// let state = LogCorrelationState::new()
    ///     .with_title("Log Correlation");
    /// assert_eq!(state.title(), Some("Log Correlation"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether scrolling is synchronized (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogCorrelationState;
    ///
    /// let state = LogCorrelationState::new()
    ///     .with_sync_scroll(false);
    /// assert!(!state.sync_scroll());
    /// ```
    pub fn with_sync_scroll(mut self, sync: bool) -> Self {
        self.sync_scroll = sync;
        self
    }

    // ---- Accessors ----

    /// Returns all streams.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LogCorrelationState, LogStream};
    ///
    /// let state = LogCorrelationState::new()
    ///     .with_streams(vec![LogStream::new("API"), LogStream::new("DB")]);
    /// assert_eq!(state.streams().len(), 2);
    /// assert_eq!(state.streams()[0].name, "API");
    /// ```
    pub fn streams(&self) -> &[LogStream] {
        &self.streams
    }

    /// Returns the number of streams.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LogCorrelationState, LogStream};
    ///
    /// let state = LogCorrelationState::new()
    ///     .with_streams(vec![LogStream::new("A"), LogStream::new("B")]);
    /// assert_eq!(state.stream_count(), 2);
    /// ```
    pub fn stream_count(&self) -> usize {
        self.streams.len()
    }

    /// Returns the index of the active (focused) stream.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogCorrelationState;
    ///
    /// let state = LogCorrelationState::new();
    /// assert_eq!(state.active_stream(), 0);
    /// ```
    pub fn active_stream(&self) -> usize {
        self.active_stream
    }

    /// Returns whether synchronized scrolling is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogCorrelationState;
    ///
    /// let state = LogCorrelationState::new();
    /// assert!(state.sync_scroll()); // enabled by default
    /// ```
    pub fn sync_scroll(&self) -> bool {
        self.sync_scroll
    }

    /// Returns the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogCorrelationState;
    ///
    /// let state = LogCorrelationState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Returns the current scroll timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogCorrelationState;
    ///
    /// let state = LogCorrelationState::new();
    /// assert_eq!(state.scroll_timestamp(), 0.0);
    /// ```
    pub fn scroll_timestamp(&self) -> f64 {
        self.scroll_timestamp
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogCorrelationState;
    ///
    /// let state = LogCorrelationState::new().with_title("Logs");
    /// assert_eq!(state.title(), Some("Logs"));
    ///
    /// let no_title = LogCorrelationState::new();
    /// assert_eq!(no_title.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogCorrelationState;
    ///
    /// let mut state = LogCorrelationState::new();
    /// state.set_title("Correlated Logs");
    /// assert_eq!(state.title(), Some("Correlated Logs"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    // ---- Mutation ----

    /// Adds a new log stream.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LogCorrelationState, LogStream};
    ///
    /// let mut state = LogCorrelationState::new();
    /// state.add_stream(LogStream::new("API"));
    /// assert_eq!(state.stream_count(), 1);
    /// ```
    pub fn add_stream(&mut self, stream: LogStream) {
        self.streams.push(stream);
    }

    /// Pushes an entry to the specified stream.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     LogCorrelationState, LogStream,
    ///     CorrelationEntry, CorrelationLevel,
    /// };
    ///
    /// let mut state = LogCorrelationState::new()
    ///     .with_streams(vec![LogStream::new("API")]);
    /// state.push_entry(0, CorrelationEntry::new(1.0, CorrelationLevel::Info, "Hello"));
    /// assert_eq!(state.streams()[0].entries.len(), 1);
    /// ```
    pub fn push_entry(&mut self, stream_idx: usize, entry: CorrelationEntry) {
        if stream_idx < self.streams.len() {
            self.streams[stream_idx].entries.push(entry);
        }
    }

    // ---- Time alignment ----

    /// Computes time-aligned rows across all streams.
    ///
    /// Collects all unique timestamps (within tolerance), sorts them,
    /// and for each timestamp group, gathers entries from each stream.
    pub(crate) fn aligned_rows(&self) -> Vec<AlignedRow> {
        // Collect filtered entries from each stream
        let filtered: Vec<Vec<&CorrelationEntry>> =
            self.streams.iter().map(|s| s.filtered_entries()).collect();

        // Collect all timestamps from filtered entries
        let mut timestamps: Vec<f64> = Vec::new();
        for stream_entries in &filtered {
            for entry in stream_entries {
                timestamps.push(entry.timestamp);
            }
        }

        if timestamps.is_empty() {
            return Vec::new();
        }

        // Sort and deduplicate within tolerance
        timestamps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let unique_timestamps = Self::deduplicate_timestamps(&timestamps);

        // Build aligned rows
        let mut rows = Vec::with_capacity(unique_timestamps.len());
        for &ts in &unique_timestamps {
            let mut stream_entries = Vec::with_capacity(self.streams.len());
            for stream_filtered in &filtered {
                let indices: Vec<usize> = stream_filtered
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| (e.timestamp - ts).abs() <= TIMESTAMP_TOLERANCE)
                    .map(|(i, _)| i)
                    .collect();
                stream_entries.push(indices);
            }
            rows.push(AlignedRow {
                timestamp: ts,
                stream_entries,
            });
        }

        rows
    }

    /// Deduplicates timestamps within tolerance, keeping representative values.
    fn deduplicate_timestamps(sorted: &[f64]) -> Vec<f64> {
        if sorted.is_empty() {
            return Vec::new();
        }

        let mut result = vec![sorted[0]];
        for &ts in &sorted[1..] {
            let last = *result.last().unwrap();
            if (ts - last).abs() > TIMESTAMP_TOLERANCE {
                result.push(ts);
            }
        }
        result
    }

    /// Returns the total number of display rows (accounting for
    /// multi-entry timestamp groups).
    pub(crate) fn total_display_rows(&self) -> usize {
        let rows = self.aligned_rows();
        rows.iter()
            .map(|row| {
                row.stream_entries
                    .iter()
                    .map(|indices| indices.len().max(1))
                    .max()
                    .unwrap_or(1)
            })
            .sum()
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     LogCorrelationState, LogCorrelationMessage, LogCorrelationOutput, LogStream,
    /// };
    ///
    /// let mut state = LogCorrelationState::new()
    ///     .with_streams(vec![LogStream::new("A"), LogStream::new("B")]);
    /// let output = state.update(LogCorrelationMessage::FocusNextStream);
    /// assert_eq!(output, Some(LogCorrelationOutput::StreamFocused(1)));
    /// ```
    pub fn update(&mut self, msg: LogCorrelationMessage) -> Option<LogCorrelationOutput> {
        LogCorrelation::update(self, msg)
    }
}

/// A side-by-side time-aligned log correlation viewer.
///
/// Displays multiple log streams side by side, aligning entries by
/// timestamp. Each stream can be independently filtered while
/// scrolling stays synchronized.
///
/// # Key Bindings
///
/// - `Up` / `k` -- Scroll up
/// - `Down` / `j` -- Scroll down
/// - `Home` -- Scroll to top
/// - `End` -- Scroll to bottom
/// - `Tab` -- Focus next stream
/// - `Shift+Tab` -- Focus previous stream
/// - `s` -- Toggle synchronized scrolling
pub struct LogCorrelation(PhantomData<()>);

impl Component for LogCorrelation {
    type State = LogCorrelationState;
    type Message = LogCorrelationMessage;
    type Output = LogCorrelationOutput;

    fn init() -> Self::State {
        LogCorrelationState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            Key::Up | Key::Char('k') => Some(LogCorrelationMessage::ScrollUp),
            Key::Down | Key::Char('j') => Some(LogCorrelationMessage::ScrollDown),
            Key::Home => Some(LogCorrelationMessage::ScrollToTop),
            Key::End => Some(LogCorrelationMessage::ScrollToBottom),
            Key::Tab => {
                if key.modifiers.shift() {
                    Some(LogCorrelationMessage::FocusPrevStream)
                } else {
                    Some(LogCorrelationMessage::FocusNextStream)
                }
            }
            Key::Char('s') => Some(LogCorrelationMessage::ToggleSyncScroll),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            LogCorrelationMessage::AddStream(stream) => {
                state.streams.push(stream);
                None
            }
            LogCorrelationMessage::SetStreams(streams) => {
                state.streams = streams;
                state.active_stream = 0;
                state.scroll.set_offset(0);
                None
            }
            LogCorrelationMessage::PushEntry { stream, entry } => {
                if stream < state.streams.len() {
                    state.streams[stream].entries.push(entry);
                    Some(LogCorrelationOutput::EntryAdded { stream })
                } else {
                    None
                }
            }
            LogCorrelationMessage::Clear => {
                for s in &mut state.streams {
                    s.entries.clear();
                }
                state.scroll.set_offset(0);
                state.scroll_timestamp = 0.0;
                None
            }
            LogCorrelationMessage::ScrollUp => {
                let total = state.total_display_rows();
                state.scroll.set_content_length(total);
                state.scroll.set_viewport_height(1.min(total));
                state.scroll.scroll_up();
                Self::update_scroll_timestamp(state);
                None
            }
            LogCorrelationMessage::ScrollDown => {
                let total = state.total_display_rows();
                state.scroll.set_content_length(total);
                state.scroll.set_viewport_height(1.min(total));
                state.scroll.scroll_down();
                Self::update_scroll_timestamp(state);
                None
            }
            LogCorrelationMessage::ScrollToTop => {
                let total = state.total_display_rows();
                state.scroll.set_content_length(total);
                state.scroll.set_viewport_height(1.min(total));
                state.scroll.scroll_to_start();
                Self::update_scroll_timestamp(state);
                None
            }
            LogCorrelationMessage::ScrollToBottom => {
                let total = state.total_display_rows();
                state.scroll.set_content_length(total);
                state.scroll.set_viewport_height(1.min(total));
                state.scroll.scroll_to_end();
                Self::update_scroll_timestamp(state);
                None
            }
            LogCorrelationMessage::SetStreamFilter { stream, filter } => {
                if stream < state.streams.len() {
                    state.streams[stream].filter = filter;
                }
                None
            }
            LogCorrelationMessage::SetStreamLevelFilter { stream, level } => {
                if stream < state.streams.len() {
                    state.streams[stream].min_level = level;
                }
                None
            }
            LogCorrelationMessage::FocusNextStream => {
                if !state.streams.is_empty() {
                    state.active_stream = (state.active_stream + 1) % state.streams.len();
                    Some(LogCorrelationOutput::StreamFocused(state.active_stream))
                } else {
                    None
                }
            }
            LogCorrelationMessage::FocusPrevStream => {
                if !state.streams.is_empty() {
                    state.active_stream = if state.active_stream == 0 {
                        state.streams.len() - 1
                    } else {
                        state.active_stream - 1
                    };
                    Some(LogCorrelationOutput::StreamFocused(state.active_stream))
                } else {
                    None
                }
            }
            LogCorrelationMessage::ToggleSyncScroll => {
                state.sync_scroll = !state.sync_scroll;
                None
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        render::render(
            state,
            ctx.frame,
            ctx.area,
            ctx.theme,
            ctx.focused,
            ctx.disabled,
        );
    }
}

impl LogCorrelation {
    /// Updates the scroll timestamp based on the current scroll offset.
    fn update_scroll_timestamp(state: &mut LogCorrelationState) {
        let rows = state.aligned_rows();
        let offset = state.scroll.offset();

        let mut row_index = 0;
        for row in &rows {
            let max_entries = row
                .stream_entries
                .iter()
                .map(|indices| indices.len().max(1))
                .max()
                .unwrap_or(1);
            if row_index + max_entries > offset {
                state.scroll_timestamp = row.timestamp;
                return;
            }
            row_index += max_entries;
        }

        // Past the end, use last timestamp
        if let Some(last) = rows.last() {
            state.scroll_timestamp = last.timestamp;
        }
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
