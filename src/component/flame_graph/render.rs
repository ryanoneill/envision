//! Rendering functions for the FlameGraph component.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::node::FlameNode;
use super::FlameGraphState;
use crate::theme::Theme;

/// Renders the full flame graph including border, depth rows, and detail bar.
pub(super) fn render_flame_graph(
    state: &FlameGraphState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let border_style = if state.disabled {
        theme.disabled_style()
    } else if state.focused {
        theme.focused_border_style()
    } else {
        theme.normal_style()
    };

    let mut block = Block::default().borders(Borders::ALL).style(border_style);
    if let Some(title) = &state.title {
        block = block.title(format!(" {} ", title));
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
                "FlameGraph".to_string(),
            ))
            .with_id("flame_graph")
            .with_focus(state.focused)
            .with_disabled(state.disabled),
        );
    });

    let view_root = match state.current_view_root() {
        Some(r) => r,
        None => {
            // Render empty state
            let style = if state.disabled {
                theme.disabled_style()
            } else {
                theme.normal_style()
            };
            let msg = "No data";
            let centered_x = inner.x + inner.width.saturating_sub(msg.len() as u16) / 2;
            let centered_y = inner.y + inner.height / 2;
            let msg_area = Rect::new(centered_x, centered_y, msg.len() as u16, 1);
            frame.render_widget(Paragraph::new(msg).style(style), msg_area);
            return;
        }
    };

    // Reserve last 2 lines for separator and detail bar
    let detail_height: u16 = 2;
    let graph_height = inner.height.saturating_sub(detail_height) as usize;
    if graph_height == 0 {
        return;
    }

    let max_depth = state.max_depth();
    let root_total = view_root.total_value();

    // Render each depth level (root at top)
    for depth in 0..=max_depth {
        let y = inner.y + depth as u16;
        if depth >= graph_height {
            break;
        }
        let row_area = Rect::new(inner.x, y, inner.width, 1);
        render_depth_row(state, frame, view_root, root_total, depth, row_area, theme);
    }

    // Render separator and detail bar
    let sep_y = inner.y + inner.height.saturating_sub(detail_height);
    if sep_y < inner.bottom() {
        let sep_area = Rect::new(inner.x, sep_y, inner.width, 1);
        let sep_line: String = "\u{2500}".repeat(inner.width as usize);
        let sep_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.normal_style()
        };
        frame.render_widget(Paragraph::new(sep_line).style(sep_style), sep_area);
    }

    let detail_y = sep_y + 1;
    if detail_y < inner.bottom() {
        let detail_area = Rect::new(inner.x, detail_y, inner.width, 1);
        render_detail_bar(state, frame, root_total, detail_area, theme);
    }
}

/// Renders a single depth row of the flame graph.
fn render_depth_row(
    state: &FlameGraphState,
    frame: &mut Frame,
    view_root: &FlameNode,
    root_total: u64,
    depth: usize,
    area: Rect,
    theme: &Theme,
) {
    let width = area.width as usize;
    if width == 0 {
        return;
    }

    let frames = FlameGraphState::frames_at_depth(view_root, depth);
    if frames.is_empty() {
        return;
    }

    // Build spans for each frame at this depth
    let mut spans: Vec<Span<'_>> = Vec::new();
    let mut total_cols_used = 0;

    for (idx, node) in frames.iter().enumerate() {
        let frame_width = if root_total == 0 {
            width
        } else {
            let raw = (node.total_value() as f64 / root_total as f64 * width as f64) as usize;
            raw.max(1).min(width.saturating_sub(total_cols_used))
        };

        if frame_width == 0 || total_cols_used >= width {
            break;
        }

        let is_selected = state.selected_depth == depth && state.selected_index == idx;
        let matches_search = !state.search_query.is_empty()
            && node
                .label
                .to_lowercase()
                .contains(&state.search_query.to_lowercase());

        let style = compute_frame_style(state, node, is_selected, matches_search, theme);

        // Build the label, truncating or padding to fit
        let label = &node.label;
        let content = if label.len() <= frame_width {
            // Label fits: render label + fill remaining with block chars
            let label_len = label.len();
            let remaining = frame_width - label_len;
            if remaining > 1 {
                format!(" {}{}", label, "\u{2588}".repeat(remaining - 1))
            } else if remaining == 1 {
                format!(" {}", &label[..label_len.min(frame_width - 1)])
            } else {
                label[..frame_width].to_string()
            }
        } else if frame_width > 2 {
            // Label doesn't fit: truncate
            format!(" {}", &label[..frame_width - 1])
        } else {
            // Very narrow: just blocks
            "\u{2588}".repeat(frame_width)
        };

        // Ensure content is exactly frame_width characters
        let content_chars: Vec<char> = content.chars().collect();
        let final_content: String = if content_chars.len() >= frame_width {
            content_chars[..frame_width].iter().collect()
        } else {
            let pad = frame_width - content_chars.len();
            let mut s: String = content_chars.into_iter().collect();
            s.push_str(&"\u{2588}".repeat(pad));
            s
        };

        spans.push(Span::styled(final_content, style));
        total_cols_used += frame_width;
    }

    // Fill remaining space
    if total_cols_used < width {
        let remaining = width - total_cols_used;
        let bg_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.normal_style()
        };
        spans.push(Span::styled(" ".repeat(remaining), bg_style));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

/// Computes the style for a frame.
fn compute_frame_style(
    state: &FlameGraphState,
    node: &FlameNode,
    is_selected: bool,
    matches_search: bool,
    theme: &Theme,
) -> Style {
    if state.disabled {
        return theme.disabled_style();
    }

    if is_selected && state.focused {
        // Selected + focused: use highlight style with the frame's color
        Style::default()
            .fg(Color::Black)
            .bg(node.color)
            .add_modifier(Modifier::BOLD)
    } else if is_selected {
        // Selected but not focused
        Style::default().fg(Color::Black).bg(node.color)
    } else if matches_search {
        // Search match: bright yellow background
        Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        // Normal frame
        Style::default().fg(node.color).bg(Color::Reset)
    }
}

/// Renders the detail bar showing information about the selected frame.
fn render_detail_bar(
    state: &FlameGraphState,
    frame: &mut Frame,
    root_total: u64,
    area: Rect,
    theme: &Theme,
) {
    let style = if state.disabled {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };

    let detail_text = if let Some(selected) = state.selected_frame() {
        let percentage = if root_total > 0 {
            selected.total_value() as f64 / root_total as f64 * 100.0
        } else {
            0.0
        };
        format!(
            " Selected: {}  {} samples ({:.1}%)  self: {}",
            selected.label,
            selected.total_value(),
            percentage,
            selected.self_value(),
        )
    } else {
        " No selection".to_string()
    };

    // Truncate to fit area
    let chars: Vec<char> = detail_text.chars().collect();
    let truncated: String = chars.into_iter().take(area.width as usize).collect();
    frame.render_widget(Paragraph::new(truncated).style(style), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_frame_style_disabled() {
        let state = FlameGraphState::with_root(FlameNode::new("main()", 500)).with_disabled(true);
        let node = FlameNode::new("test()", 100);
        let theme = Theme::default();
        let style = compute_frame_style(&state, &node, false, false, &theme);
        assert_eq!(style, theme.disabled_style());
    }

    #[test]
    fn test_compute_frame_style_selected_focused() {
        let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
        state.set_focused(true);
        let node = FlameNode::new("test()", 100).with_color(Color::Red);
        let theme = Theme::default();
        let style = compute_frame_style(&state, &node, true, false, &theme);
        assert_eq!(style.fg, Some(Color::Black));
        assert_eq!(style.bg, Some(Color::Red));
    }

    #[test]
    fn test_compute_frame_style_search_match() {
        let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
        state.set_search("test".to_string());
        let node = FlameNode::new("test()", 100);
        let theme = Theme::default();
        let style = compute_frame_style(&state, &node, false, true, &theme);
        assert_eq!(style.bg, Some(Color::Yellow));
    }

    #[test]
    fn test_compute_frame_style_normal() {
        let state = FlameGraphState::with_root(FlameNode::new("main()", 500));
        let node = FlameNode::new("test()", 100).with_color(Color::Green);
        let theme = Theme::default();
        let style = compute_frame_style(&state, &node, false, false, &theme);
        assert_eq!(style.fg, Some(Color::Green));
    }
}
