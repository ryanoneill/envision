use super::*;
use crate::component::test_utils;
use crate::component::Focusable;

fn focused_state() -> ChatViewState {
    let mut state = ChatViewState::new();
    ChatView::set_focused(&mut state, true);
    state
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state = ChatViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_user_message() {
    let mut state = focused_state();
    state.push_user("Hello, world!");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_multi_role() {
    let mut state = focused_state();
    state.push_system("Welcome to the chat!");
    state.push_user("Hello");
    state.push_assistant("Hi there! How can I help?");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_input() {
    let mut state = focused_state();
    state.push_user("Hello");
    ChatView::update(&mut state, ChatViewMessage::Input('T'));
    ChatView::update(&mut state, ChatViewMessage::Input('e'));
    ChatView::update(&mut state, ChatViewMessage::Input('s'));
    ChatView::update(&mut state, ChatViewMessage::Input('t'));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_history() {
    let mut state = focused_state();
    state.push_user("Message 1");
    state.push_assistant("Reply 1");
    ChatView::update(&mut state, ChatViewMessage::FocusHistory);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let mut state = ChatViewState::new().with_disabled(true);
    state.push_user("Hello");
    state.push_assistant("Hi!");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_timestamps() {
    let mut state = ChatViewState::new().with_timestamps(true);
    ChatView::set_focused(&mut state, true);
    state.push_user_with_timestamp("Hello", "12:00");
    state.push_assistant_with_timestamp("Hi there!", "12:01");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
