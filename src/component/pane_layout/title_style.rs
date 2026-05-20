//! `PaneConfig::with_title_style` builder + getter (G4).
//!
//! Houses the title-style accessors in a separate file to keep
//! `mod.rs` under the 1000-line cap. The field declaration itself
//! lives on the `PaneConfig` struct in `mod.rs`; this module only
//! contains the inherent `impl` for accessor methods.
//!
//! See [`PaneConfig::with_title_style`] for the full contract.

use ratatui::style::Style;

use super::PaneConfig;

impl PaneConfig {
    /// Sets the title style (builder pattern).
    ///
    /// When `Some(style)`, the pane title renders with the given style instead
    /// of inheriting the border style. When `None` (default), title styling
    /// follows the border (current behavior).
    ///
    /// # Focus invariance
    ///
    /// `title_style` is focus-invariant: when set, it applies whether the pane
    /// is focused, unfocused, or disabled. The render arm consults `title_style`
    /// unconditionally — consumer-set styles aren't silently overridden by focus
    /// state. If a future use case needs focused-vs-unfocused title styling, that
    /// would be a separate builder (`with_focused_title_style`, etc.), not a
    /// surprise in the existing one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    /// use ratatui::style::{Color, Modifier, Style};
    ///
    /// let pane = PaneConfig::new("brand")
    ///     .with_title("leadline")
    ///     .with_title_style(
    ///         Style::default()
    ///             .fg(Color::Magenta)
    ///             .add_modifier(Modifier::BOLD),
    ///     );
    /// assert!(pane.title_style().is_some());
    /// ```
    pub fn with_title_style(mut self, style: Style) -> Self {
        self.title_style = Some(style);
        self
    }

    /// Returns the title style, if explicitly set.
    ///
    /// `None` means the title inherits the border style (default behavior).
    pub fn title_style(&self) -> Option<Style> {
        self.title_style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn with_title_style_sets_field() {
        let style = Style::default().fg(Color::Magenta);
        let pane = PaneConfig::new("p").with_title("t").with_title_style(style);
        assert_eq!(pane.title_style(), Some(style));
    }

    #[test]
    fn title_style_default_none() {
        let pane = PaneConfig::new("p");
        assert_eq!(pane.title_style(), None);
    }

    #[test]
    fn snapshot_pane_with_branded_title_style() {
        use crate::component::pane_layout::{PaneDirection, PaneLayout, PaneLayoutState};
        use crate::component::test_utils::setup_render;
        use crate::component::RenderContext;
        use ratatui::style::Modifier;

        // 2-pane horizontal: left pane has a magenta+bold title; right is plain.
        let panes = vec![
            PaneConfig::new("left")
                .with_title("Brand")
                .with_title_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
                .with_proportion(0.5),
            PaneConfig::new("right").with_title("Plain").with_proportion(0.5),
        ];
        let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);

        let (mut terminal, theme) = setup_render(40, 5);
        terminal
            .draw(|frame| {
                PaneLayout::view_with(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme),
                    |_, _| {},
                );
            })
            .unwrap();

        let plain = terminal.backend().to_string();
        let ansi = terminal.backend().to_ansi();

        // Magenta fg = \x1b[35m; BOLD = \x1b[1m. Branded title carries both.
        assert!(
            ansi.contains("\x1b[35m"),
            "expected magenta (35m) for branded title, got:\n{ansi}",
        );
        assert!(
            ansi.contains("\x1b[1m"),
            "expected BOLD (1m) for branded title, got:\n{ansi}",
        );

        insta::assert_snapshot!(plain);
    }

    #[test]
    fn snapshot_pane_title_style_focus_invariant() {
        use crate::component::pane_layout::{PaneDirection, PaneLayout, PaneLayoutState};
        use crate::component::test_utils::setup_render;
        use crate::component::RenderContext;

        // 2-pane horizontal: BOTH panes get the same with_title_style(magenta).
        // One pane is focused, the other isn't. The title rendering must be
        // identical regardless of focus — title_style wins over focus-driven
        // border style adjustments.
        let style = Style::default().fg(Color::Magenta);
        let panes = vec![
            PaneConfig::new("focused")
                .with_title("F")
                .with_title_style(style)
                .with_proportion(0.5),
            PaneConfig::new("unfocused")
                .with_title("U")
                .with_title_style(style)
                .with_proportion(0.5),
        ];
        let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
        // Focus the left pane. If `focused_pane` is a public field, set it
        // directly; if there's a public setter, use it; if neither, the state
        // defaults to first-pane-focused which is fine for this test.

        let (mut terminal, theme) = setup_render(40, 5);
        terminal
            .draw(|frame| {
                let mut ctx = RenderContext::new(frame, frame.area(), &theme);
                ctx.focused = true;
                PaneLayout::view_with(&state, &mut ctx, |_, _| {});
            })
            .unwrap();

        let ansi = terminal.backend().to_ansi();
        let plain = terminal.backend().to_string();

        // The magenta escape \x1b[35m must appear at least twice in the ANSI
        // output — once per title. If focus-driven border-style adjustments
        // were inadvertently overriding title_style, the two titles would
        // render differently and only one would carry magenta.
        //
        // We assert >=2 rather than ==2 to stay robust against future
        // ratatui versions that might emit redundant SGR escapes between
        // consecutive same-style runs (which could inflate the count without
        // changing visible output). The contract is "magenta appears on both
        // titles," not "exactly two SGR escapes."
        let magenta_count = ansi.matches("\x1b[35m").count();
        assert!(
            magenta_count >= 2,
            "expected magenta (35m) at least twice (once per title regardless of focus), got {magenta_count} occurrences:\n{ansi}",
        );

        insta::assert_snapshot!(plain);
    }
}
