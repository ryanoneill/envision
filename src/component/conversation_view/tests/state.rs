use super::*;

// =============================================================================
// Construction and defaults
// =============================================================================

#[test]
fn test_new() {
    let state = ConversationViewState::new();
    assert_eq!(state.message_count(), 0);
    assert!(state.is_empty());
    assert!(state.auto_scroll());
    assert!(!state.show_timestamps());
    assert!(state.show_role_labels());
    assert_eq!(state.max_messages(), 1000);
    assert!(state.title().is_none());
}

#[test]
fn test_default() {
    let state = ConversationViewState::default();
    assert_eq!(state.max_messages(), 1000);
    assert!(state.auto_scroll());
    assert!(state.show_role_labels());
}

#[test]
fn test_with_title() {
    let state = ConversationViewState::new().with_title("My Chat");
    assert_eq!(state.title(), Some("My Chat"));
}

#[test]
fn test_with_max_messages() {
    let state = ConversationViewState::new().with_max_messages(50);
    assert_eq!(state.max_messages(), 50);
}

#[test]
fn test_with_timestamps() {
    let state = ConversationViewState::new().with_timestamps(true);
    assert!(state.show_timestamps());
}

#[test]
fn test_with_role_labels() {
    let state = ConversationViewState::new().with_role_labels(false);
    assert!(!state.show_role_labels());
}

// =============================================================================
// Message manipulation
// =============================================================================

#[test]
fn test_push_user() {
    let mut state = ConversationViewState::new();
    state.push_user("Hello");
    assert_eq!(state.message_count(), 1);
    assert_eq!(*state.messages()[0].role(), ConversationRole::User);
    assert_eq!(state.messages()[0].text_content(), "Hello");
}

#[test]
fn test_push_assistant() {
    let mut state = ConversationViewState::new();
    state.push_assistant("Hi there!");
    assert_eq!(*state.messages()[0].role(), ConversationRole::Assistant);
}

#[test]
fn test_push_system() {
    let mut state = ConversationViewState::new();
    state.push_system("Init");
    assert_eq!(*state.messages()[0].role(), ConversationRole::System);
}

#[test]
fn test_push_tool() {
    let mut state = ConversationViewState::new();
    state.push_tool("Result: 42");
    assert_eq!(*state.messages()[0].role(), ConversationRole::Tool);
}

#[test]
fn test_push_message_structured() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::thinking("Let me think..."),
            MessageBlock::text("The answer is 42."),
            MessageBlock::code("answer = 42", Some("python")),
        ],
    ));
    assert_eq!(state.messages()[0].blocks().len(), 3);
}

#[test]
fn test_clear_messages() {
    let mut state = state_with_messages();
    state.clear_messages();
    assert!(state.is_empty());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_eviction() {
    let mut state = ConversationViewState::new().with_max_messages(3);
    state.push_user("a");
    state.push_user("b");
    state.push_user("c");
    state.push_user("d");
    assert_eq!(state.message_count(), 3);
    assert_eq!(state.messages()[0].text_content(), "b");
}

#[test]
fn test_set_max_messages() {
    let mut state = ConversationViewState::new();
    state.push_user("a");
    state.push_user("b");
    state.push_user("c");
    state.set_max_messages(2);
    assert_eq!(state.message_count(), 2);
    assert_eq!(state.messages()[0].text_content(), "b");
}

#[test]
fn test_set_max_messages_no_eviction_when_under_limit() {
    let mut state = ConversationViewState::new();
    state.push_user("a");
    state.push_user("b");
    assert_eq!(state.message_count(), 2);

    state.set_max_messages(10);
    assert_eq!(state.message_count(), 2);
}

#[test]
fn test_last_message_mut() {
    let mut state = ConversationViewState::new();
    state.push_assistant("Starting...");
    if let Some(msg) = state.last_message_mut() {
        msg.push_block(MessageBlock::code("let x = 1;", Some("rust")));
        msg.set_streaming(false);
    }
    assert_eq!(state.messages()[0].blocks().len(), 2);
}

#[test]
fn test_last_message_mut_empty() {
    let mut state = ConversationViewState::new();
    assert!(state.last_message_mut().is_none());
}

#[test]
fn test_update_message() {
    let mut state = ConversationViewState::new();
    state.push_user("Hello");
    state.push_assistant("Hi");
    state.update_message(1, |msg| {
        msg.set_blocks(vec![MessageBlock::text("Updated")]);
    });
    assert_eq!(state.messages()[1].blocks().len(), 1);
}

#[test]
fn test_update_last_message_empty() {
    let mut state = ConversationViewState::new();
    // Should no-op, not panic
    state.update_last_message(|_msg| {
        panic!("should not be called on empty conversation");
    });
}

// =============================================================================
// Accessors
// =============================================================================

#[test]
fn test_set_title() {
    let mut state = ConversationViewState::new();
    state.set_title("New Title");
    assert_eq!(state.title(), Some("New Title"));
}

#[test]
fn test_set_show_timestamps() {
    let mut state = ConversationViewState::new();
    state.set_show_timestamps(true);
    assert!(state.show_timestamps());
}

#[test]
fn test_set_show_role_labels() {
    let mut state = ConversationViewState::new();
    state.set_show_role_labels(false);
    assert!(!state.show_role_labels());
}

#[test]
fn test_set_auto_scroll() {
    let mut state = ConversationViewState::new();
    state.set_auto_scroll(false);
    assert!(!state.auto_scroll());
    state.set_auto_scroll(true);
    assert!(state.auto_scroll());
}

// =============================================================================
// Collapse management
// =============================================================================

#[test]
fn test_toggle_collapse() {
    let mut state = ConversationViewState::new();
    assert!(!state.is_collapsed("thinking"));
    state.toggle_collapse("thinking");
    assert!(state.is_collapsed("thinking"));
    state.toggle_collapse("thinking");
    assert!(!state.is_collapsed("thinking"));
}

#[test]
fn test_collapse_expand() {
    let mut state = ConversationViewState::new();
    state.collapse("tool:search");
    assert!(state.is_collapsed("tool:search"));
    state.expand("tool:search");
    assert!(!state.is_collapsed("tool:search"));
}

#[test]
fn test_collapse_multiple() {
    let mut state = ConversationViewState::new();
    state.collapse("thinking");
    state.collapse("tool:search");
    assert!(state.is_collapsed("thinking"));
    assert!(state.is_collapsed("tool:search"));
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = state_with_messages();
    let state2 = state_with_messages();
    assert_eq!(state1, state2);
}

#[test]
fn test_partial_eq_different_messages() {
    let state1 = state_with_messages();
    let mut state2 = state_with_messages();
    state2.push_user("Extra");
    assert_ne!(state1, state2);
}

// =============================================================================
// Auto-scroll on new messages
// =============================================================================

#[test]
fn test_auto_scroll_on_push() {
    let mut state = ConversationViewState::new();
    state.push_user("a");
    state.push_user("b");
    assert!(state.auto_scroll());
}

#[test]
fn test_auto_scroll_after_manual_scroll_down_to_end() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    state.scroll.set_offset(0);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollToBottom);
    assert!(state.auto_scroll());
}
