//! Rendering helpers for the Heatmap component.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{HeatmapState, value_to_color};
use crate::theme::Theme;

/// Renders the heatmap grid inside the border.
pub(super) fn render_heatmap(
    state: &HeatmapState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    let num_rows = state.rows();
    let num_cols = state.cols();

    if num_rows == 0 || num_cols == 0 {
        return;
    }

    // Calculate row label width
    let row_label_width: u16 = if state.row_labels().is_empty() {
        0
    } else {
        state
            .row_labels()
            .iter()
            .map(|l| l.len() as u16)
            .max()
            .unwrap_or(0)
            + 1 // +1 for padding
    };

    // Determine if we have column labels
    let col_label_height: u16 = if state.col_labels().is_empty() { 0 } else { 1 };

    // Available space for the grid
    let grid_x = area.x + row_label_width;
    let grid_y = area.y + col_label_height;
    let grid_width = area.width.saturating_sub(row_label_width);
    let grid_height = area.height.saturating_sub(col_label_height);

    if grid_width == 0 || grid_height == 0 {
        return;
    }

    // Calculate cell width (at least 1 char wide)
    let cell_width = (grid_width / num_cols as u16).max(1);
    // Each row is 1 line tall
    let cell_height: u16 = 1;

    let min_val = state.effective_min();
    let max_val = state.effective_max();

    // Render column labels
    if col_label_height > 0 {
        render_col_labels(state, frame, area, grid_x, cell_width, theme, disabled);
    }

    // Render row labels and cells
    for ri in 0..num_rows {
        let y = grid_y + (ri as u16) * cell_height;
        if y >= area.bottom() {
            break;
        }

        // Row label
        if !state.row_labels().is_empty() {
            render_row_label(
                state,
                frame,
                area.x,
                y,
                row_label_width,
                ri,
                theme,
                disabled,
            );
        }

        // Cells
        render_row_cells(
            state,
            frame,
            ri,
            grid_x,
            y,
            cell_width,
            cell_height,
            area,
            min_val,
            max_val,
            theme,
            focused,
            disabled,
        );
    }
}

/// Renders column labels across the top of the grid.
fn render_col_labels(
    state: &HeatmapState,
    frame: &mut Frame,
    area: Rect,
    grid_x: u16,
    cell_width: u16,
    theme: &Theme,
    disabled: bool,
) {
    for (ci, label) in state.col_labels().iter().enumerate() {
        let x = grid_x + (ci as u16) * cell_width;
        if x >= area.right() {
            break;
        }
        let available = cell_width.min(area.right().saturating_sub(x));
        if available == 0 {
            continue;
        }
        let label_area = Rect::new(x, area.y, available, 1);
        let truncated = truncate_str(label, available as usize);
        let style = if disabled {
            theme.disabled_style()
        } else {
            theme.normal_style().add_modifier(Modifier::BOLD)
        };
        let p = Paragraph::new(truncated)
            .style(style)
            .alignment(Alignment::Center);
        frame.render_widget(p, label_area);
    }
}

/// Renders a single row label.
#[allow(clippy::too_many_arguments)]
fn render_row_label(
    state: &HeatmapState,
    frame: &mut Frame,
    x: u16,
    y: u16,
    width: u16,
    row_index: usize,
    theme: &Theme,
    disabled: bool,
) {
    if let Some(label) = state.row_labels().get(row_index) {
        let label_area = Rect::new(x, y, width, 1);
        let truncated = truncate_str(label, width as usize);
        let style = if disabled {
            theme.disabled_style()
        } else {
            theme.normal_style()
        };
        let p = Paragraph::new(truncated).style(style);
        frame.render_widget(p, label_area);
    }
}

/// Renders all cells in a single row.
#[allow(clippy::too_many_arguments)]
fn render_row_cells(
    state: &HeatmapState,
    frame: &mut Frame,
    ri: usize,
    grid_x: u16,
    y: u16,
    cell_width: u16,
    cell_height: u16,
    area: Rect,
    min_val: f64,
    max_val: f64,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    let _ = theme; // reserved for future style customization
    let row_data = &state.data()[ri];
    for (ci, &value) in row_data.iter().enumerate() {
        let x = grid_x + (ci as u16) * cell_width;
        if x >= area.right() {
            break;
        }
        let available_w = cell_width.min(area.right().saturating_sub(x));
        if available_w == 0 {
            continue;
        }
        let cell_area = Rect::new(x, y, available_w, cell_height);

        let bg_color = if disabled {
            Color::DarkGray
        } else {
            value_to_color(value, min_val, max_val, state.color_scale())
        };

        let is_selected = state.selected() == Some((ri, ci));

        let cell_style = if is_selected && focused && !disabled {
            // Selected cell: invert colors for visibility
            Style::default()
                .fg(bg_color)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            // Determine a readable foreground color for value text
            let fg = contrasting_fg(bg_color);
            Style::default().bg(bg_color).fg(fg)
        };

        let text = if state.show_values() {
            format_value(value, available_w as usize)
        } else {
            " ".repeat(available_w as usize)
        };

        let p = Paragraph::new(text)
            .style(cell_style)
            .alignment(Alignment::Center);
        frame.render_widget(p, cell_area);
    }
}

/// Truncates a string to fit within the given width.
pub(super) fn truncate_str(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width > 0 {
        s[..max_width].to_string()
    } else {
        String::new()
    }
}

/// Formats a value to fit within the given width.
pub(super) fn format_value(value: f64, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    // Try different formats to fit
    let formatted = if width >= 6 {
        format!("{value:.1}")
    } else if width >= 3 {
        format!("{value:.0}")
    } else {
        // Very narrow -- just use first chars
        let s = format!("{value:.0}");
        s[..s.len().min(width)].to_string()
    };

    if formatted.len() <= width {
        formatted
    } else {
        formatted[..width].to_string()
    }
}

/// Returns a contrasting foreground color for readability on the given background.
pub(super) fn contrasting_fg(bg: Color) -> Color {
    match bg {
        Color::Rgb(r, g, b) => {
            // Use relative luminance to pick black or white text
            let luminance = 0.299 * (r as f64) + 0.587 * (g as f64) + 0.114 * (b as f64);
            if luminance > 128.0 {
                Color::Black
            } else {
                Color::White
            }
        }
        Color::DarkGray | Color::Black => Color::White,
        _ => Color::Black,
    }
}
