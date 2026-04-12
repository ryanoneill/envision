//! Rendering helpers for the [`Tree`] component.
//!
//! This module contains the functions that translate [`TreeState`] into
//! ratatui widgets and render them to the terminal frame.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::component::{EventContext, RenderContext};
use crate::scroll::ScrollState;
use crate::theme::Theme;

use super::{Tree, TreeState};

impl<T: Clone + 'static> Tree<T> {
    /// Renders the tree to a list of styled lines.
    pub(super) fn render_lines(
        state: &TreeState<T>,
        width: u16,
        theme: &Theme,
        ctx: &EventContext,
    ) -> Vec<Line<'static>> {
        let flat = state.flatten();
        let mut lines = Vec::with_capacity(flat.len());

        // Pre-compute indent strings to avoid per-node allocations.
        let max_depth = flat.iter().map(|n| n.depth).max().unwrap_or(0);
        let indents: Vec<String> = (0..=max_depth).map(|d| "  ".repeat(d)).collect();

        // Pre-compute styles to avoid per-node method calls.
        let normal_style = theme.normal_style();
        let disabled_style = if ctx.disabled {
            Some(theme.disabled_style())
        } else {
            None
        };
        let highlight_style = if !ctx.disabled {
            Some(theme.selected_highlight_style(ctx.focused))
        } else {
            None
        };

        // Reusable buffer to avoid per-node String allocations.
        let mut buf = String::with_capacity(width as usize);

        for (idx, node) in flat.iter().enumerate() {
            let is_selected = state.selected_index == Some(idx);

            let indicator = if node.has_children {
                if node.is_expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            // Build padded line text in a reusable buffer.
            buf.clear();
            buf.push_str(&indents[node.depth]);
            buf.push_str(indicator);
            buf.push_str(&node.label);
            let pad = (width as usize).saturating_sub(buf.len());
            for _ in 0..pad {
                buf.push(' ');
            }

            let style = if let Some(ds) = disabled_style {
                ds
            } else if is_selected {
                highlight_style.unwrap()
            } else {
                normal_style
            };

            lines.push(Line::from(Span::styled(buf.clone(), style)));
        }

        lines
    }
}

/// Renders the tree view into `frame` within `area`.
pub(super) fn view<T: Clone + 'static>(state: &TreeState<T>, ctx: &mut RenderContext<'_, '_>) {
    let event_ctx = ctx.event_context();
    let all_lines = Tree::render_lines(state, ctx.area.width, ctx.theme, &event_ctx);
    let viewport_height = ctx.area.height as usize;

    // Use a local ScrollState for scrollbar rendering and virtual scrolling
    let mut bar_scroll = ScrollState::new(all_lines.len());
    bar_scroll.set_viewport_height(viewport_height);
    if let Some(idx) = state.selected_index {
        bar_scroll.ensure_visible(idx);
    }

    let range = bar_scroll.visible_range();
    let visible_lines: Vec<Line<'static>> = all_lines
        .into_iter()
        .skip(range.start)
        .take(range.len())
        .collect();

    let text = Text::from(visible_lines);
    let paragraph = Paragraph::new(text);

    let annotation = crate::annotation::Annotation::new(crate::annotation::WidgetType::Tree)
        .with_id("tree")
        .with_focus(ctx.focused)
        .with_disabled(ctx.disabled);
    let annotated = crate::annotation::Annotate::new(paragraph, annotation);
    ctx.frame.render_widget(annotated, ctx.area);

    // Render scrollbar if content exceeds viewport
    crate::scroll::render_scrollbar(&bar_scroll, ctx.frame, ctx.area, ctx.theme);
}
