use super::*;

fn sample_state() -> EventStreamState {
    let mut state = EventStreamState::new();
    state.push_event(EventLevel::Info, "Request received");
    state.push_event_with_fields(
        EventLevel::Debug,
        "Query executed",
        vec![("ms".into(), "12".into()), ("table".into(), "users".into())],
    );
    state.push_event(EventLevel::Warning, "Cache miss");
    state.push_event(EventLevel::Error, "Connection timeout");
    state
}

fn sample_state_with_sources() -> EventStreamState {
    let mut state = EventStreamState::new();

    let id = state.next_id;
    state.next_id += 1;
    state.events.push(
        StreamEvent::new(id, 1000.0, EventLevel::Info, "Request received")
            .with_source("api")
            .with_field("path", "/users"),
    );

    let id = state.next_id;
    state.next_id += 1;
    state.events.push(
        StreamEvent::new(id, 1001.0, EventLevel::Debug, "Query executed")
            .with_source("db")
            .with_field("ms", "12"),
    );

    let id = state.next_id;
    state.next_id += 1;
    state.events.push(
        StreamEvent::new(id, 1002.0, EventLevel::Warning, "Cache miss")
            .with_source("cache")
            .with_field("key", "usr"),
    );

    let id = state.next_id;
    state.next_id += 1;
    state.events.push(
        StreamEvent::new(id, 1003.0, EventLevel::Error, "Timeout")
            .with_source("api")
            .with_field("ms", "5000"),
    );

    state
}

fn focused_state() -> EventStreamState {
    let mut state = sample_state();
    EventStream::set_focused(&mut state, true);
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = EventStreamState::new();
    assert_eq!(state.event_count(), 0);
    assert_eq!(state.max_events(), 5000);
    assert!(state.auto_scroll());
    assert!(state.show_timestamps());
    assert!(state.show_level());
    assert!(state.show_source());
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
}

#[test]
fn test_default() {
    let state = EventStreamState::default();
    assert_eq!(state.event_count(), 0);
    assert_eq!(state.max_events(), 5000);
    assert!(state.auto_scroll());
    assert_eq!(state.title(), None);
    assert!(state.filter_text().is_empty());
    assert!(state.level_filter().is_none());
    assert!(state.source_filter().is_none());
    assert!(state.visible_columns().is_empty());
}

#[test]
fn test_with_max_events() {
    let state = EventStreamState::new().with_max_events(500);
    assert_eq!(state.max_events(), 500);
}

#[test]
fn test_with_visible_columns() {
    let state = EventStreamState::new().with_visible_columns(vec!["path".into(), "method".into()]);
    assert_eq!(state.visible_columns().len(), 2);
    assert_eq!(state.visible_columns()[0], "path");
    assert_eq!(state.visible_columns()[1], "method");
}

#[test]
fn test_with_title() {
    let state = EventStreamState::new().with_title("Events");
    assert_eq!(state.title(), Some("Events"));
}

#[test]
fn test_with_show_timestamps() {
    let state = EventStreamState::new().with_show_timestamps(false);
    assert!(!state.show_timestamps());
}

#[test]
fn test_with_show_level() {
    let state = EventStreamState::new().with_show_level(false);
    assert!(!state.show_level());
}

#[test]
fn test_with_show_source() {
    let state = EventStreamState::new().with_show_source(false);
    assert!(!state.show_source());
}

#[test]
fn test_with_disabled() {
    let state = EventStreamState::new().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Event operations
// =============================================================================

#[test]
fn test_push_event() {
    let mut state = EventStreamState::new();
    let id = state.push_event(EventLevel::Info, "hello");
    assert_eq!(state.event_count(), 1);
    assert_eq!(state.events()[0].id, id);
    assert_eq!(state.events()[0].message, "hello");
    assert_eq!(state.events()[0].level, EventLevel::Info);
}

#[test]
fn test_push_event_with_fields() {
    let mut state = EventStreamState::new();
    let id = state.push_event_with_fields(
        EventLevel::Warning,
        "Slow query",
        vec![
            ("ms".into(), "1200".into()),
            ("table".into(), "users".into()),
        ],
    );
    assert_eq!(state.event_count(), 1);
    assert_eq!(state.events()[0].id, id);
    assert_eq!(state.events()[0].fields.len(), 2);
    assert_eq!(state.events()[0].fields[0], ("ms".into(), "1200".into()));
}

#[test]
fn test_push_event_ids_increment() {
    let mut state = EventStreamState::new();
    let id1 = state.push_event(EventLevel::Info, "a");
    let id2 = state.push_event(EventLevel::Info, "b");
    let id3 = state.push_event(EventLevel::Info, "c");
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(id3, 2);
}

#[test]
fn test_clear() {
    let mut state = sample_state();
    state.clear();
    assert_eq!(state.event_count(), 0);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_eviction_at_max() {
    let mut state = EventStreamState::new().with_max_events(3);
    state.push_event(EventLevel::Info, "a");
    state.push_event(EventLevel::Info, "b");
    state.push_event(EventLevel::Info, "c");
    state.push_event(EventLevel::Info, "d");
    assert_eq!(state.event_count(), 3);
    // Oldest entry ("a") should have been evicted
    assert_eq!(state.events()[0].message, "b");
}

#[test]
fn test_eviction_preserves_newest() {
    let mut state = EventStreamState::new().with_max_events(2);
    state.push_event(EventLevel::Info, "a");
    state.push_event(EventLevel::Info, "b");
    state.push_event(EventLevel::Info, "c");
    assert_eq!(state.event_count(), 2);
    assert_eq!(state.events()[0].message, "b");
    assert_eq!(state.events()[1].message, "c");
}

// =============================================================================
// Filtering: text filter
// =============================================================================

#[test]
fn test_text_filter_on_message() {
    let mut state = sample_state();
    state.set_filter("request".to_string());
    let visible = state.visible_events();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].message, "Request received");
}

#[test]
fn test_text_filter_case_insensitive() {
    let mut state = sample_state();
    state.set_filter("REQUEST".to_string());
    assert_eq!(state.visible_events().len(), 1);
}

#[test]
fn test_text_filter_on_field_value() {
    let mut state = sample_state();
    state.set_filter("users".to_string());
    let visible = state.visible_events();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].message, "Query executed");
}

#[test]
fn test_text_filter_on_field_key() {
    let mut state = sample_state();
    state.set_filter("table".to_string());
    let visible = state.visible_events();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].message, "Query executed");
}

#[test]
fn test_text_filter_on_source() {
    let mut state = sample_state_with_sources();
    state.set_filter("cache".to_string());
    let visible = state.visible_events();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].message, "Cache miss");
}

#[test]
fn test_text_filter_no_matches() {
    let mut state = sample_state();
    state.set_filter("zzz".to_string());
    assert!(state.visible_events().is_empty());
}

#[test]
fn test_text_filter_empty_shows_all() {
    let mut state = sample_state();
    state.set_filter(String::new());
    assert_eq!(state.visible_events().len(), 4);
}

// =============================================================================
// Filtering: level filter
// =============================================================================

#[test]
fn test_level_filter_info() {
    let mut state = sample_state();
    state.level_filter = Some(EventLevel::Info);
    let visible = state.visible_events();
    // Info, Warning, Error pass; Debug does not
    assert_eq!(visible.len(), 3);
}

#[test]
fn test_level_filter_warning() {
    let mut state = sample_state();
    state.level_filter = Some(EventLevel::Warning);
    let visible = state.visible_events();
    // Warning, Error pass; Info, Debug do not
    assert_eq!(visible.len(), 2);
}

#[test]
fn test_level_filter_error() {
    let mut state = sample_state();
    state.level_filter = Some(EventLevel::Error);
    let visible = state.visible_events();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].message, "Connection timeout");
}

#[test]
fn test_level_filter_trace_shows_all() {
    let mut state = sample_state();
    state.level_filter = Some(EventLevel::Trace);
    assert_eq!(state.visible_events().len(), 4);
}

#[test]
fn test_level_filter_none_shows_all() {
    let state = sample_state();
    assert!(state.level_filter().is_none());
    assert_eq!(state.visible_events().len(), 4);
}

// =============================================================================
// Filtering: source filter
// =============================================================================

#[test]
fn test_source_filter() {
    let mut state = sample_state_with_sources();
    state.source_filter = Some("api".to_string());
    let visible = state.visible_events();
    assert_eq!(visible.len(), 2);
}

#[test]
fn test_source_filter_case_insensitive() {
    let mut state = sample_state_with_sources();
    state.source_filter = Some("API".to_string());
    let visible = state.visible_events();
    assert_eq!(visible.len(), 2);
}

#[test]
fn test_source_filter_no_matches() {
    let mut state = sample_state_with_sources();
    state.source_filter = Some("nonexistent".to_string());
    assert!(state.visible_events().is_empty());
}

#[test]
fn test_source_filter_excludes_no_source() {
    let mut state = sample_state();
    // sample_state events have no source
    state.source_filter = Some("api".to_string());
    assert!(state.visible_events().is_empty());
}

// =============================================================================
// Filtering: combined filters
// =============================================================================

#[test]
fn test_combined_level_and_text_filter() {
    let mut state = sample_state_with_sources();
    state.level_filter = Some(EventLevel::Warning);
    state.set_filter("api".to_string());
    let visible = state.visible_events();
    // Only "Timeout" has source=api AND level >= Warning
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].message, "Timeout");
}

#[test]
fn test_combined_level_and_source_filter() {
    let mut state = sample_state_with_sources();
    state.level_filter = Some(EventLevel::Info);
    state.source_filter = Some("api".to_string());
    let visible = state.visible_events();
    // api events: Info "Request received" and Error "Timeout" -- both >= Info
    assert_eq!(visible.len(), 2);
}

#[test]
fn test_combined_all_filters() {
    let mut state = sample_state_with_sources();
    state.level_filter = Some(EventLevel::Info);
    state.source_filter = Some("api".to_string());
    state.set_filter("timeout".to_string());
    let visible = state.visible_events();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].message, "Timeout");
}

// =============================================================================
// Scroll
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = focused_state();
    // Disable auto-scroll to test manual scrolling
    state.auto_scroll = false;
    state.scroll.set_offset(0);
    EventStream::update(&mut state, EventStreamMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = focused_state();
    state.auto_scroll = false;
    state.scroll.set_offset(0);
    EventStream::update(&mut state, EventStreamMessage::ScrollDown);
    EventStream::update(&mut state, EventStreamMessage::ScrollDown);
    EventStream::update(&mut state, EventStreamMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = focused_state();
    state.auto_scroll = false;
    state.scroll.set_offset(0);
    EventStream::update(&mut state, EventStreamMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_top() {
    let mut state = focused_state();
    state.auto_scroll = false;
    state.scroll.set_offset(2);
    EventStream::update(&mut state, EventStreamMessage::ScrollToTop);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_bottom() {
    let mut state = focused_state();
    state.auto_scroll = false;
    state.scroll.set_offset(0);
    EventStream::update(&mut state, EventStreamMessage::ScrollToBottom);
    assert_eq!(state.scroll_offset(), 3);
}

#[test]
fn test_scroll_up_disables_auto_scroll() {
    let mut state = focused_state();
    assert!(state.auto_scroll());
    EventStream::update(&mut state, EventStreamMessage::ScrollUp);
    assert!(!state.auto_scroll());
}

#[test]
fn test_scroll_to_top_disables_auto_scroll() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::ScrollToTop);
    assert!(!state.auto_scroll());
}

#[test]
fn test_auto_scroll_on_new_event() {
    let mut state = EventStreamState::new();
    state.push_event(EventLevel::Info, "a");
    state.push_event(EventLevel::Info, "b");
    state.push_event(EventLevel::Info, "c");
    // auto_scroll is on, so scroll should be at end
    assert!(state.auto_scroll());
    // Before rendering, viewport_height is 0, so max_offset = content_length
    let len = state.visible_events().len();
    assert_eq!(state.scroll_offset(), len);
}

// =============================================================================
// Toggle auto-scroll
// =============================================================================

#[test]
fn test_toggle_auto_scroll() {
    let mut state = focused_state();
    assert!(state.auto_scroll());
    EventStream::update(&mut state, EventStreamMessage::ToggleAutoScroll);
    assert!(!state.auto_scroll());
    EventStream::update(&mut state, EventStreamMessage::ToggleAutoScroll);
    assert!(state.auto_scroll());
}

// =============================================================================
// Level coloring and types
// =============================================================================

#[test]
fn test_level_colors() {
    assert_eq!(EventLevel::Trace.color(), Color::DarkGray);
    assert_eq!(EventLevel::Debug.color(), Color::Gray);
    assert_eq!(EventLevel::Info.color(), Color::Blue);
    assert_eq!(EventLevel::Warning.color(), Color::Yellow);
    assert_eq!(EventLevel::Error.color(), Color::Red);
    assert_eq!(EventLevel::Fatal.color(), Color::LightRed);
}

#[test]
fn test_level_abbreviations() {
    assert_eq!(EventLevel::Trace.abbreviation(), "TRC");
    assert_eq!(EventLevel::Debug.abbreviation(), "DBG");
    assert_eq!(EventLevel::Info.abbreviation(), "INF");
    assert_eq!(EventLevel::Warning.abbreviation(), "WRN");
    assert_eq!(EventLevel::Error.abbreviation(), "ERR");
    assert_eq!(EventLevel::Fatal.abbreviation(), "FTL");
}

#[test]
fn test_level_ordering() {
    assert!(EventLevel::Trace < EventLevel::Debug);
    assert!(EventLevel::Debug < EventLevel::Info);
    assert!(EventLevel::Info < EventLevel::Warning);
    assert!(EventLevel::Warning < EventLevel::Error);
    assert!(EventLevel::Error < EventLevel::Fatal);
}

#[test]
fn test_level_default() {
    assert_eq!(EventLevel::default(), EventLevel::Info);
}

#[test]
fn test_level_display() {
    assert_eq!(format!("{}", EventLevel::Info), "INF");
    assert_eq!(format!("{}", EventLevel::Error), "ERR");
}

// =============================================================================
// StreamEvent
// =============================================================================

#[test]
fn test_stream_event_new() {
    let event = StreamEvent::new(1, 1000.0, EventLevel::Info, "hello");
    assert_eq!(event.id, 1);
    assert!((event.timestamp - 1000.0).abs() < f64::EPSILON);
    assert_eq!(event.level, EventLevel::Info);
    assert_eq!(event.message, "hello");
    assert!(event.fields.is_empty());
    assert!(event.source.is_none());
}

#[test]
fn test_stream_event_with_field() {
    let event = StreamEvent::new(1, 0.0, EventLevel::Info, "msg")
        .with_field("key", "value")
        .with_field("key2", "value2");
    assert_eq!(event.fields.len(), 2);
    assert_eq!(event.fields[0], ("key".to_string(), "value".to_string()));
}

#[test]
fn test_stream_event_with_source() {
    let event = StreamEvent::new(1, 0.0, EventLevel::Info, "msg").with_source("api");
    assert_eq!(event.source, Some("api".to_string()));
}

// =============================================================================
// Messages
// =============================================================================

#[test]
fn test_push_event_message() {
    let mut state = EventStreamState::new();
    state.set_focused(true);
    let event = StreamEvent::new(0, 100.0, EventLevel::Info, "test");
    let output = EventStream::update(&mut state, EventStreamMessage::PushEvent(event));
    assert_eq!(state.event_count(), 1);
    assert!(matches!(output, Some(EventStreamOutput::EventAdded(_))));
}

#[test]
fn test_push_event_message_auto_assigns_id() {
    let mut state = EventStreamState::new();
    state.push_event(EventLevel::Info, "first");
    let event = StreamEvent::new(0, 100.0, EventLevel::Info, "second");
    EventStream::update(&mut state, EventStreamMessage::PushEvent(event));
    assert_eq!(state.event_count(), 2);
    // IDs should be unique
    assert_ne!(state.events()[0].id, state.events()[1].id);
}

#[test]
fn test_push_event_message_with_explicit_id() {
    let mut state = EventStreamState::new();
    let event = StreamEvent::new(42, 100.0, EventLevel::Info, "test");
    EventStream::update(&mut state, EventStreamMessage::PushEvent(event));
    assert_eq!(state.events()[0].id, 42);
    // next_id should be updated to 43
    assert_eq!(state.next_id, 43);
}

#[test]
fn test_set_events_message() {
    let mut state = sample_state();
    let events = vec![
        StreamEvent::new(10, 0.0, EventLevel::Info, "a"),
        StreamEvent::new(20, 0.0, EventLevel::Error, "b"),
    ];
    EventStream::update(&mut state, EventStreamMessage::SetEvents(events));
    assert_eq!(state.event_count(), 2);
    assert_eq!(state.events()[0].id, 10);
    assert_eq!(state.events()[1].id, 20);
}

#[test]
fn test_clear_message() {
    let mut state = sample_state();
    let output = EventStream::update(&mut state, EventStreamMessage::Clear);
    assert_eq!(state.event_count(), 0);
    assert_eq!(output, Some(EventStreamOutput::EventsCleared));
}

#[test]
fn test_set_filter_message() {
    let mut state = focused_state();
    let output = EventStream::update(
        &mut state,
        EventStreamMessage::SetFilter("request".to_string()),
    );
    assert_eq!(state.filter_text(), "request");
    assert_eq!(output, Some(EventStreamOutput::FilterChanged));
}

#[test]
fn test_set_level_filter_message() {
    let mut state = focused_state();
    let output = EventStream::update(
        &mut state,
        EventStreamMessage::SetLevelFilter(Some(EventLevel::Warning)),
    );
    assert_eq!(state.level_filter(), Some(&EventLevel::Warning));
    assert_eq!(output, Some(EventStreamOutput::FilterChanged));
}

#[test]
fn test_set_source_filter_message() {
    let mut state = focused_state();
    let output = EventStream::update(
        &mut state,
        EventStreamMessage::SetSourceFilter(Some("api".to_string())),
    );
    assert_eq!(state.source_filter(), Some("api"));
    assert_eq!(output, Some(EventStreamOutput::FilterChanged));
}

#[test]
fn test_set_visible_columns_message() {
    let mut state = focused_state();
    EventStream::update(
        &mut state,
        EventStreamMessage::SetVisibleColumns(vec!["path".into()]),
    );
    assert_eq!(state.visible_columns(), &["path".to_string()]);
}

// =============================================================================
// Quick level filter
// =============================================================================

#[test]
fn test_quick_level_filter_1_trace() {
    let mut state = focused_state();
    let output = EventStream::update(&mut state, EventStreamMessage::QuickLevelFilter(1));
    assert_eq!(state.level_filter(), Some(&EventLevel::Trace));
    assert_eq!(output, Some(EventStreamOutput::FilterChanged));
}

#[test]
fn test_quick_level_filter_3_info() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::QuickLevelFilter(3));
    assert_eq!(state.level_filter(), Some(&EventLevel::Info));
}

#[test]
fn test_quick_level_filter_toggle_off() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::QuickLevelFilter(3));
    assert_eq!(state.level_filter(), Some(&EventLevel::Info));
    // Toggle same level again clears filter
    EventStream::update(&mut state, EventStreamMessage::QuickLevelFilter(3));
    assert!(state.level_filter().is_none());
}

#[test]
fn test_quick_level_filter_switch() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::QuickLevelFilter(3));
    assert_eq!(state.level_filter(), Some(&EventLevel::Info));
    EventStream::update(&mut state, EventStreamMessage::QuickLevelFilter(4));
    assert_eq!(state.level_filter(), Some(&EventLevel::Warning));
}

// =============================================================================
// Search bar + disabled/unfocused + instance methods + traits + edge cases
// =============================================================================

#[test]
fn test_focus_search() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    assert!(state.is_search_focused());
}

#[test]
fn test_focus_list() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    EventStream::update(&mut state, EventStreamMessage::FocusList);
    assert!(!state.is_search_focused());
}

#[test]
fn test_clear_search() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    EventStream::update(&mut state, EventStreamMessage::SearchInput('x'));
    assert_eq!(state.filter_text(), "x");

    EventStream::update(&mut state, EventStreamMessage::ClearSearch);
    assert_eq!(state.filter_text(), "");
    assert!(!state.is_search_focused());
}

#[test]
fn test_search_backspace() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::SearchInput('a'));
    EventStream::update(&mut state, EventStreamMessage::SearchInput('b'));
    EventStream::update(&mut state, EventStreamMessage::SearchBackspace);
    assert_eq!(state.filter_text(), "a");
}

#[test]
fn test_search_resets_scroll() {
    let mut state = focused_state();
    state.auto_scroll = false;
    state.scroll.set_offset(0);
    EventStream::update(&mut state, EventStreamMessage::ScrollDown);
    EventStream::update(&mut state, EventStreamMessage::ScrollDown);
    assert!(state.scroll_offset() > 0);

    EventStream::update(&mut state, EventStreamMessage::SearchInput('a'));
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_disabled_ignores_messages() {
    let mut state = focused_state();
    state.set_disabled(true);
    let output = EventStream::update(&mut state, EventStreamMessage::ScrollDown);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = EventStream::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_unfocused_ignores_events() {
    let state = sample_state();
    let msg =
        EventStream::handle_event(&state, &Event::key(KeyCode::Down), &ViewContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::key(KeyCode::Down));
    assert_eq!(msg, Some(EventStreamMessage::ScrollDown));
}

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    state.auto_scroll = false;
    state.scroll.set_offset(0);
    state.update(EventStreamMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state();
    state.auto_scroll = false;
    state.scroll.set_offset(0);
    state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_focusable_trait() {
    let mut state = EventStream::init();
    assert!(!EventStream::is_focused(&state));

    EventStream::focus(&mut state);
    assert!(EventStream::is_focused(&state));

    EventStream::blur(&mut state);
    assert!(!EventStream::is_focused(&state));
}

#[test]
fn test_disableable_trait() {
    let mut state = EventStream::init();
    assert!(!EventStream::is_disabled(&state));

    EventStream::disable(&mut state);
    assert!(EventStream::is_disabled(&state));

    EventStream::enable(&mut state);
    assert!(!EventStream::is_disabled(&state));
}

#[test]
fn test_partial_eq() {
    let state1 = sample_state();
    let state2 = sample_state();
    assert_eq!(state1, state2);
}

#[test]
fn test_partial_eq_different_focus() {
    let state1 = focused_state();
    let state2 = sample_state();
    assert_ne!(state1, state2);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_scroll_empty_stream() {
    let mut state = EventStreamState::new();
    EventStream::set_focused(&mut state, true);
    EventStream::update(&mut state, EventStreamMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_empty_fields() {
    let mut state = EventStreamState::new();
    state.push_event_with_fields(EventLevel::Info, "msg", vec![]);
    assert!(state.events()[0].fields.is_empty());
}

#[test]
fn test_very_long_message() {
    let mut state = EventStreamState::new();
    let long_msg = "x".repeat(500);
    state.push_event(EventLevel::Info, long_msg.as_str());
    assert_eq!(state.events()[0].message.len(), 500);
}

#[test]
fn test_max_events_eviction_via_push() {
    let mut state = EventStreamState::new().with_max_events(2);
    let id1 = state.push_event(EventLevel::Info, "a");
    let _id2 = state.push_event(EventLevel::Info, "b");
    let _id3 = state.push_event(EventLevel::Info, "c");
    assert_eq!(state.event_count(), 2);
    // First event should be evicted
    assert!(state.events().iter().all(|e| e.id != id1));
}

#[test]
fn test_filter_reapplied_after_search_clear() {
    let mut state = focused_state();
    // Set level filter to Error only
    state.level_filter = Some(EventLevel::Error);
    assert_eq!(state.visible_events().len(), 1);

    // Search narrows further
    EventStream::update(
        &mut state,
        EventStreamMessage::SetFilter("nonexistent".to_string()),
    );
    assert_eq!(state.visible_events().len(), 0);

    // Clear search -- should show errors again
    EventStream::update(&mut state, EventStreamMessage::SetFilter(String::new()));
    assert_eq!(state.visible_events().len(), 1);
}

#[test]
fn test_search_cursor_navigation() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    EventStream::update(&mut state, EventStreamMessage::SearchInput('a'));
    EventStream::update(&mut state, EventStreamMessage::SearchInput('b'));
    EventStream::update(&mut state, EventStreamMessage::SearchInput('c'));
    EventStream::update(&mut state, EventStreamMessage::SearchHome);
    EventStream::update(&mut state, EventStreamMessage::SearchDelete);
    assert_eq!(state.filter_text(), "bc");
    EventStream::update(&mut state, EventStreamMessage::SearchEnd);
    EventStream::update(&mut state, EventStreamMessage::SearchBackspace);
    assert_eq!(state.filter_text(), "b");
}

#[test]
fn test_set_max_events_evicts_oldest() {
    let mut state = EventStreamState::new();
    state.push_event(EventLevel::Info, "a");
    state.push_event(EventLevel::Info, "b");
    state.push_event(EventLevel::Info, "c");
    state.push_event(EventLevel::Info, "d");
    state.push_event(EventLevel::Info, "e");
    assert_eq!(state.events().len(), 5);

    state.set_max_events(2);
    assert_eq!(state.events().len(), 2);
    assert_eq!(state.events()[0].message, "d");
    assert_eq!(state.events()[1].message, "e");
}

#[test]
fn test_set_max_events_no_eviction_when_under_limit() {
    let mut state = EventStreamState::new();
    state.push_event(EventLevel::Info, "a");
    state.push_event(EventLevel::Info, "b");
    assert_eq!(state.events().len(), 2);

    state.set_max_events(10);
    assert_eq!(state.events().len(), 2);
}
