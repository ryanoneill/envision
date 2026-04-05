use super::*;

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
