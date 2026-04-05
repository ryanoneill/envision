use super::*;

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
