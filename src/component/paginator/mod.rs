//! A page navigation indicator component.
//!
//! [`Paginator`] displays the current page position within a paginated dataset.
//! It supports several display styles: page-of-total, item range, dot indicators,
//! and a compact arrows format. State is stored in [`PaginatorState`], updated via
//! [`PaginatorMessage`], and produces [`PaginatorOutput`].
//!
//!
//! See also [`StepIndicator`](super::StepIndicator) for wizard-style step display.
//!
//! # Display Styles
//!
//! - **PageOfTotal**: `"Page 3 of 12"`
//! - **RangeOfTotal**: `"Showing 21-30 of 247"`
//! - **Dots**: `"○ ○ ● ○ ○"` with current page highlighted
//! - **Compact**: `"◀ 3 / 12 ▶"` with arrows dimmed at boundaries
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Paginator, PaginatorState,
//!     PaginatorMessage, PaginatorOutput, PaginatorStyle,
//! };
//!
//! let mut state = PaginatorState::new(5)
//!     .with_current_page(2);
//!
//! assert_eq!(state.current_page(), 2);
//! assert_eq!(state.display_page(), 3);
//! assert_eq!(state.total_pages(), 5);
//!
//! // Navigate to next page
//! let output = Paginator::update(&mut state, PaginatorMessage::NextPage);
//! assert_eq!(output, Some(PaginatorOutput::PageChanged(3)));
//! assert_eq!(state.current_page(), 3);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::theme::Theme;

/// Display style for the paginator.
///
/// Controls how page position information is rendered.
///
/// # Example
///
/// ```rust
/// use envision::component::PaginatorStyle;
///
/// let style = PaginatorStyle::default();
/// assert_eq!(style, PaginatorStyle::PageOfTotal);
///
/// let style = PaginatorStyle::Dots;
/// assert_eq!(style, PaginatorStyle::Dots);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum PaginatorStyle {
    /// `"Page 3 of 12"` format.
    #[default]
    PageOfTotal,
    /// `"Showing 51-100 of 2,847"` format with item ranges.
    RangeOfTotal,
    /// `"○ ○ ● ○ ○"` dot indicators with current page highlighted.
    Dots,
    /// `"◀ 3 / 12 ▶"` compact format with navigation arrows.
    Compact,
}

/// Messages that can be sent to a Paginator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaginatorMessage {
    /// Navigate to the next page.
    NextPage,
    /// Navigate to the previous page.
    PrevPage,
    /// Navigate to the first page.
    FirstPage,
    /// Navigate to the last page.
    LastPage,
    /// Navigate to a specific page (0-indexed).
    GoToPage(usize),
    /// Update the total number of pages.
    SetTotalPages(usize),
    /// Update the total number of items (recalculates total pages).
    SetTotalItems(usize),
}

/// Output messages from a Paginator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaginatorOutput {
    /// The current page changed. Contains the new page index (0-indexed).
    PageChanged(usize),
}

/// State for a Paginator component.
///
/// Pages are 0-indexed internally but displayed as 1-indexed.
///
/// # Example
///
/// ```rust
/// use envision::component::PaginatorState;
///
/// let state = PaginatorState::new(10);
/// assert_eq!(state.current_page(), 0);
/// assert_eq!(state.display_page(), 1);
/// assert_eq!(state.total_pages(), 10);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct PaginatorState {
    /// Current page (0-indexed).
    current_page: usize,
    /// Total number of pages.
    total_pages: usize,
    /// Items per page (used for RangeOfTotal display).
    page_size: usize,
    /// Total number of items (used for RangeOfTotal display).
    total_items: usize,
    /// Display style.
    style: PaginatorStyle,
}

impl Default for PaginatorState {
    fn default() -> Self {
        Self {
            current_page: 0,
            total_pages: 1,
            page_size: 10,
            total_items: 10,
            style: PaginatorStyle::default(),
        }
    }
}

impl PaginatorState {
    /// Creates a new paginator with the given total number of pages.
    ///
    /// The page size defaults to 10 and total items is set to
    /// `total_pages * 10`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::new(5);
    /// assert_eq!(state.total_pages(), 5);
    /// assert_eq!(state.current_page(), 0);
    /// assert_eq!(state.page_size(), 10);
    /// ```
    pub fn new(total_pages: usize) -> Self {
        let total_pages = total_pages.max(1);
        Self {
            total_pages,
            total_items: total_pages * 10,
            ..Self::default()
        }
    }

    /// Creates a paginator from a total item count and page size.
    ///
    /// Calculates total pages as `ceil(total_items / page_size)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::from_items(247, 25);
    /// assert_eq!(state.total_pages(), 10);
    /// assert_eq!(state.total_items(), 247);
    /// assert_eq!(state.page_size(), 25);
    /// ```
    pub fn from_items(total_items: usize, page_size: usize) -> Self {
        let page_size = page_size.max(1);
        let total_pages = calculate_total_pages(total_items, page_size);
        Self {
            total_pages,
            page_size,
            total_items,
            ..Self::default()
        }
    }

    /// Sets the display style (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{PaginatorState, PaginatorStyle};
    ///
    /// let state = PaginatorState::new(5)
    ///     .with_style(PaginatorStyle::Dots);
    /// assert_eq!(state.style(), &PaginatorStyle::Dots);
    /// ```
    pub fn with_style(mut self, style: PaginatorStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the page size (builder pattern).
    ///
    /// Recalculates total pages based on total items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::from_items(100, 10)
    ///     .with_page_size(25);
    /// assert_eq!(state.page_size(), 25);
    /// assert_eq!(state.total_pages(), 4);
    /// ```
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size.max(1);
        self.total_pages = calculate_total_pages(self.total_items, self.page_size);
        self.current_page = self.current_page.min(self.total_pages.saturating_sub(1));
        self
    }

    /// Sets the starting page (builder pattern).
    ///
    /// The page is clamped to the valid range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::new(5)
    ///     .with_current_page(3);
    /// assert_eq!(state.current_page(), 3);
    ///
    /// // Clamped to valid range
    /// let state = PaginatorState::new(5)
    ///     .with_current_page(100);
    /// assert_eq!(state.current_page(), 4);
    /// ```
    pub fn with_current_page(mut self, page: usize) -> Self {
        self.current_page = page.min(self.total_pages.saturating_sub(1));
        self
    }

    // ---- Accessors ----

    /// Returns the current page (0-indexed).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::new(5);
    /// assert_eq!(state.current_page(), 0);
    /// ```
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// Returns the display page (1-indexed, for human-readable display).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::new(5);
    /// assert_eq!(state.display_page(), 1);
    /// ```
    pub fn display_page(&self) -> usize {
        self.current_page + 1
    }

    /// Returns the total number of pages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::new(12);
    /// assert_eq!(state.total_pages(), 12);
    /// ```
    pub fn total_pages(&self) -> usize {
        self.total_pages
    }

    /// Returns the total number of items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::from_items(247, 25);
    /// assert_eq!(state.total_items(), 247);
    /// ```
    pub fn total_items(&self) -> usize {
        self.total_items
    }

    /// Returns the page size (items per page).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::from_items(100, 20);
    /// assert_eq!(state.page_size(), 20);
    /// ```
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// Returns the display style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{PaginatorState, PaginatorStyle};
    ///
    /// let state = PaginatorState::new(5).with_style(PaginatorStyle::Compact);
    /// assert_eq!(state.style(), &PaginatorStyle::Compact);
    /// ```
    pub fn style(&self) -> &PaginatorStyle {
        &self.style
    }

    /// Returns true if currently on the first page.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::new(5);
    /// assert!(state.is_first_page());
    /// ```
    pub fn is_first_page(&self) -> bool {
        self.current_page == 0
    }

    /// Returns true if currently on the last page.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::new(5).with_current_page(4);
    /// assert!(state.is_last_page());
    /// ```
    pub fn is_last_page(&self) -> bool {
        self.current_page >= self.total_pages.saturating_sub(1)
    }

    /// Returns the index of the first item on the current page (0-indexed).
    ///
    /// For `RangeOfTotal` display: the start of the current page's item range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::from_items(247, 25).with_current_page(2);
    /// assert_eq!(state.range_start(), 50);
    /// ```
    pub fn range_start(&self) -> usize {
        self.current_page * self.page_size
    }

    /// Returns the index of the last item on the current page (0-indexed, inclusive).
    ///
    /// Clamped to total items minus one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let state = PaginatorState::from_items(247, 25).with_current_page(2);
    /// assert_eq!(state.range_end(), 74);
    ///
    /// // Last page may have fewer items
    /// let state = PaginatorState::from_items(247, 25).with_current_page(9);
    /// assert_eq!(state.range_end(), 246);
    /// ```
    pub fn range_end(&self) -> usize {
        let end = (self.current_page + 1) * self.page_size;
        end.min(self.total_items).saturating_sub(1)
    }

    // ---- Mutators ----

    /// Sets the current page, clamped to the valid range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let mut state = PaginatorState::new(5);
    /// state.set_current_page(3);
    /// assert_eq!(state.current_page(), 3);
    ///
    /// state.set_current_page(100);
    /// assert_eq!(state.current_page(), 4);
    /// ```
    pub fn set_current_page(&mut self, page: usize) {
        self.current_page = page.min(self.total_pages.saturating_sub(1));
    }

    /// Sets the total number of pages.
    ///
    /// Clamps the current page if it would exceed the new total.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let mut state = PaginatorState::new(10).with_current_page(8);
    /// state.set_total_pages(5);
    /// assert_eq!(state.total_pages(), 5);
    /// assert_eq!(state.current_page(), 4); // Clamped
    /// ```
    pub fn set_total_pages(&mut self, total: usize) {
        self.total_pages = total.max(1);
        self.total_items = self.total_pages * self.page_size;
        self.current_page = self.current_page.min(self.total_pages.saturating_sub(1));
    }

    /// Sets the total number of items and recalculates total pages.
    ///
    /// Clamps the current page if it would exceed the new total.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::PaginatorState;
    ///
    /// let mut state = PaginatorState::from_items(100, 10).with_current_page(5);
    /// state.set_total_items(30);
    /// assert_eq!(state.total_pages(), 3);
    /// assert_eq!(state.current_page(), 2); // Clamped
    /// ```
    pub fn set_total_items(&mut self, total: usize) {
        self.total_items = total;
        self.total_pages = calculate_total_pages(total, self.page_size);
        self.current_page = self.current_page.min(self.total_pages.saturating_sub(1));
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{PaginatorMessage, PaginatorOutput, PaginatorState};
    ///
    /// let mut state = PaginatorState::new(5);
    /// let output = state.update(PaginatorMessage::NextPage);
    /// assert_eq!(output, Some(PaginatorOutput::PageChanged(1)));
    /// ```
    pub fn update(&mut self, msg: PaginatorMessage) -> Option<PaginatorOutput> {
        Paginator::update(self, msg)
    }
}

/// A page navigation indicator component.
///
/// `Paginator` displays the current page position within a paginated
/// dataset. It supports four display styles and keyboard navigation.
///
/// # Key Bindings
///
/// - `Left` / `h` -- Previous page
/// - `Right` / `l` -- Next page
/// - `Home` -- First page
/// - `End` -- Last page
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, Paginator, PaginatorMessage, PaginatorState};
///
/// let mut state = PaginatorState::new(10);
/// Paginator::update(&mut state, PaginatorMessage::NextPage);
/// assert_eq!(state.current_page(), 1);
/// ```
pub struct Paginator;

impl Component for Paginator {
    type State = PaginatorState;
    type Message = PaginatorMessage;
    type Output = PaginatorOutput;

    fn init() -> Self::State {
        PaginatorState::default()
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
            Key::Left | Key::Char('h') => Some(PaginatorMessage::PrevPage),
            Key::Right | Key::Char('l') => Some(PaginatorMessage::NextPage),
            Key::Home => Some(PaginatorMessage::FirstPage),
            Key::End => Some(PaginatorMessage::LastPage),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            PaginatorMessage::NextPage => {
                if state.current_page < state.total_pages.saturating_sub(1) {
                    state.current_page += 1;
                    Some(PaginatorOutput::PageChanged(state.current_page))
                } else {
                    None
                }
            }
            PaginatorMessage::PrevPage => {
                if state.current_page > 0 {
                    state.current_page -= 1;
                    Some(PaginatorOutput::PageChanged(state.current_page))
                } else {
                    None
                }
            }
            PaginatorMessage::FirstPage => {
                if state.current_page != 0 {
                    state.current_page = 0;
                    Some(PaginatorOutput::PageChanged(0))
                } else {
                    None
                }
            }
            PaginatorMessage::LastPage => {
                let last = state.total_pages.saturating_sub(1);
                if state.current_page != last {
                    state.current_page = last;
                    Some(PaginatorOutput::PageChanged(last))
                } else {
                    None
                }
            }
            PaginatorMessage::GoToPage(page) => {
                let clamped = page.min(state.total_pages.saturating_sub(1));
                if state.current_page != clamped {
                    state.current_page = clamped;
                    Some(PaginatorOutput::PageChanged(clamped))
                } else {
                    None
                }
            }
            PaginatorMessage::SetTotalPages(total) => {
                state.set_total_pages(total);
                None
            }
            PaginatorMessage::SetTotalItems(total) => {
                state.set_total_items(total);
                None
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::paginator("paginator")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled)
                    .with_value(format!("{}/{}", state.display_page(), state.total_pages)),
            );
        });

        let text_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else if ctx.focused {
            ctx.theme.focused_style()
        } else {
            ctx.theme.normal_style()
        };

        let content = match &state.style {
            PaginatorStyle::PageOfTotal => {
                format!("Page {} of {}", state.display_page(), state.total_pages)
            }
            PaginatorStyle::RangeOfTotal => {
                if state.total_items == 0 {
                    "Showing 0 of 0".to_string()
                } else {
                    let start = state.range_start() + 1; // 1-indexed for display
                    let end = state.range_end() + 1; // 1-indexed for display
                    format!(
                        "Showing {}-{} of {}",
                        format_number(start),
                        format_number(end),
                        format_number(state.total_items),
                    )
                }
            }
            PaginatorStyle::Dots => render_dots(state),
            PaginatorStyle::Compact => render_compact(state, ctx.theme),
        };

        // For Compact style, we need special span-based rendering for arrow dimming
        if state.style == PaginatorStyle::Compact {
            let spans = render_compact_spans(state, ctx.theme, &ctx.event_context());
            let line = Line::from(spans);
            let paragraph = Paragraph::new(line).alignment(Alignment::Center);
            ctx.frame.render_widget(paragraph, ctx.area);
        } else {
            let paragraph = Paragraph::new(content)
                .style(text_style)
                .alignment(Alignment::Center);
            ctx.frame.render_widget(paragraph, ctx.area);
        }
    }
}

/// Renders the dot indicators string.
fn render_dots(state: &PaginatorState) -> String {
    const FILLED: char = '●';
    const EMPTY: char = '○';
    const MAX_DOTS: usize = 10;

    if state.total_pages <= MAX_DOTS {
        // Show all dots
        (0..state.total_pages)
            .map(|i| {
                if i == state.current_page {
                    FILLED
                } else {
                    EMPTY
                }
            })
            .collect::<Vec<_>>()
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        // Too many pages for dots; show window around current page with ellipsis
        // Show: first ... window ... last
        let mut parts: Vec<String> = Vec::new();

        // Always show first dot
        parts.push(if state.current_page == 0 {
            FILLED.to_string()
        } else {
            EMPTY.to_string()
        });

        // Calculate window around current page
        // We have budget for MAX_DOTS - 2 (first and last) = 8 positions
        // But we may need 1-2 for ellipses, leaving 6-7 for the window
        let window_size = MAX_DOTS - 2; // positions between first and last

        if state.current_page <= window_size / 2 + 1 {
            // Current page is near the start
            for i in 1..window_size {
                parts.push(if i == state.current_page {
                    FILLED.to_string()
                } else {
                    EMPTY.to_string()
                });
            }
            parts.push("…".to_string());
        } else if state.current_page >= state.total_pages - 1 - window_size / 2 {
            // Current page is near the end
            parts.push("…".to_string());
            let start = state.total_pages - window_size;
            for i in start..state.total_pages - 1 {
                parts.push(if i == state.current_page {
                    FILLED.to_string()
                } else {
                    EMPTY.to_string()
                });
            }
        } else {
            // Current page is in the middle
            parts.push("…".to_string());
            let half = (window_size - 3) / 2; // 3 = ellipsis before + current + ellipsis after
            let start = state.current_page - half;
            let end = state.current_page + half;
            for i in start..=end {
                parts.push(if i == state.current_page {
                    FILLED.to_string()
                } else {
                    EMPTY.to_string()
                });
            }
            parts.push("…".to_string());
        }

        // Always show last dot
        parts.push(if state.current_page == state.total_pages - 1 {
            FILLED.to_string()
        } else {
            EMPTY.to_string()
        });

        parts.join(" ")
    }
}

/// Renders the compact style text (for annotation/non-span display).
fn render_compact(state: &PaginatorState, _theme: &Theme) -> String {
    let left = if state.is_first_page() { " " } else { "◀" };
    let right = if state.is_last_page() { " " } else { "▶" };
    format!(
        "{} {} / {} {}",
        left,
        state.display_page(),
        state.total_pages,
        right
    )
}

/// Renders the compact style with styled spans for arrow dimming.
fn render_compact_spans<'a>(
    state: &PaginatorState,
    theme: &Theme,
    ctx: &EventContext,
) -> Vec<Span<'a>> {
    let text_style = if ctx.disabled {
        theme.disabled_style()
    } else if ctx.focused {
        theme.focused_style()
    } else {
        theme.normal_style()
    };

    let dim_style = if ctx.disabled {
        theme.disabled_style()
    } else {
        theme.border_style()
    };

    let left_arrow = if state.is_first_page() {
        Span::styled(" ", dim_style)
    } else {
        Span::styled("◀", text_style)
    };

    let right_arrow = if state.is_last_page() {
        Span::styled(" ", dim_style)
    } else {
        Span::styled("▶", text_style)
    };

    let page_text = format!(" {} / {} ", state.display_page(), state.total_pages);

    vec![left_arrow, Span::styled(page_text, text_style), right_arrow]
}

/// Formats a number with thousands separators.
fn format_number(n: usize) -> String {
    let s = n.to_string();
    if s.len() <= 3 {
        return s;
    }

    let mut result = String::with_capacity(s.len() + s.len() / 3);
    let chars: Vec<char> = s.chars().collect();
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    result
}

/// Calculates total pages from total items and page size.
fn calculate_total_pages(total_items: usize, page_size: usize) -> usize {
    if total_items == 0 {
        return 1;
    }
    let page_size = page_size.max(1);
    total_items.div_ceil(page_size)
}

#[cfg(test)]
mod tests;
