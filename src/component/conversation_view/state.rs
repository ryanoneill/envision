//! ConversationViewState constructors, builders, accessors, setters, and instance methods.
//!
//! Extracted from the main module to keep file sizes manageable.

use ratatui::style::Style;

use super::render;
use super::{
    ConversationMessage, ConversationRole, ConversationView, ConversationViewMessage,
    ConversationViewOutput, ConversationViewState, MessageHandle,
};
use crate::component::Component;
use crate::scroll::ScrollState;

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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new();
    /// assert!(!state.markdown_enabled());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationRole};
    /// use ratatui::style::{Color, Style};
    ///
    /// let state = ConversationViewState::new()
    ///     .with_role_style(ConversationRole::User, Style::default().fg(Color::Cyan));
    /// assert!(state.role_style_override(&ConversationRole::User).is_some());
    /// assert!(state.role_style_override(&ConversationRole::Assistant).is_none());
    /// ```
    pub fn role_style_override(&self, role: &ConversationRole) -> Option<&Style> {
        self.role_style_overrides.get(role)
    }

    /// Returns the style override for a specific role, if one has been set.
    ///
    /// This is an alias for [`role_style_override`](Self::role_style_override).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationRole};
    /// use ratatui::style::{Color, Style};
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_role_style(ConversationRole::User, Style::default().fg(Color::Cyan));
    /// assert_eq!(
    ///     state.role_style(&ConversationRole::User),
    ///     Some(&Style::default().fg(Color::Cyan)),
    /// );
    /// assert!(state.role_style(&ConversationRole::Assistant).is_none());
    /// ```
    pub fn role_style(&self, role: &ConversationRole) -> Option<&Style> {
        self.role_style_override(role)
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ConversationViewState, ConversationRole};
    /// use ratatui::style::{Color, Style};
    ///
    /// let mut state = ConversationViewState::new()
    ///     .with_role_style(ConversationRole::User, Style::default().fg(Color::Cyan));
    /// state.clear_role_style(&ConversationRole::User);
    /// assert!(state.role_style_override(&ConversationRole::User).is_none());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.push_user("Hello");
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
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new().with_title("Chat");
    /// assert_eq!(state.title(), Some("Chat"));
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_status(Some("Connecting..."));
    /// assert_eq!(state.status(), Some("Connecting..."));
    /// state.set_status(None::<&str>);
    /// assert!(state.status().is_none());
    /// ```
    pub fn set_status(&mut self, status: Option<impl Into<String>>) {
        self.status = status.map(|s| s.into());
    }

    /// Returns the scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Returns the maximum number of messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new();
    /// assert_eq!(state.max_messages(), 1000);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new().with_show_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn show_timestamps(&self) -> bool {
        self.show_timestamps
    }

    /// Sets whether timestamps are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_show_timestamps(true);
    /// assert!(state.show_timestamps());
    /// ```
    pub fn set_show_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }

    /// Returns whether role labels are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new();
    /// assert!(state.show_role_labels());
    /// ```
    pub fn show_role_labels(&self) -> bool {
        self.show_role_labels
    }

    /// Sets whether role labels are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_show_role_labels(false);
    /// assert!(!state.show_role_labels());
    /// ```
    pub fn set_show_role_labels(&mut self, show: bool) {
        self.show_role_labels = show;
    }

    /// Returns whether auto-scroll is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let state = ConversationViewState::new();
    /// assert!(state.auto_scroll());
    /// ```
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Sets whether auto-scroll is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.set_auto_scroll(false);
    /// assert!(!state.auto_scroll());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// assert!(!state.is_collapsed("thinking"));
    /// state.collapse("thinking");
    /// assert!(state.is_collapsed("thinking"));
    /// ```
    pub fn is_collapsed(&self, key: &str) -> bool {
        self.collapsed_blocks.contains(key)
    }

    /// Collapses a named block.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.collapse("tool-use");
    /// assert!(state.is_collapsed("tool-use"));
    /// ```
    pub fn collapse(&mut self, key: &str) {
        self.collapsed_blocks.insert(key.to_string());
    }

    /// Expands a named block.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ConversationViewState;
    ///
    /// let mut state = ConversationViewState::new();
    /// state.collapse("thinking");
    /// state.expand("thinking");
    /// assert!(!state.is_collapsed("thinking"));
    /// ```
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
    pub(super) fn scroll_to_bottom(&mut self) {
        self.update_scroll_content_length();
        self.scroll.scroll_to_end();
    }

    /// Updates the scroll content length based on the current display lines.
    ///
    /// Uses `last_known_width` from the most recent render for wrapping
    /// calculations. Defaults to 80 before the first render.
    pub(super) fn update_scroll_content_length(&mut self) {
        let width = self.last_known_width.max(20);
        let total = render::total_display_lines(self, width);
        self.scroll.set_content_length(total);
    }
}
