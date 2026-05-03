//! Rendering functions for the SpanTree component.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{FlatSpan, SpanTreeState};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Renders the full span tree including border, time axis header, and rows.
///
/// `chrome_owned` signals that the parent has already drawn the outer
/// chrome for `area`. When true, the outer `Block` draw is suppressed
/// and content is rendered against `area` directly.
pub(super) fn render_span_tree(
    state: &SpanTreeState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
    chrome_owned: bool,
) {
    let inner = if chrome_owned {
        area
    } else {
        let border_style = if disabled {
            theme.disabled_style()
        } else if focused {
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
        inner
    };

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let annotation = crate::annotation::Annotation::span_tree("span_tree")
        .with_focus(focused)
        .with_disabled(disabled);
    let empty = Paragraph::new("");
    let annotated = crate::annotation::Annotate::new(empty, annotation);
    frame.render_widget(annotated, inner);

    // Clamp label_width to available space
    let effective_label_width = state.label_width.min(inner.width.saturating_sub(5));
    let bar_area_width = inner.width.saturating_sub(effective_label_width + 1); // +1 for separator

    // Reserve first line for the time axis header
    if inner.height < 2 {
        return;
    }

    render_header(
        state,
        frame,
        inner,
        effective_label_width,
        bar_area_width,
        theme,
        disabled,
    );

    // Separator line
    let sep_y = inner.y + 1;
    if sep_y < inner.bottom() {
        let sep_area = Rect::new(inner.x, sep_y, inner.width, 1);
        let mut sep_line = String::new();
        for i in 0..inner.width {
            if i == effective_label_width {
                sep_line.push('┼');
            } else {
                sep_line.push('─');
            }
        }
        let sep_style = if disabled {
            theme.disabled_style()
        } else {
            theme.normal_style()
        };
        frame.render_widget(Paragraph::new(sep_line).style(sep_style), sep_area);
    }

    // Content rows start after header + separator
    let content_y = inner.y + 2;
    let content_height = inner.height.saturating_sub(2) as usize;
    if content_height == 0 {
        return;
    }

    let flat = state.flatten();

    // Scrolling
    let mut bar_scroll = ScrollState::new(flat.len());
    bar_scroll.set_viewport_height(content_height);
    if let Some(idx) = state.selected_index {
        bar_scroll.ensure_visible(idx);
    }

    let range = bar_scroll.visible_range();
    let visible_spans: Vec<&FlatSpan> = flat.iter().skip(range.start).take(range.len()).collect();

    let ctx = RowContext {
        state,
        label_width: effective_label_width,
        bar_width: bar_area_width,
        theme,
        focused,
        disabled,
    };

    for (row_idx, span) in visible_spans.iter().enumerate() {
        let y = content_y + row_idx as u16;
        if y >= inner.bottom() {
            break;
        }

        let global_idx = range.start + row_idx;
        let is_selected = state.selected_index == Some(global_idx);

        render_row(&ctx, frame, span, is_selected, inner.x, y);
    }

    // Render scrollbar if content exceeds viewport
    let scroll_area = Rect::new(inner.x, content_y, inner.width, content_height as u16);
    crate::scroll::render_scrollbar(&bar_scroll, frame, scroll_area, theme);
}

/// Renders the time axis header row.
fn render_header(
    state: &SpanTreeState,
    frame: &mut Frame,
    inner: Rect,
    label_width: u16,
    bar_width: u16,
    theme: &Theme,
    disabled: bool,
) {
    let header_area = Rect::new(inner.x, inner.y, inner.width, 1);

    let style = if disabled {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };

    // Build the header: "Label" on left, time ticks on right
    let mut header = String::new();

    // Left column: "Label" padded
    let label_header = "Label";
    header.push_str(label_header);
    let label_pad = label_width as usize - label_header.len().min(label_width as usize);
    for _ in 0..label_pad {
        header.push(' ');
    }
    header.push('│');

    // Right column: time axis labels
    if bar_width > 0 {
        let time_labels = format_time_axis(state.global_start, state.global_end, bar_width);
        header.push_str(&time_labels);
    }

    // Pad to full width
    let total = header.chars().count();
    let remaining = (inner.width as usize).saturating_sub(total);
    for _ in 0..remaining {
        header.push(' ');
    }

    frame.render_widget(Paragraph::new(header).style(style), header_area);
}

/// Formats time axis labels across the given width.
fn format_time_axis(global_start: f64, global_end: f64, width: u16) -> String {
    let width = width as usize;
    if width == 0 {
        return String::new();
    }

    let total_duration = global_end - global_start;
    if total_duration <= 0.0 {
        let label = format_time_value(global_start);
        let mut result = format!(" {}", label);
        let pad = width.saturating_sub(result.len());
        for _ in 0..pad {
            result.push(' ');
        }
        return result;
    }

    // Place tick labels at start, middle, and end
    let start_label = format_time_value(global_start);
    let mid_label = format_time_value(global_start + total_duration / 2.0);
    let end_label = format_time_value(global_end);

    // For very narrow widths, just show start and end
    if width < 20 {
        let mut result = format!(" {}", start_label);
        let end_pos = width.saturating_sub(end_label.len());
        let pad = end_pos.saturating_sub(result.len());
        for _ in 0..pad {
            result.push(' ');
        }
        result.push_str(&end_label);
        // Ensure we don't exceed width
        while result.chars().count() > width {
            result.pop();
        }
        return result;
    }

    let mut line = vec![' '; width];

    // Place start label at position 1
    let start_pos = 1;
    for (i, ch) in start_label.chars().enumerate() {
        let pos = start_pos + i;
        if pos < width {
            line[pos] = ch;
        }
    }

    // Place middle label at center
    let mid_pos = width / 2 - mid_label.len() / 2;
    // Only place if it doesn't overlap with start
    if mid_pos > start_pos + start_label.len() + 1 {
        for (i, ch) in mid_label.chars().enumerate() {
            let pos = mid_pos + i;
            if pos < width {
                line[pos] = ch;
            }
        }
    }

    // Place end label at right edge
    let end_pos = width.saturating_sub(end_label.len());
    // Only place if it doesn't overlap with middle
    let no_overlap =
        mid_pos + mid_label.len() + 1 < end_pos || mid_pos <= start_pos + start_label.len() + 1;
    if no_overlap && end_pos > start_pos + start_label.len() + 1 {
        for (i, ch) in end_label.chars().enumerate() {
            let pos = end_pos + i;
            if pos < width {
                line[pos] = ch;
            }
        }
    }

    line.into_iter().collect()
}

/// Formats a time value for display on the axis.
fn format_time_value(value: f64) -> String {
    if value == 0.0 {
        return "0ms".to_string();
    }
    if value >= 1000.0 {
        let secs = value / 1000.0;
        if secs == secs.round() {
            format!("{}s", secs as i64)
        } else {
            format!("{:.1}s", secs)
        }
    } else if value == value.round() {
        format!("{}ms", value as i64)
    } else {
        format!("{:.1}ms", value)
    }
}

/// Layout context for rendering individual span rows.
struct RowContext<'a> {
    state: &'a SpanTreeState,
    label_width: u16,
    bar_width: u16,
    theme: &'a Theme,
    focused: bool,
    disabled: bool,
}

/// Renders a single span row with label and timing bar.
fn render_row(
    ctx: &RowContext<'_>,
    frame: &mut Frame,
    span: &FlatSpan,
    is_selected: bool,
    x: u16,
    y: u16,
) {
    let row_area = Rect::new(x, y, ctx.label_width + 1 + ctx.bar_width, 1);

    // Determine styles
    let (label_style, bar_bg_style) = if ctx.disabled {
        (ctx.theme.disabled_style(), ctx.theme.disabled_style())
    } else if is_selected {
        let hl = ctx.theme.selected_highlight_style(ctx.focused);
        (hl, ctx.theme.normal_style())
    } else {
        (ctx.theme.normal_style(), ctx.theme.normal_style())
    };

    // Build label text with indent and expand/collapse indicator
    let indent = "  ".repeat(span.depth);
    let indicator = if span.has_children {
        if span.is_expanded { "▾ " } else { "▸ " }
    } else {
        "  "
    };

    let label_text = format!("{}{}{}", indent, indicator, span.label);

    // Truncate or pad label to fit
    let label_chars: Vec<char> = label_text.chars().collect();
    let mut label_padded = String::with_capacity(ctx.label_width as usize + 1);
    for i in 0..ctx.label_width as usize {
        if i < label_chars.len() {
            label_padded.push(label_chars[i]);
        } else {
            label_padded.push(' ');
        }
    }
    label_padded.push('│');

    // Build timing bar
    let bar_string = render_bar(
        span,
        ctx.state.global_start,
        ctx.state.global_end,
        ctx.bar_width,
    );

    // Combine into spans
    let bar_style = if ctx.disabled {
        ctx.theme.disabled_style()
    } else {
        Style::default()
            .fg(span.color)
            .bg(bar_bg_style.bg.unwrap_or(Color::Reset))
    };

    let line = Line::from(vec![
        Span::styled(label_padded, label_style),
        Span::styled(bar_string, bar_style),
    ]);

    frame.render_widget(Paragraph::new(line), row_area);
}

/// Renders the timing bar as a string of block characters.
fn render_bar(span: &FlatSpan, global_start: f64, global_end: f64, bar_width: u16) -> String {
    let bar_width = bar_width as usize;
    if bar_width == 0 {
        return String::new();
    }

    let total_duration = global_end - global_start;
    if total_duration <= 0.0 {
        // If no time range, fill entire bar
        return "█".repeat(bar_width);
    }

    let time_scale = bar_width as f64 / total_duration;
    let bar_start = ((span.start - global_start) * time_scale) as usize;
    let bar_len = ((span.end - span.start) * time_scale).max(1.0) as usize;

    let mut result = String::with_capacity(bar_width);
    for i in 0..bar_width {
        if i >= bar_start && i < bar_start + bar_len {
            result.push('█');
        } else {
            result.push(' ');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time_value_zero() {
        assert_eq!(format_time_value(0.0), "0ms");
    }

    #[test]
    fn test_format_time_value_milliseconds() {
        assert_eq!(format_time_value(500.0), "500ms");
    }

    #[test]
    fn test_format_time_value_seconds() {
        assert_eq!(format_time_value(1000.0), "1s");
    }

    #[test]
    fn test_format_time_value_fractional_seconds() {
        assert_eq!(format_time_value(1500.0), "1.5s");
    }

    #[test]
    fn test_format_time_value_fractional_ms() {
        assert_eq!(format_time_value(50.5), "50.5ms");
    }

    #[test]
    fn test_render_bar_full_span() {
        let span = FlatSpan {
            id: "test".into(),
            label: "test".into(),
            start: 0.0,
            end: 100.0,
            color: Color::Cyan,
            status: None,
            depth: 0,
            has_children: false,
            is_expanded: false,
        };
        let bar = render_bar(&span, 0.0, 100.0, 10);
        assert_eq!(bar, "██████████");
    }

    #[test]
    fn test_render_bar_half_span() {
        let span = FlatSpan {
            id: "test".into(),
            label: "test".into(),
            start: 0.0,
            end: 50.0,
            color: Color::Cyan,
            status: None,
            depth: 0,
            has_children: false,
            is_expanded: false,
        };
        let bar = render_bar(&span, 0.0, 100.0, 10);
        assert_eq!(bar, "█████     ");
    }

    #[test]
    fn test_render_bar_offset_span() {
        let span = FlatSpan {
            id: "test".into(),
            label: "test".into(),
            start: 50.0,
            end: 100.0,
            color: Color::Cyan,
            status: None,
            depth: 0,
            has_children: false,
            is_expanded: false,
        };
        let bar = render_bar(&span, 0.0, 100.0, 10);
        assert_eq!(bar, "     █████");
    }

    #[test]
    fn test_render_bar_zero_width() {
        let span = FlatSpan {
            id: "test".into(),
            label: "test".into(),
            start: 0.0,
            end: 100.0,
            color: Color::Cyan,
            status: None,
            depth: 0,
            has_children: false,
            is_expanded: false,
        };
        let bar = render_bar(&span, 0.0, 100.0, 0);
        assert_eq!(bar, "");
    }

    #[test]
    fn test_render_bar_minimum_width() {
        // Very short span should render at least 1 char
        let span = FlatSpan {
            id: "test".into(),
            label: "test".into(),
            start: 0.0,
            end: 1.0,
            color: Color::Cyan,
            status: None,
            depth: 0,
            has_children: false,
            is_expanded: false,
        };
        let bar = render_bar(&span, 0.0, 1000.0, 20);
        assert!(
            bar.contains('█'),
            "bar should contain at least one block: {bar:?}"
        );
    }

    #[test]
    fn test_format_time_axis_zero_duration() {
        let axis = format_time_axis(0.0, 0.0, 20);
        assert!(axis.contains("0ms"));
    }
}
