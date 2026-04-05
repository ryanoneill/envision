//! A markdown rendering component with scroll support.
//!
//! [`MarkdownRenderer`] parses a markdown string using pulldown-cmark and
//! renders the result as styled terminal output with headings, inline
//! formatting, code blocks, lists, links, blockquotes, and horizontal rules.
//!
//! State is stored in [`MarkdownRendererState`], updated via
//! [`MarkdownRendererMessage`], and produces no output (unit `()`).
//!
//!
//! # Feature Gate
//!
//! This component requires the `markdown` feature.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, MarkdownRenderer, MarkdownRendererState,
//!     MarkdownRendererMessage,
//! };
//!
//! let mut state = MarkdownRendererState::new()
//!     .with_source("# Hello\n\nSome **bold** text.")
//!     .with_title("Preview");
//!
//! assert_eq!(state.source(), "# Hello\n\nSome **bold** text.");
//! assert_eq!(state.title(), Some("Preview"));
//! assert_eq!(state.scroll_offset(), 0);
//! assert!(!state.show_source());
//! ```

pub mod render;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Messages that can be sent to a [`MarkdownRenderer`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum MarkdownRendererMessage {
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
    /// Replace the markdown source.
    SetSource(String),
    /// Toggle between rendered markdown and raw source views.
    ToggleSource,
}

/// State for a [`MarkdownRenderer`] component.
///
/// Contains the markdown source, scroll position, and display options.
///
/// # Example
///
/// ```rust
/// use envision::component::MarkdownRendererState;
///
/// let state = MarkdownRendererState::new()
///     .with_source("# Title\n\nBody text.")
///     .with_title("Document");
///
/// assert_eq!(state.source(), "# Title\n\nBody text.");
/// assert_eq!(state.title(), Some("Document"));
/// assert!(!state.is_focused());
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MarkdownRendererState {
    /// The markdown source text.
    source: String,
    /// Scroll state tracking offset and providing scrollbar support.
    scroll: ScrollState,
    /// Optional title for the border.
    title: Option<String>,
    /// Whether to show raw source instead of rendered markdown.
    show_source: bool,
}

impl MarkdownRendererState {
    /// Creates a new empty markdown renderer state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MarkdownRendererState;
    ///
    /// let state = MarkdownRendererState::new();
    /// assert!(state.source().is_empty());
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the markdown source (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MarkdownRendererState;
    ///
    /// let state = MarkdownRendererState::new()
    ///     .with_source("# Hello");
    /// assert_eq!(state.source(), "# Hello");
    /// ```
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self.scroll
            .set_content_length(self.source.lines().count().max(1));
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MarkdownRendererState;
    ///
    /// let state = MarkdownRendererState::new()
    ///     .with_title("Preview");
    /// assert_eq!(state.title(), Some("Preview"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the show_source flag (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MarkdownRendererState;
    ///
    /// let state = MarkdownRendererState::new()
    ///     .with_show_source(true);
    /// assert!(state.show_source());
    /// ```
    pub fn with_show_source(mut self, show: bool) -> Self {
        self.show_source = show;
        self
    }

    // ---- Source accessors ----

    /// Returns the markdown source text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MarkdownRendererState;
    ///
    /// let state = MarkdownRendererState::new()
    ///     .with_source("hello");
    /// assert_eq!(state.source(), "hello");
    /// ```
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Sets the markdown source text and resets scroll to the top.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MarkdownRendererState;
    ///
    /// let mut state = MarkdownRendererState::new();
    /// state.set_source("# New");
    /// assert_eq!(state.source(), "# New");
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn set_source(&mut self, source: impl Into<String>) {
        self.source = source.into();
        self.scroll = ScrollState::new(self.source.lines().count().max(1));
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MarkdownRendererState;
    ///
    /// let state = MarkdownRendererState::new()
    ///     .with_title("Title");
    /// assert_eq!(state.title(), Some("Title"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    // ---- Display options ----

    /// Returns whether the raw source view is active.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MarkdownRendererState;
    ///
    /// let state = MarkdownRendererState::new();
    /// assert!(!state.show_source());
    /// ```
    pub fn show_source(&self) -> bool {
        self.show_source
    }

    /// Sets whether to show raw source.
    pub fn set_show_source(&mut self, show: bool) {
        self.show_source = show;
    }

    // ---- Scroll accessors ----

    /// Returns the current scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MarkdownRendererState;
    ///
    /// let state = MarkdownRendererState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Sets the scroll offset.
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll.set_offset(offset);
    }

    // ---- State accessors ----

    // ---- Instance methods ----

    /// Updates the state with a message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MarkdownRendererState, MarkdownRendererMessage};
    ///
    /// let mut state = MarkdownRendererState::new()
    ///     .with_source("line 1\nline 2");
    /// state.update(MarkdownRendererMessage::ScrollDown);
    /// assert_eq!(state.scroll_offset(), 1);
    /// ```
    pub fn update(&mut self, msg: MarkdownRendererMessage) {
        MarkdownRenderer::update(self, msg);
    }
}

/// A markdown rendering component with scroll support.
///
/// Parses markdown source text and renders it with styled headings, bold,
/// italic, strikethrough, inline code, code blocks, lists, links,
/// blockquotes, and horizontal rules. Supports toggling between rendered
/// and raw source views.
///
/// # Key Bindings
///
/// - `Up` / `k` -- Scroll up one line
/// - `Down` / `j` -- Scroll down one line
/// - `PageUp` / `Ctrl+u` -- Scroll up half a page
/// - `PageDown` / `Ctrl+d` -- Scroll down half a page
/// - `Home` / `g` -- Scroll to top
/// - `End` / `G` -- Scroll to bottom
/// - `s` -- Toggle between rendered and raw source views
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, MarkdownRenderer, MarkdownRendererState};
///
/// let state = MarkdownRendererState::new()
///     .with_source("# Hello\n\nWorld");
/// assert_eq!(state.source(), "# Hello\n\nWorld");
/// ```
pub struct MarkdownRenderer;

impl Component for MarkdownRenderer {
    type State = MarkdownRendererState;
    type Message = MarkdownRendererMessage;
    type Output = ();

    fn init() -> Self::State {
        MarkdownRendererState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !ctrl => Some(MarkdownRendererMessage::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') if !ctrl => {
                Some(MarkdownRendererMessage::ScrollDown)
            }
            KeyCode::PageUp => Some(MarkdownRendererMessage::PageUp(10)),
            KeyCode::PageDown => Some(MarkdownRendererMessage::PageDown(10)),
            KeyCode::Char('u') if ctrl => Some(MarkdownRendererMessage::PageUp(10)),
            KeyCode::Char('d') if ctrl => Some(MarkdownRendererMessage::PageDown(10)),
            KeyCode::Home | KeyCode::Char('g') if !shift => Some(MarkdownRendererMessage::Home),
            KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => {
                Some(MarkdownRendererMessage::End)
            }
            KeyCode::Char('s') if !ctrl && !shift => Some(MarkdownRendererMessage::ToggleSource),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            MarkdownRendererMessage::ScrollUp => {
                state.scroll.scroll_up();
            }
            MarkdownRendererMessage::ScrollDown => {
                state.scroll.scroll_down();
            }
            MarkdownRendererMessage::PageUp(n) => {
                state.scroll.page_up(n);
            }
            MarkdownRendererMessage::PageDown(n) => {
                state.scroll.page_down(n);
            }
            MarkdownRendererMessage::Home => {
                state.scroll.scroll_to_start();
            }
            MarkdownRendererMessage::End => {
                state.scroll.scroll_to_end();
            }
            MarkdownRendererMessage::SetSource(source) => {
                state.source = source;
                state.scroll = ScrollState::new(state.source.lines().count().max(1));
            }
            MarkdownRendererMessage::ToggleSource => {
                state.show_source = !state.show_source;
                // Reset scroll when switching views
                state.scroll.set_offset(0);
            }
        }
        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
                    "MarkdownRenderer".to_string(),
                ))
                .with_id("markdown_renderer")
                .with_focus(ctx.focused)
                .with_disabled(ctx.disabled),
            );
        });

        let border_style = if ctx.disabled {
            theme.disabled_style()
        } else if ctx.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(title) = &state.title {
            let suffix = if state.show_source { " [source]" } else { "" };
            block = block.title(format!("{}{}", title, suffix));
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        if state.show_source {
            // Raw source view
            let text_style = if ctx.disabled {
                theme.disabled_style()
            } else {
                theme.normal_style()
            };

            let total_lines = crate::util::wrapped_line_count(&state.source, inner.width as usize);
            let visible = inner.height as usize;
            let max_scroll = total_lines.saturating_sub(visible);
            let effective_scroll = state.scroll.offset().min(max_scroll);

            let paragraph = Paragraph::new(state.source.as_str())
                .style(text_style)
                .wrap(Wrap { trim: false })
                .scroll((effective_scroll as u16, 0));

            frame.render_widget(paragraph, inner);

            if total_lines > visible {
                let mut bar_scroll = ScrollState::new(total_lines);
                bar_scroll.set_viewport_height(visible);
                bar_scroll.set_offset(effective_scroll);
                crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
            }
        } else {
            // Rendered markdown view
            let rendered_lines = render::render_markdown(&state.source, inner.width, theme);
            let total_lines = rendered_lines.len();
            let visible = inner.height as usize;
            let max_scroll = total_lines.saturating_sub(visible);
            let effective_scroll = state.scroll.offset().min(max_scroll);

            let text = Text::from(rendered_lines);
            let paragraph = Paragraph::new(text)
                .wrap(Wrap { trim: false })
                .scroll((effective_scroll as u16, 0));

            frame.render_widget(paragraph, inner);

            if total_lines > visible {
                let mut bar_scroll = ScrollState::new(total_lines);
                bar_scroll.set_viewport_height(visible);
                bar_scroll.set_offset(effective_scroll);
                crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
            }
        }
    }
}

#[cfg(test)]
mod tests;
