//! Rendering helpers for the ConversationView component.

use super::types::{ConversationMessage, MessageBlock};
use super::{ConversationViewState, MessageSource};

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

use crate::theme::Theme;

/// Renders the full conversation view using the messages stored in state.
pub(super) fn render(
    state: &ConversationViewState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    render_from(state, state, frame, area, theme, focused, disabled);
}

/// Renders the conversation view using messages from an external [`MessageSource`].
pub(super) fn render_from(
    source: &dyn MessageSource,
    state: &ConversationViewState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    let border_style = if disabled {
        theme.disabled_style()
    } else if focused {
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

    let (message_area, status_area) = if state.status.is_some() && inner.height > 1 {
        let msg = Rect {
            height: inner.height - 1,
            ..inner
        };
        let status = Rect {
            y: inner.y + inner.height - 1,
            height: 1,
            ..inner
        };
        (msg, Some(status))
    } else {
        (inner, None)
    };

    render_messages_from(source, state, frame, message_area, theme);

    if let Some((status_rect, text)) = status_area.zip(state.status.as_deref()) {
        let style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC);
        let line = Line::from(Span::styled(text, style));
        let paragraph = ratatui::widgets::Paragraph::new(line);
        frame.render_widget(paragraph, status_rect);
    }
}

fn render_messages_from(
    source: &dyn MessageSource,
    state: &ConversationViewState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let messages = source.source_messages();

    let preliminary_lines = build_display_lines(messages, state, area.width as usize, theme);
    let needs_scrollbar = preliminary_lines.len() > area.height as usize;
    let content_width = if needs_scrollbar {
        (area.width as usize).saturating_sub(1)
    } else {
        area.width as usize
    };

    let display_lines = if needs_scrollbar {
        build_display_lines(messages, state, content_width, theme)
    } else {
        preliminary_lines
    };

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

fn build_display_lines<'a>(
    messages: &[ConversationMessage],
    state: &ConversationViewState,
    width: usize,
    _theme: &Theme,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    for (i, msg) in messages.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
        }
        format_message(msg, state, width, &mut lines);
    }

    lines
}

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

    let indent = if state.show_role_labels { "  " } else { "" };
    for block in msg.blocks() {
        format_block(block, state, width, indent, role_style, lines);
    }
}

fn format_block<'a>(
    block: &MessageBlock,
    state: &ConversationViewState,
    width: usize,
    indent: &str,
    role_style: Style,
    lines: &mut Vec<Line<'a>>,
) {
    match block {
        MessageBlock::Text(text) => {
            format_text_block(
                text,
                width,
                indent,
                role_style,
                state.markdown_enabled,
                lines,
            );
        }
        MessageBlock::Code { code, language } => {
            format_code_block(code, language.as_deref(), width, indent, lines);
        }
        MessageBlock::ToolUse {
            name,
            input,
            output,
        } => {
            format_tool_use_block(
                name,
                input.as_deref(),
                output.as_deref(),
                width,
                indent,
                state,
                lines,
            );
        }
        MessageBlock::Thinking(content) => {
            format_thinking_block(content, width, indent, state, lines);
        }
        MessageBlock::Error(content) => {
            format_error_block(content, width, indent, lines);
        }
    }
}

fn wrap_lines(text: &str, prefix: &str, width: usize) -> Vec<String> {
    let prefix_width = UnicodeWidthStr::width(prefix);
    let effective_width = width.saturating_sub(prefix_width);
    if effective_width == 0 {
        return text.lines().map(|l| format!("{}{}", prefix, l)).collect();
    }

    let mut result = Vec::new();
    for line in text.lines() {
        let line_width = UnicodeWidthStr::width(line);
        if line_width <= effective_width {
            result.push(format!("{}{}", prefix, line));
        } else {
            let mut remaining = line;
            while !remaining.is_empty() {
                let rem_width = UnicodeWidthStr::width(remaining);
                if rem_width <= effective_width {
                    result.push(format!("{}{}", prefix, remaining));
                    break;
                }
                let mut col = 0;
                let mut last_space_byte = None;
                let mut byte_at_width = remaining.len();
                for (byte_idx, ch) in remaining.char_indices() {
                    let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
                    if col + ch_w > effective_width {
                        byte_at_width = byte_idx;
                        break;
                    }
                    if ch == ' ' {
                        last_space_byte = Some(byte_idx + 1);
                    }
                    col += ch_w;
                }
                let break_at = last_space_byte.unwrap_or(byte_at_width);
                let segment = &remaining[..break_at];
                result.push(format!("{}{}", prefix, segment.trim_end()));
                remaining = &remaining[break_at..];
                if remaining.starts_with(' ') {
                    remaining = &remaining[1..];
                }
            }
        }
    }
    if result.is_empty() {
        result.push(prefix.to_string());
    }
    result
}

fn format_text_block<'a>(
    text: &str,
    width: usize,
    indent: &str,
    style: Style,
    markdown_enabled: bool,
    lines: &mut Vec<Line<'a>>,
) {
    if text.is_empty() {
        lines.push(Line::from(Span::styled(indent.to_string(), style)));
        return;
    }

    #[cfg(feature = "markdown")]
    if markdown_enabled {
        let theme = crate::theme::Theme::default();
        let indent_display_width = UnicodeWidthStr::width(indent);
        let available_width = width.saturating_sub(indent_display_width);
        let md_lines = crate::component::markdown_renderer::render::render_markdown(
            text,
            available_width as u16,
            &theme,
        );
        for md_line in md_lines {
            if indent.is_empty() {
                lines.push(md_line);
            } else {
                let mut spans: Vec<Span> = vec![Span::raw(indent.to_string())];
                spans.extend(md_line.spans);
                lines.push(Line::from(spans));
            }
        }
        return;
    }

    let _ = markdown_enabled;

    for wrapped in wrap_lines(text, indent, width) {
        lines.push(Line::from(Span::styled(wrapped, style)));
    }
}

fn format_code_block<'a>(
    code: &str,
    language: Option<&str>,
    _width: usize,
    indent: &str,
    lines: &mut Vec<Line<'a>>,
) {
    let code_style = Style::default().fg(Color::White);
    let border_style = Style::default().fg(Color::DarkGray);

    let lang_label = language.unwrap_or("code");
    lines.push(Line::from(vec![
        Span::styled(format!("{}\u{2502} ", indent), border_style),
        Span::styled(
            lang_label.to_string(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
        ),
    ]));

    let code_prefix = format!("{}\u{2502} ", indent);
    for line in code.lines() {
        lines.push(Line::from(vec![
            Span::styled(code_prefix.clone(), border_style),
            Span::styled(line.to_string(), code_style),
        ]));
    }

    if code.is_empty() {
        lines.push(Line::from(vec![Span::styled(code_prefix, border_style)]));
    }
}

fn format_tool_use_block<'a>(
    name: &str,
    input: Option<&str>,
    output: Option<&str>,
    width: usize,
    indent: &str,
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

    let inner_indent = format!("{}  ", indent);
    lines.push(Line::from(vec![
        Span::styled(format!("{}{} ", indent, toggle_char), dim_style),
        Span::styled(
            format!("Tool: {}", name),
            tool_style.add_modifier(Modifier::BOLD),
        ),
    ]));

    if !collapsed {
        match input {
            Some(text) if !text.is_empty() => {
                for wrapped in wrap_lines(text, &inner_indent, width) {
                    lines.push(Line::from(Span::styled(wrapped, dim_style)));
                }
            }
            _ => {
                lines.push(Line::from(Span::styled(
                    format!("{}(no input)", inner_indent),
                    dim_style,
                )));
            }
        }
        if let Some(out) = output {
            let output_style = Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::DIM | Modifier::ITALIC);
            let out_prefix = format!("{}-> ", inner_indent);
            for wrapped in wrap_lines(out, &out_prefix, width) {
                lines.push(Line::from(Span::styled(wrapped, output_style)));
            }
        }
    }
}

fn format_thinking_block<'a>(
    content: &str,
    width: usize,
    indent: &str,
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

    let inner_indent = format!("{}  ", indent);
    lines.push(Line::from(vec![
        Span::styled(format!("{}{} ", indent, toggle_char), thinking_style),
        Span::styled("Thinking...", header_style),
    ]));

    if !collapsed {
        for wrapped in wrap_lines(content, &inner_indent, width) {
            lines.push(Line::from(Span::styled(wrapped, thinking_style)));
        }
    }
}

fn format_error_block<'a>(content: &str, width: usize, indent: &str, lines: &mut Vec<Line<'a>>) {
    let error_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

    let prefix = format!("{}\u{2716} Error: ", indent);
    for wrapped in wrap_lines(content, &prefix, width) {
        lines.push(Line::from(Span::styled(wrapped, error_style)));
    }
}

/// Calculates the total number of display lines for the current message list.
pub(super) fn total_display_lines(state: &ConversationViewState, width: usize) -> usize {
    let theme = Theme::default();
    let messages = state.source_messages();
    let lines = build_display_lines(messages, state, width, &theme);
    lines.len()
}
