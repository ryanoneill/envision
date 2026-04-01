//! Core types for the ConversationView component.
//!
//! Contains [`ConversationRole`], [`MessageBlock`], and [`ConversationMessage`]
//! which model an AI conversation with structured message content including
//! text, code blocks, tool use, thinking, and errors.

use ratatui::style::Color;

/// An opaque handle to a conversation message for streaming updates.
///
/// Returned by `ConversationViewState::push_message` and related methods.
/// Use with `ConversationViewState::update_by_handle` to update a specific
/// message, even after other messages have been pushed or evicted.
///
/// # Example
///
/// ```rust
/// use envision::component::{ConversationViewState, MessageBlock};
///
/// let mut state = ConversationViewState::new();
/// let handle = state.push_assistant("Thinking...");
/// state.update_by_handle(handle, |msg| {
///     msg.push_block(MessageBlock::code("let x = 42;", Some("rust")));
///     msg.set_streaming(false);
/// });
/// assert_eq!(state.messages()[0].blocks().len(), 2);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MessageHandle(pub(super) u64);

/// The role of a conversation participant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// ```
    pub fn label(&self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Assistant => "Assistant",
            Self::System => "System",
            Self::Tool => "Tool",
        }
    }

    /// Returns the indicator symbol for this role.
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
    /// ```
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::User => "\u{25cf}",      // ●
            Self::Assistant => "\u{25c6}", // ◆
            Self::System => "\u{2699}",    // ⚙
            Self::Tool => "\u{2692}",      // ⚒
        }
    }

    /// Returns the default color for this role.
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
    /// ```
    pub fn color(&self) -> Color {
        match self {
            Self::User => Color::Green,
            Self::Assistant => Color::Blue,
            Self::System => Color::DarkGray,
            Self::Tool => Color::Yellow,
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
        /// A textual representation of the tool input.
        input: String,
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

    /// Creates a tool use block.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MessageBlock;
    ///
    /// let block = MessageBlock::tool_use("search", "query: rust TUI");
    /// if let MessageBlock::ToolUse { name, input } = &block {
    ///     assert_eq!(name, "search");
    ///     assert_eq!(input, "query: rust TUI");
    /// }
    /// ```
    pub fn tool_use(name: impl Into<String>, input: impl Into<String>) -> Self {
        Self::ToolUse {
            name: name.into(),
            input: input.into(),
        }
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
#[derive(Clone, Debug, Eq)]
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
    /// Internal identifier for handle-based lookup (not serialized).
    #[cfg_attr(feature = "serialization", serde(skip, default))]
    pub(super) id: u64,
}

impl PartialEq for ConversationMessage {
    fn eq(&self, other: &Self) -> bool {
        self.role == other.role
            && self.blocks == other.blocks
            && self.timestamp == other.timestamp
            && self.streaming == other.streaming
    }
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
    /// assert_eq!(msg.role(), ConversationRole::User);
    /// assert_eq!(msg.blocks().len(), 1);
    /// ```
    pub fn new(role: ConversationRole, text: impl Into<String>) -> Self {
        Self {
            role,
            blocks: vec![MessageBlock::Text(text.into())],
            timestamp: None,
            streaming: false,
            id: 0,
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
            id: 0,
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
    pub fn role(&self) -> ConversationRole {
        self.role
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
