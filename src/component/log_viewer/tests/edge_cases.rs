use super::*;

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = sample_state();
    let state2 = sample_state();
    assert_eq!(state1, state2);
}

#[test]
fn test_partial_eq_same_state() {
    let state1 = focused_state();
    let state2 = sample_state();
    assert_eq!(state1, state2);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_scroll_empty_log() {
    let mut state = LogViewerState::new();
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_search_cursor_navigation() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('a'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('b'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('c'));
    LogViewer::update(&mut state, LogViewerMessage::SearchHome);
    LogViewer::update(&mut state, LogViewerMessage::SearchDelete);
    assert_eq!(state.search_text(), "bc");
    LogViewer::update(&mut state, LogViewerMessage::SearchEnd);
    LogViewer::update(&mut state, LogViewerMessage::SearchBackspace);
    assert_eq!(state.search_text(), "b");
}

#[test]
fn test_filter_reapplied_after_search_clear() {
    let mut state = focused_state();
    // Filter to show only errors
    state.set_show_info(false);
    state.set_show_success(false);
    state.set_show_warning(false);
    assert_eq!(state.visible_entries().len(), 1);

    // Search narrows further
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('x'));
    assert_eq!(state.visible_entries().len(), 0);

    // Clear search -- should show errors again
    LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
    assert_eq!(state.visible_entries().len(), 1);
}

#[test]
fn test_title_with_filtered_count() {
    let mut state = LogViewerState::new().with_title("Log");
    state.push_info("a");
    state.push_info("b");
    state.push_error("c");
    // Filter off info, so only 1 visible out of 3
    state.set_show_info(false);
    assert_eq!(state.visible_entries().len(), 1);
    assert_eq!(state.len(), 3);
}

#[test]
fn test_set_title() {
    let mut state = LogViewerState::new();
    state.set_title(Some("Test".to_string()));
    assert_eq!(state.title(), Some("Test"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_set_show_timestamps() {
    let mut state = LogViewerState::new();
    state.set_show_timestamps(true);
    assert!(state.show_timestamps());
    state.set_show_timestamps(false);
    assert!(!state.show_timestamps());
}

// =============================================================================
// Annotation tests
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = LogViewerState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                LogViewer::view(
                    &state,
                    frame,
                    frame.area(),
                    &theme,
                    &ViewContext::new().focused(true),
                );
            })
            .unwrap();
    });
    assert!(registry.get_by_id("log_viewer").is_some());
}
