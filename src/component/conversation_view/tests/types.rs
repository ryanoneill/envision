use super::*;

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

#[test]
fn test_message_equality_ignores_id() {
    let msg1 = ConversationMessage::new(ConversationRole::User, "Hello");
    let msg2 = ConversationMessage::new(ConversationRole::User, "Hello");
    // Both have id=0 by default, but equality should not depend on id
    assert_eq!(msg1, msg2);
}
