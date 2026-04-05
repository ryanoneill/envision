//! Rendering logic for the TerminalOutput component.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::TerminalOutputState;
use super::ansi::parse_ansi;
use crate::theme::Theme;

/// Renders the terminal output component.
///
/// Layout (top to bottom):
/// - Border with optional title
/// - ANSI-colored content lines with optional line numbers
/// - Status bar at the bottom (inside the border)
/// - Scrollbar on the right edge (inside the border)
pub(super) fn render(
    state: &TerminalOutputState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::terminal_output("terminal_output")
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

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    if let Some(title) = &state.title {
        block = block.title(title.as_str());
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Reserve 1 row for the status bar
    let status_bar_height = 1u16;
    let content_height = inner.height.saturating_sub(status_bar_height);

    if content_height == 0 {
        // Only room for status bar
        let status_area = Rect::new(inner.x, inner.y, inner.width, inner.height.min(1));
        render_status_bar(state, frame, status_area, theme, disabled);
        return;
    }

    let content_area = Rect::new(inner.x, inner.y, inner.width, content_height);
    let status_area = Rect::new(
        inner.x,
        inner.y + content_height,
        inner.width,
        status_bar_height,
    );

    render_content(state, frame, content_area, theme, disabled);
    render_status_bar(state, frame, status_area, theme, disabled);

    // Render scrollbar when content exceeds viewport
    let total_lines = state.lines.len();
    let visible_lines = content_height as usize;
    if total_lines > visible_lines {
        let mut bar_scroll = crate::scroll::ScrollState::new(total_lines);
        bar_scroll.set_viewport_height(visible_lines);
        bar_scroll.set_offset(state.scroll.offset());
        crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
    }
}

/// Renders the ANSI-colored content lines.
fn render_content(
    state: &TerminalOutputState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    disabled: bool,
) {
    let visible_lines = area.height as usize;
    let offset = state.scroll.offset();

    let line_number_width = if state.show_line_numbers {
        // Calculate width for line numbers (at least 3 chars + separator)
        let max_line_num = state.lines.len();
        let digits = if max_line_num == 0 {
            1
        } else {
            max_line_num.to_string().len()
        };
        digits + 1 // digits + space separator
    } else {
        0
    };

    let text_style = if disabled {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };

    let line_num_style = if disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    for (i, line_idx) in (offset..state.lines.len().min(offset + visible_lines)).enumerate() {
        let y = area.y + i as u16;

        if y >= area.y + area.height {
            break;
        }

        let mut x = area.x;
        let max_x = area.x + area.width;

        // Render line number if enabled
        if state.show_line_numbers && line_number_width > 0 {
            let num_str = format!("{:>width$} ", line_idx + 1, width = line_number_width - 1);
            let num_width = (num_str.len() as u16).min(max_x.saturating_sub(x));
            if num_width > 0 {
                let span = Span::styled(&num_str[..num_width as usize], line_num_style);
                frame.render_widget(span, Rect::new(x, y, num_width, 1));
                x += num_width;
            }
        }

        // Render the ANSI-styled line content
        let line = &state.lines[line_idx];

        if disabled {
            // When disabled, render without ANSI colors
            let remaining = (max_x.saturating_sub(x)) as usize;
            let display: String = line.chars().take(remaining).collect();
            let truncated_len = display.len() as u16;
            if truncated_len > 0 {
                let span = Span::styled(display, text_style);
                frame.render_widget(span, Rect::new(x, y, truncated_len, 1));
            }
        } else {
            let segments = parse_ansi(line);
            for segment in &segments {
                if x >= max_x {
                    break;
                }
                let remaining = (max_x - x) as usize;
                let display: String = segment.text.chars().take(remaining).collect();
                let display_width = display.len() as u16;
                if display_width > 0 {
                    let effective_style = text_style.patch(segment.style);
                    let span = Span::styled(display, effective_style);
                    frame.render_widget(span, Rect::new(x, y, display_width, 1));
                    x += display_width;
                }
            }
        }
    }
}

/// Renders the status bar at the bottom of the component.
fn render_status_bar(
    state: &TerminalOutputState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    disabled: bool,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let status_style = if disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(Color::DarkGray).bg(Color::Black)
    };

    // Build status segments
    let mut parts: Vec<String> = Vec::new();

    // Running / exit code status
    if state.running {
        parts.push("Running".to_string());
    } else if let Some(code) = state.exit_code {
        if code == 0 {
            parts.push("Exit: 0".to_string());
        } else {
            parts.push(format!("Exit: {code}"));
        }
    }

    // Line count
    parts.push(format!("{} lines", state.lines.len()));

    // Auto-scroll indicator
    if state.auto_scroll {
        parts.push("Auto-scroll".to_string());
    }

    // Scroll position
    if !state.lines.is_empty() {
        let offset = state.scroll.offset();
        parts.push(format!("Ln {}", offset + 1));
    }

    let status_text = format!(" {}", parts.join(" | "));
    let truncated: String = status_text.chars().take(area.width as usize).collect();
    let span = Span::styled(truncated, status_style);
    frame.render_widget(span, Rect::new(area.x, area.y, area.width, 1));
}
