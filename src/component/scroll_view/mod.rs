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
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{ScrollView, ScrollViewMessage, ScrollViewState, Component, Focusable};
//!
//! let mut state = ScrollViewState::new()
//!     .with_content_height(100)
//!     .with_title("Log Output");
//! ScrollView::focus(&mut state);
//!
//! // Scroll down
//! let output = ScrollView::update(&mut state, ScrollViewMessage::ScrollDown);
//! assert_eq!(state.scroll_offset(), 1);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, Disableable, Focusable, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

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
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for ScrollViewState {
    fn default() -> Self {
        Self {
            content_height: 0,
            scroll: ScrollState::default(),
            title: None,
            show_scrollbar: true,
            focused: false,
            disabled: false,
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
    /// assert!(!state.is_focused());
    /// assert!(!state.is_disabled());
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

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new().with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    /// Returns true if the component is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new();
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
    /// use envision::component::ScrollViewState;
    ///
    /// let mut state = ScrollViewState::new();
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
    /// use envision::component::ScrollViewState;
    ///
    /// let state = ScrollViewState::new();
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
    /// use envision::component::ScrollViewState;
    ///
    /// let mut state = ScrollViewState::new();
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

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

    /// Maps an input event to a scroll view message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ScrollView, ScrollViewMessage, ScrollViewState, Component, Focusable};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = ScrollViewState::new();
    /// ScrollView::focus(&mut state);
    ///
    /// let msg = state.handle_event(&Event::key(KeyCode::Up));
    /// assert_eq!(msg, Some(ScrollViewMessage::ScrollUp));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<ScrollViewMessage> {
        ScrollView::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ScrollView, ScrollViewState, Focusable};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = ScrollViewState::new().with_content_height(100);
    /// ScrollView::focus(&mut state);
    ///
    /// let output = state.dispatch_event(&Event::key(KeyCode::Down));
    /// assert_eq!(state.scroll_offset(), 1);
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<()> {
        ScrollView::dispatch_event(self, event)
    }

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
/// use envision::component::{ScrollView, ScrollViewMessage, ScrollViewState, Component, Focusable};
///
/// let mut state = ScrollViewState::new()
///     .with_content_height(200)
///     .with_title("Log Output");
/// ScrollView::focus(&mut state);
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

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !ctrl => Some(ScrollViewMessage::ScrollUp),
            KeyCode::Down | KeyCode::Char('j') if !ctrl => Some(ScrollViewMessage::ScrollDown),
            KeyCode::PageUp => Some(ScrollViewMessage::PageUp),
            KeyCode::PageDown => Some(ScrollViewMessage::PageDown),
            KeyCode::Char('u') if ctrl => Some(ScrollViewMessage::PageUp),
            KeyCode::Char('d') if ctrl => Some(ScrollViewMessage::PageDown),
            KeyCode::Home | KeyCode::Char('g') if !shift => Some(ScrollViewMessage::Home),
            KeyCode::End | KeyCode::Char('G') if shift || key.code == KeyCode::End => {
                Some(ScrollViewMessage::End)
            }
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
                    "ScrollView".to_string(),
                ))
                .with_id("scroll_view")
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
            block = block.title(title.as_str());
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

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
            crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
        }
    }
}

impl Focusable for ScrollView {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for ScrollView {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod tests;
