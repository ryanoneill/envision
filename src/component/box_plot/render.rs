//! Rendering functions for the BoxPlot component.
//!
//! Renders box-and-whisker plots in vertical or horizontal orientation.
//! Uses block characters to draw whiskers, boxes, and median lines.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::BoxPlotState;
use crate::theme::Theme;

/// Maps a data value to a position within a pixel range.
///
/// Returns the position (0-based) within `length` pixels that corresponds
/// to `value` in the range `[data_min, data_max]`.
fn value_to_position(value: f64, data_min: f64, data_max: f64, length: u16) -> u16 {
    if length == 0 {
        return 0;
    }
    let range = data_max - data_min;
    if range <= 0.0 {
        return length / 2;
    }
    let ratio = (value - data_min) / range;
    let pos = (ratio * (length.saturating_sub(1) as f64)).round() as u16;
    pos.min(length.saturating_sub(1))
}

/// Renders vertical box plots (values on Y axis, datasets on X axis).
pub(super) fn render_vertical(
    state: &BoxPlotState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    let dataset_count = state.datasets().len() as u16;
    if dataset_count == 0 || area.height < 3 || area.width < 3 {
        return;
    }

    // Reserve bottom row for labels
    let label_height = 1u16;
    let chart_height = area.height.saturating_sub(label_height);
    if chart_height < 3 {
        return;
    }

    let chart_area = Rect::new(area.x, area.y, area.width, chart_height);
    let label_area = Rect::new(area.x, area.y + chart_height, area.width, label_height);

    // Calculate column width for each dataset
    let col_width = area.width / dataset_count.max(1);
    if col_width < 3 {
        // Not enough space for meaningful rendering
        return;
    }

    let data_min = state.global_min();
    let data_max = state.global_max();

    let style = if disabled {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };

    // Render each box plot
    for (i, dataset) in state.datasets().iter().enumerate() {
        let col_x = area.x + (i as u16) * col_width;
        let center_x = col_x + col_width / 2;

        let box_color = if disabled {
            Color::DarkGray
        } else {
            dataset.color()
        };
        let box_style = Style::default().fg(box_color);

        let selected_indicator = if i == state.selected() && focused {
            Style::default().fg(box_color).add_modifier(Modifier::BOLD)
        } else {
            box_style
        };

        // Map values to vertical positions (inverted: top = max, bottom = min)
        let min_y = value_to_y(dataset.min(), data_min, data_max, chart_area);
        let q1_y = value_to_y(dataset.q1(), data_min, data_max, chart_area);
        let median_y = value_to_y(dataset.median(), data_min, data_max, chart_area);
        let q3_y = value_to_y(dataset.q3(), data_min, data_max, chart_area);
        let max_y = value_to_y(dataset.max(), data_min, data_max, chart_area);

        // Draw upper whisker (from max to q3)
        if max_y < q3_y {
            for y in max_y..q3_y {
                if center_x < area.x + area.width && y < chart_area.y + chart_area.height {
                    frame
                        .buffer_mut()
                        .set_string(center_x, y, "|", selected_indicator);
                }
            }
            // Whisker cap at max
            if center_x > col_x && center_x + 1 < col_x + col_width {
                frame.buffer_mut().set_string(
                    center_x.saturating_sub(1),
                    max_y,
                    "---",
                    selected_indicator,
                );
            }
        }

        // Draw the box (from q3 to q1)
        let box_left = center_x.saturating_sub(1);
        let box_right = (center_x + 1).min(col_x + col_width - 1);
        let box_width = box_right.saturating_sub(box_left) + 1;

        // Top of box (Q3 line)
        if q3_y >= chart_area.y && q3_y < chart_area.y + chart_area.height {
            let top_str: String = "-".repeat(box_width as usize);
            frame
                .buffer_mut()
                .set_string(box_left, q3_y, &top_str, selected_indicator);
        }

        // Box sides
        let box_top = q3_y + 1;
        let box_bottom = q1_y;
        for y in box_top..box_bottom {
            if y >= chart_area.y && y < chart_area.y + chart_area.height {
                if box_left >= area.x && box_left < area.x + area.width {
                    frame
                        .buffer_mut()
                        .set_string(box_left, y, "|", selected_indicator);
                }
                if box_right >= area.x && box_right < area.x + area.width {
                    frame
                        .buffer_mut()
                        .set_string(box_right, y, "|", selected_indicator);
                }
            }
        }

        // Median line
        if median_y >= chart_area.y && median_y < chart_area.y + chart_area.height {
            let median_str: String = "=".repeat(box_width as usize);
            frame
                .buffer_mut()
                .set_string(box_left, median_y, &median_str, selected_indicator);
        }

        // Bottom of box (Q1 line)
        if q1_y >= chart_area.y && q1_y < chart_area.y + chart_area.height {
            let bottom_str: String = "-".repeat(box_width as usize);
            frame
                .buffer_mut()
                .set_string(box_left, q1_y, &bottom_str, selected_indicator);
        }

        // Draw lower whisker (from q1 to min)
        if q1_y < min_y {
            for y in (q1_y + 1)..=min_y {
                if center_x < area.x + area.width && y < chart_area.y + chart_area.height {
                    frame
                        .buffer_mut()
                        .set_string(center_x, y, "|", selected_indicator);
                }
            }
            // Whisker cap at min
            if center_x > col_x
                && center_x + 1 < col_x + col_width
                && min_y < chart_area.y + chart_area.height
            {
                frame.buffer_mut().set_string(
                    center_x.saturating_sub(1),
                    min_y,
                    "---",
                    selected_indicator,
                );
            }
        }

        // Draw outliers
        if state.show_outliers() {
            for &outlier in dataset.outliers() {
                let oy = value_to_y(outlier, data_min, data_max, chart_area);
                if oy >= chart_area.y
                    && oy < chart_area.y + chart_area.height
                    && center_x < area.x + area.width
                {
                    frame.buffer_mut().set_string(center_x, oy, "*", box_style);
                }
            }
        }

        // Render label below the chart
        let label = dataset.label();
        let max_label_width = col_width as usize;
        let truncated_label = if label.len() > max_label_width {
            &label[..max_label_width]
        } else {
            label
        };
        let label_x = col_x + col_width.saturating_sub(truncated_label.len() as u16) / 2;
        if label_area.y < area.y + area.height {
            frame
                .buffer_mut()
                .set_string(label_x, label_area.y, truncated_label, style);
        }
    }
}

/// Renders horizontal box plots (values on X axis, datasets on Y axis).
pub(super) fn render_horizontal(
    state: &BoxPlotState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    let dataset_count = state.datasets().len() as u16;
    if dataset_count == 0 || area.height < 3 || area.width < 3 {
        return;
    }

    // Reserve left column for labels
    let max_label_len = state
        .datasets()
        .iter()
        .map(|d| d.label().len())
        .max()
        .unwrap_or(0) as u16;
    let label_width = max_label_len.min(area.width / 3).max(1);
    let chart_width = area.width.saturating_sub(label_width + 1);
    if chart_width < 3 {
        return;
    }

    let chart_x = area.x + label_width + 1;

    // Calculate row height for each dataset
    let row_height = area.height / dataset_count.max(1);
    if row_height < 1 {
        return;
    }

    let data_min = state.global_min();
    let data_max = state.global_max();

    let style = if disabled {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };

    for (i, dataset) in state.datasets().iter().enumerate() {
        let row_y = area.y + (i as u16) * row_height;
        let center_y = row_y + row_height / 2;

        let box_color = if disabled {
            Color::DarkGray
        } else {
            dataset.color()
        };
        let box_style = Style::default().fg(box_color);

        let selected_indicator = if i == state.selected() && focused {
            Style::default().fg(box_color).add_modifier(Modifier::BOLD)
        } else {
            box_style
        };

        // Map values to horizontal positions
        let min_x = value_to_position(dataset.min(), data_min, data_max, chart_width) + chart_x;
        let q1_x = value_to_position(dataset.q1(), data_min, data_max, chart_width) + chart_x;
        let median_x =
            value_to_position(dataset.median(), data_min, data_max, chart_width) + chart_x;
        let q3_x = value_to_position(dataset.q3(), data_min, data_max, chart_width) + chart_x;
        let max_x = value_to_position(dataset.max(), data_min, data_max, chart_width) + chart_x;

        // Draw left whisker (min to q1)
        if min_x < q1_x && center_y < area.y + area.height {
            let whisker_len = q1_x.saturating_sub(min_x);
            let whisker_str: String = "-".repeat(whisker_len as usize);
            frame
                .buffer_mut()
                .set_string(min_x, center_y, &whisker_str, selected_indicator);
            // Whisker cap at min
            if center_y > row_y && center_y < row_y + row_height {
                frame
                    .buffer_mut()
                    .set_string(min_x, center_y, "|", selected_indicator);
            }
        }

        // Draw the box (q1 to q3)
        let box_top = center_y.saturating_sub(row_height.min(3) / 2);
        let box_bottom = (center_y + row_height.min(3) / 2).min(row_y + row_height - 1);

        // Top and bottom borders of box
        if box_top >= area.y && box_top < area.y + area.height {
            let box_len = q3_x.saturating_sub(q1_x) + 1;
            let border_str: String = "-".repeat(box_len as usize);
            frame
                .buffer_mut()
                .set_string(q1_x, box_top, &border_str, selected_indicator);
        }
        if box_bottom > box_top && box_bottom >= area.y && box_bottom < area.y + area.height {
            let box_len = q3_x.saturating_sub(q1_x) + 1;
            let border_str: String = "-".repeat(box_len as usize);
            frame
                .buffer_mut()
                .set_string(q1_x, box_bottom, &border_str, selected_indicator);
        }

        // Left and right sides of box
        for y in box_top..=box_bottom {
            if y >= area.y && y < area.y + area.height {
                if q1_x < area.x + area.width {
                    frame
                        .buffer_mut()
                        .set_string(q1_x, y, "|", selected_indicator);
                }
                if q3_x < area.x + area.width {
                    frame
                        .buffer_mut()
                        .set_string(q3_x, y, "|", selected_indicator);
                }
            }
        }

        // Median line (vertical within the box)
        for y in box_top..=box_bottom {
            if y >= area.y
                && y < area.y + area.height
                && median_x < area.x + area.width
                && median_x >= q1_x
                && median_x <= q3_x
            {
                frame
                    .buffer_mut()
                    .set_string(median_x, y, "|", selected_indicator);
            }
        }

        // Draw right whisker (q3 to max)
        if q3_x < max_x && center_y < area.y + area.height {
            let whisker_start = q3_x + 1;
            let whisker_len = max_x.saturating_sub(whisker_start);
            if whisker_len > 0 {
                let whisker_str: String = "-".repeat(whisker_len as usize);
                frame.buffer_mut().set_string(
                    whisker_start,
                    center_y,
                    &whisker_str,
                    selected_indicator,
                );
            }
            // Whisker cap at max
            if max_x < area.x + area.width {
                frame
                    .buffer_mut()
                    .set_string(max_x, center_y, "|", selected_indicator);
            }
        }

        // Draw outliers
        if state.show_outliers() {
            for &outlier in dataset.outliers() {
                let ox = value_to_position(outlier, data_min, data_max, chart_width) + chart_x;
                if ox < area.x + area.width && center_y < area.y + area.height {
                    frame.buffer_mut().set_string(ox, center_y, "*", box_style);
                }
            }
        }

        // Render label on the left
        let label = dataset.label();
        let label_width_usize = label_width as usize;
        let truncated_label = if label.len() > label_width_usize {
            &label[..label_width_usize]
        } else {
            label
        };
        if center_y < area.y + area.height {
            let p = Paragraph::new(truncated_label).style(style);
            let label_rect = Rect::new(area.x, center_y, label_width, 1);
            frame.render_widget(p, label_rect);
        }
    }
}

/// Converts a data value to a Y coordinate (inverted: higher values at top).
fn value_to_y(value: f64, data_min: f64, data_max: f64, area: Rect) -> u16 {
    let height = area.height;
    if height == 0 {
        return area.y;
    }
    let range = data_max - data_min;
    if range <= 0.0 {
        return area.y + height / 2;
    }
    let ratio = (value - data_min) / range;
    // Invert: top of area = data_max, bottom = data_min
    let y_offset = ((1.0 - ratio) * (height.saturating_sub(1) as f64)).round() as u16;
    area.y + y_offset.min(height.saturating_sub(1))
}

#[cfg(test)]
mod render_tests {
    use super::*;

    #[test]
    fn test_value_to_position_basic() {
        assert_eq!(value_to_position(0.0, 0.0, 100.0, 100), 0);
        assert_eq!(value_to_position(50.0, 0.0, 100.0, 100), 50);
        assert_eq!(value_to_position(100.0, 0.0, 100.0, 100), 99);
    }

    #[test]
    fn test_value_to_position_zero_length() {
        assert_eq!(value_to_position(50.0, 0.0, 100.0, 0), 0);
    }

    #[test]
    fn test_value_to_position_zero_range() {
        assert_eq!(value_to_position(50.0, 50.0, 50.0, 100), 50);
    }

    #[test]
    fn test_value_to_y_basic() {
        let area = Rect::new(0, 0, 10, 100);
        // Max value should be at top (y = 0)
        assert_eq!(value_to_y(100.0, 0.0, 100.0, area), 0);
        // Min value should be at bottom (y = 99)
        assert_eq!(value_to_y(0.0, 0.0, 100.0, area), 99);
        // Middle value should be around middle
        let mid = value_to_y(50.0, 0.0, 100.0, area);
        assert!((49..=50).contains(&mid));
    }

    #[test]
    fn test_value_to_y_zero_height() {
        let area = Rect::new(5, 10, 10, 0);
        assert_eq!(value_to_y(50.0, 0.0, 100.0, area), 10);
    }

    #[test]
    fn test_value_to_y_zero_range() {
        let area = Rect::new(0, 0, 10, 20);
        assert_eq!(value_to_y(5.0, 5.0, 5.0, area), 10);
    }
}
