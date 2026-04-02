//! Rendering helpers for the Treemap component.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::layout::{squarified_layout, LayoutRect};
use super::TreemapState;
use crate::theme::Theme;

/// Renders the treemap inside the border area.
pub(super) fn render_treemap(
    state: &TreemapState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    let view_node = state.current_view_node();
    let children = match view_node {
        Some(node) => &node.children,
        None => return,
    };

    if children.is_empty() {
        // Leaf node -- render as a single colored rectangle.
        if let Some(node) = view_node {
            render_leaf(
                node.label.as_str(),
                node.value,
                node.color,
                state,
                frame,
                area,
                disabled,
            );
        }
        return;
    }

    // Reserve bottom row for detail bar.
    let (grid_area, detail_area) = if area.height > 2 {
        let grid = Rect::new(area.x, area.y, area.width, area.height - 1);
        let detail = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        (grid, Some(detail))
    } else {
        (area, None)
    };

    // Compute layout.
    let rects = squarified_layout(children, grid_area, 0, &[]);

    // Determine which node index is selected.
    let selected_index = state.selected_child_index();

    // Render each rectangle.
    for rect in &rects {
        let is_selected = focused && !disabled && rect.node_index.first() == Some(&selected_index);

        render_rect(rect, is_selected, state, frame, disabled);
    }

    // Render detail bar.
    if let Some(detail) = detail_area {
        render_detail_bar(state, frame, detail, theme, disabled);
    }
}

/// Render a single layout rectangle.
fn render_rect(
    rect: &LayoutRect,
    is_selected: bool,
    state: &TreemapState,
    frame: &mut Frame,
    disabled: bool,
) {
    let cell_area = Rect::new(rect.x, rect.y, rect.width, rect.height);

    if cell_area.width == 0 || cell_area.height == 0 {
        return;
    }

    let bg = if disabled {
        Color::DarkGray
    } else {
        rect.color
    };

    let style = if is_selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        let fg = contrasting_fg(bg);
        Style::default().bg(bg).fg(fg)
    };

    // Build the cell content.
    let label = if state.show_labels && rect.width >= 3 && rect.height >= 1 {
        truncate_label(&rect.label, rect.width as usize)
    } else {
        " ".repeat(rect.width as usize)
    };

    let value_str = if state.show_values && rect.height >= 2 && rect.width >= 3 {
        Some(format_value(rect.value, rect.width as usize))
    } else {
        None
    };

    // First line: label.
    let p = Paragraph::new(label)
        .style(style)
        .alignment(Alignment::Center);
    let first_line = Rect::new(cell_area.x, cell_area.y, cell_area.width, 1);
    frame.render_widget(p, first_line);

    // Fill remaining lines with background color.
    for row in 1..cell_area.height {
        let line_area = Rect::new(cell_area.x, cell_area.y + row, cell_area.width, 1);
        let text = if row == 1 {
            if let Some(ref vs) = value_str {
                vs.clone()
            } else {
                " ".repeat(cell_area.width as usize)
            }
        } else {
            " ".repeat(cell_area.width as usize)
        };
        let p = Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center);
        frame.render_widget(p, line_area);
    }
}

/// Render a leaf node as a single colored rectangle.
fn render_leaf(
    label: &str,
    value: f64,
    color: Color,
    state: &TreemapState,
    frame: &mut Frame,
    area: Rect,

    disabled: bool,
) {
    let bg = if disabled { Color::DarkGray } else { color };
    let fg = contrasting_fg(bg);
    let style = Style::default().bg(bg).fg(fg);

    let text = if state.show_labels {
        truncate_label(label, area.width as usize)
    } else {
        " ".repeat(area.width as usize)
    };

    let p = Paragraph::new(text)
        .style(style)
        .alignment(Alignment::Center);
    frame.render_widget(p, Rect::new(area.x, area.y, area.width, 1.min(area.height)));

    // Fill remaining rows.
    for row in 1..area.height {
        let line_area = Rect::new(area.x, area.y + row, area.width, 1);
        let text = if row == 1 && state.show_values {
            format_value(value, area.width as usize)
        } else {
            " ".repeat(area.width as usize)
        };
        let p = Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center);
        frame.render_widget(p, line_area);
    }
}

/// Render the detail bar at the bottom.
fn render_detail_bar(
    state: &TreemapState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    disabled: bool,
) {
    let style = if disabled {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };

    let text = if let Some(node) = state.selected_node() {
        let total = state
            .current_view_node()
            .map(|n| n.total_value())
            .unwrap_or(0.0);
        let pct = if total > 0.0 {
            node.total_value() / total * 100.0
        } else {
            0.0
        };
        format!(
            "Selected: {} \u{2014} {:.1} ({:.1}%)",
            node.label,
            node.total_value(),
            pct
        )
    } else {
        String::new()
    };

    let truncated = if text.len() > area.width as usize {
        text[..area.width as usize].to_string()
    } else {
        text
    };

    let p = Paragraph::new(truncated).style(style);
    frame.render_widget(p, area);
}

/// Truncate a label to fit within the given width.
fn truncate_label(label: &str, max_width: usize) -> String {
    if label.len() <= max_width {
        label.to_string()
    } else if max_width > 2 {
        format!("{}..", &label[..max_width - 2])
    } else if max_width > 0 {
        label[..max_width].to_string()
    } else {
        String::new()
    }
}

/// Format a value to fit within the given width.
fn format_value(value: f64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let formatted = if width >= 6 {
        format!("{value:.1}")
    } else if width >= 3 {
        format!("{value:.0}")
    } else {
        let s = format!("{value:.0}");
        s[..s.len().min(width)].to_string()
    };

    if formatted.len() <= width {
        formatted
    } else {
        formatted[..width].to_string()
    }
}

/// Returns a contrasting foreground color for readability.
fn contrasting_fg(bg: Color) -> Color {
    match bg {
        Color::Rgb(r, g, b) => {
            let luminance = 0.299 * (r as f64) + 0.587 * (g as f64) + 0.114 * (b as f64);
            if luminance > 128.0 {
                Color::Black
            } else {
                Color::White
            }
        }
        Color::DarkGray | Color::Black => Color::White,
        Color::White
        | Color::Yellow
        | Color::LightYellow
        | Color::LightGreen
        | Color::LightCyan => Color::Black,
        _ => Color::White,
    }
}
