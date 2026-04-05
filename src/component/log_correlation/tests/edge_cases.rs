use super::*;

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = two_stream_state();
    let state2 = two_stream_state();
    assert_eq!(state1, state2);
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
                LogCorrelation::view(
                    &state,
                    frame,
                    frame.area(),
                    &theme,
                    &ViewContext::new().focused(true),
                );
            })
            .unwrap();
    });
    assert!(registry.get_by_id("log_correlation").is_some());
}
