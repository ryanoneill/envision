//! A chat interface with message history and multi-line input.
//!
//! `ChatView` provides a scrollable message history display and a
//! [`TextArea`](super::TextArea) input field. Messages can be typed
//! as user, system, or assistant and are styled differently per role.
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

mod render_helpers;

use std::collections::HashMap;
use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, Focusable, TextAreaMessage, TextAreaOutput, TextAreaState};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

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
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::User => "You",
            Self::System => "System",
            Self::Assistant => "Assistant",
        }
    }

    /// Returns the display color for this role.
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
pub struct ChatMessage {
    /// The role of the sender.
    role: ChatRole,
    /// The message content.
    content: String,
    /// Optional timestamp.
    timestamp: Option<String>,
    /// Optional username override.
    username: Option<String>,
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
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Sets the username override (builder pattern).
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Returns the role.
    pub fn role(&self) -> ChatRole {
        self.role
    }

    /// Returns the content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> Option<&str> {
        self.timestamp.as_deref()
    }

    /// Returns the username (or the role's default prefix).
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
enum Focus {
    /// The message history is focused.
    History,
    /// The input field is focused.
    #[default]
    Input,
}

/// Messages that can be sent to a ChatView.
#[derive(Clone, Debug, PartialEq, Eq)]
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
pub enum ChatViewOutput {
    /// The user submitted a message. Contains the message text.
    Submitted(String),
    /// The input text changed.
    InputChanged(String),
}

/// State for a ChatView component.
///
/// Contains the message history, input field, and scroll state.
#[derive(Clone, Debug)]
pub struct ChatViewState {
    /// Message history.
    messages: Vec<ChatMessage>,
    /// The text input area.
    input: TextAreaState,
    /// Scroll offset for the message history.
    scroll_offset: usize,
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
}

impl Default for ChatViewState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input: TextAreaState::with_placeholder("Type a message..."),
            scroll_offset: 0,
            auto_scroll: true,
            max_messages: 1000,
            focus: Focus::Input,
            focused: false,
            disabled: false,
            show_timestamps: false,
            input_height: 3,
            role_styles: None,
        }
    }
}

impl PartialEq for ChatViewState {
    fn eq(&self, other: &Self) -> bool {
        self.messages == other.messages
            && self.scroll_offset == other.scroll_offset
            && self.auto_scroll == other.auto_scroll
            && self.max_messages == other.max_messages
            && self.focus == other.focus
            && self.focused == other.focused
            && self.disabled == other.disabled
            && self.show_timestamps == other.show_timestamps
            && self.input_height == other.input_height
            && self.role_styles == other.role_styles
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
    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }

    /// Sets whether to show timestamps (builder pattern).
    pub fn with_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = show;
        self
    }

    /// Sets the input area height (builder pattern).
    pub fn with_input_height(mut self, height: u16) -> Self {
        self.input_height = height.max(1);
        self
    }

    /// Sets the disabled state (builder pattern).
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the input placeholder text (builder pattern).
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.input.set_placeholder(placeholder);
        self
    }

    // ---- Message manipulation ----

    /// Adds a message from any role.
    pub fn push_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        while self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
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
    pub fn push_system(&mut self, content: impl Into<String>) {
        self.push_message(ChatMessage::new(ChatRole::System, content));
    }

    /// Adds an assistant message.
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
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
    }

    // ---- Accessors ----

    /// Returns the messages.
    pub fn messages(&self) -> &[ChatMessage] {
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

    /// Returns the current input text.
    pub fn input_value(&self) -> String {
        self.input.value()
    }

    /// Sets the input text.
    pub fn set_input_value(&mut self, value: impl Into<String>) {
        self.input.set_value(value);
    }

    /// Returns the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Returns the maximum number of messages.
    pub fn max_messages(&self) -> usize {
        self.max_messages
    }

    /// Sets the maximum number of messages.
    pub fn set_max_messages(&mut self, max: usize) {
        self.max_messages = max;
        while self.messages.len() > self.max_messages {
            self.messages.remove(0);
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

    /// Returns whether auto-scroll is enabled.
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Sets whether auto-scroll is enabled.
    pub fn set_auto_scroll(&mut self, auto_scroll: bool) {
        self.auto_scroll = auto_scroll;
    }

    /// Returns the input area height.
    pub fn input_height(&self) -> u16 {
        self.input_height
    }

    /// Sets the input area height.
    pub fn set_input_height(&mut self, height: u16) {
        self.input_height = height.max(1);
    }

    /// Returns whether the input field is focused.
    pub fn is_input_focused(&self) -> bool {
        self.focus == Focus::Input
    }

    /// Returns whether the history is focused.
    pub fn is_history_focused(&self) -> bool {
        self.focus == Focus::History
    }

    /// Scrolls the message history to the bottom (newest).
    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
    }

    // ---- Instance methods ----

    /// Returns true if the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
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
    pub fn clear_role_styles(&mut self) {
        self.role_styles = None;
    }

    /// Maps an input event to a chat view message.
    pub fn handle_event(&self, event: &Event) -> Option<ChatViewMessage> {
        ChatView::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<ChatViewOutput> {
        ChatView::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
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

impl Component for ChatView {
    type State = ChatViewState;
    type Message = ChatViewMessage;
    type Output = ChatViewOutput;

    fn init() -> Self::State {
        ChatViewState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;

        match state.focus {
            Focus::Input => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Enter if ctrl => Some(ChatViewMessage::Submit),
                    KeyCode::Enter => Some(ChatViewMessage::NewLine),
                    KeyCode::Tab => Some(ChatViewMessage::ToggleFocus),
                    KeyCode::Char(c) if !ctrl => Some(ChatViewMessage::Input(c)),
                    KeyCode::Char('k') if ctrl => Some(ChatViewMessage::DeleteToEnd),
                    KeyCode::Char('u') if ctrl => Some(ChatViewMessage::DeleteToStart),
                    KeyCode::Backspace => Some(ChatViewMessage::Backspace),
                    KeyCode::Delete => Some(ChatViewMessage::Delete),
                    KeyCode::Left => Some(ChatViewMessage::Left),
                    KeyCode::Right => Some(ChatViewMessage::Right),
                    KeyCode::Up => Some(ChatViewMessage::Up),
                    KeyCode::Down => Some(ChatViewMessage::Down),
                    KeyCode::Home if ctrl => Some(ChatViewMessage::InputStart),
                    KeyCode::Home => Some(ChatViewMessage::Home),
                    KeyCode::End if ctrl => Some(ChatViewMessage::InputEnd),
                    KeyCode::End => Some(ChatViewMessage::End),
                    _ => None,
                }
            }
            Focus::History => match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(ChatViewMessage::ScrollUp),
                KeyCode::Down | KeyCode::Char('j') => Some(ChatViewMessage::ScrollDown),
                KeyCode::Home => Some(ChatViewMessage::ScrollToTop),
                KeyCode::End => Some(ChatViewMessage::ScrollToBottom),
                KeyCode::Tab => Some(ChatViewMessage::ToggleFocus),
                _ => None,
            },
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            ChatViewMessage::Input(c) => {
                state.input.update(TextAreaMessage::Insert(c));
                Some(ChatViewOutput::InputChanged(state.input.value()))
            }
            ChatViewMessage::NewLine => {
                state.input.update(TextAreaMessage::NewLine);
                Some(ChatViewOutput::InputChanged(state.input.value()))
            }
            ChatViewMessage::Backspace => {
                if let Some(TextAreaOutput::Changed(_)) =
                    state.input.update(TextAreaMessage::Backspace)
                {
                    Some(ChatViewOutput::InputChanged(state.input.value()))
                } else {
                    None
                }
            }
            ChatViewMessage::Delete => {
                if let Some(TextAreaOutput::Changed(_)) =
                    state.input.update(TextAreaMessage::Delete)
                {
                    Some(ChatViewOutput::InputChanged(state.input.value()))
                } else {
                    None
                }
            }
            ChatViewMessage::Left => {
                state.input.update(TextAreaMessage::Left);
                None
            }
            ChatViewMessage::Right => {
                state.input.update(TextAreaMessage::Right);
                None
            }
            ChatViewMessage::Up => {
                state.input.update(TextAreaMessage::Up);
                None
            }
            ChatViewMessage::Down => {
                state.input.update(TextAreaMessage::Down);
                None
            }
            ChatViewMessage::Home => {
                state.input.update(TextAreaMessage::Home);
                None
            }
            ChatViewMessage::End => {
                state.input.update(TextAreaMessage::End);
                None
            }
            ChatViewMessage::InputStart => {
                state.input.update(TextAreaMessage::TextStart);
                None
            }
            ChatViewMessage::InputEnd => {
                state.input.update(TextAreaMessage::TextEnd);
                None
            }
            ChatViewMessage::DeleteToEnd => {
                if let Some(TextAreaOutput::Changed(_)) =
                    state.input.update(TextAreaMessage::DeleteToEnd)
                {
                    Some(ChatViewOutput::InputChanged(state.input.value()))
                } else {
                    None
                }
            }
            ChatViewMessage::DeleteToStart => {
                if let Some(TextAreaOutput::Changed(_)) =
                    state.input.update(TextAreaMessage::DeleteToStart)
                {
                    Some(ChatViewOutput::InputChanged(state.input.value()))
                } else {
                    None
                }
            }
            ChatViewMessage::Submit => {
                let value = state.input.value();
                if value.trim().is_empty() {
                    return None;
                }
                state.push_user(&value);
                state.input.update(TextAreaMessage::Clear);
                Some(ChatViewOutput::Submitted(value))
            }
            ChatViewMessage::ToggleFocus => {
                match state.focus {
                    Focus::Input => {
                        state.focus = Focus::History;
                        state.input.set_focused(false);
                    }
                    Focus::History => {
                        state.focus = Focus::Input;
                        state.input.set_focused(true);
                    }
                }
                None
            }
            ChatViewMessage::FocusInput => {
                state.focus = Focus::Input;
                state.input.set_focused(true);
                None
            }
            ChatViewMessage::FocusHistory => {
                state.focus = Focus::History;
                state.input.set_focused(false);
                None
            }
            ChatViewMessage::ScrollUp => {
                if state.scroll_offset > 0 {
                    state.scroll_offset -= 1;
                    state.auto_scroll = false;
                }
                None
            }
            ChatViewMessage::ScrollDown => {
                let max = state.messages.len().saturating_sub(1);
                if state.scroll_offset < max {
                    state.scroll_offset += 1;
                    if state.scroll_offset >= max {
                        state.auto_scroll = true;
                    }
                }
                None
            }
            ChatViewMessage::ScrollToTop => {
                state.scroll_offset = 0;
                state.auto_scroll = false;
                None
            }
            ChatViewMessage::ScrollToBottom => {
                state.scroll_to_bottom();
                state.auto_scroll = true;
                None
            }
            ChatViewMessage::ClearInput => {
                state.input.update(TextAreaMessage::Clear);
                Some(ChatViewOutput::InputChanged(String::new()))
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if area.height < 4 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.open(
                area,
                crate::annotation::Annotation::container("chat_view")
                    .with_focus(state.focused)
                    .with_disabled(state.disabled),
            );
        });

        // Layout: history + input
        let input_h = state.input_height + 2; // +2 for border
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(input_h)])
            .split(area);

        let history_area = chunks[0];
        let input_area = chunks[1];

        render_helpers::render_history(state, frame, history_area, theme);
        render_helpers::render_input(state, frame, input_area, theme);

        crate::annotation::with_registry(|reg| {
            reg.close();
        });
    }
}

impl Focusable for ChatView {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
        if focused && state.focus == Focus::Input {
            state.input.set_focused(true);
        }
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
