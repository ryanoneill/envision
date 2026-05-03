//! Closure-based per-pane child rendering.
//!
//! The `view_with` method on [`PaneLayout`] draws pane chrome (borders,
//! titles, focus rings) and invokes the consumer's render closure once
//! per pane with a chrome-inset child [`RenderContext`] whose
//! `chrome_owned` flag is set to `true` so embedded components suppress
//! their own chrome.
//!
//! See [`PaneLayout::view_with`] for the public API.

use ratatui::layout::Margin;
use ratatui::widgets::{Block, Borders};

use super::{PaneLayout, PaneLayoutState};
use crate::component::RenderContext;

/// Draws pane chrome (border + title + focus ring) for every pane and
/// invokes `render_child(pane_id, &mut child_ctx)` per pane.
///
/// Shared implementation for both [`PaneLayout::view_with`] and the
/// `Component::view` trait impl. The trait impl passes a no-op closure
/// to get the chrome-only behavior.
fn render_panes<F>(state: &PaneLayoutState, ctx: &mut RenderContext<'_, '_>, mut render_child: F)
where
    F: FnMut(&str, &mut RenderContext<'_, '_>),
{
    crate::annotation::with_registry(|reg| {
        reg.register(
            ctx.area,
            crate::annotation::Annotation::new(crate::annotation::WidgetType::PaneLayout)
                .with_id("pane_layout")
                .with_focus(ctx.focused)
                .with_disabled(ctx.disabled),
        );
    });

    let rects = state.layout(ctx.area);

    for (i, (pane, rect)) in state.panes.iter().zip(rects.iter()).enumerate() {
        let is_focused_pane = ctx.focused && i == state.focused_pane;
        let border_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else if is_focused_pane {
            ctx.theme.focused_border_style()
        } else {
            ctx.theme.border_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(title) = &pane.title {
            block = block.title(format!(" {} ", title));
        }

        ctx.frame.render_widget(block, *rect);

        let inner = rect.inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        let mut child_ctx = ctx.with_area(inner);
        child_ctx.focused = is_focused_pane;
        child_ctx.chrome_owned = true;

        render_child(pane.id.as_str(), &mut child_ctx);
    }
}

/// Renders pane chrome only (no children). Used by the `Component::view`
/// trait impl on [`PaneLayout`] so generic-`Component` callers get the
/// chrome-only path.
pub(super) fn render_chrome_only(state: &PaneLayoutState, ctx: &mut RenderContext<'_, '_>) {
    render_panes(state, ctx, |_, _| {});
}

impl PaneLayout {
    /// Render pane chrome and invoke `render_child` once per pane.
    ///
    /// `render_child` receives the pane's id (string slice) and a child
    /// [`RenderContext`] whose:
    /// - `area` is the pane's inner rect (chrome already accounted for)
    /// - `chrome_owned` is `true` (children suppress their own chrome)
    /// - `focused` is `true` only for the focused pane's child context
    /// - `disabled` is propagated from the parent context
    /// - `frame` and `theme` are propagated unchanged
    ///
    /// # Common pitfall
    ///
    /// Do **not** construct a fresh `RenderContext::new(frame, area, theme)`
    /// inside the closure body. Doing so bypasses `chrome_owned = true`
    /// and re-introduces the double-render bug this method exists to
    /// solve. Always pass the provided `child_ctx` (or a reborrow of it
    /// via `child_ctx.with_area(...)`) to embedded components.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use envision::component::{
    ///     PaneLayout, PaneLayoutState, RenderContext,
    ///     pane_layout::{PaneConfig, PaneDirection},
    /// };
    /// use envision::theme::Theme;
    /// use ratatui::Frame;
    /// use ratatui::prelude::Rect;
    ///
    /// fn render(frame: &mut Frame, area: Rect, theme: &Theme) {
    ///     let panes = vec![
    ///         PaneConfig::new("left").with_title("Left").with_proportion(0.5),
    ///         PaneConfig::new("right").with_title("Right").with_proportion(0.5),
    ///     ];
    ///     let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
    ///     PaneLayout::view_with(
    ///         &state,
    ///         &mut RenderContext::new(frame, area, theme),
    ///         |pane_id, _child_ctx| match pane_id {
    ///             "left" => { /* render left child into child_ctx */ }
    ///             "right" => { /* render right child into child_ctx */ }
    ///             _ => {}
    ///         },
    ///     );
    /// }
    /// ```
    pub fn view_with<F>(state: &PaneLayoutState, ctx: &mut RenderContext<'_, '_>, render_child: F)
    where
        F: FnMut(&str, &mut RenderContext<'_, '_>),
    {
        render_panes(state, ctx, render_child);
    }
}
