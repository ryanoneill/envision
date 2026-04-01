use super::*;
use crate::component::Focusable;
use crate::input::{Event, KeyCode};

fn focused_state() -> ConversationViewState {
    let mut state = ConversationViewState::new();
    ConversationView::set_focused(&mut state, true);
    state
}

fn state_with_messages() -> ConversationViewState {
    let mut state = focused_state();
    state.push_system("Welcome to the conversation.");
    state.push_user("Hello, can you help me?");
    state.push_assistant("Of course! What do you need?");
    state
}

// =============================================================================
// ConversationRole
// =============================================================================

#[test]
fn test_role_label() {
    assert_eq!(ConversationRole::User.label(), "User");
    assert_eq!(ConversationRole::Assistant.label(), "Assistant");
    assert_eq!(ConversationRole::System.label(), "System");
    assert_eq!(ConversationRole::Tool.label(), "Tool");
}

#[test]
fn test_role_indicator() {
    assert_eq!(ConversationRole::User.indicator(), "\u{25cf}");
    assert_eq!(ConversationRole::Assistant.indicator(), "\u{25c6}");
    assert_eq!(ConversationRole::System.indicator(), "\u{2699}");
    assert_eq!(ConversationRole::Tool.indicator(), "\u{2692}");
}

#[test]
fn test_role_color() {
    use ratatui::style::Color;
    assert_eq!(ConversationRole::User.color(), Color::Green);
    assert_eq!(ConversationRole::Assistant.color(), Color::Blue);
    assert_eq!(ConversationRole::System.color(), Color::DarkGray);
    assert_eq!(ConversationRole::Tool.color(), Color::Yellow);
}

#[test]
fn test_role_equality() {
    assert_eq!(ConversationRole::User, ConversationRole::User);
    assert_ne!(ConversationRole::User, ConversationRole::Assistant);
}

#[test]
fn test_role_clone() {
    let role = ConversationRole::Tool;
    let cloned = role.clone();
    assert_eq!(role, cloned);
}

// =============================================================================
// MessageBlock
// =============================================================================

#[test]
fn test_text_block() {
    let block = MessageBlock::text("Hello");
    assert!(block.is_text());
    assert!(!block.is_code());
    assert!(!block.is_tool_use());
    assert!(!block.is_thinking());
    assert!(!block.is_error());
}

#[test]
fn test_code_block() {
    let block = MessageBlock::code("let x = 1;", Some("rust"));
    assert!(block.is_code());
    assert!(!block.is_text());
    if let MessageBlock::Code { code, language } = &block {
        assert_eq!(code, "let x = 1;");
        assert_eq!(language.as_deref(), Some("rust"));
    } else {
        panic!("Expected Code block");
    }
}

#[test]
fn test_code_block_no_language() {
    let block = MessageBlock::code("echo hello", None);
    if let MessageBlock::Code { language, .. } = &block {
        assert!(language.is_none());
    } else {
        panic!("Expected Code block");
    }
}

#[test]
fn test_tool_use_block() {
    let block = MessageBlock::tool_use("search").with_input("query: TUI");
    assert!(block.is_tool_use());
    if let MessageBlock::ToolUse { name, input, .. } = &block {
        assert_eq!(name, "search");
        assert_eq!(input.as_deref(), Some("query: TUI"));
    } else {
        panic!("Expected ToolUse block");
    }
}

#[test]
fn test_thinking_block() {
    let block = MessageBlock::thinking("Reasoning...");
    assert!(block.is_thinking());
    if let MessageBlock::Thinking(content) = &block {
        assert_eq!(content, "Reasoning...");
    } else {
        panic!("Expected Thinking block");
    }
}

#[test]
fn test_error_block() {
    let block = MessageBlock::error("Something failed");
    assert!(block.is_error());
    if let MessageBlock::Error(content) = &block {
        assert_eq!(content, "Something failed");
    } else {
        panic!("Expected Error block");
    }
}

#[test]
fn test_block_equality() {
    assert_eq!(MessageBlock::text("a"), MessageBlock::text("a"));
    assert_ne!(MessageBlock::text("a"), MessageBlock::text("b"));
    assert_ne!(MessageBlock::text("a"), MessageBlock::error("a"));
}

#[test]
fn test_block_clone() {
    let block = MessageBlock::code("x", Some("py"));
    let cloned = block.clone();
    assert_eq!(block, cloned);
}

// =============================================================================
// ConversationMessage
// =============================================================================

#[test]
fn test_message_new() {
    let msg = ConversationMessage::new(ConversationRole::User, "Hello");
    assert_eq!(*msg.role(), ConversationRole::User);
    assert_eq!(msg.blocks().len(), 1);
    assert!(matches!(&msg.blocks()[0], MessageBlock::Text(s) if s == "Hello"));
    assert!(!msg.is_streaming());
    assert!(msg.timestamp().is_none());
}

#[test]
fn test_message_with_blocks() {
    let msg = ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::text("Here:"),
            MessageBlock::code("fn main() {}", Some("rust")),
        ],
    );
    assert_eq!(msg.blocks().len(), 2);
    assert!(msg.blocks()[0].is_text());
    assert!(msg.blocks()[1].is_code());
}

#[test]
fn test_message_with_timestamp() {
    let msg = ConversationMessage::new(ConversationRole::User, "Hi").with_timestamp("14:30");
    assert_eq!(msg.timestamp(), Some("14:30"));
}

#[test]
fn test_message_with_streaming() {
    let msg = ConversationMessage::new(ConversationRole::Assistant, "...").with_streaming(true);
    assert!(msg.is_streaming());
}

#[test]
fn test_message_set_streaming() {
    let mut msg = ConversationMessage::new(ConversationRole::Assistant, "...");
    assert!(!msg.is_streaming());
    msg.set_streaming(true);
    assert!(msg.is_streaming());
    msg.set_streaming(false);
    assert!(!msg.is_streaming());
}

#[test]
fn test_message_push_block() {
    let mut msg = ConversationMessage::new(ConversationRole::Assistant, "Result:");
    msg.push_block(MessageBlock::code("42", None));
    assert_eq!(msg.blocks().len(), 2);
}

#[test]
fn test_message_blocks_mut() {
    let mut msg = ConversationMessage::new(ConversationRole::User, "Hello");
    msg.blocks_mut().push(MessageBlock::text(" world"));
    assert_eq!(msg.blocks().len(), 2);
}

#[test]
fn test_message_set_blocks() {
    let mut msg = ConversationMessage::new(ConversationRole::User, "Hello");
    msg.set_blocks(vec![
        MessageBlock::text("Replaced"),
        MessageBlock::code("x = 1", Some("py")),
    ]);
    assert_eq!(msg.blocks().len(), 2);
    assert!(msg.blocks()[0].is_text());
    assert!(msg.blocks()[1].is_code());
}

#[test]
fn test_message_set_blocks_empty() {
    let mut msg = ConversationMessage::new(ConversationRole::User, "Hello");
    msg.set_blocks(vec![]);
    assert!(msg.blocks().is_empty());
}

#[test]
fn test_message_text_content() {
    let msg = ConversationMessage::with_blocks(
        ConversationRole::User,
        vec![
            MessageBlock::text("Hello "),
            MessageBlock::code("code", None),
            MessageBlock::text("world"),
        ],
    );
    assert_eq!(msg.text_content(), "Hello world");
}

#[test]
fn test_message_text_content_no_text_blocks() {
    let msg = ConversationMessage::with_blocks(
        ConversationRole::Tool,
        vec![MessageBlock::code("x = 1", Some("python"))],
    );
    assert_eq!(msg.text_content(), "");
}

#[test]
fn test_message_equality() {
    let msg1 = ConversationMessage::new(ConversationRole::User, "Hi");
    let msg2 = ConversationMessage::new(ConversationRole::User, "Hi");
    assert_eq!(msg1, msg2);
}

#[test]
fn test_message_inequality_role() {
    let msg1 = ConversationMessage::new(ConversationRole::User, "Hi");
    let msg2 = ConversationMessage::new(ConversationRole::Assistant, "Hi");
    assert_ne!(msg1, msg2);
}

#[test]
fn test_message_clone() {
    let msg = ConversationMessage::new(ConversationRole::User, "Hi")
        .with_timestamp("12:00")
        .with_streaming(true);
    let cloned = msg.clone();
    assert_eq!(msg, cloned);
}

// =============================================================================
// Construction and defaults
// =============================================================================

#[test]
fn test_new() {
    let state = ConversationViewState::new();
    assert_eq!(state.message_count(), 0);
    assert!(state.is_empty());
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
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

#[test]
fn test_with_disabled() {
    let state = ConversationViewState::new().with_disabled(true);
    assert!(state.is_disabled());
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
// Focus and disabled state
// =============================================================================

#[test]
fn test_focused() {
    let mut state = ConversationViewState::new();
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());
}

#[test]
fn test_disabled() {
    let mut state = ConversationViewState::new();
    assert!(!state.is_disabled());
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_focusable_trait() {
    let mut state = ConversationView::init();
    assert!(!ConversationView::is_focused(&state));
    ConversationView::focus(&mut state);
    assert!(ConversationView::is_focused(&state));
    ConversationView::blur(&mut state);
    assert!(!ConversationView::is_focused(&state));
}

#[test]
fn test_disableable_trait() {
    let mut state = ConversationView::init();
    assert!(!ConversationView::is_disabled(&state));
    ConversationView::disable(&mut state);
    assert!(ConversationView::is_disabled(&state));
    ConversationView::enable(&mut state);
    assert!(!ConversationView::is_disabled(&state));
}

// =============================================================================
// Event handling
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = ConversationViewState::new();
    assert_eq!(
        ConversationView::handle_event(&state, &Event::char('k')),
        None
    );
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    assert_eq!(
        ConversationView::handle_event(&state, &Event::char('k')),
        None
    );
}

#[test]
fn test_disabled_ignores_updates() {
    let mut state = focused_state();
    state.set_disabled(true);
    let output = ConversationView::update(&mut state, ConversationViewMessage::ScrollUp);
    assert_eq!(output, None);
}

#[test]
fn test_scroll_up_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(ConversationViewMessage::ScrollUp)
    );
    assert_eq!(
        ConversationView::handle_event(&state, &Event::char('k')),
        Some(ConversationViewMessage::ScrollUp)
    );
}

#[test]
fn test_scroll_down_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(ConversationViewMessage::ScrollDown)
    );
    assert_eq!(
        ConversationView::handle_event(&state, &Event::char('j')),
        Some(ConversationViewMessage::ScrollDown)
    );
}

#[test]
fn test_scroll_to_top_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(ConversationViewMessage::ScrollToTop)
    );
    assert_eq!(
        ConversationView::handle_event(&state, &Event::char('g')),
        Some(ConversationViewMessage::ScrollToTop)
    );
}

#[test]
fn test_scroll_to_bottom_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(&state, &Event::key(KeyCode::End)),
        Some(ConversationViewMessage::ScrollToBottom)
    );
    assert_eq!(
        ConversationView::handle_event(&state, &Event::char('G')),
        Some(ConversationViewMessage::ScrollToBottom)
    );
}

#[test]
fn test_page_up_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(&state, &Event::key(KeyCode::PageUp)),
        Some(ConversationViewMessage::PageUp)
    );
}

#[test]
fn test_page_down_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(&state, &Event::key(KeyCode::PageDown)),
        Some(ConversationViewMessage::PageDown)
    );
}

#[test]
fn test_unrecognized_key_ignored() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(&state, &Event::char('x')),
        None
    );
}

// =============================================================================
// Scrolling behavior
// =============================================================================

#[test]
fn test_scroll_up_disables_auto_scroll() {
    let mut state = state_with_messages();
    assert!(state.auto_scroll());
    ConversationView::update(&mut state, ConversationViewMessage::ScrollUp);
    assert!(!state.auto_scroll());
}

#[test]
fn test_scroll_to_bottom_enables_auto_scroll() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollToBottom);
    assert!(state.auto_scroll());
}

#[test]
fn test_scroll_to_top_disables_auto_scroll() {
    let mut state = state_with_messages();
    ConversationView::update(&mut state, ConversationViewMessage::ScrollToTop);
    assert!(!state.auto_scroll());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_down_output() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    let output = ConversationView::update(&mut state, ConversationViewMessage::ScrollDown);
    assert!(matches!(
        output,
        Some(ConversationViewOutput::ScrollChanged { .. })
    ));
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    state.scroll.set_offset(0);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_page_up_disables_auto_scroll() {
    let mut state = state_with_messages();
    ConversationView::update(&mut state, ConversationViewMessage::PageUp);
    assert!(!state.auto_scroll());
}

#[test]
fn test_scroll_empty_conversation() {
    let mut state = focused_state();
    let output = ConversationView::update(&mut state, ConversationViewMessage::ScrollDown);
    assert!(matches!(
        output,
        Some(ConversationViewOutput::ScrollChanged { offset: 0 })
    ));
}

// =============================================================================
// Toggle collapse
// =============================================================================

#[test]
fn test_toggle_collapse_via_message() {
    let mut state = ConversationViewState::new();
    ConversationView::update(
        &mut state,
        ConversationViewMessage::ToggleCollapse("thinking".to_string()),
    );
    assert!(state.is_collapsed("thinking"));
    ConversationView::update(
        &mut state,
        ConversationViewMessage::ToggleCollapse("thinking".to_string()),
    );
    assert!(!state.is_collapsed("thinking"));
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::char('k'));
    assert_eq!(msg, Some(ConversationViewMessage::ScrollUp));
}

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    state.push_user("Hello");
    let output = state.update(ConversationViewMessage::ScrollDown);
    assert!(output.is_some());
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state();
    state.push_user("Hello");
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert!(output.is_some());
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
