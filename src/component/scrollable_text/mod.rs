//! A scrollable text display component.
//!
//! [`ScrollableText`] provides a read-only text buffer with scroll support.
//! It wraps text within its display area and allows the user to scroll
//! through content that exceeds the visible height. State is stored in
//! [`ScrollableTextState`], updated via [`ScrollableTextMessage`], and
//! produces [`ScrollableTextOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! See also [`StyledText`](super::StyledText) for rich text with semantic blocks.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, ScrollableText, ScrollableTextState,
//!     ScrollableTextMessage, ScrollableTextOutput,
//! };
//!
//! let mut state = ScrollableTextState::new()
//!     .with_content("Hello, world!\nThis is scrollable text.")
//!     .with_title("Preview");
//!
//! assert_eq!(state.content(), "Hello, world!\nThis is scrollable text.");
//! assert_eq!(state.title(), Some("Preview"));
//! assert_eq!(state.scroll_offset(), 0);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::{Component, Disableable, Focusable};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Messages that can be sent to a ScrollableText.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScrollableTextMessage {
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll up by a page (given number of lines).
    PageUp(usize),
    /// Scroll down by a page (given number of lines).
    PageDown(usize),
    /// Scroll to the top.
    Home,
    /// Scroll to the bottom.
    End,
    /// Replace the content.
    SetContent(String),
}

/// Output messages from a ScrollableText.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScrollableTextOutput {
    /// The scroll position changed.
    ScrollChanged(usize),
}

/// State for a ScrollableText component.
///
/// Contains the text content, scroll position, and display options.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ScrollableTextState {
    /// The text content.
    content: String,
    /// Scroll state tracking offset and providing scrollbar support.
    scroll: ScrollState,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
    /// Optional title for the border.
    title: Option<String>,
}

impl ScrollableTextState {
    /// Creates a new empty scrollable text state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollableTextState;
    ///
    /// let state = ScrollableTextState::new();
    /// assert!(state.content().is_empty());
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial content (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollableTextState;
    ///
    /// let state = ScrollableTextState::new()
    ///     .with_content("Hello!");
    /// assert_eq!(state.content(), "Hello!");
    /// ```
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self.scroll
            .set_content_length(self.content.lines().count().max(1));
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollableTextState;
    ///
    /// let state = ScrollableTextState::new()
    ///     .with_title("Preview");
    /// assert_eq!(state.title(), Some("Preview"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollableTextState;
    ///
    /// let state = ScrollableTextState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Content accessors ----

    /// Returns the text content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Sets the text content.
    ///
    /// Resets the scroll offset to 0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollableTextState;
    ///
    /// let mut state = ScrollableTextState::new();
    /// state.set_content("New content");
    /// assert_eq!(state.content(), "New content");
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.scroll = ScrollState::new(self.content.lines().count().max(1));
    }

    /// Appends text to the content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollableTextState;
    ///
    /// let mut state = ScrollableTextState::new()
    ///     .with_content("Hello");
    /// state.append(", world!");
    /// assert_eq!(state.content(), "Hello, world!");
    /// ```
    pub fn append(&mut self, text: &str) {
        self.content.push_str(text);
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    // ---- Scroll accessors ----

    /// Returns the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Sets the scroll offset.
    ///
    /// The offset is clamped to the valid range based on the current
    /// content length estimate. The precise clamping to wrapped line
    /// count happens during rendering.
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll.set_offset(offset);
    }

    /// Returns the number of visual lines the content would occupy
    /// when wrapped at the given `width`.
    ///
    /// Delegates to [`crate::util::wrapped_line_count`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollableTextState;
    ///
    /// let state = ScrollableTextState::new()
    ///     .with_content("hello world");
    /// assert_eq!(state.line_count(5), 3);
    /// ```
    pub fn line_count(&self, width: usize) -> usize {
        crate::util::wrapped_line_count(&self.content, width)
    }

    // ---- State accessors ----

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

    // ---- Instance methods ----

    /// Maps an input event to a scrollable text message.
    pub fn handle_event(&self, event: &Event) -> Option<ScrollableTextMessage> {
        ScrollableText::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<ScrollableTextOutput> {
        ScrollableText::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ScrollableTextMessage, ScrollableTextOutput, ScrollableTextState};
    ///
    /// let mut state = ScrollableTextState::new()
    ///     .with_content("Line 1\nLine 2\nLine 3");
    /// let output = state.update(ScrollableTextMessage::ScrollDown);
    /// assert_eq!(output, Some(ScrollableTextOutput::ScrollChanged(1)));
    /// ```
    pub fn update(&mut self, msg: ScrollableTextMessage) -> Option<ScrollableTextOutput> {
        ScrollableText::update(self, msg)
    }
}

/// A scrollable text display component.
///
/// Displays text content that can be scrolled when it exceeds the visible
/// area. Text is wrapped at the component's width using character-level
/// wrapping.
///
/// # Key Bindings
///
/// - `Up` / `k` — Scroll up one line
/// - `Down` / `j` — Scroll down one line
/// - `PageUp` / `Ctrl+u` — Scroll up half a page
/// - `PageDown` / `Ctrl+d` — Scroll down half a page
/// - `Home` / `g` — Scroll to top
/// - `End` / `G` — Scroll to bottom
pub struct ScrollableText;

impl Component for ScrollableText {
    type State = ScrollableTextState;
    type Message = ScrollableTextMessage;
    type Output = ScrollableTextOutput;

    fn init() -> Self::State {
        ScrollableTextState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !ctrl => Some(ScrollableTextMessage::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') if !ctrl => Some(ScrollableTextMessage::ScrollDown),
            KeyCode::PageUp => Some(ScrollableTextMessage::PageUp(10)),
            KeyCode::PageDown => Some(ScrollableTextMessage::PageDown(10)),
            KeyCode::Char('u') if ctrl => Some(ScrollableTextMessage::PageUp(10)),
            KeyCode::Char('d') if ctrl => Some(ScrollableTextMessage::PageDown(10)),
            KeyCode::Home | KeyCode::Char('g') if !shift => Some(ScrollableTextMessage::Home),
            KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => {
                Some(ScrollableTextMessage::End)
            }
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ScrollableTextMessage::ScrollUp => {
                if state.scroll.scroll_up() {
                    Some(ScrollableTextOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            ScrollableTextMessage::ScrollDown => {
                if state.scroll.scroll_down() {
                    Some(ScrollableTextOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            ScrollableTextMessage::PageUp(n) => {
                if state.scroll.page_up(n) {
                    Some(ScrollableTextOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            ScrollableTextMessage::PageDown(n) => {
                if state.scroll.page_down(n) {
                    Some(ScrollableTextOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            ScrollableTextMessage::Home => {
                if state.scroll.scroll_to_start() {
                    Some(ScrollableTextOutput::ScrollChanged(0))
                } else {
                    None
                }
            }
            ScrollableTextMessage::End => {
                if state.scroll.scroll_to_end() {
                    Some(ScrollableTextOutput::ScrollChanged(state.scroll.offset()))
                } else {
                    None
                }
            }
            ScrollableTextMessage::SetContent(content) => {
                state.content = content;
                state.scroll = ScrollState::new(state.content.lines().count().max(1));
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::scrollable_text("scrollable_text")
                    .with_focus(state.focused)
                    .with_disabled(state.disabled),
            );
        });

        let border_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let text_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_style()
        } else {
            theme.normal_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(title) = &state.title {
            block = block.title(title.as_str());
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Compute scroll dimensions and clamp offset
        let total_lines = state.line_count(inner.width as usize);
        let visible_lines = inner.height as usize;
        let max_scroll = total_lines.saturating_sub(visible_lines);
        let effective_scroll = state.scroll.offset().min(max_scroll);

        let paragraph = Paragraph::new(state.content.as_str())
            .style(text_style)
            .wrap(Wrap { trim: false })
            .scroll((effective_scroll as u16, 0));

        frame.render_widget(paragraph, inner);

        // Render scrollbar when content exceeds viewport
        if total_lines > visible_lines {
            let mut bar_scroll = ScrollState::new(total_lines);
            bar_scroll.set_viewport_height(visible_lines);
            bar_scroll.set_offset(effective_scroll);
            crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
        }
    }
}

impl Focusable for ScrollableText {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for ScrollableText {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod tests;
