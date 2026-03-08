//! A rich text display component with semantic block elements and inline styling.
//!
//! [`StyledText`] renders structured content composed of headings, paragraphs,
//! lists, code blocks, and horizontal rules with scrolling support. State is
//! stored in [`StyledTextState`], updated via [`StyledTextMessage`], and
//! produces [`StyledTextOutput`]. Content is built with [`StyledContent`],
//! [`StyledBlock`], and [`StyledInline`].
//!
//! Implements [`Focusable`] and [`Disableable`](super::Disableable).
//!
//! See also [`ScrollableText`](super::ScrollableText) for plain text display.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     StyledText, StyledTextMessage, StyledTextState, Component,
//!     styled_text::{StyledContent, StyledInline},
//! };
//!
//! let content = StyledContent::new()
//!     .heading(1, "Welcome")
//!     .text("This is a styled paragraph.")
//!     .bullet_list(vec![
//!         vec![StyledInline::Bold("Important".to_string())],
//!         vec![StyledInline::Plain("Normal item".to_string())],
//!     ]);
//!
//! let mut state = StyledTextState::new()
//!     .with_content(content);
//!
//! // Scroll down
//! StyledText::update(&mut state, StyledTextMessage::ScrollDown);
//! ```

pub mod content;

pub use content::{StyledBlock, StyledContent, StyledInline};

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, Focusable};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

/// Messages that can be sent to a StyledText component.
#[derive(Clone, Debug, PartialEq)]
pub enum StyledTextMessage {
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
    SetContent(StyledContent),
}

/// Output messages from a StyledText component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StyledTextOutput {
    /// The scroll position changed.
    ScrollChanged(usize),
}

/// State for a StyledText component.
///
/// Contains the styled content, scroll position, and display options.
///
/// # Example
///
/// ```rust
/// use envision::component::styled_text::{StyledContent, StyledInline};
/// use envision::component::StyledTextState;
///
/// let content = StyledContent::new()
///     .heading(1, "Title")
///     .text("Body text");
///
/// let state = StyledTextState::new()
///     .with_content(content)
///     .with_title("Preview");
///
/// assert_eq!(state.title(), Some("Preview"));
/// assert_eq!(state.scroll_offset(), 0);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct StyledTextState {
    content: StyledContent,
    scroll_offset: usize,
    focused: bool,
    disabled: bool,
    title: Option<String>,
    show_border: bool,
}

impl Default for StyledTextState {
    /// Creates a default styled text state with border enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StyledTextState;
    ///
    /// let state = StyledTextState::default();
    /// assert_eq!(state.scroll_offset(), 0);
    /// assert!(state.show_border());
    /// assert!(!state.is_focused());
    /// ```
    fn default() -> Self {
        Self {
            content: StyledContent::default(),
            scroll_offset: 0,
            focused: false,
            disabled: false,
            title: None,
            show_border: true,
        }
    }
}

impl StyledTextState {
    /// Creates a new empty styled text state with a border.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StyledTextState;
    ///
    /// let state = StyledTextState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// assert!(state.show_border());
    /// assert!(!state.is_focused());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the content (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::styled_text::StyledContent;
    /// use envision::component::StyledTextState;
    ///
    /// let content = StyledContent::new().text("Hello");
    /// let state = StyledTextState::new().with_content(content);
    /// assert!(!state.content().is_empty());
    /// ```
    pub fn with_content(mut self, content: StyledContent) -> Self {
        self.content = content;
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StyledTextState;
    ///
    /// let state = StyledTextState::new().with_title("Preview");
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
    /// use envision::component::StyledTextState;
    ///
    /// let state = StyledTextState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets whether to show the border (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StyledTextState;
    ///
    /// let state = StyledTextState::new().with_show_border(false);
    /// assert!(!state.show_border());
    /// ```
    pub fn with_show_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

    // ---- Content accessors ----

    /// Returns the styled content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StyledTextState;
    /// use envision::component::styled_text::StyledContent;
    ///
    /// let content = StyledContent::new().text("Hello");
    /// let state = StyledTextState::new().with_content(content);
    /// assert!(!state.content().is_empty());
    /// ```
    pub fn content(&self) -> &StyledContent {
        &self.content
    }

    /// Sets the styled content and resets scroll to top.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StyledTextState;
    /// use envision::component::styled_text::StyledContent;
    ///
    /// let mut state = StyledTextState::new();
    /// state.set_content(StyledContent::new().text("New content"));
    /// assert_eq!(state.scroll_offset(), 0);
    /// assert!(!state.content().is_empty());
    /// ```
    pub fn set_content(&mut self, content: StyledContent) {
        self.content = content;
        self.scroll_offset = 0;
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StyledTextState;
    ///
    /// let state = StyledTextState::new().with_title("Readme");
    /// assert_eq!(state.title(), Some("Readme"));
    ///
    /// let state2 = StyledTextState::new();
    /// assert_eq!(state2.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns whether the border is shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StyledTextState;
    ///
    /// let state = StyledTextState::new();
    /// assert!(state.show_border());
    /// ```
    pub fn show_border(&self) -> bool {
        self.show_border
    }

    // ---- Scroll accessors ----

    /// Returns the current scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StyledTextState, StyledTextMessage};
    ///
    /// let mut state = StyledTextState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// state.update(StyledTextMessage::ScrollDown);
    /// assert_eq!(state.scroll_offset(), 1);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    // ---- State accessors ----

    /// Returns true if the component is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StyledTextState;
    ///
    /// let state = StyledTextState::new();
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
    /// use envision::component::StyledTextState;
    ///
    /// let mut state = StyledTextState::new();
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
    /// use envision::component::StyledTextState;
    ///
    /// let state = StyledTextState::new();
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
    /// use envision::component::StyledTextState;
    ///
    /// let mut state = StyledTextState::new();
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    // ---- Instance methods ----

    /// Maps an input event to a styled text message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StyledTextState, StyledTextMessage};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = StyledTextState::new();
    /// state.set_focused(true);
    /// let event = Event::key(KeyCode::Down);
    /// assert_eq!(state.handle_event(&event), Some(StyledTextMessage::ScrollDown));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<StyledTextMessage> {
        StyledText::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StyledTextState, StyledTextOutput};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = StyledTextState::new();
    /// state.set_focused(true);
    /// let event = Event::key(KeyCode::Down);
    /// let output = state.dispatch_event(&event);
    /// assert_eq!(output, Some(StyledTextOutput::ScrollChanged(1)));
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<StyledTextOutput> {
        StyledText::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StyledTextState, StyledTextMessage, StyledTextOutput};
    ///
    /// let mut state = StyledTextState::new();
    /// let output = state.update(StyledTextMessage::ScrollDown);
    /// assert_eq!(output, Some(StyledTextOutput::ScrollChanged(1)));
    /// assert_eq!(state.scroll_offset(), 1);
    /// ```
    pub fn update(&mut self, msg: StyledTextMessage) -> Option<StyledTextOutput> {
        StyledText::update(self, msg)
    }
}

/// A rich text display component with semantic styling.
///
/// `StyledText` renders [`StyledContent`] with proper formatting for headings,
/// paragraphs, lists, code blocks, and other semantic elements.
///
/// # Key Bindings
///
/// - `Up` / `k` — Scroll up one line
/// - `Down` / `j` — Scroll down one line
/// - `PageUp` / `Ctrl+u` — Scroll up half a page
/// - `PageDown` / `Ctrl+d` — Scroll down half a page
/// - `Home` / `g` — Scroll to top
/// - `End` / `G` — Scroll to bottom
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     StyledText, StyledTextMessage, StyledTextState, Component,
///     styled_text::StyledContent,
/// };
///
/// let content = StyledContent::new()
///     .heading(1, "Title")
///     .text("Hello, world!");
///
/// let mut state = StyledTextState::new()
///     .with_content(content);
///
/// StyledText::update(&mut state, StyledTextMessage::ScrollDown);
/// assert_eq!(state.scroll_offset(), 1);
/// ```
pub struct StyledText;

impl Component for StyledText {
    type State = StyledTextState;
    type Message = StyledTextMessage;
    type Output = StyledTextOutput;

    fn init() -> Self::State {
        StyledTextState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !ctrl => Some(StyledTextMessage::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') if !ctrl => Some(StyledTextMessage::ScrollDown),
            KeyCode::PageUp => Some(StyledTextMessage::PageUp(10)),
            KeyCode::PageDown => Some(StyledTextMessage::PageDown(10)),
            KeyCode::Char('u') if ctrl => Some(StyledTextMessage::PageUp(10)),
            KeyCode::Char('d') if ctrl => Some(StyledTextMessage::PageDown(10)),
            KeyCode::Home | KeyCode::Char('g') if !shift => Some(StyledTextMessage::Home),
            KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => {
                Some(StyledTextMessage::End)
            }
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            StyledTextMessage::ScrollUp => {
                if state.scroll_offset > 0 {
                    state.scroll_offset -= 1;
                    Some(StyledTextOutput::ScrollChanged(state.scroll_offset))
                } else {
                    None
                }
            }
            StyledTextMessage::ScrollDown => {
                state.scroll_offset += 1;
                Some(StyledTextOutput::ScrollChanged(state.scroll_offset))
            }
            StyledTextMessage::PageUp(n) => {
                let old = state.scroll_offset;
                state.scroll_offset = state.scroll_offset.saturating_sub(n);
                if state.scroll_offset != old {
                    Some(StyledTextOutput::ScrollChanged(state.scroll_offset))
                } else {
                    None
                }
            }
            StyledTextMessage::PageDown(n) => {
                state.scroll_offset += n;
                Some(StyledTextOutput::ScrollChanged(state.scroll_offset))
            }
            StyledTextMessage::Home => {
                if state.scroll_offset > 0 {
                    state.scroll_offset = 0;
                    Some(StyledTextOutput::ScrollChanged(0))
                } else {
                    None
                }
            }
            StyledTextMessage::End => {
                state.scroll_offset = usize::MAX;
                Some(StyledTextOutput::ScrollChanged(state.scroll_offset))
            }
            StyledTextMessage::SetContent(content) => {
                state.content = content;
                state.scroll_offset = 0;
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::StyledText)
                    .with_id("styled_text")
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

        let (inner, render_area) = if state.show_border {
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            if let Some(title) = &state.title {
                block = block.title(title.as_str());
            }

            let inner = block.inner(area);
            frame.render_widget(block, area);
            (inner, inner)
        } else {
            (area, area)
        };

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        let rendered_lines = state.content.render_lines(inner.width, theme);
        let total_lines = rendered_lines.len();
        let visible_lines = inner.height as usize;
        let max_scroll = total_lines.saturating_sub(visible_lines);
        let effective_scroll = state.scroll_offset.min(max_scroll);

        let text = Text::from(rendered_lines);
        let paragraph = Paragraph::new(text).scroll((effective_scroll as u16, 0));

        frame.render_widget(paragraph, render_area);
    }
}

impl Focusable for StyledText {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
