use super::types::{EventLevel, StreamEvent};
use super::{EventStream, EventStreamMessage, EventStreamOutput, Focus};
use crate::component::{Component, InputFieldState, ViewContext};
use crate::input::Event;
use crate::scroll::ScrollState;

/// State for an EventStream component.
///
/// Contains structured events, filter settings, scroll state, and display
/// configuration for a real-time event feed.
///
/// # Example
///
/// ```rust
/// use envision::component::{EventLevel, EventStreamState};
///
/// let mut state = EventStreamState::new()
///     .with_max_events(1000)
///     .with_title("System Events");
///
/// state.push_event(EventLevel::Info, "Service started");
/// state.push_event(EventLevel::Warning, "High latency detected");
///
/// assert_eq!(state.event_count(), 2);
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct EventStreamState {
    /// All events in insertion order.
    pub(super) events: Vec<StreamEvent>,
    /// Maximum events to retain.
    pub(super) max_events: usize,
    /// Auto-incrementing ID counter.
    pub(super) next_id: u64,
    /// Scroll state for virtual scrolling.
    pub(super) scroll: ScrollState,
    /// Text filter (searches message + field values).
    pub(super) filter_text: String,
    /// Minimum level to show.
    pub(super) level_filter: Option<EventLevel>,
    /// Filter by source.
    pub(super) source_filter: Option<String>,
    /// Which field keys to show as columns.
    pub(super) visible_columns: Vec<String>,
    /// Whether to follow new events.
    pub(super) auto_scroll: bool,
    /// Whether to show the timestamp column.
    pub(super) show_timestamps: bool,
    /// Whether to show the level column.
    pub(super) show_level: bool,
    /// Whether to show the source column.
    pub(super) show_source: bool,
    /// Optional title for the block.
    pub(super) title: Option<String>,
    /// Internal focus state.
    pub(super) focus: Focus,
    /// The search input field state.
    #[cfg_attr(feature = "serialization", serde(skip))]
    pub(super) search: InputFieldState,
    /// Whether the component is focused.
    pub(super) focused: bool,
    /// Whether the component is disabled.
    pub(super) disabled: bool,
}

impl Default for EventStreamState {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            max_events: 5000,
            next_id: 0,
            scroll: ScrollState::default(),
            filter_text: String::new(),
            level_filter: None,
            source_filter: None,
            visible_columns: Vec::new(),
            auto_scroll: true,
            show_timestamps: true,
            show_level: true,
            show_source: true,
            title: None,
            focus: Focus::List,
            search: InputFieldState::new(),
            focused: false,
            disabled: false,
        }
    }
}

impl PartialEq for EventStreamState {
    fn eq(&self, other: &Self) -> bool {
        self.events == other.events
            && self.max_events == other.max_events
            && self.next_id == other.next_id
            && self.scroll == other.scroll
            && self.filter_text == other.filter_text
            && self.level_filter == other.level_filter
            && self.source_filter == other.source_filter
            && self.visible_columns == other.visible_columns
            && self.auto_scroll == other.auto_scroll
            && self.show_timestamps == other.show_timestamps
            && self.show_level == other.show_level
            && self.show_source == other.show_source
            && self.title == other.title
            && self.focus == other.focus
            && self.focused == other.focused
            && self.disabled == other.disabled
    }
}

impl EventStreamState {
    /// Creates a new empty event stream state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let state = EventStreamState::new();
    /// assert_eq!(state.event_count(), 0);
    /// assert_eq!(state.max_events(), 5000);
    /// assert!(state.auto_scroll());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    // =========================================================================
    // Builder methods
    // =========================================================================

    /// Sets the maximum number of events to retain (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let state = EventStreamState::new().with_max_events(500);
    /// assert_eq!(state.max_events(), 500);
    /// ```
    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    /// Sets which field keys to show as columns (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let state = EventStreamState::new()
    ///     .with_visible_columns(vec!["path".into(), "method".into()]);
    /// assert_eq!(state.visible_columns().len(), 2);
    /// ```
    pub fn with_visible_columns(mut self, columns: Vec<String>) -> Self {
        self.visible_columns = columns;
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let state = EventStreamState::new().with_title("Events");
    /// assert_eq!(state.title(), Some("Events"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show timestamps (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let state = EventStreamState::new().with_show_timestamps(false);
    /// assert!(!state.show_timestamps());
    /// ```
    pub fn with_show_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    /// Sets whether to show the level column (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let state = EventStreamState::new().with_show_level(false);
    /// assert!(!state.show_level());
    /// ```
    pub fn with_show_level(mut self, show: bool) -> Self {
        self.show_level = show;
        self
    }

    /// Sets whether to show the source column (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let state = EventStreamState::new().with_show_source(false);
    /// assert!(!state.show_source());
    /// ```
    pub fn with_show_source(mut self, show: bool) -> Self {
        self.show_source = show;
        self
    }

    /// Sets whether auto-scroll is enabled (builder pattern).
    ///
    /// When enabled, the view automatically scrolls to show new events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let state = EventStreamState::new().with_auto_scroll(false);
    /// assert!(!state.auto_scroll());
    /// ```
    pub fn with_auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let state = EventStreamState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // =========================================================================
    // Event manipulation
    // =========================================================================

    /// Pushes a new event with the given level and message, returning its ID.
    ///
    /// The event is assigned an auto-incrementing ID and a timestamp of 0.0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, EventStreamState};
    ///
    /// let mut state = EventStreamState::new();
    /// let id = state.push_event(EventLevel::Info, "Server started");
    /// assert_eq!(state.event_count(), 1);
    /// assert_eq!(state.events()[0].id, id);
    /// ```
    pub fn push_event(&mut self, level: EventLevel, message: impl Into<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let event = StreamEvent::new(id, 0.0, level, message);
        self.events.push(event);
        self.evict_oldest();
        if self.auto_scroll {
            let len = self.visible_events().len();
            self.scroll.set_content_length(len);
            self.scroll.scroll_to_end();
        }
        id
    }

    /// Pushes a new event with fields, returning its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, EventStreamState};
    ///
    /// let mut state = EventStreamState::new();
    /// let id = state.push_event_with_fields(
    ///     EventLevel::Warning,
    ///     "Slow query",
    ///     vec![("ms".into(), "1200".into())],
    /// );
    /// assert_eq!(state.events()[0].fields.len(), 1);
    /// ```
    pub fn push_event_with_fields(
        &mut self,
        level: EventLevel,
        message: impl Into<String>,
        fields: Vec<(String, String)>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut event = StreamEvent::new(id, 0.0, level, message);
        event.fields = fields;
        self.events.push(event);
        self.evict_oldest();
        if self.auto_scroll {
            let len = self.visible_events().len();
            self.scroll.set_content_length(len);
            self.scroll.scroll_to_end();
        }
        id
    }

    /// Clears all events and resets scroll.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, EventStreamState};
    ///
    /// let mut state = EventStreamState::new();
    /// state.push_event(EventLevel::Info, "hello");
    /// state.clear();
    /// assert_eq!(state.event_count(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.events.clear();
        self.scroll.set_offset(0);
    }

    /// Sets the text filter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, EventStreamState};
    ///
    /// let mut state = EventStreamState::new();
    /// state.push_event(EventLevel::Info, "Request received");
    /// state.push_event(EventLevel::Error, "Connection failed");
    /// state.set_filter("request".to_string());
    /// assert_eq!(state.visible_events().len(), 1);
    /// ```
    pub fn set_filter(&mut self, filter: String) {
        self.filter_text = filter;
        self.scroll.set_offset(0);
    }

    /// Evicts oldest events if over the max_events limit.
    pub(super) fn evict_oldest(&mut self) {
        while self.events.len() > self.max_events {
            self.events.remove(0);
        }
    }

    // =========================================================================
    // Accessors
    // =========================================================================

    /// Returns all events in insertion order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, EventStreamState};
    ///
    /// let mut state = EventStreamState::new();
    /// state.push_event(EventLevel::Info, "hello");
    /// assert_eq!(state.events().len(), 1);
    /// ```
    pub fn events(&self) -> &[StreamEvent] {
        &self.events
    }

    /// Returns a mutable reference to the events.
    ///
    /// After modifying the events through this reference, the scroll
    /// content length should be updated. Call
    /// [`set_auto_scroll`](Self::set_auto_scroll) or push a new event
    /// to trigger a scroll recalculation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, EventStreamState};
    ///
    /// let mut state = EventStreamState::new();
    /// state.push_event(EventLevel::Info, "hello");
    /// state.push_event(EventLevel::Warning, "warn");
    /// state.events_mut().retain(|e| e.level == EventLevel::Warning);
    /// assert_eq!(state.event_count(), 1);
    /// ```
    /// **Note**: After modifying the collection, the scrollbar may be inaccurate
    /// until the next render. Prefer dedicated methods (e.g., `push_event()`) when available.
    pub fn events_mut(&mut self) -> &mut Vec<StreamEvent> {
        &mut self.events
    }

    /// Returns the total number of events.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, EventStreamState};
    ///
    /// let mut state = EventStreamState::new();
    /// state.push_event(EventLevel::Info, "hello");
    /// assert_eq!(state.event_count(), 1);
    /// ```
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Returns the maximum number of events to retain.
    pub fn max_events(&self) -> usize {
        self.max_events
    }

    /// Returns the current text filter.
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Returns the current level filter.
    pub fn level_filter(&self) -> Option<&EventLevel> {
        self.level_filter.as_ref()
    }

    /// Returns the current source filter.
    pub fn source_filter(&self) -> Option<&str> {
        self.source_filter.as_deref()
    }

    /// Returns the visible column keys.
    pub fn visible_columns(&self) -> &[String] {
        &self.visible_columns
    }

    /// Returns whether auto-scroll is enabled.
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Returns whether the timestamp column is shown.
    pub fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    /// Returns whether the level column is shown.
    pub fn show_level(&self) -> bool {
        self.show_level
    }

    /// Returns whether the source column is shown.
    pub fn show_source(&self) -> bool {
        self.show_source
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let mut state = EventStreamState::new();
    /// state.set_title("System Events");
    /// assert_eq!(state.title(), Some("System Events"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Sets the maximum number of events to retain.
    ///
    /// If the current count exceeds the new maximum, oldest events are removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let mut state = EventStreamState::new();
    /// state.set_max_events(500);
    /// assert_eq!(state.max_events(), 500);
    /// ```
    pub fn set_max_events(&mut self, max: usize) {
        self.max_events = max;
        if self.events.len() > max {
            let excess = self.events.len() - max;
            self.events.drain(..excess);
            self.scroll.set_content_length(self.events.len());
        }
    }

    /// Sets whether to show the timestamp column.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let mut state = EventStreamState::new();
    /// state.set_show_timestamps(false);
    /// assert!(!state.show_timestamps());
    /// ```
    pub fn set_show_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }

    /// Sets whether to show the level column.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let mut state = EventStreamState::new();
    /// state.set_show_level(false);
    /// assert!(!state.show_level());
    /// ```
    pub fn set_show_level(&mut self, show: bool) {
        self.show_level = show;
    }

    /// Sets whether to show the source column.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let mut state = EventStreamState::new();
    /// state.set_show_source(false);
    /// assert!(!state.show_source());
    /// ```
    pub fn set_show_source(&mut self, show: bool) {
        self.show_source = show;
    }

    /// Sets whether auto-scroll is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let mut state = EventStreamState::new();
    /// state.set_auto_scroll(false);
    /// assert!(!state.auto_scroll());
    /// ```
    pub fn set_auto_scroll(&mut self, auto_scroll: bool) {
        self.auto_scroll = auto_scroll;
    }

    /// Sets the visible column keys.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::EventStreamState;
    ///
    /// let mut state = EventStreamState::new();
    /// state.set_visible_columns(vec!["path".into(), "method".into()]);
    /// assert_eq!(state.visible_columns().len(), 2);
    /// ```
    pub fn set_visible_columns(&mut self, columns: Vec<String>) {
        self.visible_columns = columns;
    }

    /// Returns the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
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

    // =========================================================================
    // Filtering
    // =========================================================================

    /// Returns events matching all active filters.
    ///
    /// Events are returned in chronological order (oldest first).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{EventLevel, EventStreamState};
    ///
    /// let mut state = EventStreamState::new();
    /// state.push_event(EventLevel::Info, "hello");
    /// state.push_event(EventLevel::Error, "oops");
    /// state.set_filter("hello".to_string());
    /// assert_eq!(state.visible_events().len(), 1);
    /// ```
    pub fn visible_events(&self) -> Vec<&StreamEvent> {
        self.events
            .iter()
            .filter(|event| self.matches_filters(event))
            .collect()
    }

    /// Returns true if an event passes all active filters.
    fn matches_filters(&self, event: &StreamEvent) -> bool {
        // Level filter: event level must be >= minimum
        if let Some(ref min_level) = self.level_filter {
            if event.level < *min_level {
                return false;
            }
        }

        // Source filter
        if let Some(ref source) = self.source_filter {
            match &event.source {
                Some(event_source) => {
                    if !event_source.eq_ignore_ascii_case(source) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Text filter (case-insensitive, searches message + field values)
        if !self.filter_text.is_empty() {
            let search_lower = self.filter_text.to_lowercase();
            let message_matches = event.message.to_lowercase().contains(&search_lower);
            let field_matches = event.fields.iter().any(|(k, v)| {
                k.to_lowercase().contains(&search_lower) || v.to_lowercase().contains(&search_lower)
            });
            let source_matches = event
                .source
                .as_ref()
                .is_some_and(|s| s.to_lowercase().contains(&search_lower));

            if !message_matches && !field_matches && !source_matches {
                return false;
            }
        }

        true
    }

    // =========================================================================
    // Instance methods
    // =========================================================================

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

    /// Maps an input event to an event stream message.
    pub fn handle_event(&self, event: &Event) -> Option<EventStreamMessage> {
        let ctx = ViewContext::new()
            .focused(self.focused)
            .disabled(self.disabled);
        EventStream::handle_event(self, event, &ctx)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<EventStreamOutput> {
        let ctx = ViewContext::new()
            .focused(self.focused)
            .disabled(self.disabled);
        EventStream::dispatch_event(self, event, &ctx)
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: EventStreamMessage) -> Option<EventStreamOutput> {
        EventStream::update(self, msg)
    }
}
