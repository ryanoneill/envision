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
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, ConversationView, ConversationViewState,
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

pub use types::{
    ConversationMessage, ConversationRole, MessageBlock, MessageHandle, MessageSource,
};

use std::collections::HashSet;
use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, Disableable, Focusable, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

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
    /// Whether the component is focused.
    pub(super) focused: bool,
    /// Whether the component is disabled.
    pub(super) disabled: bool,
    /// Set of collapsed block keys (e.g., "tool:search", "thinking").
    pub(super) collapsed_blocks: HashSet<String>,
    /// Optional status text rendered at the bottom of the viewport, above the border.
    pub(super) status: Option<String>,
    /// Next unique ID for message handles.
    #[cfg_attr(feature = "serialization", serde(skip, default))]
    pub(super) next_id: u64,
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
            focused: false,
            disabled: false,
            collapsed_blocks: HashSet::new(),
            status: None,
            next_id: 1,
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
            && self.focused == other.focused
            && self.disabled == other.disabled
            && self.collapsed_blocks == other.collapsed_blocks
            && self.status == other.status
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
    /// let state = ConversationViewState::new().with_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn with_timestamps(mut self, show: bool) -> Self {
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
    /// let state = ConversationViewState::new().with_role_labels(false);
    /// assert!(!state.show_role_labels());
    /// ```
    pub fn with_role_labels(mut self, show: bool) -> Self {
        self.show_role_labels = show;
        self
    }

    /// Enables or disables markdown rendering for text blocks (builder pattern).
    ///
    /// When enabled and the `markdown` feature is active, text blocks are
    /// rendered as markdown (headings, bold, italic, code, lists, etc.)
    /// instead of plain text.
    pub fn with_markdown(mut self, enabled: bool) -> Self {
        self.markdown_enabled = enabled;
        self
    }

    /// Returns whether markdown rendering is enabled.
    pub fn markdown_enabled(&self) -> bool {
        self.markdown_enabled
    }

    /// Sets whether markdown rendering is enabled.
    pub fn set_markdown_enabled(&mut self, enabled: bool) {
        self.markdown_enabled = enabled;
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
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

    /// Returns true if the component is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new();
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
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
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
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new();
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
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    // ---- Instance methods for dispatch ----

    /// Maps an input event to a conversation view message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationViewMessage};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_focused(true);
    /// let event = Event::key(KeyCode::Up);
    /// assert_eq!(state.handle_event(&event), Some(ConversationViewMessage::ScrollUp));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<ConversationViewMessage> {
        ConversationView::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_focused(true);
    /// state.push_user("Hello");
    /// let _output = state.dispatch_event(&Event::key(KeyCode::Up));
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<ConversationViewOutput> {
        ConversationView::dispatch_event(self, event)
    }

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

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(ConversationViewMessage::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') => Some(ConversationViewMessage::ScrollDown),
            KeyCode::Home | KeyCode::Char('g') => Some(ConversationViewMessage::ScrollToTop),
            KeyCode::End if shift => Some(ConversationViewMessage::ScrollToBottom),
            KeyCode::End => Some(ConversationViewMessage::ScrollToBottom),
            KeyCode::Char('G') => Some(ConversationViewMessage::ScrollToBottom),
            KeyCode::PageUp => Some(ConversationViewMessage::PageUp),
            KeyCode::PageDown => Some(ConversationViewMessage::PageDown),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if area.height < 3 || area.width < 5 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.open(
                area,
                crate::annotation::Annotation::container("conversation_view")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        render::render(state, frame, area, theme, ctx.focused, ctx.disabled);

        crate::annotation::with_registry(|reg| {
            reg.close();
        });
    }
}

impl ConversationView {
    /// Renders the conversation using messages from an external [`MessageSource`]
    /// instead of the messages stored in the state.
    ///
    /// This is the key method for avoiding the "dual-store" pattern: your
    /// application can own the canonical message list and pass it directly
    /// for rendering, while [`ConversationViewState`] only tracks scroll
    /// position, collapsed blocks, and display configuration.
    ///
    /// The `state` parameter still provides all non-message configuration
    /// (scroll offset, collapsed blocks, auto-scroll, title, etc.).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     ConversationView, ConversationViewState, ConversationMessage,
    ///     ConversationRole, ViewContext,
    /// };
    /// use envision::prelude::*;
    /// use envision::component::test_utils;
    ///
    /// // Application owns the canonical message list
    /// let messages = vec![
    ///     ConversationMessage::new(ConversationRole::User, "Hello"),
    ///     ConversationMessage::new(ConversationRole::Assistant, "Hi!"),
    /// ];
    ///
    /// // State tracks only view configuration (scroll, collapsed blocks, etc.)
    /// let state = ConversationViewState::new();
    /// let (mut terminal, theme) = test_utils::setup_render(60, 20);
    ///
    /// terminal.draw(|frame| {
    ///     ConversationView::view_from(
    ///         &messages,
    ///         &state,
    ///         frame,
    ///         frame.area(),
    ///         &theme,
    ///         &ViewContext::default(),
    ///     );
    /// }).unwrap();
    /// ```
    pub fn view_from(
        source: &dyn MessageSource,
        state: &ConversationViewState,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        ctx: &ViewContext,
    ) {
        if area.height < 3 || area.width < 5 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.open(
                area,
                crate::annotation::Annotation::container("conversation_view")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        render::render_from(source, state, frame, area, theme, ctx.focused, ctx.disabled);

        crate::annotation::with_registry(|reg| {
            reg.close();
        });
    }
}

impl Focusable for ConversationView {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for ConversationView {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod render_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
