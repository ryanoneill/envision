//! A chat interface with message history and multi-line input.
//!
//! [`ChatView`] provides a scrollable message history display and a
//! [`TextArea`](super::TextArea) input field. Messages can be typed
//! as user, system, or assistant and are styled differently per role.
//! State is stored in [`ChatViewState`], updated via [`ChatViewMessage`],
//! and produces [`ChatViewOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, ChatView, ChatViewState,
//!     ChatViewMessage, ChatViewOutput, ChatRole,
//! };
//!
//! let mut state = ChatViewState::new();
//! state.push_system("Welcome to the chat!");
//! state.push_user("Hello");
//! state.push_assistant("Hi! How can I help?");
//!
//! assert_eq!(state.message_count(), 3);
//! assert_eq!(state.messages()[2].role(), ChatRole::Assistant);
//! ```

mod component_impl;
#[cfg(feature = "markdown")]
mod markdown;
pub mod message;
mod render_helpers;

use message::Focus;
pub use message::{ChatMessage, ChatRole, ChatViewMessage, ChatViewOutput};

use std::collections::HashMap;
use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, TextAreaState, ViewContext};
use crate::input::Event;
use crate::scroll::ScrollState;

/// State for a ChatView component.
///
/// Contains the message history, input field, and scroll state.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ChatViewState {
    /// Message history.
    messages: Vec<ChatMessage>,
    /// The text input area.
    #[cfg_attr(feature = "serialization", serde(skip))]
    input: TextAreaState,
    /// Scroll state for the message history.
    scroll: ScrollState,
    /// Whether to auto-scroll to bottom on new messages.
    auto_scroll: bool,
    /// Maximum number of messages to keep.
    max_messages: usize,
    /// Internal focus target.
    focus: Focus,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
    /// Whether to show timestamps.
    show_timestamps: bool,
    /// Input area height in lines.
    input_height: u16,
    /// Custom styles per role (overrides default role colors).
    role_styles: Option<HashMap<ChatRole, Style>>,
    /// Whether to parse message content as markdown for rendering.
    markdown_enabled: bool,
}

impl Default for ChatViewState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input: TextAreaState::new().with_placeholder("Type a message..."),
            scroll: ScrollState::default(),
            auto_scroll: true,
            max_messages: 1000,
            focus: Focus::Input,
            focused: false,
            disabled: false,
            show_timestamps: false,
            input_height: 3,
            role_styles: None,
            markdown_enabled: false,
        }
    }
}

impl PartialEq for ChatViewState {
    fn eq(&self, other: &Self) -> bool {
        self.messages == other.messages
            && self.scroll == other.scroll
            && self.auto_scroll == other.auto_scroll
            && self.max_messages == other.max_messages
            && self.focus == other.focus
            && self.focused == other.focused
            && self.disabled == other.disabled
            && self.show_timestamps == other.show_timestamps
            && self.input_height == other.input_height
            && self.role_styles == other.role_styles
            && self.markdown_enabled == other.markdown_enabled
    }
}

impl ChatViewState {
    /// Creates a new empty chat view state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert_eq!(state.message_count(), 0);
    /// assert!(state.input_value().is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of messages (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new().with_max_messages(50);
    /// assert_eq!(state.max_messages(), 50);
    /// ```
    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }

    /// Sets whether to show timestamps (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new().with_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn with_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    /// Sets the input area height (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new().with_input_height(5);
    /// assert_eq!(state.input_height(), 5);
    /// ```
    pub fn with_input_height(mut self, height: u16) -> Self {
        self.input_height = height.max(1);
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the input placeholder text (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new()
    ///     .with_placeholder("Enter your message...");
    /// ```
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.input.set_placeholder(placeholder);
        self
    }

    // ---- Message manipulation ----

    /// Adds a message from any role.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatMessage, ChatRole};
    ///
    /// let mut state = ChatViewState::new();
    /// state.push_message(ChatMessage::new(ChatRole::System, "Welcome!"));
    /// assert_eq!(state.message_count(), 1);
    /// ```
    pub fn push_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        while self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
        self.scroll.set_content_length(self.messages.len());
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Adds a user message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatRole};
    ///
    /// let mut state = ChatViewState::new();
    /// state.push_user("Hello!");
    /// assert_eq!(state.messages()[0].role(), ChatRole::User);
    /// ```
    pub fn push_user(&mut self, content: impl Into<String>) {
        self.push_message(ChatMessage::new(ChatRole::User, content));
    }

    /// Adds a system message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatRole};
    ///
    /// let mut state = ChatViewState::new();
    /// state.push_system("Server started");
    /// assert_eq!(state.messages()[0].role(), ChatRole::System);
    /// ```
    pub fn push_system(&mut self, content: impl Into<String>) {
        self.push_message(ChatMessage::new(ChatRole::System, content));
    }

    /// Adds an assistant message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatRole};
    ///
    /// let mut state = ChatViewState::new();
    /// state.push_assistant("How can I help?");
    /// assert_eq!(state.messages()[0].role(), ChatRole::Assistant);
    /// ```
    pub fn push_assistant(&mut self, content: impl Into<String>) {
        self.push_message(ChatMessage::new(ChatRole::Assistant, content));
    }

    /// Adds a user message with a timestamp.
    pub fn push_user_with_timestamp(
        &mut self,
        content: impl Into<String>,
        timestamp: impl Into<String>,
    ) {
        self.push_message(ChatMessage::new(ChatRole::User, content).with_timestamp(timestamp));
    }

    /// Adds a system message with a timestamp.
    pub fn push_system_with_timestamp(
        &mut self,
        content: impl Into<String>,
        timestamp: impl Into<String>,
    ) {
        self.push_message(ChatMessage::new(ChatRole::System, content).with_timestamp(timestamp));
    }

    /// Adds an assistant message with a timestamp.
    pub fn push_assistant_with_timestamp(
        &mut self,
        content: impl Into<String>,
        timestamp: impl Into<String>,
    ) {
        self.push_message(ChatMessage::new(ChatRole::Assistant, content).with_timestamp(timestamp));
    }

    /// Clears all messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.push_user("Hello");
    /// state.push_assistant("Hi!");
    /// state.clear_messages();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll = ScrollState::default();
    }

    // ---- Accessors ----

    /// Returns the messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.push_user("Hello");
    /// assert_eq!(state.messages().len(), 1);
    /// assert_eq!(state.messages()[0].content(), "Hello");
    /// ```
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Returns the number of messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// assert_eq!(state.message_count(), 0);
    /// state.push_user("Hi");
    /// assert_eq!(state.message_count(), 1);
    /// ```
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Returns true if there are no messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Returns the current input text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert!(state.input_value().is_empty());
    /// ```
    pub fn input_value(&self) -> String {
        self.input.value()
    }

    /// Sets the input text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_input_value("Draft message");
    /// assert_eq!(state.input_value(), "Draft message");
    /// ```
    pub fn set_input_value(&mut self, value: impl Into<String>) {
        self.input.set_value(value);
    }

    /// Returns the placeholder text for the input field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new()
    ///     .with_placeholder("Type here...");
    /// assert_eq!(state.placeholder(), "Type here...");
    /// ```
    pub fn placeholder(&self) -> &str {
        self.input.placeholder()
    }

    /// Sets the placeholder text for the input field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_placeholder("Enter your message...");
    /// assert_eq!(state.placeholder(), "Enter your message...");
    /// ```
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.input.set_placeholder(placeholder);
    }

    /// Returns the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Returns the maximum number of messages.
    pub fn max_messages(&self) -> usize {
        self.max_messages
    }

    /// Sets the maximum number of messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_max_messages(100);
    /// assert_eq!(state.max_messages(), 100);
    /// ```
    pub fn set_max_messages(&mut self, max: usize) {
        self.max_messages = max;
        if self.messages.len() > max {
            let excess = self.messages.len() - max;
            self.messages.drain(..excess);
            self.scroll.set_content_length(self.messages.len());
        }
    }

    /// Returns whether timestamps are shown.
    pub fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    /// Sets whether timestamps are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_show_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn set_show_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }

    /// Returns whether auto-scroll is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert!(state.auto_scroll()); // enabled by default
    /// ```
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Sets whether auto-scroll is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_auto_scroll(false);
    /// assert!(!state.auto_scroll());
    /// ```
    pub fn set_auto_scroll(&mut self, auto_scroll: bool) {
        self.auto_scroll = auto_scroll;
    }

    /// Returns the input area height.
    pub fn input_height(&self) -> u16 {
        self.input_height
    }

    /// Sets the input area height.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_input_height(5);
    /// assert_eq!(state.input_height(), 5);
    /// ```
    pub fn set_input_height(&mut self, height: u16) {
        self.input_height = height.max(1);
    }

    /// Returns whether the input field is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert!(state.is_input_focused()); // input focused by default
    /// ```
    pub fn is_input_focused(&self) -> bool {
        self.focus == Focus::Input
    }

    /// Returns whether the history is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert!(!state.is_history_focused());
    /// ```
    pub fn is_history_focused(&self) -> bool {
        self.focus == Focus::History
    }

    /// Scrolls the message history to the bottom (newest).
    fn scroll_to_bottom(&mut self) {
        self.scroll.set_content_length(self.messages.len());
        self.scroll.scroll_to_end();
    }

    // ---- Instance methods ----

    /// Returns true if the component is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert!(!state.is_focused());
    /// ```
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_focused(true);
    /// assert!(state.is_focused());
    /// ```
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    // ---- Role style configuration ----

    /// Returns the style for a given role.
    ///
    /// If a custom style has been set via [`set_role_style`](Self::set_role_style),
    /// that style is returned. Otherwise returns `Style::default().fg(role.color())`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatRole};
    /// use ratatui::style::{Color, Style};
    ///
    /// let state = ChatViewState::new();
    /// assert_eq!(state.role_style(&ChatRole::User), Style::default().fg(Color::Cyan));
    /// ```
    pub fn role_style(&self, role: &ChatRole) -> Style {
        self.role_styles
            .as_ref()
            .and_then(|styles| styles.get(role).copied())
            .unwrap_or_else(|| Style::default().fg(role.color()))
    }

    /// Sets a custom style for a chat role.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatRole};
    /// use ratatui::style::{Color, Style};
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_role_style(ChatRole::User, Style::default().fg(Color::Red));
    /// assert_eq!(state.role_style(&ChatRole::User), Style::default().fg(Color::Red));
    /// ```
    pub fn set_role_style(&mut self, role: ChatRole, style: Style) {
        self.role_styles
            .get_or_insert_with(HashMap::new)
            .insert(role, style);
    }

    /// Sets a custom style for a chat role (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatRole};
    /// use ratatui::style::{Color, Style};
    ///
    /// let state = ChatViewState::new()
    ///     .with_role_style(ChatRole::Assistant, Style::default().fg(Color::Yellow));
    /// assert_eq!(state.role_style(&ChatRole::Assistant), Style::default().fg(Color::Yellow));
    /// ```
    pub fn with_role_style(mut self, role: ChatRole, style: Style) -> Self {
        self.set_role_style(role, style);
        self
    }

    /// Clears all custom role styles, reverting to defaults.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatRole};
    /// use ratatui::style::{Color, Style};
    ///
    /// let mut state = ChatViewState::new()
    ///     .with_role_style(ChatRole::User, Style::default().fg(Color::Red));
    /// state.clear_role_styles();
    /// // Now uses default color
    /// assert_eq!(state.role_style(&ChatRole::User), Style::default().fg(Color::Cyan));
    /// ```
    pub fn clear_role_styles(&mut self) {
        self.role_styles = None;
    }

    // ---- Markdown configuration ----

    /// Enables markdown rendering for message content (builder pattern).
    ///
    /// When enabled and the `markdown` feature is active, message content
    /// is parsed as markdown and rendered with rich formatting (headings,
    /// bold, italic, code blocks, lists, etc.).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new().with_markdown(true);
    /// assert!(state.markdown_enabled());
    /// ```
    pub fn with_markdown(mut self, enabled: bool) -> Self {
        self.markdown_enabled = enabled;
        self
    }

    /// Returns whether markdown rendering is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let state = ChatViewState::new();
    /// assert!(!state.markdown_enabled()); // disabled by default
    /// ```
    pub fn markdown_enabled(&self) -> bool {
        self.markdown_enabled
    }

    /// Sets whether markdown rendering is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ChatViewState;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_markdown_enabled(true);
    /// assert!(state.markdown_enabled());
    /// ```
    pub fn set_markdown_enabled(&mut self, enabled: bool) {
        self.markdown_enabled = enabled;
    }

    /// Maps an input event to a chat view message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatViewMessage};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_focused(true);
    /// let event = Event::key(KeyCode::Tab);
    /// assert_eq!(state.handle_event(&event), Some(ChatViewMessage::ToggleFocus));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<ChatViewMessage> {
        ChatView::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatViewOutput};
    /// use envision::input::Event;
    ///
    /// let mut state = ChatViewState::new();
    /// state.set_focused(true);
    /// let event = Event::char('H');
    /// let output = state.dispatch_event(&event);
    /// assert!(matches!(output, Some(ChatViewOutput::InputChanged(_))));
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<ChatViewOutput> {
        ChatView::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ChatViewState, ChatViewMessage, ChatViewOutput};
    ///
    /// let mut state = ChatViewState::new();
    /// let output = state.update(ChatViewMessage::Input('H'));
    /// assert!(matches!(output, Some(ChatViewOutput::InputChanged(_))));
    /// assert_eq!(state.input_value(), "H");
    /// ```
    pub fn update(&mut self, msg: ChatViewMessage) -> Option<ChatViewOutput> {
        ChatView::update(self, msg)
    }
}

/// A chat interface with message history and multi-line input.
///
/// The input area uses a [`TextArea`](super::TextArea) for multi-line
/// editing. Press Ctrl+Enter to submit a message, Tab to toggle between
/// history scrolling and input editing.
///
/// # Key Bindings (Input Mode)
///
/// - Characters — Type text
/// - `Enter` — New line in input
/// - `Ctrl+Enter` — Submit message
/// - `Tab` — Switch to history mode
/// - `Backspace` / `Delete` — Edit text
/// - `Left` / `Right` / `Home` / `End` — Cursor movement
///
/// # Key Bindings (History Mode)
///
/// - `Up` / `k` — Scroll history up
/// - `Down` / `j` — Scroll history down
/// - `Home` — Scroll to top (oldest)
/// - `End` — Scroll to bottom (newest)
/// - `Tab` — Switch to input mode
pub struct ChatView(PhantomData<()>);

#[cfg(test)]
mod scroll_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
