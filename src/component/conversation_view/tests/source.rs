use super::*;

// =============================================================================
// MessageSource trait
// =============================================================================

#[test]
fn test_message_source_vec() {
    let messages = vec![
        ConversationMessage::new(ConversationRole::User, "Hello"),
        ConversationMessage::new(ConversationRole::Assistant, "Hi!"),
    ];
    assert_eq!(messages.source_messages().len(), 2);
    assert_eq!(MessageSource::message_count(&messages), 2);
}

#[test]
fn test_message_source_vec_three_messages() {
    let messages = vec![
        ConversationMessage::new(ConversationRole::User, "Hello"),
        ConversationMessage::new(ConversationRole::Assistant, "Hi!"),
        ConversationMessage::new(ConversationRole::System, "Init"),
    ];
    assert_eq!(messages.source_messages().len(), 3);
    assert_eq!(MessageSource::message_count(&messages), 3);
}

#[test]
fn test_message_source_state() {
    let mut state = ConversationViewState::new();
    state.push_user("Hello");
    state.push_assistant("Hi!");
    assert_eq!(state.source_messages().len(), 2);
    assert_eq!(MessageSource::message_count(&state), 2);
    assert_eq!(*state.source_messages()[0].role(), ConversationRole::User);
}

#[test]
fn test_message_source_empty_vec() {
    let messages: Vec<ConversationMessage> = Vec::new();
    assert_eq!(messages.source_messages().len(), 0);
    assert_eq!(MessageSource::message_count(&messages), 0);
}

#[test]
fn test_message_source_empty_state() {
    let state = ConversationViewState::new();
    assert_eq!(state.source_messages().len(), 0);
    assert_eq!(MessageSource::message_count(&state), 0);
}

#[test]
fn test_message_source_dyn_dispatch_vec() {
    let messages = vec![ConversationMessage::new(ConversationRole::User, "Hello")];
    let source: &dyn MessageSource = &messages;
    assert_eq!(source.source_messages().len(), 1);
    assert_eq!(source.message_count(), 1);
}

#[test]
fn test_message_source_dyn_dispatch_state() {
    let mut state = ConversationViewState::new();
    state.push_user("Hello");
    let source: &dyn MessageSource = &state;
    assert_eq!(source.source_messages().len(), 1);
    assert_eq!(source.message_count(), 1);
}
