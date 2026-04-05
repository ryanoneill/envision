use super::*;

// =============================================================================
// MessageHandle
// =============================================================================

#[test]
fn test_push_message_returns_handle() {
    let mut state = ConversationViewState::new();
    let h1 = state.push_user("Hello");
    let h2 = state.push_assistant("Hi");
    assert_ne!(h1, h2);
}

#[test]
fn test_handle_survives_additional_pushes() {
    let mut state = ConversationViewState::new();
    let handle = state.push_assistant("Thinking...");
    state.push_user("Another message");
    state.push_system("System note");
    state.update_by_handle(handle, |msg| {
        msg.push_block(MessageBlock::text(" Done."));
        msg.set_streaming(false);
    });
    assert_eq!(state.messages()[0].blocks().len(), 2);
    assert!(!state.messages()[0].is_streaming());
}

#[test]
fn test_handle_noop_after_clear() {
    let mut state = ConversationViewState::new();
    let handle = state.push_assistant("Thinking...");
    state.clear_messages();
    // Should not panic; no-op
    state.update_by_handle(handle, |msg| {
        msg.set_streaming(false);
    });
    assert!(state.is_empty());
}

#[test]
fn test_handle_noop_after_eviction() {
    let mut state = ConversationViewState::new().with_max_messages(2);
    let handle = state.push_user("a");
    state.push_user("b");
    state.push_user("c"); // "a" is evicted
                          // handle points to evicted message, should no-op
    state.update_by_handle(handle, |_msg| {
        panic!("should not be called on evicted message");
    });
    assert_eq!(state.message_count(), 2);
}

#[test]
fn test_push_user_returns_handle() {
    let mut state = ConversationViewState::new();
    let handle = state.push_user("Hello");
    state.update_by_handle(handle, |msg| {
        msg.push_block(MessageBlock::text(" - updated"));
    });
    assert_eq!(state.messages()[0].blocks().len(), 2);
}

#[test]
fn test_push_assistant_returns_handle() {
    let mut state = ConversationViewState::new();
    let handle = state.push_assistant("Hi");
    state.update_by_handle(handle, |msg| {
        msg.set_streaming(true);
    });
    assert!(state.messages()[0].is_streaming());
}

#[test]
fn test_push_system_returns_handle() {
    let mut state = ConversationViewState::new();
    let handle = state.push_system("Init");
    state.update_by_handle(handle, |msg| {
        msg.push_block(MessageBlock::text(" - complete"));
    });
    assert_eq!(state.messages()[0].blocks().len(), 2);
}

#[test]
fn test_push_tool_returns_handle() {
    let mut state = ConversationViewState::new();
    let handle = state.push_tool("Result");
    state.update_by_handle(handle, |msg| {
        msg.push_block(MessageBlock::code("42", None));
    });
    assert_eq!(state.messages()[0].blocks().len(), 2);
}

#[test]
fn test_handle_uniqueness_across_many_pushes() {
    let mut state = ConversationViewState::new();
    let mut handles = Vec::new();
    for i in 0..100 {
        handles.push(state.push_user(format!("msg {i}")));
    }
    // All handles should be unique
    let set: std::collections::HashSet<_> = handles.iter().copied().collect();
    assert_eq!(set.len(), 100);
}

#[test]
fn test_handle_clone_copy() {
    let mut state = ConversationViewState::new();
    let handle = state.push_user("Hello");
    let copied = handle;
    let cloned = handle;
    assert_eq!(handle, copied);
    assert_eq!(handle, cloned);
}

#[test]
fn test_handle_debug_format() {
    let mut state = ConversationViewState::new();
    let handle = state.push_user("Hello");
    let debug = format!("{:?}", handle);
    assert!(debug.starts_with("MessageHandle("));
}

#[test]
fn test_streaming_workflow_with_handle() {
    let mut state = ConversationViewState::new();
    // Start streaming response
    let handle = state.push_message(
        ConversationMessage::new(ConversationRole::Assistant, "").with_streaming(true),
    );
    // Simulate streaming chunks
    state.update_by_handle(handle, |msg| {
        msg.set_blocks(vec![MessageBlock::text("Hello")]);
    });
    assert_eq!(state.messages()[0].text_content(), "Hello");

    state.update_by_handle(handle, |msg| {
        msg.set_blocks(vec![MessageBlock::text("Hello, world!")]);
    });
    assert_eq!(state.messages()[0].text_content(), "Hello, world!");

    // Finish streaming
    state.update_by_handle(handle, |msg| {
        msg.set_streaming(false);
    });
    assert!(!state.messages()[0].is_streaming());
}
