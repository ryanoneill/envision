//! Virtual scrolling infrastructure for TUI components.
//!
//! The [`ScrollState`] struct provides composable scroll offset tracking,
//! visible range calculation, and scrollbar rendering support. Components
//! embed a `ScrollState` in their state to gain virtual scrolling — rendering
//! only the visible portion of their content instead of allocating items for
//! every row.
//!
//! # Two Scrolling Patterns
//!
//! ## Offset-based scrolling
//!
//! For components like log viewers and text displays where the user scrolls
//! through content without a selection cursor:
//!
//! ```rust
//! use envision::scroll::ScrollState;
//!
//! let mut scroll = ScrollState::new(1000); // 1000 total lines
//! scroll.set_viewport_height(24);          // terminal is 24 lines tall
//!
//! // User presses Page Down
//! scroll.page_down(scroll.viewport_height());
//!
//! // Render only the visible range
//! let range = scroll.visible_range();
//! assert_eq!(range, 24..48);
//! ```
//!
//! ## Selection-based scrolling
//!
//! For components like lists and tables where a selected item should always
//! remain visible:
//!
//! ```rust
//! use envision::scroll::ScrollState;
//!
//! let mut scroll = ScrollState::new(100); // 100 items
//! scroll.set_viewport_height(10);         // 10 visible at a time
//!
//! // User moves selection to item 15
//! scroll.ensure_visible(15);
//!
//! // Viewport scrolled to keep item 15 in view
//! let range = scroll.visible_range();
//! assert!(range.contains(&15));
//! ```

#[cfg(test)]
mod tests;

use std::ops::Range;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::theme::Theme;

/// Tracks scroll position for virtual scrolling in TUI components.
///
/// `ScrollState` is a composable building block that any component can
/// embed in its state to gain scroll offset tracking, visible range
/// calculation, and scrollbar rendering support.
///
/// It supports two usage patterns:
/// - **Offset mode**: The scroll offset is controlled directly via
///   [`scroll_up`](Self::scroll_up), [`scroll_down`](Self::scroll_down), etc.
///   Used by LogViewer, ChatView, ScrollableText.
/// - **Selection mode**: The scroll offset follows a selected index via
///   [`ensure_visible`](Self::ensure_visible), keeping it within the viewport.
///   Used by SelectableList, Table, DataGrid, etc.
///
/// # Example
///
/// ```rust
/// use envision::scroll::ScrollState;
///
/// let mut scroll = ScrollState::new(500);
/// scroll.set_viewport_height(20);
///
/// assert_eq!(scroll.offset(), 0);
/// assert!(scroll.can_scroll());
/// assert!(scroll.at_start());
///
/// scroll.scroll_down();
/// assert_eq!(scroll.offset(), 1);
///
/// scroll.scroll_to_end();
/// assert_eq!(scroll.offset(), 480); // 500 - 20
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ScrollState {
    /// The scroll offset (index of the first visible item).
    offset: usize,
    /// Total number of scrollable items or lines.
    content_length: usize,
    /// Number of items visible in the viewport.
    viewport_height: usize,
}

impl ScrollState {
    /// Creates a new `ScrollState` with the given content length.
    ///
    /// The scroll offset starts at 0 and viewport height defaults to 0
    /// (set it in `view()` once the render area is known).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::scroll::ScrollState;
    ///
    /// let scroll = ScrollState::new(100);
    /// assert_eq!(scroll.content_length(), 100);
    /// assert_eq!(scroll.offset(), 0);
    /// ```
    pub fn new(content_length: usize) -> Self {
        Self {
            offset: 0,
            content_length,
            viewport_height: 0,
        }
    }

    // =========================================================================
    // Setters
    // =========================================================================

    /// Sets the total number of scrollable items or lines.
    ///
    /// If the current offset exceeds the new maximum, it is clamped.
    pub fn set_content_length(&mut self, len: usize) {
        self.content_length = len;
        self.clamp_offset();
    }

    /// Sets the viewport height (number of visible items).
    ///
    /// Typically called in `view()` after computing the inner render area.
    /// If the current offset exceeds the new maximum, it is clamped.
    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
        self.clamp_offset();
    }

    /// Sets the scroll offset directly, clamped to the valid range.
    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
        self.clamp_offset();
    }

    // =========================================================================
    // Scroll operations
    // =========================================================================

    /// Scrolls up by one item. Returns `true` if the offset changed.
    pub fn scroll_up(&mut self) -> bool {
        self.scroll_up_by(1)
    }

    /// Scrolls down by one item. Returns `true` if the offset changed.
    pub fn scroll_down(&mut self) -> bool {
        self.scroll_down_by(1)
    }

    /// Scrolls up by `n` items. Returns `true` if the offset changed.
    pub fn scroll_up_by(&mut self, n: usize) -> bool {
        let old = self.offset;
        self.offset = self.offset.saturating_sub(n);
        self.offset != old
    }

    /// Scrolls down by `n` items. Returns `true` if the offset changed.
    pub fn scroll_down_by(&mut self, n: usize) -> bool {
        let old = self.offset;
        self.offset = self.offset.saturating_add(n);
        self.clamp_offset();
        self.offset != old
    }

    /// Scrolls up by one page (viewport height). Returns `true` if the offset changed.
    pub fn page_up(&mut self, page_size: usize) -> bool {
        self.scroll_up_by(page_size)
    }

    /// Scrolls down by one page (viewport height). Returns `true` if the offset changed.
    pub fn page_down(&mut self, page_size: usize) -> bool {
        self.scroll_down_by(page_size)
    }

    /// Scrolls to the first item. Returns `true` if the offset changed.
    pub fn scroll_to_start(&mut self) -> bool {
        let old = self.offset;
        self.offset = 0;
        self.offset != old
    }

    /// Scrolls to the last page. Returns `true` if the offset changed.
    pub fn scroll_to_end(&mut self) -> bool {
        let old = self.offset;
        self.offset = self.max_offset();
        self.offset != old
    }

    // =========================================================================
    // Selection tracking
    // =========================================================================

    /// Adjusts the scroll offset to ensure the given index is visible.
    ///
    /// If the index is above the viewport, scrolls up. If below, scrolls
    /// down. If already visible, does nothing. Returns `true` if the
    /// offset changed.
    ///
    /// This is the key method for selection-based components: after
    /// changing the selected index, call this to keep the selection
    /// in view.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::scroll::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(100);
    /// scroll.set_viewport_height(10);
    ///
    /// // Selection at item 25 — scrolls viewport to show it
    /// assert!(scroll.ensure_visible(25));
    /// assert_eq!(scroll.offset(), 16); // 25 - 10 + 1
    ///
    /// // Selection at item 20 — already visible, no change
    /// assert!(!scroll.ensure_visible(20));
    /// ```
    pub fn ensure_visible(&mut self, index: usize) -> bool {
        let old = self.offset;

        if self.viewport_height == 0 {
            return false;
        }

        if index < self.offset {
            // Index is above the viewport — scroll up
            self.offset = index;
        } else if index >= self.offset + self.viewport_height {
            // Index is below the viewport — scroll down
            self.offset = index.saturating_sub(self.viewport_height.saturating_sub(1));
        }

        self.clamp_offset();
        self.offset != old
    }

    // =========================================================================
    // Queries
    // =========================================================================

    /// Returns the current scroll offset (index of the first visible item).
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the total content length.
    pub fn content_length(&self) -> usize {
        self.content_length
    }

    /// Returns the viewport height.
    pub fn viewport_height(&self) -> usize {
        self.viewport_height
    }

    /// Returns the range of visible item indices.
    ///
    /// The range is `offset..min(offset + viewport_height, content_length)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::scroll::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(50);
    /// scroll.set_viewport_height(10);
    /// assert_eq!(scroll.visible_range(), 0..10);
    ///
    /// scroll.scroll_to_end();
    /// assert_eq!(scroll.visible_range(), 40..50);
    /// ```
    pub fn visible_range(&self) -> Range<usize> {
        let end = (self.offset + self.viewport_height).min(self.content_length);
        self.offset..end
    }

    /// Returns the maximum valid scroll offset.
    ///
    /// This is `content_length.saturating_sub(viewport_height)`.
    pub fn max_offset(&self) -> usize {
        self.content_length.saturating_sub(self.viewport_height)
    }

    /// Returns `true` if scrolling is possible (content exceeds viewport).
    pub fn can_scroll(&self) -> bool {
        self.content_length > self.viewport_height
    }

    /// Returns `true` if at the top (offset is 0).
    pub fn at_start(&self) -> bool {
        self.offset == 0
    }

    /// Returns `true` if at the bottom (offset is at max).
    pub fn at_end(&self) -> bool {
        self.offset >= self.max_offset()
    }

    // =========================================================================
    // Scrollbar integration
    // =========================================================================

    /// Creates a ratatui [`ScrollbarState`] configured for this scroll state.
    ///
    /// Use this with ratatui's [`Scrollbar`] widget for visual scroll
    /// indicators.
    pub fn scrollbar_state(&self) -> ScrollbarState {
        ScrollbarState::default()
            .content_length(self.content_length.saturating_sub(self.viewport_height))
            .viewport_content_length(self.viewport_height)
            .position(self.offset)
    }

    // =========================================================================
    // Internal
    // =========================================================================

    /// Clamps the offset to the valid range `0..=max_offset`.
    fn clamp_offset(&mut self) {
        let max = self.max_offset();
        if self.offset > max {
            self.offset = max;
        }
    }
}

/// Renders a vertical scrollbar on the right edge of the given area.
///
/// Only renders if the content exceeds the viewport height. Uses the
/// theme's border color for the scrollbar track and foreground color
/// for the thumb.
///
/// # Example
///
/// ```rust,no_run
/// # use envision::prelude::*;
/// # use envision::scroll::{ScrollState, render_scrollbar};
/// # let scroll = ScrollState::new(100);
/// # let theme = Theme::default();
/// # let area = Rect::new(0, 0, 80, 24);
/// # let mut terminal = Terminal::new(
/// #     envision::CaptureBackend::new(80, 24)
/// # ).unwrap();
/// # terminal.draw(|frame| {
/// render_scrollbar(&scroll, frame, area, &theme);
/// # }).unwrap();
/// ```
pub fn render_scrollbar(scroll: &ScrollState, frame: &mut Frame, area: Rect, theme: &Theme) {
    if !scroll.can_scroll() {
        return;
    }

    let mut scrollbar_state = scroll.scrollbar_state();
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(theme.normal_style())
        .track_style(theme.disabled_style());
    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}

/// Renders a vertical scrollbar inside a bordered area.
///
/// This accounts for the 1-cell border on each side, positioning the
/// scrollbar within the inner area. This is the common case for
/// components that use `Block::default().borders(Borders::ALL)`.
///
/// Only renders if the content exceeds the viewport height.
pub fn render_scrollbar_inside_border(
    scroll: &ScrollState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    if !scroll.can_scroll() || area.height < 3 {
        return;
    }

    // Inset by 1 on top and bottom for borders
    let inner = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(2),
    );

    let mut scrollbar_state = scroll.scrollbar_state();
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(theme.normal_style())
        .track_style(theme.disabled_style());
    frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
}
