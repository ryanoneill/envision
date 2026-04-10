//! Layout utilities and re-exports.
//!
//! This module re-exports ratatui's layout types and provides convenience
//! functions for common layout patterns. Using these re-exports, applications
//! can write `use envision::layout::*` instead of `use ratatui::layout::*`.
//!
//! # Layout Helpers
//!
//! The [`vertical`] and [`horizontal`] functions make splitting areas concise:
//!
//! ```rust
//! use envision::layout::{vertical, horizontal, Rect, Constraint};
//!
//! let area = Rect::new(0, 0, 80, 24);
//!
//! // Split vertically into 3 rows
//! let [header, body, footer] = vertical(area, [
//!     Constraint::Length(1),
//!     Constraint::Min(0),
//!     Constraint::Length(1),
//! ]);
//!
//! // Split the body horizontally into 2 columns
//! let [left, right] = horizontal(body, [
//!     Constraint::Percentage(30),
//!     Constraint::Percentage(70),
//! ]);
//! ```
//!
//! # Centering
//!
//! The [`centered`] and [`centered_percent`] functions create centered sub-areas:
//!
//! ```rust
//! use envision::layout::{centered, centered_percent, Rect};
//!
//! let area = Rect::new(0, 0, 80, 24);
//!
//! // Center a 40x10 box
//! let dialog = centered(area, 40, 10);
//!
//! // Center at 60% width and 50% height
//! let panel = centered_percent(area, 60, 50);
//! ```

// Re-export ratatui layout types
pub use ratatui::Frame;
pub use ratatui::Terminal;
pub use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Position, Rect, Size};

/// Splits an area vertically (top to bottom) into N parts.
///
/// This is a convenience wrapper around [`Layout`] with [`Direction::Vertical`].
///
/// # Example
///
/// ```rust
/// use envision::layout::{vertical, Rect, Constraint};
///
/// let area = Rect::new(0, 0, 80, 24);
/// let [header, body, footer] = vertical(area, [
///     Constraint::Length(3),
///     Constraint::Min(0),
///     Constraint::Length(1),
/// ]);
///
/// assert_eq!(header.height, 3);
/// assert_eq!(footer.height, 1);
/// assert_eq!(body.height, 20);
/// ```
pub fn vertical<const N: usize>(area: Rect, constraints: [Constraint; N]) -> [Rect; N] {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints);
    let chunks = layout.split(area);
    std::array::from_fn(|i| chunks[i])
}

/// Splits an area horizontally (left to right) into N parts.
///
/// This is a convenience wrapper around [`Layout`] with [`Direction::Horizontal`].
///
/// # Example
///
/// ```rust
/// use envision::layout::{horizontal, Rect, Constraint};
///
/// let area = Rect::new(0, 0, 80, 24);
/// let [sidebar, content] = horizontal(area, [
///     Constraint::Length(20),
///     Constraint::Min(0),
/// ]);
///
/// assert_eq!(sidebar.width, 20);
/// assert_eq!(content.width, 60);
/// ```
pub fn horizontal<const N: usize>(area: Rect, constraints: [Constraint; N]) -> [Rect; N] {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints);
    let chunks = layout.split(area);
    std::array::from_fn(|i| chunks[i])
}

/// Creates a centered sub-area with the given width and height.
///
/// If the requested dimensions exceed the available area, they are
/// clamped to fit.
///
/// # Example
///
/// ```rust
/// use envision::layout::{centered, Rect};
///
/// let area = Rect::new(0, 0, 80, 24);
/// let dialog = centered(area, 40, 10);
///
/// assert_eq!(dialog.width, 40);
/// assert_eq!(dialog.height, 10);
/// assert_eq!(dialog.x, 20); // (80 - 40) / 2
/// assert_eq!(dialog.y, 7);  // (24 - 10) / 2
/// ```
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// Creates a centered sub-area with the given percentage of width and height.
///
/// Percentages are clamped to 0-100.
///
/// # Example
///
/// ```rust
/// use envision::layout::{centered_percent, Rect};
///
/// let area = Rect::new(0, 0, 100, 50);
/// let panel = centered_percent(area, 60, 40);
///
/// assert_eq!(panel.width, 60);  // 60% of 100
/// assert_eq!(panel.height, 20); // 40% of 50
/// assert_eq!(panel.x, 20);      // (100 - 60) / 2
/// assert_eq!(panel.y, 15);      // (50 - 20) / 2
/// ```
pub fn centered_percent(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let w_pct = width_percent.min(100);
    let h_pct = height_percent.min(100);
    let width = (area.width as u32 * w_pct as u32 / 100) as u16;
    let height = (area.height as u32 * h_pct as u32 / 100) as u16;
    centered(area, width, height)
}

/// The regions produced by [`AppShell::split`].
///
/// All three fields are always present. When no header or footer is
/// configured on the [`AppShell`], the corresponding rect has zero
/// height.
///
/// # Example
///
/// ```rust
/// use envision::layout::{AppShell, AppRegions, Constraint, Rect};
///
/// let shell = AppShell::new()
///     .header(Constraint::Length(3))
///     .footer(Constraint::Length(1));
///
/// let regions = shell.split(Rect::new(0, 0, 80, 24));
/// assert_eq!(regions.header.height, 3);
/// assert_eq!(regions.content.height, 20);
/// assert_eq!(regions.footer.height, 1);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppRegions {
    /// The header area. Zero-height if no header was configured.
    pub header: Rect,
    /// The main content area.
    pub content: Rect,
    /// The footer area. Zero-height if no footer was configured.
    pub footer: Rect,
}

/// A reusable app-level layout definition for the canonical
/// (header, content, footer) split.
///
/// Construct once at app init with your header and/or footer constraints,
/// then call [`split`](Self::split) from views and overlays to get
/// consistent rects without duplicating layout constants.
///
/// # Example
///
/// ```rust
/// use envision::layout::{AppShell, Constraint, Rect};
///
/// // Define the layout once at app init.
/// let shell = AppShell::new()
///     .header(Constraint::Length(4))
///     .footer(Constraint::Length(1));
///
/// // Use it from both the main view and overlays.
/// let area = Rect::new(0, 0, 80, 24);
/// let regions = shell.split(area);
///
/// assert_eq!(regions.header.height, 4);
/// assert_eq!(regions.content.height, 19);
/// assert_eq!(regions.footer.height, 1);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AppShell {
    header: Option<Constraint>,
    footer: Option<Constraint>,
}

impl AppShell {
    /// Creates a new `AppShell` with no header or footer.
    ///
    /// Content will occupy the full area until a header or footer is
    /// configured via [`header`](Self::header) or
    /// [`footer`](Self::footer).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::layout::{AppShell, Rect};
    ///
    /// let shell = AppShell::new();
    /// let regions = shell.split(Rect::new(0, 0, 80, 24));
    /// assert_eq!(regions.content.height, 24);
    /// ```
    pub fn new() -> Self {
        Self {
            header: None,
            footer: None,
        }
    }

    /// Sets the header constraint (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::layout::{AppShell, Constraint, Rect};
    ///
    /// let shell = AppShell::new().header(Constraint::Length(3));
    /// let regions = shell.split(Rect::new(0, 0, 80, 24));
    /// assert_eq!(regions.header.height, 3);
    /// ```
    pub fn header(mut self, constraint: Constraint) -> Self {
        self.header = Some(constraint);
        self
    }

    /// Sets the footer constraint (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::layout::{AppShell, Constraint, Rect};
    ///
    /// let shell = AppShell::new().footer(Constraint::Length(1));
    /// let regions = shell.split(Rect::new(0, 0, 80, 24));
    /// assert_eq!(regions.footer.height, 1);
    /// ```
    pub fn footer(mut self, constraint: Constraint) -> Self {
        self.footer = Some(constraint);
        self
    }

    /// Splits the given area into header, content, and footer regions.
    ///
    /// Content always receives [`Constraint::Min(0)`] so it expands
    /// to fill whatever the header and footer don't consume.
    ///
    /// When no header or footer is configured, the corresponding region
    /// has zero height.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::layout::{AppShell, Constraint, Rect};
    ///
    /// let shell = AppShell::new()
    ///     .header(Constraint::Length(4))
    ///     .footer(Constraint::Length(1));
    /// let regions = shell.split(Rect::new(0, 0, 80, 24));
    ///
    /// assert_eq!(regions.header.height, 4);
    /// assert_eq!(regions.content.height, 19);
    /// assert_eq!(regions.footer.height, 1);
    /// ```
    pub fn split(self, area: Rect) -> AppRegions {
        let h = self.header.unwrap_or(Constraint::Length(0));
        let f = self.footer.unwrap_or(Constraint::Length(0));
        let [header, content, footer] = vertical(area, [h, Constraint::Min(0), f]);
        AppRegions {
            header,
            content,
            footer,
        }
    }
}

impl Default for AppShell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
