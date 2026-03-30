use super::*;
use crate::component::test_utils;

fn sample_state() -> LogViewerState {
    let mut state = LogViewerState::new();
    state.push_info("Server started");
    state.push_success("Connected to database");
    state.push_warning("Disk space low");
    state.push_error("Connection timeout");
    state
}

fn focused_state() -> LogViewerState {
    let mut state = sample_state();
    LogViewer::set_focused(&mut state, true);
    state
}

// =============================================================================
// Follow mode
// =============================================================================

#[test]
fn test_default_follow_enabled() {
    let state = LogViewerState::new();
    assert!(state.follow());
}

#[test]
fn test_with_follow_builder() {
    let state = LogViewerState::new().with_follow(false);
    assert!(!state.follow());
}

#[test]
fn test_set_follow() {
    let mut state = LogViewerState::new();
    state.set_follow(false);
    assert!(!state.follow());
    state.set_follow(true);
    assert!(state.follow());
}

#[test]
fn test_toggle_follow() {
    let mut state = focused_state();
    assert!(state.follow());

    let output = LogViewer::update(&mut state, LogViewerMessage::ToggleFollow);
    assert!(!state.follow());
    assert_eq!(output, Some(LogViewerOutput::FollowToggled(false)));

    let output = LogViewer::update(&mut state, LogViewerMessage::ToggleFollow);
    assert!(state.follow());
    assert_eq!(output, Some(LogViewerOutput::FollowToggled(true)));
}

#[test]
fn test_follow_key_binding() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(&state, &Event::char('f')),
        Some(LogViewerMessage::ToggleFollow)
    );
}

#[test]
fn test_scroll_disables_follow() {
    let mut state = focused_state();
    assert!(state.follow());

    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert!(!state.follow());
}

#[test]
fn test_scroll_up_disables_follow() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    state.set_follow(true);

    LogViewer::update(&mut state, LogViewerMessage::ScrollUp);
    assert!(!state.follow());
}

#[test]
fn test_follow_auto_scroll_on_push() {
    let mut state = focused_state();
    // Scroll away from top
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 2);

    // Re-enable follow
    state.set_follow(true);

    // Push new entry should auto-scroll to offset 0 (newest)
    LogViewer::update(
        &mut state,
        LogViewerMessage::Push {
            message: "new entry".into(),
            level: StatusLogLevel::Info,
            timestamp: None,
        },
    );
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_no_auto_scroll_without_follow() {
    let mut state = focused_state();
    state.set_follow(false);

    // Scroll away from top
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    let offset = state.scroll_offset();

    // Push new entry should not change scroll
    LogViewer::update(
        &mut state,
        LogViewerMessage::Push {
            message: "new entry".into(),
            level: StatusLogLevel::Info,
            timestamp: None,
        },
    );
    assert_eq!(state.scroll_offset(), offset);
}

#[test]
fn test_toggle_follow_on_scrolls_to_top() {
    let mut state = focused_state();
    // Scroll away and disable follow
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert!(!state.follow());
    assert!(state.scroll_offset() > 0);

    // Toggle follow back on should scroll to top (newest)
    LogViewer::update(&mut state, LogViewerMessage::ToggleFollow);
    assert!(state.follow());
    assert_eq!(state.scroll_offset(), 0);
}

// =============================================================================
// Regex search
// =============================================================================

#[test]
fn test_default_regex_disabled() {
    let state = LogViewerState::new();
    assert!(!state.use_regex());
}

#[test]
fn test_with_regex_builder() {
    let state = LogViewerState::new().with_regex(true);
    assert!(state.use_regex());
}

#[test]
fn test_set_use_regex() {
    let mut state = LogViewerState::new();
    state.set_use_regex(true);
    assert!(state.use_regex());
    state.set_use_regex(false);
    assert!(!state.use_regex());
}

#[test]
fn test_toggle_regex() {
    let mut state = focused_state();
    assert!(!state.use_regex());

    let output = LogViewer::update(&mut state, LogViewerMessage::ToggleRegex);
    assert!(state.use_regex());
    assert_eq!(output, Some(LogViewerOutput::RegexToggled(true)));

    let output = LogViewer::update(&mut state, LogViewerMessage::ToggleRegex);
    assert!(!state.use_regex());
    assert_eq!(output, Some(LogViewerOutput::RegexToggled(false)));
}

#[test]
fn test_toggle_regex_key_binding() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::ctrl('r')),
        Some(LogViewerMessage::ToggleRegex)
    );
}

#[test]
fn test_toggle_regex_resets_scroll() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ToggleRegex);
    assert_eq!(state.scroll_offset(), 0);
}

#[cfg(feature = "regex")]
#[test]
fn test_regex_search_basic() {
    let mut state = focused_state();
    state.set_use_regex(true);

    // Search with regex pattern: "^Con" should match entries starting with "Con"
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "^Con".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }

    let visible = state.visible_entries();
    // "Connected to database" and "Connection timeout" match "^Con"
    assert_eq!(visible.len(), 2);
}

#[cfg(feature = "regex")]
#[test]
fn test_regex_search_alternation() {
    let mut state = focused_state();
    state.set_use_regex(true);

    // Search for "started|timeout"
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "started|timeout".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }

    let visible = state.visible_entries();
    // "Server started" and "Connection timeout" match
    assert_eq!(visible.len(), 2);
}

#[cfg(feature = "regex")]
#[test]
fn test_regex_search_case_insensitive() {
    let mut state = focused_state();
    state.set_use_regex(true);

    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "SERVER".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }

    let visible = state.visible_entries();
    // "Server started" matches case-insensitively
    assert_eq!(visible.len(), 1);
}

#[cfg(feature = "regex")]
#[test]
fn test_regex_invalid_falls_back_to_literal() {
    let mut state = focused_state();
    state.set_use_regex(true);

    // Invalid regex: unclosed bracket
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "[invalid".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }

    // Should fall back to literal match -- no entries contain "[invalid"
    let visible = state.visible_entries();
    assert!(visible.is_empty());
}

#[test]
fn test_substring_search_when_regex_disabled() {
    let mut state = focused_state();
    assert!(!state.use_regex());

    // "^Con" as literal should not match (no entry contains "^Con")
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "^Con".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }

    let visible = state.visible_entries();
    assert!(visible.is_empty());
}

// =============================================================================
// Search history
// =============================================================================

#[test]
fn test_default_search_history() {
    let state = LogViewerState::new();
    assert!(state.search_history().is_empty());
    assert_eq!(state.max_history(), 20);
}

#[test]
fn test_with_max_history_builder() {
    let state = LogViewerState::new().with_max_history(10);
    assert_eq!(state.max_history(), 10);
}

#[test]
fn test_set_max_history() {
    let mut state = LogViewerState::new();
    state.set_max_history(5);
    assert_eq!(state.max_history(), 5);
}

#[test]
fn test_confirm_search_saves_to_history() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('a'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('b'));
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    assert_eq!(state.search_history(), &["ab"]);
    assert!(!state.is_search_focused());
}

#[test]
fn test_confirm_empty_search_no_history() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    assert!(state.search_history().is_empty());
}

#[test]
fn test_search_history_deduplication() {
    let mut state = focused_state();

    // Search for "abc" and confirm
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "abc".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    // Clear, search for "def" and confirm
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "def".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    // Clear, search for "abc" again and confirm (should move to end, not duplicate)
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "abc".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    assert_eq!(state.search_history(), &["def", "abc"]);
}

#[test]
fn test_search_history_max_limit() {
    let mut state = focused_state();
    state.set_max_history(3);

    for i in 0..5 {
        // Clear first, then focus search and type
        LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
        LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
        LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
        LogViewer::update(
            &mut state,
            LogViewerMessage::SearchInput(char::from(b'a' + i)),
        );
        LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);
    }

    // Only last 3 should remain
    assert_eq!(state.search_history().len(), 3);
    assert_eq!(state.search_history(), &["c", "d", "e"]);
}

#[test]
fn test_search_history_up_key_binding() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(LogViewerMessage::SearchHistoryUp)
    );
}

#[test]
fn test_search_history_down_key_binding() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(LogViewerMessage::SearchHistoryDown)
    );
}

#[test]
fn test_search_history_navigation_up() {
    let mut state = focused_state();

    // Add two history entries
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "first".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    // Clear and add second entry
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "second".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    // Start a new search and navigate history
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);

    // Up should load "second" (most recent)
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryUp);
    assert_eq!(state.search_text(), "second");

    // Up again should load "first"
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryUp);
    assert_eq!(state.search_text(), "first");

    // Up again at top should stay at "first"
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryUp);
    assert_eq!(state.search_text(), "first");
}

#[test]
fn test_search_history_navigation_down() {
    let mut state = focused_state();

    // Add two history entries
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "first".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "second".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    // Start new search and navigate
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);

    // Go up to "first"
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryUp);
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryUp);
    assert_eq!(state.search_text(), "first");

    // Down should go to "second"
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryDown);
    assert_eq!(state.search_text(), "second");

    // Down past end should clear search
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryDown);
    assert_eq!(state.search_text(), "");
}

#[test]
fn test_search_history_empty_no_op() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);

    // Up/Down with empty history should be no-op
    let output = LogViewer::update(&mut state, LogViewerMessage::SearchHistoryUp);
    assert_eq!(output, None);

    let output = LogViewer::update(&mut state, LogViewerMessage::SearchHistoryDown);
    assert_eq!(output, None);
}

#[test]
fn test_search_history_down_without_browsing_no_op() {
    let mut state = focused_state();

    // Add a history entry
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "test".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    // Start new search, don't press Up first
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);

    // Down without first pressing Up should be no-op
    let output = LogViewer::update(&mut state, LogViewerMessage::SearchHistoryDown);
    assert_eq!(output, None);
}

#[test]
fn test_confirm_search_resets_history_index() {
    let mut state = focused_state();

    // Add a history entry
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "test".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    // Browse history
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryUp);
    assert_eq!(state.search_text(), "test");

    // Confirm resets history browsing state
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    // Next Up should start from the most recent again
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryUp);
    assert_eq!(state.search_text(), "test");
}

#[test]
fn test_clear_search_resets_history_index() {
    let mut state = focused_state();

    // Add a history entry
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    for c in "test".chars() {
        LogViewer::update(&mut state, LogViewerMessage::SearchInput(c));
    }
    LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);

    // Browse history
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::SearchHistoryUp);

    // Clear search resets history browsing
    LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
    assert_eq!(state.search_text(), "");
}

#[test]
fn test_set_max_history_truncates() {
    let mut state = LogViewerState::new();
    state.set_focused(true);

    // Add 5 history entries
    for i in 0..5 {
        LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
        LogViewer::update(&mut state, LogViewerMessage::ClearSearch);
        LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
        LogViewer::update(
            &mut state,
            LogViewerMessage::SearchInput(char::from(b'a' + i)),
        );
        LogViewer::update(&mut state, LogViewerMessage::ConfirmSearch);
    }
    assert_eq!(state.search_history().len(), 5);

    // Reduce max history should truncate
    state.set_max_history(3);
    assert_eq!(state.search_history().len(), 3);
    assert_eq!(state.search_history(), &["c", "d", "e"]);
}

// =============================================================================
// Rendering with new features
// =============================================================================

#[test]
fn test_render_with_follow() {
    let state = LogViewerState::new(); // follow is true by default
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_without_follow() {
    let mut state = sample_state();
    state.set_follow(false);
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_regex() {
    let mut state = LogViewerState::new();
    state.set_use_regex(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}
