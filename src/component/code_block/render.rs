//! Rendering logic for the CodeBlock component.
//!
//! Provides bordered code display with optional line number gutter,
//! syntax-highlighted lines, highlight-line backgrounds, and a vertical
//! scrollbar when content exceeds the viewport.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::CodeBlockState;
use super::highlight::highlight_line;
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Width of the line-number gutter (digits + separator).
///
/// Format: `NNNNN | ` where NNNNN is right-aligned.
const GUTTER_WIDTH: u16 = 7;

/// Renders the CodeBlock in the given area.
#[allow(clippy::too_many_lines)]
pub(super) fn render(
    state: &CodeBlockState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::custom("CodeBlock", "code_block")
                .with_focus(focused)
                .with_disabled(disabled),
        );
    });

    let border_style = if disabled {
        theme.disabled_style()
    } else if focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let title = build_title(state);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let lines: Vec<&str> = state.code.lines().collect();
    let total_lines = lines.len().max(1);
    let visible = inner.height as usize;
    let max_scroll = total_lines.saturating_sub(visible);
    let scroll_offset = state.scroll.offset().min(max_scroll);

    let gutter_width = if state.show_line_numbers {
        GUTTER_WIDTH
    } else {
        0
    };

    let code_area_width = inner.width.saturating_sub(gutter_width);

    // Render each visible line
    let end = (scroll_offset + visible).min(total_lines);
    for (row_idx, line_idx) in (scroll_offset..end).enumerate() {
        let y = inner.y + row_idx as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let line_num = line_idx + 1; // 1-based line numbers
        let line_text = if line_idx < lines.len() {
            lines[line_idx]
        } else {
            ""
        };

        let is_highlighted = state.highlight_lines.contains(&line_num);

        // Render line number gutter
        if state.show_line_numbers {
            let gutter_area = Rect::new(inner.x, y, gutter_width, 1);
            render_gutter(
                line_num,
                is_highlighted,
                state,
                frame,
                gutter_area,
                theme,
                disabled,
            );
        }

        // Render code content with horizontal scroll offset
        let code_x = inner.x + gutter_width;
        let code_line_area = Rect::new(code_x, y, code_area_width, 1);
        render_code_line(
            line_text,
            is_highlighted,
            state,
            frame,
            code_line_area,
            theme,
            disabled,
            state.horizontal_offset,
        );
    }

    // Render scrollbar when content exceeds viewport
    if total_lines > visible {
        let mut bar_scroll = ScrollState::new(total_lines);
        bar_scroll.set_viewport_height(visible);
        bar_scroll.set_offset(scroll_offset);
        crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
    }
}

/// Builds the title string for the border.
fn build_title(state: &CodeBlockState) -> String {
    match (&state.title, &state.language) {
        (Some(title), _) => format!(" {} ", title),
        (None, lang) if *lang != super::highlight::Language::Plain => {
            format!(" {} ", lang.name())
        }
        _ => String::new(),
    }
}

/// Renders the line-number gutter for a single row.
fn render_gutter(
    line_num: usize,
    is_highlighted: bool,
    _state: &CodeBlockState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,

    disabled: bool,
) {
    let gutter_style = if disabled {
        theme.disabled_style()
    } else if is_highlighted {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let gutter_text = format!("{:>5} ", line_num);
    let paragraph = Paragraph::new(gutter_text).style(gutter_style);
    frame.render_widget(paragraph, area);
}

/// Applies a horizontal offset to a list of styled spans, skipping
/// the first `offset` characters while preserving styling.
fn apply_horizontal_offset<'a>(spans: Vec<Span<'a>>, offset: usize) -> Vec<Span<'a>> {
    if offset == 0 {
        return spans;
    }

    let mut remaining = offset;
    let mut result = Vec::new();

    for span in spans {
        let len = span.content.len();
        if remaining >= len {
            // Skip this entire span
            remaining -= len;
            continue;
        }
        if remaining > 0 {
            // Partially skip this span
            let trimmed: String = span.content.chars().skip(remaining).collect();
            result.push(Span::styled(trimmed, span.style));
            remaining = 0;
        } else {
            result.push(span);
        }
    }

    result
}

/// Renders a single line of highlighted code.
#[allow(clippy::too_many_arguments)]
fn render_code_line(
    line_text: &str,
    is_highlighted: bool,
    state: &CodeBlockState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    disabled: bool,
    horizontal_offset: usize,
) {
    if disabled {
        let visible: String = line_text.chars().skip(horizontal_offset).collect();
        let paragraph = Paragraph::new(visible).style(theme.disabled_style());
        frame.render_widget(paragraph, area);
        return;
    }

    if is_highlighted {
        // Highlighted lines get a distinct background
        let hl_bg = Color::Rgb(50, 50, 20);
        let spans = highlight_line(line_text, &state.language);
        let shifted = apply_horizontal_offset(spans, horizontal_offset);
        let styled_spans: Vec<Span<'_>> = shifted
            .into_iter()
            .map(|s| {
                let mut style = s.style;
                style = style.bg(hl_bg);
                Span::styled(s.content.to_string(), style)
            })
            .collect();

        // Pad remaining width with highlight background
        let text_width: usize = styled_spans.iter().map(|s| s.content.len()).sum();
        let mut all_spans = styled_spans;
        if (text_width as u16) < area.width {
            let pad = " ".repeat((area.width as usize).saturating_sub(text_width));
            all_spans.push(Span::styled(pad, Style::default().bg(hl_bg)));
        }

        let line = Line::from(all_spans);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    } else {
        let spans = highlight_line(line_text, &state.language);
        let shifted = apply_horizontal_offset(spans, horizontal_offset);
        let line = Line::from(shifted);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }
}
