//! View rendering helpers for LineInput.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::LineInputState;
use super::chunking::{chunk_buffer, cursor_to_visual};
use crate::component::RenderContext;

/// Renders the LineInput component.
pub(super) fn render(state: &LineInputState, ctx: &mut RenderContext<'_, '_>) {
    crate::annotation::with_registry(|reg| {
        reg.register(
            ctx.area,
            crate::annotation::Annotation::line_input("line_input")
                .with_value(state.value())
                .with_focus(ctx.focused)
                .with_disabled(ctx.disabled),
        );
    });

    let border_style = if ctx.focused {
        ctx.theme.focused_border_style()
    } else {
        ctx.theme.border_style()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    // Inner ctx.area (inside borders)
    let inner = block.inner(ctx.area);
    if inner.width == 0 || inner.height == 0 {
        ctx.frame.render_widget(block, ctx.area);
        return;
    }

    let width = inner.width as usize;
    let is_placeholder = state.buffer.is_empty();

    let base_style = if ctx.disabled {
        ctx.theme.disabled_style()
    } else if ctx.focused {
        ctx.theme.focused_style()
    } else if is_placeholder {
        ctx.theme.placeholder_style()
    } else {
        ctx.theme.normal_style()
    };

    let display_text = if is_placeholder {
        &state.placeholder
    } else {
        &state.buffer
    };

    // Build visual lines from chunks
    let chunks = chunk_buffer(display_text, width);
    let selection_range = if !is_placeholder {
        state.selection_range()
    } else {
        None
    };

    let mut lines: Vec<Line> = Vec::with_capacity(chunks.len());
    for chunk in &chunks {
        let chunk_text = &display_text[chunk.clone()];
        if let Some((sel_start, sel_end)) = selection_range {
            // Compute overlap between chunk range and selection range
            let overlap_start = sel_start.max(chunk.start);
            let overlap_end = sel_end.min(chunk.end);
            if overlap_start < overlap_end {
                // There is selected text in this chunk
                let before = &display_text[chunk.start..overlap_start];
                let selected = &display_text[overlap_start..overlap_end];
                let after = &display_text[overlap_end..chunk.end];
                let mut spans = Vec::new();
                if !before.is_empty() {
                    spans.push(Span::styled(before.to_string(), base_style));
                }
                spans.push(Span::styled(
                    selected.to_string(),
                    ctx.theme.selection_style(),
                ));
                if !after.is_empty() {
                    spans.push(Span::styled(after.to_string(), base_style));
                }
                lines.push(Line::from(spans));
            } else {
                lines.push(Line::styled(chunk_text.to_string(), base_style));
            }
        } else {
            lines.push(Line::styled(chunk_text.to_string(), base_style));
        }
    }

    let paragraph = Paragraph::new(Text::from(lines)).block(block);
    ctx.frame.render_widget(paragraph, ctx.area);

    // Set cursor position when focused
    if ctx.focused && !ctx.disabled && inner.width > 0 && inner.height > 0 {
        let (cursor_row, cursor_col) = cursor_to_visual(&state.buffer, state.cursor, width);

        let cursor_x = inner.x + cursor_col as u16;
        let cursor_y = inner.y + cursor_row as u16;

        if cursor_x < inner.x + inner.width && cursor_y < inner.y + inner.height {
            ctx.frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
