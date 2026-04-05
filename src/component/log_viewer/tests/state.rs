use super::*;

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = LogViewerState::new();
    assert!(state.is_empty());
    assert_eq!(state.max_entries(), 1000);
}

#[test]
fn test_default() {
    let state = LogViewerState::default();
    assert!(state.is_empty());
    assert!(state.show_info());
    assert!(state.show_success());
    assert!(state.show_warning());
    assert!(state.show_error());
    assert!(!state.show_timestamps());
    assert_eq!(state.title(), None);
}

#[test]
fn test_with_max_entries() {
    let state = LogViewerState::new().with_max_entries(50);
    assert_eq!(state.max_entries(), 50);
}

#[test]
fn test_with_timestamps() {
    let state = LogViewerState::new().with_timestamps(true);
    assert!(state.show_timestamps());
}

#[test]
fn test_with_title() {
    let state = LogViewerState::new().with_title("Logs");
    assert_eq!(state.title(), Some("Logs"));
}

// =============================================================================
// Entry manipulation
// =============================================================================

#[test]
fn test_push_info() {
    let mut state = LogViewerState::new();
    let id = state.push_info("hello");
    assert_eq!(state.len(), 1);
    assert_eq!(state.entries()[0].id(), id);
    assert_eq!(state.entries()[0].message(), "hello");
    assert_eq!(state.entries()[0].level(), StatusLogLevel::Info);
}

#[test]
fn test_push_success() {
    let mut state = LogViewerState::new();
    state.push_success("done");
    assert_eq!(state.entries()[0].level(), StatusLogLevel::Success);
}

#[test]
fn test_push_warning() {
    let mut state = LogViewerState::new();
    state.push_warning("careful");
    assert_eq!(state.entries()[0].level(), StatusLogLevel::Warning);
}

#[test]
fn test_push_error() {
    let mut state = LogViewerState::new();
    state.push_error("oops");
    assert_eq!(state.entries()[0].level(), StatusLogLevel::Error);
}

#[test]
fn test_push_with_timestamp() {
    let mut state = LogViewerState::new();
    state.push_info_with_timestamp("msg", "12:00:00");
    assert_eq!(state.entries()[0].timestamp(), Some("12:00:00"));
}

#[test]
fn test_push_success_with_timestamp() {
    let mut state = LogViewerState::new();
    state.push_success_with_timestamp("msg", "12:00");
    assert_eq!(state.entries()[0].timestamp(), Some("12:00"));
}

#[test]
fn test_push_warning_with_timestamp() {
    let mut state = LogViewerState::new();
    state.push_warning_with_timestamp("msg", "12:00");
    assert_eq!(state.entries()[0].timestamp(), Some("12:00"));
}

#[test]
fn test_push_error_with_timestamp() {
    let mut state = LogViewerState::new();
    state.push_error_with_timestamp("msg", "12:00");
    assert_eq!(state.entries()[0].timestamp(), Some("12:00"));
}

#[test]
fn test_ids_increment() {
    let mut state = LogViewerState::new();
    let id1 = state.push_info("a");
    let id2 = state.push_info("b");
    let id3 = state.push_info("c");
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(id3, 2);
}

#[test]
fn test_remove() {
    let mut state = LogViewerState::new();
    let id = state.push_info("to remove");
    assert!(state.remove(id));
    assert!(state.is_empty());
}

#[test]
fn test_remove_nonexistent() {
    let mut state = LogViewerState::new();
    assert!(!state.remove(999));
}

#[test]
fn test_clear() {
    let mut state = sample_state();
    state.clear();
    assert!(state.is_empty());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_eviction() {
    let mut state = LogViewerState::new().with_max_entries(3);
    state.push_info("a");
    state.push_info("b");
    state.push_info("c");
    state.push_info("d");
    assert_eq!(state.len(), 3);
    // Oldest entry ("a") should have been evicted
    assert_eq!(state.entries()[0].message(), "b");
}

#[test]
fn test_set_max_entries_evicts() {
    let mut state = LogViewerState::new();
    state.push_info("a");
    state.push_info("b");
    state.push_info("c");
    state.set_max_entries(2);
    assert_eq!(state.len(), 2);
    assert_eq!(state.entries()[0].message(), "b");
}

#[test]
fn test_set_max_entries_no_eviction_when_under_limit() {
    let mut state = LogViewerState::new();
    state.push_info("a");
    state.push_info("b");
    assert_eq!(state.len(), 2);

    state.set_max_entries(10);
    assert_eq!(state.len(), 2);
}

#[test]
fn test_set_max_history_evicts_oldest() {
    let mut state = LogViewerState::new();
    state.search_history = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
        "e".to_string(),
    ];
    assert_eq!(state.search_history().len(), 5);

    state.set_max_history(2);
    assert_eq!(state.search_history().len(), 2);
    assert_eq!(state.search_history()[0], "d");
    assert_eq!(state.search_history()[1], "e");
}

#[test]
fn test_set_max_history_no_eviction_when_under_limit() {
    let mut state = LogViewerState::new();
    state.search_history = vec!["a".to_string(), "b".to_string()];
    assert_eq!(state.search_history().len(), 2);

    state.set_max_history(10);
    assert_eq!(state.search_history().len(), 2);
}

// =============================================================================
// Filtering
// =============================================================================

#[test]
fn test_visible_entries_all() {
    let state = sample_state();
    assert_eq!(state.visible_entries().len(), 4);
}

#[test]
fn test_visible_entries_newest_first() {
    let state = sample_state();
    let visible = state.visible_entries();
    assert_eq!(visible[0].message(), "Connection timeout");
    assert_eq!(visible[3].message(), "Server started");
}

#[test]
fn test_filter_info() {
    let mut state = sample_state();
    state.set_show_info(false);
    assert_eq!(state.visible_entries().len(), 3);
}

#[test]
fn test_filter_success() {
    let mut state = sample_state();
    state.set_show_success(false);
    assert_eq!(state.visible_entries().len(), 3);
}

#[test]
fn test_filter_warning() {
    let mut state = sample_state();
    state.set_show_warning(false);
    assert_eq!(state.visible_entries().len(), 3);
}

#[test]
fn test_filter_error() {
    let mut state = sample_state();
    state.set_show_error(false);
    assert_eq!(state.visible_entries().len(), 3);
}

#[test]
fn test_filter_multiple_levels() {
    let mut state = sample_state();
    state.set_show_info(false);
    state.set_show_success(false);
    assert_eq!(state.visible_entries().len(), 2);
}

#[test]
fn test_filter_all_hidden() {
    let mut state = sample_state();
    state.set_show_info(false);
    state.set_show_success(false);
    state.set_show_warning(false);
    state.set_show_error(false);
    assert!(state.visible_entries().is_empty());
}

#[test]
fn test_search_filter() {
    let mut state = focused_state();
    state.focus = Focus::Search;
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('c'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('o'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('n'));
    // "Connected to database" and "Connection timeout" match "con"
    assert_eq!(state.visible_entries().len(), 2);
}

#[test]
fn test_search_case_insensitive() {
    let mut state = focused_state();
    state.focus = Focus::Search;
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('S'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('E'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('R'));
    // "Server started" matches "SER" (case insensitive)
    assert_eq!(state.visible_entries().len(), 1);
}

#[test]
fn test_search_no_matches() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('z'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('z'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('z'));
    assert!(state.visible_entries().is_empty());
}

#[test]
fn test_search_combined_with_level_filter() {
    let mut state = focused_state();
    state.set_show_info(false);
    state.focus = Focus::Search;
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('c'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('o'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('n'));
    // "Connected to database" is success (visible), "Connection timeout" is error (visible)
    // Info filter is off, so only these two match
    assert_eq!(state.visible_entries().len(), 2);
}
