//! A conversation display for AI/LLM interactions with structured message blocks.
//!
//! [`ConversationView`] provides a scrollable, read-only view of a conversation
//! composed of structured messages. Each message has a [`ConversationRole`] (User,
//! Assistant, System, Tool) and contains one or more [`MessageBlock`]s (Text, Code,
//! ToolUse, Thinking, Error).
//!
//! Unlike [`ChatView`](super::ChatView) which includes an input field, this component
//! is purely a display widget for viewing conversation history, making it suitable
//! for read-only conversation logs, streaming AI responses, and tool-use displays.
//!
//! State is stored in [`ConversationViewState`], updated via [`ConversationViewMessage`],
//! and produces [`ConversationViewOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, ConversationView, ConversationViewState,
//!     ConversationMessage, ConversationRole, MessageBlock,
//! };
//!
//! let mut state = ConversationViewState::new();
//! state.push_message(ConversationMessage::new(ConversationRole::User, "Hello!"));
//! state.push_message(ConversationMessage::with_blocks(
//!     ConversationRole::Assistant,
//!     vec![
//!         MessageBlock::text("Here is some code:"),
//!         MessageBlock::code("fn main() {}", Some("rust")),
//!     ],
//! ));
//!
//! assert_eq!(state.message_count(), 2);
//! ```

mod render;
pub mod types;

use types::MessageSource;
pub use types::{ConversationMessage, ConversationRole, MessageBlock, MessageHandle};

use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::scroll::ScrollState;

impl MessageSource for ConversationViewState {
    fn source_messages(&self) -> &[ConversationMessage] {
        &self.messages
    }
}

/// Messages that can be sent to a ConversationView.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ConversationViewMessage {
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll to the top of the conversation.
    ScrollToTop,
    /// Scroll to the bottom of the conversation.
    ScrollToBottom,
    /// Page up (scroll by viewport height).
    PageUp,
    /// Page down (scroll by viewport height).
    PageDown,
    /// Toggle collapse state of a named block.
    ToggleCollapse(String),
}

/// Output messages from a ConversationView.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ConversationViewOutput {
    /// The scroll position changed.
    ScrollChanged {
        /// The current scroll offset.
        offset: usize,
    },
}

/// State for a ConversationView component.
///
/// Contains the message history, scroll state, and display configuration.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ConversationViewState {
    /// Message history.
    pub(super) messages: Vec<ConversationMessage>,
    /// Scroll state for the conversation.
    pub(super) scroll: ScrollState,
    /// Whether to auto-scroll to bottom on new messages.
    pub(super) auto_scroll: bool,
    /// Maximum number of messages to keep.
    pub(super) max_messages: usize,
    /// Whether to show timestamps.
    pub(super) show_timestamps: bool,
    /// Whether to show role labels/headers.
    pub(super) show_role_labels: bool,
    /// Whether to render text blocks as markdown (requires `markdown` feature).
    pub(super) markdown_enabled: bool,
    /// Last known render width for scroll content length estimation.
    pub(super) last_known_width: usize,
    /// Optional title for the conversation panel.
    pub(super) title: Option<String>,
    /// Set of collapsed block keys (e.g., "tool:search", "thinking").
    pub(super) collapsed_blocks: HashSet<String>,
    /// Optional status text rendered at the bottom of the viewport, above the border.
    pub(super) status: Option<String>,
    /// Next unique ID for message handles.
    #[cfg_attr(feature = "serialization", serde(skip, default))]
    pub(super) next_id: u64,
    /// Per-role style overrides. When set, these take precedence over
    /// the default colors from [`ConversationRole::color()`].
    #[cfg_attr(feature = "serialization", serde(skip, default))]
    pub(super) role_style_overrides: HashMap<ConversationRole, Style>,
}

impl Default for ConversationViewState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            scroll: ScrollState::default(),
            auto_scroll: true,
            max_messages: 1000,
            show_timestamps: false,
            show_role_labels: true,
            markdown_enabled: false,
            last_known_width: 80,
            title: None,
            collapsed_blocks: HashSet::new(),
            status: None,
            next_id: 1,
            role_style_overrides: HashMap::new(),
        }
    }
}

impl PartialEq for ConversationViewState {
    fn eq(&self, other: &Self) -> bool {
        self.messages == other.messages
            && self.scroll == other.scroll
            && self.auto_scroll == other.auto_scroll
            && self.max_messages == other.max_messages
            && self.show_timestamps == other.show_timestamps
            && self.show_role_labels == other.show_role_labels
            && self.title == other.title
            && self.collapsed_blocks == other.collapsed_blocks
            && self.status == other.status
            && self.role_style_overrides == other.role_style_overrides
        // next_id is intentionally excluded from equality
    }
}

impl ConversationViewState {
    /// Creates a new empty conversation view state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new();
    /// assert_eq!(state.message_count(), 0);
    /// assert!(state.auto_scroll());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new().with_title("Chat Session");
    /// assert_eq!(state.title(), Some("Chat Session"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the maximum number of messages (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new().with_max_messages(50);
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
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new().with_show_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn with_show_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    /// Sets whether to show role labels (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new().with_show_role_labels(false);
    /// assert!(!state.show_role_labels());
    /// ```
    pub fn with_show_role_labels(mut self, show: bool) -> Self {
        self.show_role_labels = show;
        self
    }

    /// Enables or disables markdown rendering for text blocks (builder pattern).
    ///
    /// When enabled, text blocks are rendered as markdown (headings, bold,
    /// italic, code, lists, etc.) instead of plain text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new().with_markdown(true);
    /// assert!(state.markdown_enabled());
    /// ```
    #[cfg(feature = "markdown")]
    pub fn with_markdown(mut self, enabled: bool) -> Self {
        self.markdown_enabled = enabled;
        self
    }

    /// Returns whether markdown rendering is enabled.
    ///
    /// Always returns `false` when the `markdown` Cargo feature is not
    /// enabled (since the setter methods are not available).
    pub fn markdown_enabled(&self) -> bool {
        self.markdown_enabled
    }

    /// Sets whether markdown rendering is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_markdown_enabled(true);
    /// assert!(state.markdown_enabled());
    /// ```
    #[cfg(feature = "markdown")]
    pub fn set_markdown_enabled(&mut self, enabled: bool) {
        self.markdown_enabled = enabled;
    }

    // ---- Role style overrides ----

    /// Sets a style override for a specific role (builder pattern).
    ///
    /// When set, this style is used instead of the default color from
    /// [`ConversationRole::color()`] for messages from this role.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationRole};
    /// use ratatui::style::{Color, Style};
    ///
    /// let state = ConversationViewState::new()
    ///     .with_role_style(ConversationRole::User, Style::default().fg(Color::Cyan))
    ///     .with_role_style(ConversationRole::Assistant, Style::default().fg(Color::Magenta));
    /// assert!(state.role_style_override(&ConversationRole::User).is_some());
    /// ```
    pub fn with_role_style(mut self, role: ConversationRole, style: Style) -> Self {
        self.role_style_overrides.insert(role, style);
        self
    }

    /// Returns the style override for a role, if one is set.
    pub fn role_style_override(&self, role: &ConversationRole) -> Option<&Style> {
        self.role_style_overrides.get(role)
    }

    /// Sets a style override for a specific role.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationRole};
    /// use ratatui::style::{Color, Style};
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_role_style(ConversationRole::Assistant, Style::default().fg(Color::Red));
    /// assert_eq!(
    ///     state.role_style_override(&ConversationRole::Assistant),
    ///     Some(&Style::default().fg(Color::Red)),
    /// );
    /// ```
    pub fn set_role_style(&mut self, role: ConversationRole, style: Style) {
        self.role_style_overrides.insert(role, style);
    }

    /// Removes a style override for a role, reverting to the default color.
    pub fn clear_role_style(&mut self, role: &ConversationRole) {
        self.role_style_overrides.remove(role);
    }

    // ---- Message manipulation ----

    /// Adds a message to the conversation and returns a [`MessageHandle`]
    /// that can be used to update it later.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationMessage, ConversationRole};
    ///
    /// let mut state = ConversationViewState::new();
    /// let handle = state.push_message(ConversationMessage::new(ConversationRole::User, "Hello"));
    /// assert_eq!(state.message_count(), 1);
    /// // The handle can be used later with update_by_handle
    /// let _ = handle;
    /// ```
    pub fn push_message(&mut self, mut message: ConversationMessage) -> MessageHandle {
        let id = self.next_id;
        self.next_id += 1;
        message.id = id;
        let handle = MessageHandle(id);
        self.messages.push(message);
        while self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
        self.update_scroll_content_length();
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
        handle
    }

    /// Adds a user text message and returns a [`MessageHandle`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationRole};
    ///
    /// let mut state = ConversationViewState::new();
    /// let handle = state.push_user("Hello!");
    /// assert_eq!(*state.messages()[0].role(), ConversationRole::User);
    /// let _ = handle;
    /// ```
    pub fn push_user(&mut self, content: impl Into<String>) -> MessageHandle {
        self.push_message(ConversationMessage::new(ConversationRole::User, content))
    }

    /// Adds an assistant text message and returns a [`MessageHandle`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationRole};
    ///
    /// let mut state = ConversationViewState::new();
    /// let handle = state.push_assistant("How can I help?");
    /// assert_eq!(*state.messages()[0].role(), ConversationRole::Assistant);
    /// let _ = handle;
    /// ```
    pub fn push_assistant(&mut self, content: impl Into<String>) -> MessageHandle {
        self.push_message(ConversationMessage::new(
            ConversationRole::Assistant,
            content,
        ))
    }

    /// Adds a system text message and returns a [`MessageHandle`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationRole};
    ///
    /// let mut state = ConversationViewState::new();
    /// let handle = state.push_system("System initialized");
    /// assert_eq!(*state.messages()[0].role(), ConversationRole::System);
    /// let _ = handle;
    /// ```
    pub fn push_system(&mut self, content: impl Into<String>) -> MessageHandle {
        self.push_message(ConversationMessage::new(ConversationRole::System, content))
    }

    /// Adds a tool result message and returns a [`MessageHandle`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationRole};
    ///
    /// let mut state = ConversationViewState::new();
    /// let handle = state.push_tool("Search results: 5 items found");
    /// assert_eq!(*state.messages()[0].role(), ConversationRole::Tool);
    /// let _ = handle;
    /// ```
    pub fn push_tool(&mut self, content: impl Into<String>) -> MessageHandle {
        self.push_message(ConversationMessage::new(ConversationRole::Tool, content))
    }

    /// Returns a mutable reference to the last message, if any.
    ///
    /// Useful for appending blocks to a streaming message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     ConversationViewState, ConversationMessage, ConversationRole, MessageBlock,
    /// };
    ///
    /// let mut state = ConversationViewState::new();
    /// state.push_assistant("Thinking...");
    /// if let Some(msg) = state.last_message_mut() {
    ///     msg.push_block(MessageBlock::code("let x = 1;", Some("rust")));
    /// }
    /// assert_eq!(state.messages()[0].blocks().len(), 2);
    /// ```
    pub fn last_message_mut(&mut self) -> Option<&mut ConversationMessage> {
        self.messages.last_mut()
    }

    /// Updates the last message via a closure.
    ///
    /// No-ops if the conversation is empty. This provides a safe way
    /// to modify a streaming message without exposing the internal
    /// vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     ConversationViewState, ConversationMessage, ConversationRole, MessageBlock,
    /// };
    ///
    /// let mut state = ConversationViewState::new();
    /// state.push_assistant("Thinking...");
    /// state.update_last_message(|msg| {
    ///     msg.push_block(MessageBlock::code("let x = 1;", Some("rust")));
    /// });
    /// assert_eq!(state.messages()[0].blocks().len(), 2);
    /// ```
    pub fn update_last_message(&mut self, f: impl FnOnce(&mut ConversationMessage)) {
        if let Some(msg) = self.messages.last_mut() {
            f(msg);
        }
    }

    /// Updates a message at the given index via a closure.
    ///
    /// No-ops if the index is out of bounds. This is safe because it
    /// does not change the number of messages, so scroll state
    /// remains valid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     ConversationViewState, ConversationMessage, ConversationRole, MessageBlock,
    /// };
    ///
    /// let mut state = ConversationViewState::new();
    /// state.push_user("Hello");
    /// state.push_assistant("Hi there");
    /// state.update_message(1, |msg| {
    ///     msg.push_block(MessageBlock::text(" - updated"));
    /// });
    /// assert_eq!(state.messages()[1].blocks().len(), 2);
    /// ```
    pub fn update_message(&mut self, index: usize, f: impl FnOnce(&mut ConversationMessage)) {
        if let Some(msg) = self.messages.get_mut(index) {
            f(msg);
        }
    }

    /// Updates a message identified by a [`MessageHandle`].
    ///
    /// No-ops if the message has been evicted (e.g., due to
    /// [`max_messages`](ConversationViewState::max_messages) eviction)
    /// or if the conversation has been cleared. This makes it safe to
    /// hold a handle across multiple push/clear operations.
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
    pub fn update_by_handle(
        &mut self,
        handle: MessageHandle,
        f: impl FnOnce(&mut ConversationMessage),
    ) {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == handle.0) {
            f(msg);
        }
    }

    /// Clears all messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.push_user("Hello");
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
    /// use envision::component::{ConversationViewState, ConversationMessage, ConversationRole};
    ///
    /// let mut state = ConversationViewState::new();
    /// state.push_user("Hello");
    /// assert_eq!(state.messages().len(), 1);
    /// ```
    pub fn messages(&self) -> &[ConversationMessage] {
        &self.messages
    }

    /// Returns the number of messages.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Returns true if there are no messages.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_title("My Chat");
    /// assert_eq!(state.title(), Some("My Chat"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns the status text, if set.
    ///
    /// The status line renders at the bottom of the viewport, above the
    /// border. Use it for transient information like rate-limit backoff
    /// or connection state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// assert!(state.status().is_none());
    ///
    /// state.set_status(Some("Rate limited"));
    /// assert_eq!(state.status(), Some("Rate limited"));
    ///
    /// state.set_status(None::<&str>);
    /// assert!(state.status().is_none());
    /// ```
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Sets or clears the status text.
    ///
    /// When `Some`, a single line is rendered at the bottom of the
    /// viewport inside the border. When `None`, the full viewport
    /// is used for messages.
    pub fn set_status(&mut self, status: Option<impl Into<String>>) {
        self.status = status.map(|s| s.into());
    }

    /// Returns the scroll offset.
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
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
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
    pub fn set_show_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }

    /// Returns whether role labels are shown.
    pub fn show_role_labels(&self) -> bool {
        self.show_role_labels
    }

    /// Sets whether role labels are shown.
    pub fn set_show_role_labels(&mut self, show: bool) {
        self.show_role_labels = show;
    }

    /// Returns whether auto-scroll is enabled.
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Sets whether auto-scroll is enabled.
    pub fn set_auto_scroll(&mut self, auto_scroll: bool) {
        self.auto_scroll = auto_scroll;
    }

    // ---- Collapse management ----

    /// Toggles the collapse state of a named block.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// assert!(!state.is_collapsed("thinking"));
    /// state.toggle_collapse("thinking");
    /// assert!(state.is_collapsed("thinking"));
    /// state.toggle_collapse("thinking");
    /// assert!(!state.is_collapsed("thinking"));
    /// ```
    pub fn toggle_collapse(&mut self, key: &str) {
        if self.collapsed_blocks.contains(key) {
            self.collapsed_blocks.remove(key);
        } else {
            self.collapsed_blocks.insert(key.to_string());
        }
    }

    /// Returns whether a block is collapsed.
    pub fn is_collapsed(&self, key: &str) -> bool {
        self.collapsed_blocks.contains(key)
    }

    /// Collapses a named block.
    pub fn collapse(&mut self, key: &str) {
        self.collapsed_blocks.insert(key.to_string());
    }

    /// Expands a named block.
    pub fn expand(&mut self, key: &str) {
        self.collapsed_blocks.remove(key);
    }

    // ---- Focus/Disabled instance methods ----

    // ---- Instance methods for dispatch ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationViewMessage};
    ///
    /// let mut state = ConversationViewState::new();
    /// state.push_user("Hello");
    /// let output = state.update(ConversationViewMessage::ScrollDown);
    /// ```
    pub fn update(&mut self, msg: ConversationViewMessage) -> Option<ConversationViewOutput> {
        ConversationView::update(self, msg)
    }

    // ---- Internal helpers ----

    /// Scrolls to the bottom of the conversation.
    fn scroll_to_bottom(&mut self) {
        self.update_scroll_content_length();
        self.scroll.scroll_to_end();
    }

    /// Updates the scroll content length based on the current display lines.
    ///
    /// Uses `last_known_width` from the most recent render for wrapping
    /// calculations. Defaults to 80 before the first render.
    fn update_scroll_content_length(&mut self) {
        let width = self.last_known_width.max(20);
        let total = render::total_display_lines(self, width);
        self.scroll.set_content_length(total);
    }
}

/// A read-only conversation display for AI/LLM interactions.
///
/// This component renders a scrollable list of conversation messages with
/// structured content blocks. It supports:
///
/// - **Role headers** with colored indicators (User, Assistant, System, Tool)
/// - **Code blocks** with left border styling
/// - **Collapsible** tool use and thinking blocks
/// - **Streaming cursor** for in-progress messages
/// - **Scrollbar** for long conversations
///
/// # Key Bindings
///
/// - `Up` / `k` -- Scroll up
/// - `Down` / `j` -- Scroll down
/// - `Home` / `g` -- Scroll to top
/// - `End` / `G` -- Scroll to bottom
/// - `PageUp` -- Page up
/// - `PageDown` -- Page down
pub struct ConversationView(PhantomData<()>);

impl Component for ConversationView {
    type State = ConversationViewState;
    type Message = ConversationViewMessage;
    type Output = ConversationViewOutput;

    fn init() -> Self::State {
        ConversationViewState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            Key::Up | Key::Char('k') => Some(ConversationViewMessage::ScrollUp),
            Key::Down | Key::Char('j') => Some(ConversationViewMessage::ScrollDown),
            Key::Char('g') if key.modifiers.shift() => {
                Some(ConversationViewMessage::ScrollToBottom)
            }
            Key::Home | Key::Char('g') => Some(ConversationViewMessage::ScrollToTop),
            Key::End => Some(ConversationViewMessage::ScrollToBottom),
            Key::PageUp => Some(ConversationViewMessage::PageUp),
            Key::PageDown => Some(ConversationViewMessage::PageDown),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ConversationViewMessage::ScrollUp => {
                state.update_scroll_content_length();
                state.scroll.scroll_up();
                state.auto_scroll = false;
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::ScrollDown => {
                state.update_scroll_content_length();
                state.scroll.scroll_down();
                if state.scroll.at_end() {
                    state.auto_scroll = true;
                }
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::ScrollToTop => {
                state.update_scroll_content_length();
                state.scroll.scroll_to_start();
                state.auto_scroll = false;
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::ScrollToBottom => {
                state.update_scroll_content_length();
                state.scroll.scroll_to_end();
                state.auto_scroll = true;
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::PageUp => {
                state.update_scroll_content_length();
                let page_size = state.scroll.viewport_height().max(1);
                state.scroll.page_up(page_size);
                state.auto_scroll = false;
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::PageDown => {
                state.update_scroll_content_length();
                let page_size = state.scroll.viewport_height().max(1);
                state.scroll.page_down(page_size);
                if state.scroll.at_end() {
                    state.auto_scroll = true;
                }
                Some(ConversationViewOutput::ScrollChanged {
                    offset: state.scroll.offset(),
                })
            }
            ConversationViewMessage::ToggleCollapse(key) => {
                state.toggle_collapse(&key);
                None
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if ctx.area.height < 3 || ctx.area.width < 5 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.open(
                ctx.area,
                crate::annotation::Annotation::container("conversation_view")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        render::render(
            state,
            ctx.frame,
            ctx.area,
            ctx.theme,
            ctx.focused,
            ctx.disabled,
        );

        crate::annotation::with_registry(|reg| {
            reg.close();
        });
    }
}

#[cfg(test)]
mod render_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
