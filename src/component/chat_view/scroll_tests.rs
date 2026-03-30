use super::*;
use crate::component::Focusable;

fn focused_state() -> ChatViewState {
    let mut state = ChatViewState::new();
    ChatView::set_focused(&mut state, true);
    state
}

fn state_with_messages() -> ChatViewState {
    let mut state = focused_state();
    state.push_system("Welcome!");
    state.push_user("Hello");
    state.push_assistant("Hi there!");
    state
}

// =============================================================================
// Scrolling
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    state.scroll.set_offset(0);
    ChatView::update(&mut state, ChatViewMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    state.scroll.set_content_length(state.messages().len());
    state.scroll.set_offset(2);
    ChatView::update(&mut state, ChatViewMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = state_with_messages();
    state.scroll.set_offset(0);
    ChatView::update(&mut state, ChatViewMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_top() {
    let mut state = state_with_messages();
    state.scroll.set_content_length(state.messages().len());
    state.scroll.set_offset(2);
    ChatView::update(&mut state, ChatViewMessage::ScrollToTop);
    assert_eq!(state.scroll_offset(), 0);
    assert!(!state.auto_scroll());
}

#[test]
fn test_scroll_to_bottom() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    state.scroll.set_offset(0);
    ChatView::update(&mut state, ChatViewMessage::ScrollToBottom);
    assert!(state.scroll.at_end());
    assert!(state.auto_scroll());
}

#[test]
fn test_auto_scroll_on_new_message() {
    let mut state = focused_state();
    state.push_user("a");
    state.push_user("b");
    // auto_scroll should keep us at the bottom
    assert!(state.auto_scroll());
}

#[test]
fn test_scroll_up_disables_auto_scroll() {
    let mut state = state_with_messages();
    ChatView::update(&mut state, ChatViewMessage::ScrollUp);
    assert!(!state.auto_scroll());
}

#[test]
fn test_set_auto_scroll() {
    let mut state = ChatViewState::new();
    state.set_auto_scroll(false);
    assert!(!state.auto_scroll());
    state.set_auto_scroll(true);
    assert!(state.auto_scroll());
}

// =============================================================================
// Edge cases involving scroll
// =============================================================================

#[test]
fn test_scroll_empty_history() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 0);
}
