use super::*;

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = LogCorrelationState::new();
    assert_eq!(state.stream_count(), 0);
    assert!(state.sync_scroll());
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
