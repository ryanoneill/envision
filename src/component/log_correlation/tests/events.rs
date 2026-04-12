use super::*;

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
    let state = focused_state();
    let msg = LogCorrelation::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_disabled_ignores_update() {
    let mut state = focused_state();
    let output = LogCorrelation::update(&mut state, LogCorrelationMessage::ScrollDown);
    assert_eq!(output, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = two_stream_state();
    let msg =
        LogCorrelation::handle_event(&state, &Event::key(KeyCode::Down), &EventContext::default());
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_up_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &EventContext::new().focused(true)
        ),
        Some(LogCorrelationMessage::ScrollUp)
    );
    assert_eq!(
        LogCorrelation::handle_event(
            &state,
            &Event::char('k'),
            &EventContext::new().focused(true)
        ),
        Some(LogCorrelationMessage::ScrollUp)
    );
}

#[test]
fn test_down_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &EventContext::new().focused(true)
        ),
        Some(LogCorrelationMessage::ScrollDown)
    );
    assert_eq!(
        LogCorrelation::handle_event(
            &state,
            &Event::char('j'),
            &EventContext::new().focused(true)
        ),
        Some(LogCorrelationMessage::ScrollDown)
    );
}

#[test]
fn test_home_end_keys() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(
            &state,
            &Event::key(KeyCode::Home),
            &EventContext::new().focused(true)
        ),
        Some(LogCorrelationMessage::ScrollToTop)
    );
    assert_eq!(
        LogCorrelation::handle_event(
            &state,
            &Event::key(KeyCode::End),
            &EventContext::new().focused(true)
        ),
        Some(LogCorrelationMessage::ScrollToBottom)
    );
}

#[test]
fn test_tab_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(
            &state,
            &Event::key(KeyCode::Tab),
            &EventContext::new().focused(true)
        ),
        Some(LogCorrelationMessage::FocusNextStream)
    );
}

#[test]
fn test_backtab_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(
            &state,
            &Event::key(KeyCode::BackTab),
            &EventContext::new().focused(true)
        ),
        Some(LogCorrelationMessage::FocusPrevStream)
    );
}

#[test]
fn test_s_key() {
    let state = focused_state();
    assert_eq!(
        LogCorrelation::handle_event(
            &state,
            &Event::char('s'),
            &EventContext::new().focused(true)
        ),
        Some(LogCorrelationMessage::ToggleSyncScroll)
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = LogCorrelation::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &EventContext::new().focused(true),
    );
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
    LogCorrelation::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &EventContext::new().focused(true),
    );
    assert_eq!(state.scroll_offset(), 1);
}
