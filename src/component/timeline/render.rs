//! Rendering functions for the Timeline component.
//!
//! Extracted from the main timeline module to keep file sizes manageable.
//! Contains renderers for the time axis, span bars, event markers, and
//! the detail bar.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{SelectedType, TimelineState};
use crate::theme::Theme;

/// Renders the complete timeline inside the block's inner area.
pub(super) fn render_timeline(
    state: &TimelineState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    _focused: bool,
    disabled: bool,
) {
    // Layout:
    //  1 row: time axis
    //  1 row: separator
    //  N rows: span lanes (one row per lane)
    //  1 row: event markers
    //  1 row: separator
    //  1 row: detail bar (if selection)

    let lane_count = state.effective_lane_count();
    let has_events = !state.events.is_empty();
    let has_selection = state.selected_index.is_some();

    // Calculate row requirements
    let axis_rows: u16 = 1;
    let sep1_rows: u16 = 1;
    let span_rows = lane_count as u16;
    let event_rows: u16 = if has_events { 1 } else { 0 };
    let sep2_rows: u16 = if has_selection { 1 } else { 0 };
    let detail_rows: u16 = if has_selection { 1 } else { 0 };

    let needed = axis_rows + sep1_rows + span_rows + event_rows + sep2_rows + detail_rows;

    if area.height < 2 {
        return;
    }

    // Allocate rows
    let mut y = area.y;
    let remaining = area.height;

    // Time axis - always present
    let axis_area = Rect::new(area.x, y, area.width, 1);
    render_time_axis(state, frame, axis_area, theme, disabled);
    y += 1;

    if remaining < 2 {
        return;
    }

    // Separator
    let sep1_area = Rect::new(area.x, y, area.width, 1);
    render_separator(frame, sep1_area, theme, disabled);
    y += 1;

    // If we don't have room for anything else, stop
    if y >= area.y + area.height {
        return;
    }

    let rows_left = (area.y + area.height).saturating_sub(y);

    // Determine how many rows we can give to spans and events
    let desired_content = span_rows + event_rows;
    let desired_footer = sep2_rows + detail_rows;
    let content_rows = if rows_left > desired_footer {
        (rows_left - desired_footer).min(desired_content)
    } else {
        rows_left
    };

    // Render span lanes
    let actual_span_rows = if content_rows > event_rows {
        (content_rows - event_rows).min(span_rows)
    } else if has_events {
        0
    } else {
        content_rows.min(span_rows)
    };

    for lane_idx in 0..actual_span_rows {
        if y >= area.y + area.height {
            break;
        }
        let lane_area = Rect::new(area.x, y, area.width, 1);
        render_span_lane(state, frame, lane_area, lane_idx as usize, theme, disabled);
        y += 1;
    }

    // Render event row
    if has_events && y < area.y + area.height {
        let event_area = Rect::new(area.x, y, area.width, 1);
        render_events(state, frame, event_area, theme, disabled);
        y += 1;
    }

    // Detail bar (separator + detail)
    if has_selection && y + 1 < area.y + area.height {
        let sep2_area = Rect::new(area.x, y, area.width, 1);
        render_separator(frame, sep2_area, theme, disabled);
        y += 1;

        if y < area.y + area.height {
            let detail_area = Rect::new(area.x, y, area.width, 1);
            render_detail_bar(state, frame, detail_area, theme, disabled);
        }
    } else if has_selection && y < area.y + area.height {
        // Just the detail bar, no separator
        let detail_area = Rect::new(area.x, y, area.width, 1);
        render_detail_bar(state, frame, detail_area, theme, disabled);
    }

    // Suppress unused variable warning
    let _ = needed;
}

/// Renders the time axis with tick labels.
fn render_time_axis(
    state: &TimelineState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    disabled: bool,
) {
    if area.width == 0 {
        return;
    }

    let style = if disabled {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };

    let view_start = state.view_start;
    let view_end = state.view_end;
    let view_range = view_end - view_start;

    if view_range <= 0.0 {
        return;
    }

    // Generate tick labels
    let width = area.width as usize;
    let mut line_chars = vec![' '; width];

    // Decide number of ticks based on width
    let num_ticks = (width / 12).clamp(2, 10);

    for i in 0..num_ticks {
        let frac = i as f64 / (num_ticks - 1).max(1) as f64;
        let time = view_start + frac * view_range;
        let label = format_time(time, view_range);
        let col = (frac * (width.saturating_sub(1)) as f64) as usize;

        // Place the label centered on the tick position
        let label_start = col.saturating_sub(label.len() / 2);
        for (j, ch) in label.chars().enumerate() {
            let pos = label_start + j;
            if pos < width {
                line_chars[pos] = ch;
            }
        }
    }

    let text: String = line_chars.into_iter().collect();
    let paragraph = Paragraph::new(text).style(style);
    frame.render_widget(paragraph, area);
}

/// Renders a horizontal separator line.
fn render_separator(frame: &mut Frame, area: Rect, theme: &Theme, disabled: bool) {
    let style = if disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let line = "─".repeat(area.width as usize);
    let paragraph = Paragraph::new(line).style(style);
    frame.render_widget(paragraph, area);
}

/// Renders spans for a specific lane.
fn render_span_lane(
    state: &TimelineState,
    frame: &mut Frame,
    area: Rect,
    lane_idx: usize,
    theme: &Theme,

    disabled: bool,
) {
    if area.width == 0 {
        return;
    }

    let width = area.width as usize;
    let view_start = state.view_start;
    let view_end = state.view_end;
    let view_range = view_end - view_start;

    if view_range <= 0.0 {
        return;
    }

    // Collect all spans for this lane
    let lane_spans: Vec<(usize, &super::TimelineSpan)> = state
        .spans
        .iter()
        .enumerate()
        .filter(|(_, s)| s.lane == lane_idx)
        .collect();

    // Build a character buffer with styles
    let mut chars: Vec<(char, Style)> = vec![(' ', Style::default()); width];

    for (span_idx, span) in &lane_spans {
        let is_selected =
            state.selected_type == SelectedType::Span && state.selected_index == Some(*span_idx);

        let style = if disabled {
            theme.disabled_style()
        } else if is_selected {
            Style::default()
                .fg(span.color)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(span.color)
        };

        // Calculate column positions
        let start_frac = (span.start - view_start) / view_range;
        let end_frac = (span.end - view_start) / view_range;

        let start_col = (start_frac * width as f64).round() as isize;
        let end_col = (end_frac * width as f64).round() as isize;

        let start_col = start_col.max(0) as usize;
        let end_col = (end_col.max(0) as usize).min(width);

        if start_col >= width || end_col == 0 || start_col >= end_col {
            continue;
        }

        // Draw the span bar
        for (offset, cell) in chars[start_col..end_col].iter_mut().enumerate() {
            let ch = if offset == 0 {
                '╺'
            } else if offset == end_col - start_col - 1 {
                '╸'
            } else {
                '━'
            };
            *cell = (ch, style);
        }

        // Overlay label if enabled and there's room
        if state.show_labels {
            let span_width = end_col - start_col;
            if span_width > 2 && span.label.len() + 2 <= span_width {
                let label_start = start_col + 1;
                for (j, ch) in span.label.chars().enumerate() {
                    let pos = label_start + j;
                    if pos < end_col - 1 && pos < width {
                        chars[pos] = (ch, style);
                    }
                }
            }
        }
    }

    // Build styled line
    let spans: Vec<Span> = chars
        .into_iter()
        .map(|(ch, style)| Span::styled(String::from(ch), style))
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

/// Renders point events as markers.
fn render_events(
    state: &TimelineState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    disabled: bool,
) {
    if area.width == 0 {
        return;
    }

    let width = area.width as usize;
    let view_start = state.view_start;
    let view_end = state.view_end;
    let view_range = view_end - view_start;

    if view_range <= 0.0 {
        return;
    }

    // Build a character buffer with styles
    let mut chars: Vec<(char, Style)> = vec![(' ', Style::default()); width];

    for (event_idx, event) in state.events.iter().enumerate() {
        let frac = (event.timestamp - view_start) / view_range;
        let col = (frac * width as f64).round() as isize;

        if col < 0 || col >= width as isize {
            continue;
        }

        let col = col as usize;
        let is_selected =
            state.selected_type == SelectedType::Event && state.selected_index == Some(event_idx);

        let style = if disabled {
            theme.disabled_style()
        } else if is_selected {
            Style::default()
                .fg(event.color)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(event.color)
        };

        chars[col] = ('\u{25bc}', style); // ▼
    }

    // Build styled line
    let spans: Vec<Span> = chars
        .into_iter()
        .map(|(ch, style)| Span::styled(String::from(ch), style))
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

/// Renders the detail bar showing information about the selected item.
fn render_detail_bar(
    state: &TimelineState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    disabled: bool,
) {
    let detail = match state.selected_type {
        SelectedType::Event => {
            if let Some(event) = state.selected_event() {
                format!(
                    "[Selected: {}  @{}]",
                    event.label,
                    format_time(event.timestamp, state.view_end - state.view_start),
                )
            } else {
                return;
            }
        }
        SelectedType::Span => {
            if let Some(span) = state.selected_span() {
                format!(
                    "[Selected: {}  {}-{}  {}]",
                    span.label,
                    format_time(span.start, state.view_end - state.view_start),
                    format_time(span.end, state.view_end - state.view_start),
                    format_time(span.duration(), state.view_end - state.view_start),
                )
            } else {
                return;
            }
        }
    };

    let style = if disabled {
        theme.disabled_style()
    } else {
        theme.normal_style().add_modifier(Modifier::DIM)
    };

    let paragraph = Paragraph::new(detail).style(style);
    frame.render_widget(paragraph, area);
}

/// Formats a time value with appropriate unit based on the view range.
///
/// Auto-scales between ms, s, min, and hr.
pub(super) fn format_time(value: f64, view_range: f64) -> String {
    let abs_range = view_range.abs();

    if abs_range < 1000.0 {
        // Sub-second range: show milliseconds
        format!("{:.0}ms", value)
    } else if abs_range < 60_000.0 {
        // Under a minute: show seconds
        format!("{:.1}s", value / 1000.0)
    } else if abs_range < 3_600_000.0 {
        // Under an hour: show minutes
        format!("{:.1}min", value / 60_000.0)
    } else {
        // Hours
        format!("{:.1}hr", value / 3_600_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time_milliseconds() {
        assert_eq!(format_time(500.0, 800.0), "500ms");
    }

    #[test]
    fn test_format_time_seconds() {
        assert_eq!(format_time(5000.0, 10_000.0), "5.0s");
    }

    #[test]
    fn test_format_time_minutes() {
        assert_eq!(format_time(120_000.0, 600_000.0), "2.0min");
    }

    #[test]
    fn test_format_time_hours() {
        assert_eq!(format_time(7_200_000.0, 14_400_000.0), "2.0hr");
    }

    #[test]
    fn test_format_time_zero() {
        assert_eq!(format_time(0.0, 500.0), "0ms");
    }
}
