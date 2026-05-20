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
//!         StyledInline::bold("world".to_string()),
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
///         StyledInline::bold("ready".to_string()),
///     ];
///     styled_line(frame, area, &inlines, theme);
/// }
/// ```
pub fn styled_line(frame: &mut Frame, area: Rect, inlines: &[StyledInline], theme: &Theme) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::styled_text::StyledInline;
    use crate::component::test_utils::setup_render;
    use ratatui::style::Color;

    #[test]
    fn test_styled_line_renders_inlines() {
        // Plain + Bold + Colored — all three appear in the rendered text in order.
        let inlines = vec![
            StyledInline::Plain("hello ".to_string()),
            StyledInline::bold("bold".to_string()),
            StyledInline::Plain(" world".to_string()),
        ];

        let (mut terminal, theme) = setup_render(40, 1);
        terminal
            .draw(|frame| {
                styled_line(frame, frame.area(), &inlines, &theme);
            })
            .unwrap();

        let plain = terminal.backend().to_string();
        insta::assert_snapshot!(plain);
    }

    #[test]
    fn test_styled_line_applies_theme_color() {
        // StyledInline::colored carries an explicit fg color. ANSI red = \x1b[31m.
        let inlines = vec![StyledInline::colored("err".to_string(), Color::Red)];

        let (mut terminal, theme) = setup_render(20, 1);
        terminal
            .draw(|frame| {
                styled_line(frame, frame.area(), &inlines, &theme);
            })
            .unwrap();

        let ansi = terminal.backend().to_ansi();
        assert!(
            ansi.contains("\x1b[31m"),
            "expected red (31m) ANSI fg for Colored(Red) inline, got:\n{ansi}",
        );
    }

    #[test]
    fn test_styled_line_truncates_to_area_width() {
        // 60 chars of plain text rendered into a 20-wide area — truncates
        // per ratatui's Paragraph default. Snapshot pins truncation behavior.
        let inlines = vec![StyledInline::Plain(
            "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWX".to_string(),
        )];

        let (mut terminal, theme) = setup_render(20, 1);
        terminal
            .draw(|frame| {
                styled_line(frame, frame.area(), &inlines, &theme);
            })
            .unwrap();

        let plain = terminal.backend().to_string();
        insta::assert_snapshot!(plain);
    }

    #[test]
    fn test_styled_line_empty_inlines_renders_nothing() {
        // Empty slice — primitive returns without panicking; buffer stays empty.
        let inlines: &[StyledInline] = &[];

        let (mut terminal, theme) = setup_render(40, 1);
        terminal
            .draw(|frame| {
                styled_line(frame, frame.area(), inlines, &theme);
            })
            .unwrap();

        let plain = terminal.backend().to_string();
        insta::assert_snapshot!(plain);
    }

    #[test]
    fn test_styled_line_no_chrome_drawn() {
        // Render into a 40x3 area but only the first row should have content;
        // rows 2 and 3 stay blank. Pins that the primitive draws no chrome /
        // no border / no fill in unused rows.
        let inlines = vec![StyledInline::Plain("only first row".to_string())];

        let (mut terminal, theme) = setup_render(40, 3);
        terminal
            .draw(|frame| {
                // Use the full frame area (40x3). styled_line renders into
                // row 0 only; rows 1 and 2 must stay blank.
                styled_line(frame, frame.area(), &inlines, &theme);
            })
            .unwrap();

        let plain = terminal.backend().to_string();
        // Snapshot is INFORMATIONAL: insta trims trailing whitespace and
        // newlines when serializing terminal.to_string() output, so the
        // snapshot won't visibly distinguish "blank rows" from "no rows at
        // all." The row-content `assert!` calls below are the load-bearing
        // chrome-no-draw assertions — they walk the raw lines() output and
        // verify every char in rows 1 and 2 is a space, which catches any
        // chrome / border / fill regression that the trimmed snapshot would
        // silently mask.
        insta::assert_snapshot!(plain);

        // Load-bearing chrome assertions: verify rows 1 and 2 are entirely
        // spaces (no chrome glyphs).
        let rows: Vec<&str> = plain.lines().collect();
        assert!(
            rows.len() >= 3,
            "expected 3 rows, got {}: {plain}",
            rows.len()
        );
        let row1_blank = rows[1].chars().all(|c| c == ' ');
        let row2_blank = rows[2].chars().all(|c| c == ' ');
        assert!(
            row1_blank,
            "row 1 should be blank (no chrome), got: {:?}",
            rows[1]
        );
        assert!(
            row2_blank,
            "row 2 should be blank (no chrome), got: {:?}",
            rows[2]
        );
    }
}
