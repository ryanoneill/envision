use super::*;

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
    let mut state = LogCorrelationState::new().with_streams(vec![
        LogStream::new("A")
            .with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Debug, "dbg"))
            .with_entry(CorrelationEntry::new(2.0, CorrelationLevel::Info, "inf"))
            .with_entry(CorrelationEntry::new(3.0, CorrelationLevel::Warning, "wrn"))
            .with_entry(CorrelationEntry::new(4.0, CorrelationLevel::Error, "err")),
    ]);

    state.streams[0].min_level = Some(CorrelationLevel::Info);
    let filtered = state.streams[0].filtered_entries();
    assert_eq!(filtered.len(), 3); // Info, Warning, Error

    state.streams[0].min_level = Some(CorrelationLevel::Error);
    let filtered = state.streams[0].filtered_entries();
    assert_eq!(filtered.len(), 1); // Error only
}

#[test]
fn test_combined_text_and_level_filter() {
    let mut state = LogCorrelationState::new().with_streams(vec![
        LogStream::new("A")
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
            )),
    ]);

    state.streams[0].filter = "query".to_string();
    state.streams[0].min_level = Some(CorrelationLevel::Info);

    let filtered = state.streams[0].filtered_entries();
    // Only "info query" passes both filters
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].message, "info query");
}
