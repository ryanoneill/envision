//! Rendering helpers for the ConversationView component.

use super::types::{ConversationMessage, MessageBlock};
use super::ConversationViewState;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::theme::Theme;

/// Renders the full conversation view into the given area.
pub(super) fn render(state: &ConversationViewState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let border_style = if state.disabled {
        theme.disabled_style()
    } else if state.focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let title = state.title.as_deref().unwrap_or("Conversation");
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    render_messages(state, frame, inner, theme);
}

/// Renders the message list inside the content area.
fn render_messages(state: &ConversationViewState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let display_lines = build_display_lines(state, area.width as usize, theme);

    let total_lines = display_lines.len();
    let visible_lines = area.height as usize;

    let line_offset = if state.auto_scroll {
        total_lines.saturating_sub(visible_lines)
    } else {
        let estimated_line = state.scroll.offset().saturating_mul(3);
        estimated_line.min(total_lines.saturating_sub(visible_lines))
    };

    let items: Vec<ListItem> = display_lines
        .into_iter()
        .skip(line_offset)
        .take(visible_lines)
        .map(ListItem::new)
        .collect();

    let list = List::new(items);
    frame.render_widget(list, area);

    // Render scrollbar when content exceeds viewport
    if total_lines > visible_lines {
        let outer_area = Rect {
            x: area.x.saturating_sub(1),
            y: area.y.saturating_sub(1),
            width: area.width + 2,
            height: area.height + 2,
        };
        let mut bar_scroll = crate::scroll::ScrollState::new(total_lines);
        bar_scroll.set_viewport_height(visible_lines);
        bar_scroll.set_offset(line_offset);
        crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, outer_area, theme);
    }
}

/// Builds all display lines from the message list.
fn build_display_lines<'a>(
    state: &ConversationViewState,
    width: usize,
    _theme: &Theme,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    for (i, msg) in state.messages.iter().enumerate() {
        if i > 0 {
            // Separator between messages
            lines.push(Line::from(""));
        }
        format_message(msg, state, width, &mut lines);
    }

    lines
}

/// Formats a single conversation message into display lines.
fn format_message<'a>(
    msg: &ConversationMessage,
    state: &ConversationViewState,
    width: usize,
    lines: &mut Vec<Line<'a>>,
) {
    let role = msg.role();
    let role_color = role.color();
    let role_style = Style::default().fg(role_color);
    let bold_role_style = role_style.add_modifier(Modifier::BOLD);

    // Role header
    if state.show_role_labels {
        let mut header_spans = Vec::new();

        if state.show_timestamps {
            if let Some(ts) = msg.timestamp() {
                header_spans.push(Span::styled(
                    format!("[{}] ", ts),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }

        header_spans.push(Span::styled(
            format!("{} {}", role.indicator(), role.label()),
            bold_role_style,
        ));

        if msg.is_streaming() {
            header_spans.push(Span::styled(
                " \u{2588}",
                Style::default()
                    .fg(role_color)
                    .add_modifier(Modifier::SLOW_BLINK),
            ));
        }

        lines.push(Line::from(header_spans));
    }

    // Content blocks
    for block in msg.blocks() {
        format_block(block, state, width, role_style, lines);
    }
}

/// Formats a single message block into display lines.
fn format_block<'a>(
    block: &MessageBlock,
    state: &ConversationViewState,
    width: usize,
    role_style: Style,
    lines: &mut Vec<Line<'a>>,
) {
    match block {
        MessageBlock::Text(text) => {
            format_text_block(text, role_style, lines);
        }
        MessageBlock::Code { code, language } => {
            format_code_block(code, language.as_deref(), width, lines);
        }
        MessageBlock::ToolUse { name, input } => {
            format_tool_use_block(name, input, state, lines);
        }
        MessageBlock::Thinking(content) => {
            format_thinking_block(content, state, lines);
        }
        MessageBlock::Error(content) => {
            format_error_block(content, lines);
        }
    }
}

/// Formats a plain text block.
fn format_text_block<'a>(text: &str, style: Style, lines: &mut Vec<Line<'a>>) {
    if text.is_empty() {
        lines.push(Line::from(Span::styled("  ", style)));
        return;
    }
    for line in text.lines() {
        lines.push(Line::from(Span::styled(format!("  {}", line), style)));
    }
}

/// Formats a code block with a left border.
fn format_code_block<'a>(
    code: &str,
    language: Option<&str>,
    _width: usize,
    lines: &mut Vec<Line<'a>>,
) {
    let code_style = Style::default().fg(Color::White);
    let border_style = Style::default().fg(Color::DarkGray);

    // Header line with language
    let lang_label = language.unwrap_or("code");
    lines.push(Line::from(vec![
        Span::styled("  \u{2502} ", border_style),
        Span::styled(
            lang_label.to_string(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
        ),
    ]));

    // Code lines
    for line in code.lines() {
        lines.push(Line::from(vec![
            Span::styled("  \u{2502} ", border_style),
            Span::styled(line.to_string(), code_style),
        ]));
    }

    if code.is_empty() {
        lines.push(Line::from(vec![Span::styled("  \u{2502} ", border_style)]));
    }
}

/// Formats a tool use block (collapsible).
fn format_tool_use_block<'a>(
    name: &str,
    input: &str,
    state: &ConversationViewState,
    lines: &mut Vec<Line<'a>>,
) {
    let tool_style = Style::default().fg(Color::Yellow);
    let dim_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::DIM);

    let block_key = format!("tool:{}", name);
    let collapsed = state.collapsed_blocks.contains(&block_key);
    let toggle_char = if collapsed { "\u{25b8}" } else { "\u{25be}" };

    lines.push(Line::from(vec![
        Span::styled(format!("  {} ", toggle_char), dim_style),
        Span::styled(
            format!("Tool: {}", name),
            tool_style.add_modifier(Modifier::BOLD),
        ),
    ]));

    if !collapsed {
        for line in input.lines() {
            lines.push(Line::from(Span::styled(format!("    {}", line), dim_style)));
        }
        if input.is_empty() {
            lines.push(Line::from(Span::styled("    (no input)", dim_style)));
        }
    }
}

/// Formats a thinking block (collapsible).
fn format_thinking_block<'a>(
    content: &str,
    state: &ConversationViewState,
    lines: &mut Vec<Line<'a>>,
) {
    let thinking_style = Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::DIM);
    let header_style = Style::default()
        .fg(Color::Magenta)
        .add_modifier(Modifier::ITALIC);

    let collapsed = state.collapsed_blocks.contains("thinking");
    let toggle_char = if collapsed { "\u{25b8}" } else { "\u{25be}" };

    lines.push(Line::from(vec![
        Span::styled(format!("  {} ", toggle_char), thinking_style),
        Span::styled("Thinking...", header_style),
    ]));

    if !collapsed {
        for line in content.lines() {
            lines.push(Line::from(Span::styled(
                format!("    {}", line),
                thinking_style,
            )));
        }
    }
}

/// Formats an error block.
fn format_error_block<'a>(content: &str, lines: &mut Vec<Line<'a>>) {
    let error_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

    lines.push(Line::from(Span::styled(
        format!("  \u{2716} Error: {}", content),
        error_style,
    )));
}

/// Calculates the total number of display lines for the current message list.
///
/// Used internally to set scroll content length.
pub(super) fn total_display_lines(state: &ConversationViewState) -> usize {
    let theme = Theme::default();
    let lines = build_display_lines(state, 80, &theme);
    lines.len()
}
