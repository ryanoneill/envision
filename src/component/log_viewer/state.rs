use super::{
    Focus, InputFieldState, LogViewer, LogViewerMessage, LogViewerOutput, StatusLogEntry,
    StatusLogLevel,
};
use crate::component::Component;
use crate::scroll::ScrollState;

/// State for a LogViewer component.
///
/// Contains the log entries, search state, and severity filter toggles.
#[derive(Clone, Debug)]
pub struct LogViewerState {
    /// All log entries in insertion order.
    pub(super) entries: Vec<StatusLogEntry>,
    /// Next entry ID counter.
    pub(super) next_id: u64,
    /// Maximum number of entries to keep.
    pub(super) max_entries: usize,
    /// The search input field state.
    pub(super) search: InputFieldState,
    /// The current search text (cached for filtering).
    pub(super) search_text: String,
    /// Scroll state for the visible log.
    pub(super) scroll: ScrollState,
    /// Severity filter toggles (true = show).
    pub(super) show_info: bool,
    /// Whether to show success entries.
    pub(super) show_success: bool,
    /// Whether to show warning entries.
    pub(super) show_warning: bool,
    /// Whether to show error entries.
    pub(super) show_error: bool,
    /// Whether to show timestamps.
    pub(super) show_timestamps: bool,
    /// Optional title for the log block.
    pub(super) title: Option<String>,
    /// Whether follow mode is enabled (auto-scroll on new entries).
    pub(super) follow: bool,
    /// Whether regex search mode is enabled.
    pub(super) use_regex: bool,
    /// Search history (most recent last).
    pub(super) search_history: Vec<String>,
    /// Maximum number of search history entries to keep.
    pub(super) max_history: usize,
    /// Current position in search history (None means not browsing history).
    pub(super) history_index: Option<usize>,
    /// Internal focus state.
    pub(super) focus: Focus,
}

impl Default for LogViewerState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
            max_entries: 1000,
            search: InputFieldState::new(),
            search_text: String::new(),
            scroll: ScrollState::default(),
            show_info: true,
            show_success: true,
            show_warning: true,
            show_error: true,
            show_timestamps: false,
            title: None,
            follow: true,
            use_regex: false,
            search_history: Vec::new(),
            max_history: 20,
            history_index: None,
            focus: Focus::Log,
        }
    }
}

impl PartialEq for LogViewerState {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
            && self.next_id == other.next_id
            && self.max_entries == other.max_entries
            && self.search_text == other.search_text
            && self.scroll == other.scroll
            && self.show_info == other.show_info
            && self.show_success == other.show_success
            && self.show_warning == other.show_warning
            && self.show_error == other.show_error
            && self.show_timestamps == other.show_timestamps
            && self.title == other.title
            && self.follow == other.follow
            && self.use_regex == other.use_regex
            && self.search_history == other.search_history
            && self.max_history == other.max_history
            && self.focus == other.focus
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

    /// Sets whether follow mode is enabled (builder pattern).
    ///
    /// When follow mode is enabled and new entries are added, the log
    /// automatically scrolls to show the newest entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new().with_follow(false);
    /// assert!(!state.follow());
    /// ```
    pub fn with_follow(mut self, follow: bool) -> Self {
        self.follow = follow;
        self
    }

    /// Sets whether regex search mode is enabled (builder pattern).
    ///
    /// When regex mode is enabled and the `regex` feature flag is active,
    /// search text is compiled as a regular expression pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new().with_regex(true);
    /// assert!(state.use_regex());
    /// ```
    pub fn with_regex(mut self, use_regex: bool) -> Self {
        self.use_regex = use_regex;
        self
    }

    /// Sets the maximum search history size (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new().with_max_history(10);
    /// assert_eq!(state.max_history(), 10);
    /// ```
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// let id = state.push_warning("Disk space low");
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn push_warning(&mut self, message: impl Into<String>) -> u64 {
        self.push_entry(message.into(), StatusLogLevel::Warning, None)
    }

    /// Adds an error-level entry, returning its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// let id = state.push_error("Connection refused");
    /// assert_eq!(state.len(), 1);
    /// ```
    pub fn push_error(&mut self, message: impl Into<String>) -> u64 {
        self.push_entry(message.into(), StatusLogLevel::Error, None)
    }

    /// Adds an info-level entry with a timestamp, returning its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new().with_timestamps(true);
    /// let id = state.push_info_with_timestamp("Server started", "12:00:00");
    /// assert_eq!(state.len(), 1);
    /// ```
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

    /// Adds an entry with the given level and optional timestamp, returning its ID.
    ///
    /// This is the general-purpose entry method. For convenience, use
    /// [`push_info`](Self::push_info), [`push_warning`](Self::push_warning),
    /// [`push_error`](Self::push_error), or [`push_success`](Self::push_success).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{LogViewerState, StatusLogLevel};
    ///
    /// let mut state = LogViewerState::new();
    /// let id = state.push_entry("Custom entry".into(), StatusLogLevel::Info, None);
    /// assert_eq!(state.entries().len(), 1);
    /// ```
    pub fn push_entry(
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
        self.scroll.set_offset(0);
    }

    // ---- Accessors ----

    /// Returns all entries in insertion order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.push_info("first");
    /// state.push_error("second");
    /// assert_eq!(state.entries().len(), 2);
    /// ```
    pub fn entries(&self) -> &[StatusLogEntry] {
        &self.entries
    }

    /// Returns the number of entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.push_info("first");
    /// state.push_error("second");
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if there are no entries.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
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
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.set_max_entries(200);
    /// assert_eq!(state.max_entries(), 200);
    /// ```
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
        if self.entries.len() > max {
            let excess = self.entries.len() - max;
            self.entries.drain(..excess);
            self.scroll.set_content_length(self.entries.len());
        }
    }

    /// Returns the current search text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert_eq!(state.search_text(), "");
    /// ```
    pub fn search_text(&self) -> &str {
        &self.search_text
    }

    /// Returns the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Sets the scroll offset.
    pub fn set_scroll_offset(&mut self, offset: usize) {
        let len = self.visible_entries().len();
        self.scroll.set_content_length(len);
        self.scroll.set_viewport_height(1.min(len));
        self.scroll.set_offset(offset);
    }

    /// Returns whether timestamps are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert!(!state.show_timestamps()); // disabled by default
    ///
    /// let state = LogViewerState::new().with_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    /// Sets whether timestamps are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.set_show_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn set_show_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new().with_title("System Log");
    /// assert_eq!(state.title(), Some("System Log"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.set_title(Some("Error Log".to_string()));
    /// assert_eq!(state.title(), Some("Error Log"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns true if info entries are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert!(state.show_info());
    /// ```
    pub fn show_info(&self) -> bool {
        self.show_info
    }

    /// Returns true if success entries are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert!(state.show_success()); // enabled by default
    /// ```
    pub fn show_success(&self) -> bool {
        self.show_success
    }

    /// Returns true if warning entries are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert!(state.show_warning()); // enabled by default
    /// ```
    pub fn show_warning(&self) -> bool {
        self.show_warning
    }

    /// Returns true if error entries are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert!(state.show_error()); // enabled by default
    /// ```
    pub fn show_error(&self) -> bool {
        self.show_error
    }

    /// Sets the info filter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.set_show_info(false);
    /// assert!(!state.show_info());
    /// ```
    pub fn set_show_info(&mut self, show: bool) {
        self.show_info = show;
    }

    /// Sets the success filter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.set_show_success(false);
    /// assert!(!state.show_success());
    /// ```
    pub fn set_show_success(&mut self, show: bool) {
        self.show_success = show;
    }

    /// Sets the warning filter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.set_show_warning(false);
    /// assert!(!state.show_warning());
    /// ```
    pub fn set_show_warning(&mut self, show: bool) {
        self.show_warning = show;
    }

    /// Sets the error filter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let mut state = LogViewerState::new();
    /// state.set_show_error(false);
    /// assert!(!state.show_error());
    /// ```
    pub fn set_show_error(&mut self, show: bool) {
        self.show_error = show;
    }

    /// Returns whether follow mode is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert!(state.follow());
    /// ```
    pub fn follow(&self) -> bool {
        self.follow
    }

    /// Sets follow mode.
    pub fn set_follow(&mut self, follow: bool) {
        self.follow = follow;
    }

    /// Returns whether regex search mode is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LogViewerState;
    ///
    /// let state = LogViewerState::new();
    /// assert!(!state.use_regex());
    /// ```
    pub fn use_regex(&self) -> bool {
        self.use_regex
    }

    /// Sets regex search mode.
    pub fn set_use_regex(&mut self, use_regex: bool) {
        self.use_regex = use_regex;
    }

    /// Returns the search history.
    pub fn search_history(&self) -> &[String] {
        &self.search_history
    }

    /// Returns the maximum search history size.
    pub fn max_history(&self) -> usize {
        self.max_history
    }

    /// Sets the maximum search history size.
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        if self.search_history.len() > max {
            let excess = self.search_history.len() - max;
            self.search_history.drain(..excess);
        }
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

        // Search text filter
        if self.search_text.is_empty() {
            return true;
        }

        self.matches_search(entry.message())
    }

    /// Returns true if a message matches the current search text.
    ///
    /// When regex mode is enabled and the `regex` feature is active, the
    /// search text is compiled as a case-insensitive regex. If compilation
    /// fails, falls back to literal substring search.
    fn matches_search(&self, message: &str) -> bool {
        #[cfg(feature = "regex")]
        if self.use_regex {
            if let Ok(re) = regex::RegexBuilder::new(&self.search_text)
                .case_insensitive(true)
                .build()
            {
                return re.is_match(message);
            }
            // Fall through to literal search on invalid regex
        }

        // Default: case-insensitive substring search
        let search_lower = self.search_text.to_lowercase();
        message.to_lowercase().contains(&search_lower)
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: LogViewerMessage) -> Option<LogViewerOutput> {
        LogViewer::update(self, msg)
    }
}
