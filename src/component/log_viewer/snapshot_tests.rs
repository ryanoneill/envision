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

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_default_empty() {
    let state = LogViewerState::new();
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_populated() {
    let state = sample_state();
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused() {
    let mut state = sample_state();
    LogViewer::set_focused(&mut state, true);
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_title() {
    let mut state = LogViewerState::new().with_title("Application Log");
    state.push_info("Started successfully");
    state.push_warning("High memory usage");
    state.push_error("Failed to connect");
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_timestamps() {
    let mut state = LogViewerState::new().with_timestamps(true);
    state.push_info_with_timestamp("Server started", "10:00:00");
    state.push_warning_with_timestamp("Memory high", "10:05:30");
    state.push_error_with_timestamp("Connection lost", "10:10:15");
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_search_active() {
    let mut state = sample_state();
    LogViewer::set_focused(&mut state, true);
    LogViewer::update(&mut state, LogViewerMessage::FocusSearch);
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('c'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('o'));
    LogViewer::update(&mut state, LogViewerMessage::SearchInput('n'));
    let (mut terminal, theme) = test_utils::setup_render(70, 20);
    terminal
        .draw(|frame| {
            LogViewer::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
