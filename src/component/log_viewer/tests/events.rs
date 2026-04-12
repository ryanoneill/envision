use super::*;

// =============================================================================
// Event mapping -- log mode
// =============================================================================

#[test]
fn test_log_mode_up_key() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ScrollUp)
    );
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::char('k'),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ScrollUp)
    );
}

#[test]
fn test_log_mode_down_key() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ScrollDown)
    );
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::char('j'),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ScrollDown)
    );
}

#[test]
fn test_log_mode_home_end() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Home),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ScrollToTop)
    );
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::End),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ScrollToBottom)
    );
}

#[test]
fn test_log_mode_slash() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::char('/'),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::FocusSearch)
    );
}

#[test]
fn test_log_mode_number_keys() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::char('1'),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ToggleInfo)
    );
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::char('2'),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ToggleSuccess)
    );
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::char('3'),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ToggleWarning)
    );
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::char('4'),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ToggleError)
    );
}

// =============================================================================
// Event mapping -- search mode
// =============================================================================

#[test]
fn test_search_mode_char_input() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::char('a'),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::SearchInput('a'))
    );
}

#[test]
fn test_search_mode_esc() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Esc),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ClearSearch)
    );
}

#[test]
fn test_search_mode_enter() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Enter),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::ConfirmSearch)
    );
}

#[test]
fn test_search_mode_backspace() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Backspace),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::SearchBackspace)
    );
}

#[test]
fn test_search_mode_delete() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Delete),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::SearchDelete)
    );
}

#[test]
fn test_search_mode_left_right() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Left),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::SearchLeft)
    );
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Right),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::SearchRight)
    );
}

#[test]
fn test_search_mode_home_end() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::Home),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::SearchHome)
    );
    assert_eq!(
        LogViewer::handle_event(
            &state,
            &Event::key(KeyCode::End),
            &EventContext::new().focused(true)
        ),
        Some(LogViewerMessage::SearchEnd)
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = LogViewer::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(LogViewerMessage::ScrollDown));
}

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state();
    LogViewer::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &EventContext::new().focused(true),
    );
    assert_eq!(state.scroll_offset(), 1);
}
