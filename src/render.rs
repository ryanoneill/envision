//! Standalone render primitives that don't belong to a single component.
//!
//! These functions take a `Frame`, a `Rect`, the data to render, and a `Theme`,
//! and produce immediate rendered output. They carry no state, draw no chrome,
//! and own no border — primitives consume an externally-owned area and produce
//! styled content directly into it.
//!
//! Compose freely with the chrome-ownership protocol: a primitive called inside
//! a chrome_owned context owns no chrome of its own, so there's nothing to
//! suppress.
//!
//! # Example
//!
//! ```rust,no_run
//! use envision::component::styled_text::StyledInline;
//! use envision::render::styled_line;
//! use envision::theme::Theme;
//! use ratatui::layout::Rect;
//!
//! fn render(frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
//!     let inlines = vec![
//!         StyledInline::Plain("Hello, ".to_string()),
//!         StyledInline::Bold("world".to_string()),
//!     ];
//!     styled_line(frame, area, &inlines, theme);
//! }
//! ```

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;

use crate::component::styled_text::{StyledContent, StyledInline};
use crate::theme::Theme;

/// Render a sequence of styled inline elements as a single line into `area`.
///
/// Equivalent to a borderless `StyledText` with one line of content, but with
/// no state plumbing — pass inlines + frame + area + theme, get rendered output.
///
/// Empty inlines render an empty buffer (no error, no panic). Inlines that
/// exceed the area width are truncated by the underlying `ratatui::Paragraph`
/// widget; no wrapping is applied (single-line semantics).
///
/// # Example
///
/// ```rust,no_run
/// use envision::component::styled_text::StyledInline;
/// use envision::render::styled_line;
/// use envision::theme::Theme;
/// use ratatui::layout::Rect;
///
/// fn render(frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
///     let inlines = vec![
///         StyledInline::Plain("status: ".to_string()),
///         StyledInline::Bold("ready".to_string()),
///     ];
///     styled_line(frame, area, &inlines, theme);
/// }
/// ```
pub fn styled_line(
    frame: &mut Frame,
    area: Rect,
    inlines: &[StyledInline],
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    // Build a one-block StyledContent and use the existing render path.
    // Reuses render_block -> render_line -> render_inline; no new rendering logic.
    let content = StyledContent::new().line(inlines.to_vec());
    let lines = content.render_lines(area.width, theme);
    let text = Text::from(lines);
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}
