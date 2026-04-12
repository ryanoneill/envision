use super::*;

// =============================================================================
// Search bar focus
// =============================================================================

#[test]
fn test_focus_search() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert!(state.is_search_focused());
}

#[test]
fn test_focus_log() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::FocusLog);
    assert!(!state.is_search_focused());
}

#[test]
fn test_clear_search() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('x'));
    assert_eq!(state.search_text(), "x");

    LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
    assert_eq!(state.search_text(), "");
    assert!(!state.is_search_focused());
}

#[test]
fn test_search_backspace() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('a'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('b'));
    LogViewer::update(&mut state, LogViewerMessage::SearchBackspace);
    assert_eq!(state.search_text(), "a");
}

#[test]
fn test_search_resets_scroll() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 2);

    LogViewer::update(&mut state, LogViewerMessage::SearchInput('a'));
    assert_eq!(state.scroll_offset(), 0);
}

// =============================================================================
// Toggle filters via messages
// =============================================================================

#[test]
fn test_toggle_info() {
    let mut state = focused_state();
    let output = LogViewer::update(&mut state, LogViewerMessage::ToggleInfo);
    assert!(!state.show_info());
    assert_eq!(output, Some(LogViewerOutput::FilterChanged));
}

#[test]
fn test_toggle_success() {
    let mut state = focused_state();
    let output = LogViewer::update(&mut state, LogViewerMessage::ToggleSuccess);
    assert!(!state.show_success());
    assert_eq!(output, Some(LogViewerOutput::FilterChanged));
}

#[test]
fn test_toggle_warning() {
    let mut state = focused_state();
    let output = LogViewer::update(&mut state, LogViewerMessage::ToggleWarning);
    assert!(!state.show_warning());
    assert_eq!(output, Some(LogViewerOutput::FilterChanged));
}

#[test]
fn test_toggle_error() {
    let mut state = focused_state();
    let output = LogViewer::update(&mut state, LogViewerMessage::ToggleError);
    assert!(!state.show_error());
    assert_eq!(output, Some(LogViewerOutput::FilterChanged));
}

#[test]
fn test_toggle_resets_scroll() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ToggleInfo);
    assert_eq!(state.scroll_offset(), 0);
}

// =============================================================================
// Push/Clear/Remove via messages
// =============================================================================

#[test]
fn test_push_via_message() {
    let mut state = focused_state();
    let output = LogViewer::update(
        &mut state,
        LogViewerMessage::Push {
            message: "new entry".into(),
            level: StatusLogLevel::Info,
            timestamp: None,
        },
    );
    assert_eq!(state.len(), 5);
    assert!(matches!(output, Some(LogViewerOutput::Added(_))));
}

#[test]
fn test_clear_via_message() {
    let mut state = focused_state();
    let output = LogViewer::update(&mut state, LogViewerMessage::Clear);
    assert!(state.is_empty());
    assert_eq!(output, Some(LogViewerOutput::Cleared));
}

#[test]
fn test_remove_via_message() {
    let mut state = focused_state();
    let id = state.entries()[0].id();
    let output = LogViewer::update(&mut state, LogViewerMessage::Remove(id));
    assert_eq!(state.len(), 3);
    assert_eq!(output, Some(LogViewerOutput::Removed(id)));
}

#[test]
fn test_remove_nonexistent_via_message() {
    let mut state = focused_state();
    let output = LogViewer::update(&mut state, LogViewerMessage::Remove(999));
    assert_eq!(output, None);
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_messages() {
    let mut state = focused_state();
    let output = LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_events() {
    let state = focused_state();
    let msg = LogViewer::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = sample_state();
    let msg = LogViewer::handle_event(&state, &Event::key(KeyCode::Down), &EventContext::default());
    assert_eq!(msg, None);
}
