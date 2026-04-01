//! Core types for the ConversationView component.
//!
//! Contains [`ConversationRole`], [`MessageBlock`], and [`ConversationMessage`]
//! which model an AI conversation with structured message content including
//! text, code blocks, tool use, thinking, and errors.

use ratatui::style::Color;

/// The role of a conversation participant.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ConversationRole {
    /// A message from the user.
    User,
    /// A message from an AI assistant.
    Assistant,
    /// A system message (instructions, configuration).
    System,
    /// A tool/function call result.
    Tool,
    /// A custom role with a user-defined label.
    Custom(String),
}

impl ConversationRole {
    /// Returns the display label for this role.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationRole;
    ///
    /// assert_eq!(ConversationRole::User.label(), "User");
    /// assert_eq!(ConversationRole::Assistant.label(), "Assistant");
    /// assert_eq!(ConversationRole::System.label(), "System");
    /// assert_eq!(ConversationRole::Tool.label(), "Tool");
    /// assert_eq!(ConversationRole::Custom("Moderator".into()).label(), "Moderator");
    /// ```
    pub fn label(&self) -> &str {
        match self {
            Self::User => "User",
            Self::Assistant => "Assistant",
            Self::System => "System",
            Self::Tool => "Tool",
            Self::Custom(name) => name,
        }
    }

    /// Returns the indicator symbol for this role.
    ///
    /// Custom roles use a neutral dash indicator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationRole;
    ///
    /// assert_eq!(ConversationRole::User.indicator(), "\u{25cf}");
    /// assert_eq!(ConversationRole::Assistant.indicator(), "\u{25c6}");
    /// assert_eq!(ConversationRole::System.indicator(), "\u{2699}");
    /// assert_eq!(ConversationRole::Tool.indicator(), "\u{2692}");
    /// assert_eq!(ConversationRole::Custom("Moderator".into()).indicator(), "\u{25cb}");
    /// ```
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::User => "\u{25cf}",      // ●
            Self::Assistant => "\u{25c6}", // ◆
            Self::System => "\u{2699}",    // ⚙
            Self::Tool => "\u{2692}",      // ⚒
            Self::Custom(_) => "\u{25cb}", // ○ (neutral open circle)
        }
    }

    /// Returns the default color for this role.
    ///
    /// Custom roles use a neutral gray color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationRole;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(ConversationRole::User.color(), Color::Green);
    /// assert_eq!(ConversationRole::Assistant.color(), Color::Blue);
    /// assert_eq!(ConversationRole::System.color(), Color::DarkGray);
    /// assert_eq!(ConversationRole::Tool.color(), Color::Yellow);
    /// assert_eq!(ConversationRole::Custom("Moderator".into()).color(), Color::Gray);
    /// ```
    pub fn color(&self) -> Color {
        match self {
            Self::User => Color::Green,
            Self::Assistant => Color::Blue,
            Self::System => Color::DarkGray,
            Self::Tool => Color::Yellow,
            Self::Custom(_) => Color::Gray,
        }
    }
}

/// A block of content within a conversation message.
///
/// Messages are composed of one or more blocks, each representing
/// a different type of content (text, code, tool use, etc.).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum MessageBlock {
    /// Plain text content.
    Text(String),
    /// A code block with optional language identifier.
    Code {
        /// The code content.
        code: String,
        /// Optional programming language for syntax indication.
        language: Option<String>,
    },
    /// A tool/function use block.
    ToolUse {
        /// The name of the tool being invoked.
        name: String,
        /// A textual representation of the tool input, if any.
        input: Option<String>,
        /// A textual representation of the tool output, if any.
        output: Option<String>,
    },
    /// A thinking/reasoning block (e.g., chain-of-thought).
    Thinking(String),
    /// An error block.
    Error(String),
}

impl MessageBlock {
    /// Creates a text block.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MessageBlock;
    ///
    /// let block = MessageBlock::text("Hello, world!");
    /// assert!(matches!(block, MessageBlock::Text(ref s) if s == "Hello, world!"));
    /// ```
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Creates a code block with a language.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MessageBlock;
    ///
    /// let block = MessageBlock::code("fn main() {}", Some("rust"));
    /// if let MessageBlock::Code { code, language } = &block {
    ///     assert_eq!(code, "fn main() {}");
    ///     assert_eq!(language.as_deref(), Some("rust"));
    /// }
    /// ```
    pub fn code(code: impl Into<String>, language: Option<&str>) -> Self {
        Self::Code {
            code: code.into(),
            language: language.map(String::from),
        }
    }

    /// Creates a tool use block with only a name.
    ///
    /// Use [`with_input`](Self::with_input) and [`with_output`](Self::with_output)
    /// to attach input and output data via the builder pattern.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MessageBlock;
    ///
    /// let block = MessageBlock::tool_use("search");
    /// if let MessageBlock::ToolUse { name, input, output } = &block {
    ///     assert_eq!(name, "search");
    ///     assert!(input.is_none());
    ///     assert!(output.is_none());
    /// }
    /// ```
    pub fn tool_use(name: impl Into<String>) -> Self {
        Self::ToolUse {
            name: name.into(),
            input: None,
            output: None,
        }
    }

    /// Sets the input for a tool use block (builder pattern).
    ///
    /// Has no effect on non-`ToolUse` variants.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MessageBlock;
    ///
    /// let block = MessageBlock::tool_use("search")
    ///     .with_input("query: rust TUI");
    /// if let MessageBlock::ToolUse { name, input, .. } = &block {
    ///     assert_eq!(name, "search");
    ///     assert_eq!(input.as_deref(), Some("query: rust TUI"));
    /// }
    /// ```
    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        if let Self::ToolUse {
            input: ref mut i, ..
        } = self
        {
            *i = Some(input.into());
        }
        self
    }

    /// Sets the output for a tool use block (builder pattern).
    ///
    /// Has no effect on non-`ToolUse` variants.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MessageBlock;
    ///
    /// let block = MessageBlock::tool_use("search")
    ///     .with_input("query: rust TUI")
    ///     .with_output("Found 5 results");
    /// if let MessageBlock::ToolUse { name, input, output } = &block {
    ///     assert_eq!(name, "search");
    ///     assert_eq!(input.as_deref(), Some("query: rust TUI"));
    ///     assert_eq!(output.as_deref(), Some("Found 5 results"));
    /// }
    /// ```
    pub fn with_output(mut self, output: impl Into<String>) -> Self {
        if let Self::ToolUse {
            output: ref mut o, ..
        } = self
        {
            *o = Some(output.into());
        }
        self
    }

    /// Creates a thinking block.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MessageBlock;
    ///
    /// let block = MessageBlock::thinking("Let me reason about this...");
    /// assert!(matches!(block, MessageBlock::Thinking(ref s) if s == "Let me reason about this..."));
    /// ```
    pub fn thinking(content: impl Into<String>) -> Self {
        Self::Thinking(content.into())
    }

    /// Creates an error block.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MessageBlock;
    ///
    /// let block = MessageBlock::error("Connection timeout");
    /// assert!(matches!(block, MessageBlock::Error(ref s) if s == "Connection timeout"));
    /// ```
    pub fn error(content: impl Into<String>) -> Self {
        Self::Error(content.into())
    }

    /// Returns true if this is a text block.
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Returns true if this is a code block.
    pub fn is_code(&self) -> bool {
        matches!(self, Self::Code { .. })
    }

    /// Returns true if this is a tool use block.
    pub fn is_tool_use(&self) -> bool {
        matches!(self, Self::ToolUse { .. })
    }

    /// Returns true if this is a thinking block.
    pub fn is_thinking(&self) -> bool {
        matches!(self, Self::Thinking(_))
    }

    /// Returns true if this is an error block.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

/// A single message in a conversation.
///
/// Each message has a role, a sequence of content blocks, an optional
/// timestamp, and a streaming flag to indicate whether the message is
/// still being received.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ConversationMessage {
    /// The role of the message sender.
    pub(super) role: ConversationRole,
    /// The content blocks of the message.
    pub(super) blocks: Vec<MessageBlock>,
    /// Optional timestamp string.
    pub(super) timestamp: Option<String>,
    /// Whether this message is still being streamed.
    pub(super) streaming: bool,
}

impl ConversationMessage {
    /// Creates a new message with a single text block.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationMessage, ConversationRole};
    ///
    /// let msg = ConversationMessage::new(ConversationRole::User, "Hello!");
    /// assert_eq!(*msg.role(), ConversationRole::User);
    /// assert_eq!(msg.blocks().len(), 1);
    /// ```
    pub fn new(role: ConversationRole, text: impl Into<String>) -> Self {
        Self {
            role,
            blocks: vec![MessageBlock::Text(text.into())],
            timestamp: None,
            streaming: false,
        }
    }

    /// Creates a new message with the given blocks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationMessage, ConversationRole, MessageBlock};
    ///
    /// let msg = ConversationMessage::with_blocks(
    ///     ConversationRole::Assistant,
    ///     vec![
    ///         MessageBlock::text("Here is the code:"),
    ///         MessageBlock::code("println!(\"hi\");", Some("rust")),
    ///     ],
    /// );
    /// assert_eq!(msg.blocks().len(), 2);
    /// ```
    pub fn with_blocks(role: ConversationRole, blocks: Vec<MessageBlock>) -> Self {
        Self {
            role,
            blocks,
            timestamp: None,
            streaming: false,
        }
    }

    /// Sets the timestamp (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationMessage, ConversationRole};
    ///
    /// let msg = ConversationMessage::new(ConversationRole::User, "Hi")
    ///     .with_timestamp("14:30");
    /// assert_eq!(msg.timestamp(), Some("14:30"));
    /// ```
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Sets the streaming flag (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationMessage, ConversationRole};
    ///
    /// let msg = ConversationMessage::new(ConversationRole::Assistant, "Thinking")
    ///     .with_streaming(true);
    /// assert!(msg.is_streaming());
    /// ```
    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    /// Returns the role of the message sender.
    pub fn role(&self) -> &ConversationRole {
        &self.role
    }

    /// Returns the content blocks.
    pub fn blocks(&self) -> &[MessageBlock] {
        &self.blocks
    }

    /// Returns a mutable reference to the content blocks.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationMessage, ConversationRole, MessageBlock};
    ///
    /// let mut msg = ConversationMessage::new(ConversationRole::User, "Hello");
    /// msg.blocks_mut().push(MessageBlock::text(" world"));
    /// assert_eq!(msg.blocks().len(), 2);
    /// ```
    pub fn blocks_mut(&mut self) -> &mut Vec<MessageBlock> {
        &mut self.blocks
    }

    /// Replace all blocks in this message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationMessage, ConversationRole, MessageBlock};
    ///
    /// let mut msg = ConversationMessage::new(ConversationRole::User, "Hello");
    /// msg.set_blocks(vec![MessageBlock::text("Replaced"), MessageBlock::code("x = 1", Some("py"))]);
    /// assert_eq!(msg.blocks().len(), 2);
    /// assert!(msg.blocks()[0].is_text());
    /// assert!(msg.blocks()[1].is_code());
    /// ```
    pub fn set_blocks(&mut self, blocks: Vec<MessageBlock>) {
        self.blocks = blocks;
    }

    /// Returns the timestamp if set.
    pub fn timestamp(&self) -> Option<&str> {
        self.timestamp.as_deref()
    }

    /// Returns whether the message is still being streamed.
    pub fn is_streaming(&self) -> bool {
        self.streaming
    }

    /// Sets the streaming flag.
    pub fn set_streaming(&mut self, streaming: bool) {
        self.streaming = streaming;
    }

    /// Appends a block to this message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationMessage, ConversationRole, MessageBlock};
    ///
    /// let mut msg = ConversationMessage::new(ConversationRole::Assistant, "Result:");
    /// msg.push_block(MessageBlock::code("let x = 42;", Some("rust")));
    /// assert_eq!(msg.blocks().len(), 2);
    /// ```
    pub fn push_block(&mut self, block: MessageBlock) {
        self.blocks.push(block);
    }

    /// Returns the plain text content of all text blocks concatenated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationMessage, ConversationRole, MessageBlock};
    ///
    /// let msg = ConversationMessage::with_blocks(
    ///     ConversationRole::User,
    ///     vec![
    ///         MessageBlock::text("Hello "),
    ///         MessageBlock::text("world"),
    ///     ],
    /// );
    /// assert_eq!(msg.text_content(), "Hello world");
    /// ```
    pub fn text_content(&self) -> String {
        self.blocks
            .iter()
            .filter_map(|b| match b {
                MessageBlock::Text(s) => Some(s.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}
