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
pub use ratatui::layout::{
    Alignment, Constraint, Direction, Layout, Margin, Position, Rect, Size,
};
pub use ratatui::Frame;
pub use ratatui::Terminal;

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

#[cfg(test)]
mod tests;
