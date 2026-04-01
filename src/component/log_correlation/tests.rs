use super::*;
use crate::component::test_utils;

fn entry(ts: f64, level: CorrelationLevel, msg: &str) -> CorrelationEntry {
    CorrelationEntry::new(ts, level, msg)
}

fn two_stream_state() -> LogCorrelationState {
    use CorrelationLevel::*;
    let api = LogStream::new("API Server")
        .with_color(Color::Cyan)
        .with_entry(entry(1.0, Info, "Request received"))
        .with_entry(entry(1.0, Debug, "Parsing body"))
        .with_entry(entry(2.0, Info, "Query sent"))
        .with_entry(entry(3.0, Info, "Response sent"))
        .with_entry(entry(3.0, Warning, "Slow response"));

    let db = LogStream::new("Database")
        .with_color(Color::Green)
        .with_entry(entry(1.0, Info, "Connected"))
        .with_entry(entry(2.0, Info, "Query start"))
        .with_entry(entry(2.0, Debug, "Query plan"))
        .with_entry(entry(3.0, Info, "Query done"))
        .with_entry(entry(3.0, Warning, "Slow query"));

    LogCorrelationState::new().with_streams(vec![api, db])
}

fn focused_state() -> LogCorrelationState {
    let mut state = two_stream_state();
    LogCorrelation::set_focused(&mut state, true);
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = LogCorrelationState::new();
    assert_eq!(state.stream_count(), 0);
    assert!(state.sync_scroll());
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert_eq!(state.title(), None);
    assert_eq!(state.active_stream(), 0);
}

#[test]
fn test_default() {
    let state = LogCorrelationState::default();
    assert_eq!(state.stream_count(), 0);
    assert!(state.sync_scroll());
}

#[test]
fn test_with_streams() {
    let state =
        LogCorrelationState::new().with_streams(vec![LogStream::new("A"), LogStream::new("B")]);
    assert_eq!(state.stream_count(), 2);
    assert_eq!(state.streams()[0].name, "A");
    assert_eq!(state.streams()[1].name, "B");
}

#[test]
fn test_with_title() {
    let state = LogCorrelationState::new().with_title("Logs");
    assert_eq!(state.title(), Some("Logs"));
}

#[test]
fn test_with_sync_scroll() {
    let state = LogCorrelationState::new().with_sync_scroll(false);
    assert!(!state.sync_scroll());
}

#[test]
fn test_with_disabled() {
    let state = LogCorrelationState::new().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// LogStream
// =============================================================================

#[test]
fn test_log_stream_new() {
    let stream = LogStream::new("Test");
    assert_eq!(stream.name, "Test");
    assert_eq!(stream.color, Color::White);
    assert!(stream.entries.is_empty());
    assert!(stream.filter.is_empty());
    assert_eq!(stream.min_level, None);
}

#[test]
fn test_log_stream_with_color() {
    let stream = LogStream::new("Test").with_color(Color::Red);
    assert_eq!(stream.color, Color::Red);
}

#[test]
fn test_log_stream_with_entry() {
    let stream = LogStream::new("Test").with_entry(CorrelationEntry::new(
        1.0,
        CorrelationLevel::Info,
        "msg",
    ));
    assert_eq!(stream.entries.len(), 1);
    assert_eq!(stream.entries[0].message, "msg");
}

// =============================================================================
// CorrelationEntry
// =============================================================================

#[test]
fn test_correlation_entry_new() {
    let entry = CorrelationEntry::new(1.5, CorrelationLevel::Warning, "alert");
    assert_eq!(entry.timestamp, 1.5);
    assert_eq!(entry.level, CorrelationLevel::Warning);
    assert_eq!(entry.message, "alert");
}

// =============================================================================
// CorrelationLevel
// =============================================================================

#[test]
fn test_level_ordering() {
    assert!(CorrelationLevel::Debug < CorrelationLevel::Info);
    assert!(CorrelationLevel::Info < CorrelationLevel::Warning);
    assert!(CorrelationLevel::Warning < CorrelationLevel::Error);
}

#[test]
fn test_level_default() {
    assert_eq!(CorrelationLevel::default(), CorrelationLevel::Info);
}

#[test]
fn test_level_labels() {
    assert_eq!(CorrelationLevel::Debug.label(), "DBG");
    assert_eq!(CorrelationLevel::Info.label(), "INF");
    assert_eq!(CorrelationLevel::Warning.label(), "WRN");
    assert_eq!(CorrelationLevel::Error.label(), "ERR");
}

#[test]
fn test_level_colors() {
    assert_eq!(CorrelationLevel::Debug.color(), Color::DarkGray);
    assert_eq!(CorrelationLevel::Info.color(), Color::Blue);
    assert_eq!(CorrelationLevel::Warning.color(), Color::Yellow);
    assert_eq!(CorrelationLevel::Error.color(), Color::Red);
}

#[test]
fn test_level_display() {
    assert_eq!(format!("{}", CorrelationLevel::Info), "INF");
    assert_eq!(format!("{}", CorrelationLevel::Error), "ERR");
}

// =============================================================================
// Stream operations
// =============================================================================

#[test]
fn test_add_stream() {
    let mut state = LogCorrelationState::new();
    state.add_stream(LogStream::new("API"));
    assert_eq!(state.stream_count(), 1);
    state.add_stream(LogStream::new("DB"));
    assert_eq!(state.stream_count(), 2);
}

#[test]
fn test_push_entry() {
    let mut state =
        LogCorrelationState::new().with_streams(vec![LogStream::new("API"), LogStream::new("DB")]);

    state.push_entry(
        0,
        CorrelationEntry::new(1.0, CorrelationLevel::Info, "hello"),
    );
    assert_eq!(state.streams()[0].entries.len(), 1);
    assert_eq!(state.streams()[1].entries.len(), 0);

    state.push_entry(
        1,
        CorrelationEntry::new(1.0, CorrelationLevel::Info, "world"),
    );
    assert_eq!(state.streams()[1].entries.len(), 1);
}

#[test]
fn test_push_entry_out_of_bounds() {
    let mut state = LogCorrelationState::new().with_streams(vec![LogStream::new("API")]);
    state.push_entry(5, CorrelationEntry::new(1.0, CorrelationLevel::Info, "oob"));
    // Should not panic; entry is silently dropped
    assert_eq!(state.streams()[0].entries.len(), 0);
}

// =============================================================================
// Time alignment
// =============================================================================

#[test]
fn test_aligned_rows_basic() {
    let state = two_stream_state();
    let rows = state.aligned_rows();

    // Should have 3 timestamp groups: 1.0, 2.0, 3.0
    assert_eq!(rows.len(), 3);

    // Timestamp 1.0: API has 2 entries, DB has 1
    assert_eq!(rows[0].timestamp, 1.0);
    assert_eq!(rows[0].stream_entries[0].len(), 2);
    assert_eq!(rows[0].stream_entries[1].len(), 1);

    // Timestamp 2.0: API has 1 entry, DB has 2
    assert_eq!(rows[1].timestamp, 2.0);
    assert_eq!(rows[1].stream_entries[0].len(), 1);
    assert_eq!(rows[1].stream_entries[1].len(), 2);

    // Timestamp 3.0: API has 2 entries, DB has 2
    assert_eq!(rows[2].timestamp, 3.0);
    assert_eq!(rows[2].stream_entries[0].len(), 2);
    assert_eq!(rows[2].stream_entries[1].len(), 2);
}

#[test]
fn test_aligned_rows_empty() {
    let state = LogCorrelationState::new();
    let rows = state.aligned_rows();
    assert!(rows.is_empty());
}

#[test]
fn test_aligned_rows_single_stream() {
    let state = LogCorrelationState::new().with_streams(vec![LogStream::new("API")
        .with_entry(CorrelationEntry::new(
            1.0,
            CorrelationLevel::Info,
            "entry 1",
        ))
        .with_entry(CorrelationEntry::new(
            2.0,
            CorrelationLevel::Info,
            "entry 2",
        ))]);

    let rows = state.aligned_rows();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].stream_entries[0].len(), 1);
    assert_eq!(rows[1].stream_entries[0].len(), 1);
}

#[test]
fn test_aligned_rows_mismatched_timestamps() {
    let state = LogCorrelationState::new().with_streams(vec![
        LogStream::new("A")
            .with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "a1"))
            .with_entry(CorrelationEntry::new(3.0, CorrelationLevel::Info, "a2")),
        LogStream::new("B")
            .with_entry(CorrelationEntry::new(2.0, CorrelationLevel::Info, "b1"))
            .with_entry(CorrelationEntry::new(4.0, CorrelationLevel::Info, "b2")),
    ]);

    let rows = state.aligned_rows();
    assert_eq!(rows.len(), 4); // 1.0, 2.0, 3.0, 4.0

    // At ts 1.0: A has 1, B has 0
    assert_eq!(rows[0].stream_entries[0].len(), 1);
    assert!(rows[0].stream_entries[1].is_empty());

    // At ts 2.0: A has 0, B has 1
    assert!(rows[1].stream_entries[0].is_empty());
    assert_eq!(rows[1].stream_entries[1].len(), 1);
}

#[test]
fn test_total_display_rows() {
    let state = two_stream_state();
    let total = state.total_display_rows();
    // ts 1.0: max(2,1)=2, ts 2.0: max(1,2)=2, ts 3.0: max(2,2)=2
    assert_eq!(total, 6);
}

#[test]
fn test_timestamp_tolerance() {
    // Two entries within 100ms tolerance should align
    let state = LogCorrelationState::new().with_streams(vec![
        LogStream::new("A").with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "a1")),
        LogStream::new("B").with_entry(CorrelationEntry::new(1.05, CorrelationLevel::Info, "b1")),
    ]);

    let rows = state.aligned_rows();
    // Should be grouped into one row (within 0.1s tolerance)
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].stream_entries[0].len(), 1);
    assert_eq!(rows[0].stream_entries[1].len(), 1);
}

#[test]
fn test_timestamp_beyond_tolerance() {
    // Two entries beyond 100ms tolerance should be separate rows
    let state = LogCorrelationState::new().with_streams(vec![
        LogStream::new("A").with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "a1")),
        LogStream::new("B").with_entry(CorrelationEntry::new(1.2, CorrelationLevel::Info, "b1")),
    ]);

    let rows = state.aligned_rows();
    assert_eq!(rows.len(), 2);
}

// =============================================================================
// Per-stream filtering
// =============================================================================

#[test]
fn test_text_filter() {
    let mut state = two_stream_state();
    state.streams[0].filter = "Query".to_string();

    let rows = state.aligned_rows();
    // API stream should only have "Query sent" at ts 2.0
    // Find the row at ts 2.0
    let ts2_row = rows
        .iter()
        .find(|r| (r.timestamp - 2.0).abs() < f64::EPSILON);
    assert!(ts2_row.is_some());
    let ts2_row = ts2_row.unwrap();
    assert_eq!(ts2_row.stream_entries[0].len(), 1);

    // ts 1.0 should have no API entries (filtered out)
    let ts1_row = rows
        .iter()
        .find(|r| (r.timestamp - 1.0).abs() < f64::EPSILON);
    assert!(ts1_row.is_some());
    let ts1_row = ts1_row.unwrap();
    assert!(ts1_row.stream_entries[0].is_empty());
}

#[test]
fn test_text_filter_case_insensitive() {
    let mut state = LogCorrelationState::new().with_streams(vec![LogStream::new("A").with_entry(
        CorrelationEntry::new(1.0, CorrelationLevel::Info, "Hello World"),
    )]);

    state.streams[0].filter = "hello".to_string();
    let filtered = state.streams[0].filtered_entries();
    assert_eq!(filtered.len(), 1);
}

#[test]
fn test_level_filter() {
    let mut state = two_stream_state();
    state.streams[0].min_level = Some(CorrelationLevel::Warning);

    let filtered = state.streams[0].filtered_entries();
    // Only Warning entries from API stream: "Slow response"
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].message, "Slow response");
}

#[test]
fn test_level_filter_excludes_lower() {
    let mut state = LogCorrelationState::new().with_streams(vec![LogStream::new("A")
        .with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Debug, "dbg"))
        .with_entry(CorrelationEntry::new(2.0, CorrelationLevel::Info, "inf"))
        .with_entry(CorrelationEntry::new(3.0, CorrelationLevel::Warning, "wrn"))
        .with_entry(CorrelationEntry::new(4.0, CorrelationLevel::Error, "err"))]);

    state.streams[0].min_level = Some(CorrelationLevel::Info);
    let filtered = state.streams[0].filtered_entries();
    assert_eq!(filtered.len(), 3); // Info, Warning, Error

    state.streams[0].min_level = Some(CorrelationLevel::Error);
    let filtered = state.streams[0].filtered_entries();
    assert_eq!(filtered.len(), 1); // Error only
}

#[test]
fn test_combined_text_and_level_filter() {
    let mut state = LogCorrelationState::new().with_streams(vec![LogStream::new("A")
        .with_entry(CorrelationEntry::new(
            1.0,
            CorrelationLevel::Debug,
            "debug query",
        ))
        .with_entry(CorrelationEntry::new(
            2.0,
            CorrelationLevel::Info,
            "info query",
        ))
        .with_entry(CorrelationEntry::new(
            3.0,
            CorrelationLevel::Warning,
            "warn other",
        ))]);

    state.streams[0].filter = "query".to_string();
    state.streams[0].min_level = Some(CorrelationLevel::Info);

    let filtered = state.streams[0].filtered_entries();
    // Only "info query" passes both filters
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].message, "info query");
}

// =============================================================================
// Sync scroll
// =============================================================================

#[test]
fn test_sync_scroll_default() {
    let state = LogCorrelationState::new();
    assert!(state.sync_scroll());
}

#[test]
fn test_toggle_sync_scroll() {
    let mut state = focused_state();
    LogCorrelation::update(&mut state, LogCorrelationMessage::ToggleSyncScroll);
    assert!(!state.sync_scroll());
    LogCorrelation::update(&mut state, LogCorrelationMessage::ToggleSyncScroll);
    assert!(state.sync_scroll());
}

// =============================================================================
// Active stream
// =============================================================================

#[test]
fn test_focus_next_stream() {
    let mut state = focused_state();
    assert_eq!(state.active_stream(), 0);

    let output = LogCorrelation::update(&mut state, LogCorrelationMessage::FocusNextStream);
    assert_eq!(state.active_stream(), 1);
    assert_eq!(output, Some(LogCorrelationOutput::StreamFocused(1)));

    // Wraps around
    let output = LogCorrelation::update(&mut state, LogCorrelationMessage::FocusNextStream);
    assert_eq!(state.active_stream(), 0);
    assert_eq!(output, Some(LogCorrelationOutput::StreamFocused(0)));
}

#[test]
fn test_focus_prev_stream() {
    let mut state = focused_state();
    assert_eq!(state.active_stream(), 0);

    let output = LogCorrelation::update(&mut state, LogCorrelationMessage::FocusPrevStream);
    assert_eq!(state.active_stream(), 1);
    assert_eq!(output, Some(LogCorrelationOutput::StreamFocused(1)));

    let output = LogCorrelation::update(&mut state, LogCorrelationMessage::FocusPrevStream);
    assert_eq!(state.active_stream(), 0);
    assert_eq!(output, Some(LogCorrelationOutput::StreamFocused(0)));
}

#[test]
fn test_focus_stream_no_streams() {
    let mut state = LogCorrelationState::new();
    state.set_focused(true);

    let output = LogCorrelation::update(&mut state, LogCorrelationMessage::FocusNextStream);
    assert_eq!(output, None);

    let output = LogCorrelation::update(&mut state, LogCorrelationMessage::FocusPrevStream);
    assert_eq!(output, None);
}

// =============================================================================
// Scrolling
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = focused_state();
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = focused_state();
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollDown);
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollDown);
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = focused_state();
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_top() {
    let mut state = focused_state();
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollDown);
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollDown);
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollToTop);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_bottom() {
    let mut state = focused_state();
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollToBottom);
    assert!(state.scroll_offset() > 0);
}

#[test]
fn test_scroll_updates_timestamp() {
    let mut state = focused_state();
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollDown);
    // After scrolling, timestamp should still be at or near the first group
    assert!(state.scroll_timestamp() >= 0.0);
}

// =============================================================================
// Messages via update
// =============================================================================

#[test]
fn test_add_stream_message() {
    let mut state = focused_state();
    LogCorrelation::update(
        &mut state,
        LogCorrelationMessage::AddStream(LogStream::new("Cache")),
    );
    assert_eq!(state.stream_count(), 3);
}

#[test]
fn test_set_streams_message() {
    let mut state = focused_state();
    LogCorrelation::update(
        &mut state,
        LogCorrelationMessage::SetStreams(vec![LogStream::new("New")]),
    );
    assert_eq!(state.stream_count(), 1);
    assert_eq!(state.active_stream(), 0);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_push_entry_message() {
    let mut state = focused_state();
    let output = LogCorrelation::update(
        &mut state,
        LogCorrelationMessage::PushEntry {
            stream: 0,
            entry: CorrelationEntry::new(4.0, CorrelationLevel::Info, "new entry"),
        },
    );
    assert_eq!(output, Some(LogCorrelationOutput::EntryAdded { stream: 0 }));
    assert_eq!(state.streams()[0].entries.len(), 6);
}

#[test]
fn test_push_entry_oob_message() {
    let mut state = focused_state();
    let output = LogCorrelation::update(
        &mut state,
        LogCorrelationMessage::PushEntry {
            stream: 99,
            entry: CorrelationEntry::new(4.0, CorrelationLevel::Info, "oob"),
        },
    );
    assert_eq!(output, None);
}

#[test]
fn test_clear_message() {
    let mut state = focused_state();
    LogCorrelation::update(&mut state, LogCorrelationMessage::Clear);
    for stream in state.streams() {
        assert!(stream.entries.is_empty());
    }
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_set_stream_filter_message() {
    let mut state = focused_state();
    LogCorrelation::update(
        &mut state,
        LogCorrelationMessage::SetStreamFilter {
            stream: 0,
            filter: "Query".to_string(),
        },
    );
    assert_eq!(state.streams()[0].filter, "Query");
}

#[test]
fn test_set_stream_level_filter_message() {
    let mut state = focused_state();
    LogCorrelation::update(
        &mut state,
        LogCorrelationMessage::SetStreamLevelFilter {
            stream: 1,
            level: Some(CorrelationLevel::Warning),
        },
    );
    assert_eq!(
        state.streams()[1].min_level,
        Some(CorrelationLevel::Warning)
    );
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = LogCorrelation::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

#[test]
fn test_disabled_ignores_update() {
    let mut state = focused_state();
    state.set_disabled(true);
    let output = LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollDown);
    assert_eq!(output, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = two_stream_state();
    let msg = LogCorrelation::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_up_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(LogCorrelationMessage::ScrollUp)
    );
    assert_eq!(
        LogCorrelation::handle_event(&state, &Event::char('k')),
        Some(LogCorrelationMessage::ScrollUp)
    );
}

#[test]
fn test_down_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(LogCorrelationMessage::ScrollDown)
    );
    assert_eq!(
        LogCorrelation::handle_event(&state, &Event::char('j')),
        Some(LogCorrelationMessage::ScrollDown)
    );
}

#[test]
fn test_home_end_keys() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(LogCorrelationMessage::ScrollToTop)
    );
    assert_eq!(
        LogCorrelation::handle_event(&state, &Event::key(KeyCode::End)),
        Some(LogCorrelationMessage::ScrollToBottom)
    );
}

#[test]
fn test_tab_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(&state, &Event::key(KeyCode::Tab)),
        Some(LogCorrelationMessage::FocusNextStream)
    );
}

#[test]
fn test_backtab_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(&state, &Event::key(KeyCode::BackTab)),
        Some(LogCorrelationMessage::FocusPrevStream)
    );
}

#[test]
fn test_s_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(&state, &Event::char('s')),
        Some(LogCorrelationMessage::ToggleSyncScroll)
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::key(KeyCode::Down));
    assert_eq!(msg, Some(LogCorrelationMessage::ScrollDown));
}

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    state.update(LogCorrelationMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state();
    state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(state.scroll_offset(), 1);
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = LogCorrelationState::new();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_two_streams() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = two_stream_state().with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_title() {
    let state = LogCorrelationState::new()
        .with_title("Log Correlation")
        .with_streams(vec![
            LogStream::new("API").with_entry(CorrelationEntry::new(
                1.0,
                CorrelationLevel::Info,
                "ok",
            )),
            LogStream::new("DB").with_entry(CorrelationEntry::new(
                1.0,
                CorrelationLevel::Info,
                "ok",
            )),
        ]);
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_with_filter() {
    let mut state = two_stream_state();
    state.streams[0].filter = "Query".to_string();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 2);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_narrow_area() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(20, 10);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = LogCorrelation::init();
    assert!(!LogCorrelation::is_focused(&state));

    LogCorrelation::focus(&mut state);
    assert!(LogCorrelation::is_focused(&state));

    LogCorrelation::blur(&mut state);
    assert!(!LogCorrelation::is_focused(&state));
}

// =============================================================================
// Disableable trait
// =============================================================================

#[test]
fn test_disableable_trait() {
    let mut state = LogCorrelation::init();
    assert!(!LogCorrelation::is_disabled(&state));

    LogCorrelation::disable(&mut state);
    assert!(LogCorrelation::is_disabled(&state));

    LogCorrelation::enable(&mut state);
    assert!(!LogCorrelation::is_disabled(&state));
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = two_stream_state();
    let state2 = two_stream_state();
    assert_eq!(state1, state2);
}

#[test]
fn test_partial_eq_different_focus() {
    let state1 = focused_state();
    let state2 = two_stream_state();
    assert_ne!(state1, state2);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_empty_streams() {
    let state =
        LogCorrelationState::new().with_streams(vec![LogStream::new("A"), LogStream::new("B")]);
    let rows = state.aligned_rows();
    assert!(rows.is_empty());
    assert_eq!(state.total_display_rows(), 0);
}

#[test]
fn test_single_stream_operation() {
    let mut state = LogCorrelationState::new().with_streams(vec![LogStream::new("Single")]);
    state.push_entry(
        0,
        CorrelationEntry::new(1.0, CorrelationLevel::Info, "only"),
    );
    let rows = state.aligned_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].stream_entries[0].len(), 1);
}

#[test]
fn test_four_streams() {
    let state = LogCorrelationState::new().with_streams(vec![
        LogStream::new("A").with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "a")),
        LogStream::new("B").with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "b")),
        LogStream::new("C").with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "c")),
        LogStream::new("D").with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "d")),
    ]);
    assert_eq!(state.stream_count(), 4);
    let rows = state.aligned_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].stream_entries.len(), 4);
}

#[test]
fn test_scroll_empty_streams() {
    let mut state = LogCorrelationState::new();
    state.set_focused(true);
    LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 0);
}

// =============================================================================
// Annotation
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = LogCorrelationState::new();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                LogCorrelation::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert!(registry.get_by_id("log_correlation").is_some());
}
