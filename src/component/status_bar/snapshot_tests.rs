use super::*;
use crate::component::test_utils;

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state = StatusBarState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_left_only() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("INSERT"));
    state.push_left(StatusBarItem::new("main.rs"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_center_only() {
    let mut state = StatusBarState::new();
    state.push_center(StatusBarItem::new("filename.rs"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_right_only() {
    let mut state = StatusBarState::new();
    state.push_right(StatusBarItem::new("UTF-8"));
    state.push_right(StatusBarItem::new("Ln 42, Col 8"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_all_sections() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));
    state.push_center(StatusBarItem::new("main.rs"));
    state.push_right(StatusBarItem::new("UTF-8"));
    state.push_right(StatusBarItem::new("Ln 42, Col 8"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_styled_items() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("ERROR").with_style(StatusBarStyle::Error));
    state.push_left(StatusBarItem::new("WARN").with_style(StatusBarStyle::Warning));
    state.push_right(StatusBarItem::new("OK").with_style(StatusBarStyle::Success));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_counter_item() {
    let mut state = StatusBarState::new();
    state.push_left(StatusBarItem::new("Mode"));
    state.push_right(StatusBarItem::counter().with_label("Items"));
    StatusBar::update(
        &mut state,
        StatusBarMessage::SetCounter {
            section: Section::Right,
            index: 0,
            value: 42,
        },
    );
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_custom_separator() {
    let mut state = StatusBarState::with_separator(" :: ");
    state.push_left(StatusBarItem::new("A"));
    state.push_left(StatusBarItem::new("B"));
    state.push_right(StatusBarItem::new("X"));
    state.push_right(StatusBarItem::new("Y"));
    let (mut terminal, theme) = test_utils::setup_render(60, 1);
    terminal
        .draw(|frame| {
            StatusBar::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
