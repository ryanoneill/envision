//! A generic scrollable container component.
//!
//! [`ScrollView`] provides a bordered, scrollable viewport that wraps arbitrary
//! content. It manages scroll state and renders a scrollbar indicator when the
//! content exceeds the visible area. The parent application is responsible for
//! rendering actual content into the area returned by
//! [`ScrollViewState::content_area()`].
//!
//! This is the missing layout primitive: unlike [`ScrollableText`](super::ScrollableText)
//! which owns and renders text content directly, `ScrollView` is a generic
//! container that provides scrolling behavior for any content the parent
//! renders.
//!
//! State is stored in [`ScrollViewState`], updated via [`ScrollViewMessage`],
//! and produces no output (Output = `()`).
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{ScrollView, ScrollViewMessage, ScrollViewState, Component};
//!
//! let mut state = ScrollViewState::new()
//!     .with_content_height(100)
//!     .with_title("Log Output");
//!
//! // Scroll down
//! let output = ScrollView::update(&mut state, ScrollViewMessage::ScrollDown);
//! assert_eq!(state.scroll_offset(), 1);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::scroll::ScrollState;

/// Messages that can be sent to a ScrollView.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum ScrollViewMessage {
    /// Scroll up by one line.
    ScrollUp,
    /// Scroll down by one line.
    ScrollDown,
    /// Scroll up by one page.
    PageUp,
    /// Scroll down by one page.
    PageDown,
    /// Scroll to the top.
    Home,
    /// Scroll to the bottom.
    End,
    /// Set the total content height (in lines).
    SetContentHeight(u16),
}

/// State for a ScrollView component.
///
/// Contains the scroll position, content dimensions, and display options.
///
/// # Example
///
/// ```rust
/// use envision::component::ScrollViewState;
///
/// let state = ScrollViewState::new()
///     .with_content_height(50)
///     .with_title("Preview");
/// assert_eq!(state.content_height(), 50);
/// assert_eq!(state.title(), Some("Preview"));
/// assert_eq!(state.scroll_offset(), 0);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ScrollViewState {
    /// Total height of the content in lines.
    content_height: u16,
    /// Scroll state tracking offset and scrollbar.
    scroll: ScrollState,
    /// Optional title for the border.
    title: Option<String>,
    /// Whether to show the scrollbar when content overflows.
    show_scrollbar: bool,
}

impl Default for ScrollViewState {
    fn default() -> Self {
        Self {
            content_height: 0,
            scroll: ScrollState::default(),
            title: None,
            show_scrollbar: true,
        }
    }
}

impl ScrollViewState {
    /// Creates a new scroll view state with default settings.
    ///
    /// The content height starts at 0, with no title, scrollbar enabled,
    /// unfocused, and enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new();
    /// assert_eq!(state.content_height(), 0);
    /// assert_eq!(state.scroll_offset(), 0);
    /// assert!(state.show_scrollbar());
    /// assert_eq!(state.title(), None);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the total content height (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new().with_content_height(100);
    /// assert_eq!(state.content_height(), 100);
    /// ```
    pub fn with_content_height(mut self, height: u16) -> Self {
        self.content_height = height;
        self.scroll.set_content_length(height as usize);
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new().with_title("Preview");
    /// assert_eq!(state.title(), Some("Preview"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether the scrollbar is shown (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new().with_show_scrollbar(false);
    /// assert!(!state.show_scrollbar());
    /// ```
    pub fn with_show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    // ---- Content accessors ----

    /// Returns the total content height.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new().with_content_height(50);
    /// assert_eq!(state.content_height(), 50);
    /// ```
    pub fn content_height(&self) -> u16 {
        self.content_height
    }

    /// Sets the total content height.
    ///
    /// Updates the scroll state's content length accordingly.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let mut state = ScrollViewState::new();
    /// state.set_content_height(75);
    /// assert_eq!(state.content_height(), 75);
    /// ```
    pub fn set_content_height(&mut self, height: u16) {
        self.content_height = height;
        self.scroll.set_content_length(height as usize);
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new().with_title("Output");
    /// assert_eq!(state.title(), Some("Output"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let mut state = ScrollViewState::new();
    /// state.set_title(Some("Output".to_string()));
    /// assert_eq!(state.title(), Some("Output"));
    /// state.set_title(None);
    /// assert_eq!(state.title(), None);
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns whether the scrollbar is shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new();
    /// assert!(state.show_scrollbar());
    /// ```
    pub fn show_scrollbar(&self) -> bool {
        self.show_scrollbar
    }

    /// Sets whether the scrollbar is shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let mut state = ScrollViewState::new();
    /// state.set_show_scrollbar(false);
    /// assert!(!state.show_scrollbar());
    /// ```
    pub fn set_show_scrollbar(&mut self, show: bool) {
        self.show_scrollbar = show;
    }

    // ---- Scroll accessors ----

    /// Returns the current scroll offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new();
    /// assert_eq!(state.scroll_offset(), 0);
    /// ```
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    /// Sets the scroll offset directly.
    ///
    /// The offset is clamped to the valid range based on the content height
    /// and viewport height.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let mut state = ScrollViewState::new().with_content_height(100);
    /// state.set_scroll_offset(50);
    /// assert_eq!(state.scroll_offset(), 50);
    /// ```
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll.set_offset(offset);
    }

    /// Returns a reference to the internal scroll state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new().with_content_height(50);
    /// assert_eq!(state.scroll_state().content_length(), 50);
    /// ```
    pub fn scroll_state(&self) -> &ScrollState {
        &self.scroll
    }

    // ---- State accessors ----

    // ---- Content area ----

    /// Returns the visible content `Rect` accounting for the scroll offset.
    ///
    /// This is the area inside the border where the parent should render content.
    /// The returned `Rect` has its `y` coordinate adjusted by the scroll offset,
    /// so the parent can use it directly to position content.
    ///
    /// The border uses 1 cell on each side (top, bottom, left, right), so the
    /// inner area is smaller than the provided area by 2 in each dimension.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    /// use ratatui::prelude::Rect;
    ///
    /// let state = ScrollViewState::new().with_content_height(100);
    /// let area = Rect::new(0, 0, 40, 20);
    ///
    /// let content = state.content_area(area);
    /// // Inner area: x+1, y+1, width-2, height-2 = (1, 1, 38, 18)
    /// assert_eq!(content.x, 1);
    /// assert_eq!(content.y, 1);
    /// assert_eq!(content.width, 38);
    /// assert_eq!(content.height, 18);
    /// ```
    pub fn content_area(&self, area: Rect) -> Rect {
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);

        if inner.width == 0 || inner.height == 0 {
            return Rect::new(area.x, area.y, 0, 0);
        }

        inner
    }

    /// Returns the viewport height for the given render area.
    ///
    /// This is the number of visible lines inside the bordered area.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    /// use ratatui::prelude::Rect;
    ///
    /// let state = ScrollViewState::new();
    /// let area = Rect::new(0, 0, 40, 20);
    /// assert_eq!(state.viewport_height(area), 18);
    /// ```
    pub fn viewport_height(&self, area: Rect) -> u16 {
        let content = self.content_area(area);
        content.height
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ScrollViewMessage, ScrollViewState};
    ///
    /// let mut state = ScrollViewState::new().with_content_height(100);
    /// state.update(ScrollViewMessage::ScrollDown);
    /// assert_eq!(state.scroll_offset(), 1);
    /// ```
    pub fn update(&mut self, msg: ScrollViewMessage) -> Option<()> {
        ScrollView::update(self, msg)
    }
}

/// A generic scrollable container component.
///
/// ScrollView renders a bordered area with an optional scrollbar. The actual
/// content is rendered by the parent application using the area returned by
/// [`ScrollViewState::content_area()`]. This follows the same delegation
/// pattern as [`Collapsible`](super::Collapsible).
///
/// # Keyboard Navigation
///
/// When focused:
/// - `Up` / `k` -- Scroll up one line
/// - `Down` / `j` -- Scroll down one line
/// - `PageUp` / `Ctrl+u` -- Scroll up one page
/// - `PageDown` / `Ctrl+d` -- Scroll down one page
/// - `Home` / `g` -- Scroll to top
/// - `End` / `G` (Shift+g) -- Scroll to bottom
///
/// # Visual Layout
///
/// ```text
/// +--- Title -------------------------+
/// | [content rendered by parent]      |
/// |                                  ▓|
/// |                                  ░|
/// |                                  ░|
/// +-----------------------------------+
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{ScrollView, ScrollViewMessage, ScrollViewState, Component};
///
/// let mut state = ScrollViewState::new()
///     .with_content_height(200)
///     .with_title("Log Output");
///
/// ScrollView::update(&mut state, ScrollViewMessage::ScrollDown);
/// assert_eq!(state.scroll_offset(), 1);
///
/// ScrollView::update(&mut state, ScrollViewMessage::Home);
/// assert_eq!(state.scroll_offset(), 0);
/// ```
pub struct ScrollView;

impl Component for ScrollView {
    type State = ScrollViewState;
    type Message = ScrollViewMessage;
    type Output = ();

    fn init() -> Self::State {
        ScrollViewState::default()
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
        let ctrl = key.modifiers.ctrl();
        let shift = key.modifiers.shift();

        match key.key {
            Key::Up | Key::Char('k') if !ctrl => Some(ScrollViewMessage::ScrollUp),
            Key::Down | Key::Char('j') if !ctrl => Some(ScrollViewMessage::ScrollDown),
            Key::PageUp => Some(ScrollViewMessage::PageUp),
            Key::PageDown => Some(ScrollViewMessage::PageDown),
            Key::Char('u') if ctrl => Some(ScrollViewMessage::PageUp),
            Key::Char('d') if ctrl => Some(ScrollViewMessage::PageDown),
            Key::Home | Key::Char('g') if !shift => Some(ScrollViewMessage::Home),
            Key::End | Key::Char('g') if key.modifiers.shift() || key.key == Key::End => {
                Some(ScrollViewMessage::End)
            }
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ScrollViewMessage::ScrollUp => {
                if state.scroll.scroll_up() {
                    Some(())
                } else {
                    None
                }
            }
            ScrollViewMessage::ScrollDown => {
                if state.scroll.scroll_down() {
                    Some(())
                } else {
                    None
                }
            }
            ScrollViewMessage::PageUp => {
                let page = state.scroll.viewport_height().max(1);
                if state.scroll.page_up(page) {
                    Some(())
                } else {
                    None
                }
            }
            ScrollViewMessage::PageDown => {
                let page = state.scroll.viewport_height().max(1);
                if state.scroll.page_down(page) {
                    Some(())
                } else {
                    None
                }
            }
            ScrollViewMessage::Home => {
                if state.scroll.scroll_to_start() {
                    Some(())
                } else {
                    None
                }
            }
            ScrollViewMessage::End => {
                if state.scroll.scroll_to_end() {
                    Some(())
                } else {
                    None
                }
            }
            ScrollViewMessage::SetContentHeight(height) => {
                state.content_height = height;
                state.scroll.set_content_length(height as usize);
                None
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if ctx.area.width == 0 || ctx.area.height == 0 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
                    "ScrollView".to_string(),
                ))
                .with_id("scroll_view")
                .with_focus(ctx.focused)
                .with_disabled(ctx.disabled),
            );
        });

        let border_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else if ctx.focused {
            ctx.theme.focused_border_style()
        } else {
            ctx.theme.border_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(title) = &state.title {
            block = block.title(title.as_str());
        }

        let inner = block.inner(ctx.area);
        ctx.frame.render_widget(block, ctx.area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Update viewport for scrollbar calculation
        let viewport_height = inner.height as usize;
        let total = state.content_height as usize;

        // Render scrollbar when content exceeds viewport
        if state.show_scrollbar && total > viewport_height {
            let mut bar_scroll = ScrollState::new(total);
            bar_scroll.set_viewport_height(viewport_height);
            bar_scroll.set_offset(
                state
                    .scroll
                    .offset()
                    .min(total.saturating_sub(viewport_height)),
            );
            crate::scroll::render_scrollbar_inside_border(
                &bar_scroll,
                ctx.frame,
                ctx.area,
                ctx.theme,
            );
        }
    }
}

#[cfg(test)]
mod tests;
