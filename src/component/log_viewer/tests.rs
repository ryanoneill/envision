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
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = LogViewerState::new();
    assert!(state.is_empty());
    assert_eq!(state.max_entries(), 1000);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
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

#[test]
fn test_with_disabled() {
    let state = LogViewerState::new().with_disabled(true);
    assert!(state.is_disabled());
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

// =============================================================================
// Scrolling
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_down_at_bottom() {
    let mut state = focused_state();
    // 4 entries, max offset = 3
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 3);
}

#[test]
fn test_scroll_to_top() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    LogViewer::update(&mut state, LogViewerMessage::ScrollToTop);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_bottom() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::ScrollToBottom);
    assert_eq!(state.scroll_offset(), 3);
}

#[test]
fn test_set_scroll_offset() {
    let mut state = sample_state();
    state.set_scroll_offset(2);
    assert_eq!(state.scroll_offset(), 2);
}

#[test]
fn test_set_scroll_offset_clamped() {
    let mut state = sample_state();
    state.set_scroll_offset(100);
    assert_eq!(state.scroll_offset(), 3);
}

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
    state.set_disabled(true);
    let output = LogViewer::update(&mut state, LogViewerMessage::ScrollDown);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = LogViewer::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = sample_state();
    let msg = LogViewer::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping — log mode
// =============================================================================

#[test]
fn test_log_mode_up_key() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(LogViewerMessage::ScrollUp)
    );
    assert_eq!(
        LogViewer::handle_event(&state, &Event::char('k')),
        Some(LogViewerMessage::ScrollUp)
    );
}

#[test]
fn test_log_mode_down_key() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(LogViewerMessage::ScrollDown)
    );
    assert_eq!(
        LogViewer::handle_event(&state, &Event::char('j')),
        Some(LogViewerMessage::ScrollDown)
    );
}

#[test]
fn test_log_mode_home_end() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(LogViewerMessage::ScrollToTop)
    );
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::End)),
        Some(LogViewerMessage::ScrollToBottom)
    );
}

#[test]
fn test_log_mode_slash() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(&state, &Event::char('/')),
        Some(LogViewerMessage::FocusSearch)
    );
}

#[test]
fn test_log_mode_number_keys() {
    let state = focused_state();
    assert_eq!(
        LogViewer::handle_event(&state, &Event::char('1')),
        Some(LogViewerMessage::ToggleInfo)
    );
    assert_eq!(
        LogViewer::handle_event(&state, &Event::char('2')),
        Some(LogViewerMessage::ToggleSuccess)
    );
    assert_eq!(
        LogViewer::handle_event(&state, &Event::char('3')),
        Some(LogViewerMessage::ToggleWarning)
    );
    assert_eq!(
        LogViewer::handle_event(&state, &Event::char('4')),
        Some(LogViewerMessage::ToggleError)
    );
}

// =============================================================================
// Event mapping — search mode
// =============================================================================

#[test]
fn test_search_mode_char_input() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::char('a')),
        Some(LogViewerMessage::SearchInput('a'))
    );
}

#[test]
fn test_search_mode_esc() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Esc)),
        Some(LogViewerMessage::ClearSearch)
    );
}

#[test]
fn test_search_mode_enter() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Enter)),
        Some(LogViewerMessage::ConfirmSearch)
    );
}

#[test]
fn test_search_mode_backspace() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Backspace)),
        Some(LogViewerMessage::SearchBackspace)
    );
}

#[test]
fn test_search_mode_delete() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Delete)),
        Some(LogViewerMessage::SearchDelete)
    );
}

#[test]
fn test_search_mode_left_right() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Left)),
        Some(LogViewerMessage::SearchLeft)
    );
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Right)),
        Some(LogViewerMessage::SearchRight)
    );
}

#[test]
fn test_search_mode_home_end() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(LogViewerMessage::SearchHome)
    );
    assert_eq!(
        LogViewer::handle_event(&state, &Event::key(KeyCode::End)),
        Some(LogViewerMessage::SearchEnd)
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::key(KeyCode::Down));
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
    state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(state.scroll_offset(), 1);
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = LogViewerState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_entries() {
    let state = sample_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_search_focused() {
    let mut state = focused_state();
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('c'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('o'));
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = LogViewerState::new().with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_title() {
    let mut state = LogViewerState::new().with_title("Application Log");
    state.push_info("entry 1");
    state.push_error("entry 2");
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_timestamps() {
    let mut state = LogViewerState::new().with_timestamps(true);
    state.push_info_with_timestamp("entry 1", "12:00:00");
    state.push_error_with_timestamp("entry 2", "12:00:01");
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = sample_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 2);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = LogViewer::init();
    assert!(!LogViewer::is_focused(&state));

    LogViewer::focus(&mut state);
    assert!(LogViewer::is_focused(&state));

    LogViewer::blur(&mut state);
    assert!(!LogViewer::is_focused(&state));
}

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
fn test_partial_eq_different_focus() {
    let state1 = focused_state();
    let state2 = sample_state();
    assert_ne!(state1, state2);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_scroll_empty_log() {
    let mut state = LogViewerState::new();
    LogViewer::set_focused(&mut state, true);
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

    // Clear search — should show errors again
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

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = LogViewerState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                LogViewer::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    assert!(registry.get_by_id("log_viewer").is_some());
}
