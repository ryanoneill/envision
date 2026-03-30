//! Chat message types and event definitions.
//!
//! Contains [`ChatRole`], [`ChatMessage`], [`ChatViewMessage`], [`ChatViewOutput`],
//! and the internal [`Focus`] enum used by the [`ChatView`](super::ChatView) component.

use ratatui::prelude::*;

/// The role of a chat message sender.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ChatRole {
    /// A message from the user.
    User,
    /// A system message (announcements, status, etc.).
    System,
    /// A message from an assistant or bot.
    Assistant,
}

impl ChatRole {
    /// Returns the display prefix for this role.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatRole;
    ///
    /// assert_eq!(ChatRole::User.prefix(), "You");
    /// assert_eq!(ChatRole::System.prefix(), "System");
    /// assert_eq!(ChatRole::Assistant.prefix(), "Assistant");
    /// ```
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::User => "You",
            Self::System => "System",
            Self::Assistant => "Assistant",
        }
    }

    /// Returns the display color for this role.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatRole;
    /// use ratatui::style::Color;
    ///
    /// assert_eq!(ChatRole::User.color(), Color::Cyan);
    /// assert_eq!(ChatRole::System.color(), Color::DarkGray);
    /// assert_eq!(ChatRole::Assistant.color(), Color::Green);
    /// ```
    pub fn color(&self) -> Color {
        match self {
            Self::User => Color::Cyan,
            Self::System => Color::DarkGray,
            Self::Assistant => Color::Green,
        }
    }
}

/// A single chat message.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ChatMessage {
    /// The role of the sender.
    pub(super) role: ChatRole,
    /// The message content.
    pub(super) content: String,
    /// Optional timestamp.
    pub(super) timestamp: Option<String>,
    /// Optional username override.
    pub(super) username: Option<String>,
}

impl ChatMessage {
    /// Creates a new chat message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatMessage, ChatRole};
    ///
    /// let msg = ChatMessage::new(ChatRole::User, "Hello!");
    /// assert_eq!(msg.role(), ChatRole::User);
    /// assert_eq!(msg.content(), "Hello!");
    /// ```
    pub fn new(role: ChatRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            timestamp: None,
            username: None,
        }
    }

    /// Sets the timestamp (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatMessage, ChatRole};
    ///
    /// let msg = ChatMessage::new(ChatRole::User, "Hi")
    ///     .with_timestamp("10:30");
    /// assert_eq!(msg.timestamp(), Some("10:30"));
    /// ```
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Sets the username override (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatMessage, ChatRole};
    ///
    /// let msg = ChatMessage::new(ChatRole::User, "Hi")
    ///     .with_username("Alice");
    /// assert_eq!(msg.display_name(), "Alice");
    /// ```
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Returns the role.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatMessage, ChatRole};
    ///
    /// let msg = ChatMessage::new(ChatRole::Assistant, "Hello");
    /// assert_eq!(msg.role(), ChatRole::Assistant);
    /// ```
    pub fn role(&self) -> ChatRole {
        self.role
    }

    /// Returns the content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatMessage, ChatRole};
    ///
    /// let msg = ChatMessage::new(ChatRole::User, "Hello!");
    /// assert_eq!(msg.content(), "Hello!");
    /// ```
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the timestamp.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatMessage, ChatRole};
    ///
    /// let msg = ChatMessage::new(ChatRole::User, "Hi");
    /// assert_eq!(msg.timestamp(), None);
    /// ```
    pub fn timestamp(&self) -> Option<&str> {
        self.timestamp.as_deref()
    }

    /// Returns the username (or the role's default prefix).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatMessage, ChatRole};
    ///
    /// let msg = ChatMessage::new(ChatRole::User, "Hi");
    /// assert_eq!(msg.display_name(), "You");
    ///
    /// let msg2 = ChatMessage::new(ChatRole::User, "Hi").with_username("Bob");
    /// assert_eq!(msg2.display_name(), "Bob");
    /// ```
    pub fn display_name(&self) -> &str {
        self.username
            .as_deref()
            .unwrap_or_else(|| self.role.prefix())
    }
}

/// Internal focus target for the chat view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(crate) enum Focus {
    /// The message history is focused.
    History,
    /// The input field is focused.
    #[default]
    Input,
}

/// Messages that can be sent to a ChatView.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ChatViewMessage {
    /// Type a character in the input field.
    Input(char),
    /// Insert a newline in the input field.
    NewLine,
    /// Delete the character before the cursor.
    Backspace,
    /// Delete the character at the cursor.
    Delete,
    /// Move cursor left in the input field.
    Left,
    /// Move cursor right in the input field.
    Right,
    /// Move cursor up in the input field or scroll history.
    Up,
    /// Move cursor down in the input field or scroll history.
    Down,
    /// Move cursor to start of line.
    Home,
    /// Move cursor to end of line.
    End,
    /// Submit the current input as a user message.
    Submit,
    /// Toggle focus between history and input.
    ToggleFocus,
    /// Focus the input field.
    FocusInput,
    /// Focus the message history.
    FocusHistory,
    /// Scroll history up by one line.
    ScrollUp,
    /// Scroll history down by one line.
    ScrollDown,
    /// Scroll to the top of history.
    ScrollToTop,
    /// Scroll to the bottom of history (newest).
    ScrollToBottom,
    /// Clear the input field.
    ClearInput,
    /// Move cursor to the start of the input.
    InputStart,
    /// Move cursor to the end of the input.
    InputEnd,
    /// Delete from cursor to end of line.
    DeleteToEnd,
    /// Delete from line start to cursor.
    DeleteToStart,
}

/// Output messages from a ChatView.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ChatViewOutput {
    /// The user submitted a message. Contains the message text.
    Submitted(String),
    /// The input text changed.
    InputChanged(String),
}
