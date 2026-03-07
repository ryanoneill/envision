//! Rendering helper functions for ChatView.

use super::{ChatMessage, ChatRole, ChatViewState, Focus};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::layout::Position;
use crate::theme::Theme;

/// Renders the message history area.
pub(super) fn render_history(state: &ChatViewState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let border_style = if state.disabled {
        theme.disabled_style()
    } else if state.focused && state.focus == Focus::History {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Build display lines from messages
    let display_lines: Vec<(Line, ChatRole)> = state
        .messages
        .iter()
        .flat_map(|msg| {
            let base_style = state.role_style(&msg.role());
            format_message(msg, state.show_timestamps, inner.width as usize, base_style)
        })
        .collect();

    let total_lines = display_lines.len();
    let visible_lines = inner.height as usize;

    // Calculate scroll offset based on message-level scroll
    // We'll show from offset based on display lines
    let line_offset = if state.auto_scroll {
        total_lines.saturating_sub(visible_lines)
    } else {
        // Approximate: each message is roughly 2+ lines
        let estimated_line = state.scroll_offset.saturating_mul(2);
        estimated_line.min(total_lines.saturating_sub(visible_lines))
    };

    let items: Vec<ListItem> = display_lines
        .into_iter()
        .skip(line_offset)
        .take(visible_lines)
        .map(|(line, _)| ListItem::new(line))
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Formats a chat message into display lines.
pub(super) fn format_message(
    msg: &ChatMessage,
    show_timestamps: bool,
    _width: usize,
    base_style: Style,
) -> Vec<(Line<'_>, ChatRole)> {
    let mut result = Vec::new();
    let role = msg.role();
    let style = base_style;
    let bold_style = style.add_modifier(Modifier::BOLD);

    // Header line: [timestamp] Username:
    let mut header_spans = Vec::new();

    if show_timestamps {
        if let Some(ts) = msg.timestamp() {
            header_spans.push(Span::styled(
                format!("[{}] ", ts),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    header_spans.push(Span::styled(format!("{}:", msg.display_name()), bold_style));

    result.push((Line::from(header_spans), role));

    // Content lines
    for line in msg.content().lines() {
        result.push((Line::from(Span::styled(format!("  {}", line), style)), role));
    }

    // Handle empty content
    if msg.content().is_empty() {
        result.push((Line::from(Span::styled("  ", style)), role));
    }

    result
}

/// Renders the input area.
pub(super) fn render_input(state: &ChatViewState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let border_style = if state.disabled {
        theme.disabled_style()
    } else if state.focused && state.focus == Focus::Input {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let text_style = if state.disabled {
        theme.disabled_style()
    } else if state.focused && state.focus == Focus::Input {
        theme.focused_style()
    } else {
        theme.normal_style()
    };

    let value = state.input.value();
    let display_text = if value.is_empty() && !state.input.placeholder().is_empty() {
        state.input.placeholder().to_string()
    } else {
        value
    };

    let text_style_final = if state.input.is_empty() && !state.input.placeholder().is_empty() {
        theme.placeholder_style()
    } else {
        text_style
    };

    let paragraph = Paragraph::new(display_text)
        .style(text_style_final)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // Show cursor when input is focused
    if state.focused && state.focus == Focus::Input && !state.disabled {
        let (cursor_row, cursor_col) = state.input.cursor_display_position();
        let cursor_x = area.x + 1 + cursor_col as u16;
        let cursor_y = area.y + 1 + cursor_row as u16;
        if cursor_x < area.right() - 1 && cursor_y < area.bottom() - 1 {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }
}
