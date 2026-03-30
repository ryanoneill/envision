use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::LogCorrelationState;
use crate::theme::Theme;

/// Renders the entire LogCorrelation component.
pub(super) fn render(state: &LogCorrelationState, frame: &mut Frame, area: Rect, theme: &Theme) {
    if area.height < 3 || area.width < 3 {
        return;
    }

    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::container("log_correlation")
                .with_focus(state.is_focused())
                .with_disabled(state.is_disabled()),
        );
    });

    // Layout: streams area + status bar (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let streams_area = chunks[0];
    let status_area = chunks[1];

    render_streams(state, frame, streams_area, theme);
    render_status_bar(state, frame, status_area, theme);
}

/// Renders the side-by-side stream panels.
fn render_streams(state: &LogCorrelationState, frame: &mut Frame, area: Rect, theme: &Theme) {
    if state.streams.is_empty() {
        let border_style = if state.is_disabled() {
            theme.disabled_style()
        } else if state.is_focused() {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(title) = state.title() {
            block = block.title(format!(" {} ", title));
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let msg = Paragraph::new("No streams configured")
            .style(theme.normal_style())
            .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    // Outer border with title
    let outer_border_style = if state.is_disabled() {
        theme.disabled_style()
    } else if state.is_focused() {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let mut outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(outer_border_style);

    if let Some(title) = state.title() {
        outer_block = outer_block.title(format!(" {} ", title));
    }

    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    if inner_area.height == 0 || inner_area.width == 0 {
        return;
    }

    // Split inner area equally among streams
    let stream_count = state.streams.len() as u16;
    let constraints: Vec<Constraint> = (0..stream_count)
        .map(|i| {
            if i < stream_count - 1 {
                Constraint::Ratio(1, stream_count as u32)
            } else {
                Constraint::Min(0)
            }
        })
        .collect();

    let stream_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(inner_area);

    // Compute aligned rows once for all streams
    let aligned_rows = state.aligned_rows();
    let filtered: Vec<Vec<&super::CorrelationEntry>> =
        state.streams.iter().map(|s| s.filtered_entries()).collect();

    for (i, stream) in state.streams.iter().enumerate() {
        let stream_area = stream_areas[i];
        let is_active = i == state.active_stream();

        render_single_stream(
            state,
            stream,
            i,
            is_active,
            &aligned_rows,
            &filtered[i],
            frame,
            stream_area,
            theme,
        );
    }
}

/// Renders a single stream panel.
#[allow(clippy::too_many_arguments)]
fn render_single_stream(
    state: &LogCorrelationState,
    stream: &super::LogStream,
    _stream_idx: usize,
    is_active: bool,
    aligned_rows: &[super::AlignedRow],
    filtered_entries: &[&super::CorrelationEntry],
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let border_style = if state.is_disabled() {
        theme.disabled_style()
    } else if is_active && state.is_focused() {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let title_style = if state.is_disabled() {
        theme.disabled_style()
    } else {
        Style::default().fg(stream.color)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(format!(" {} ", stream.name), title_style));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Build display lines from aligned rows
    let mut lines: Vec<Line<'_>> = Vec::new();

    for row in aligned_rows {
        let indices = &row.stream_entries[_stream_idx];
        let max_across_streams = row
            .stream_entries
            .iter()
            .map(|idx| idx.len().max(1))
            .max()
            .unwrap_or(1);

        if indices.is_empty() {
            // No entries for this stream at this timestamp -- pad with blank lines
            for _ in 0..max_across_streams {
                lines.push(Line::from(""));
            }
        } else {
            for &idx in indices {
                if idx < filtered_entries.len() {
                    let entry = filtered_entries[idx];
                    let line = format_entry(entry, inner.width as usize);
                    let style = if state.is_disabled() {
                        theme.disabled_style()
                    } else {
                        Style::default().fg(entry.level.color())
                    };
                    lines.push(Line::styled(line, style));
                }
            }
            // Pad to match the row height
            let extra = max_across_streams.saturating_sub(indices.len());
            for _ in 0..extra {
                lines.push(Line::from(""));
            }
        }
    }

    // Apply scroll offset
    let total_lines = lines.len();
    let viewport_height = inner.height as usize;
    let offset = state.scroll_offset();
    let visible_lines: Vec<Line<'_>> = lines
        .into_iter()
        .skip(offset)
        .take(viewport_height)
        .collect();

    let paragraph = Paragraph::new(visible_lines);
    frame.render_widget(paragraph, inner);

    // Render scrollbar if content exceeds viewport
    if total_lines > viewport_height {
        let mut bar_scroll = crate::scroll::ScrollState::new(total_lines);
        bar_scroll.set_viewport_height(viewport_height);
        bar_scroll.set_offset(offset);
        crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
    }
}

/// Formats a single entry for display.
fn format_entry(entry: &super::CorrelationEntry, max_width: usize) -> String {
    // Format timestamp as HH:MM:SS
    let ts = format_timestamp(entry.timestamp);
    let level = entry.level.label();
    let prefix = format!("{} {} ", ts, level);

    let remaining = max_width.saturating_sub(prefix.len());
    let msg = if entry.message.len() > remaining {
        &entry.message[..remaining]
    } else {
        &entry.message
    };

    format!("{}{}", prefix, msg)
}

/// Formats a floating-point timestamp as HH:MM:SS.
fn format_timestamp(ts: f64) -> String {
    let total_secs = ts as u64;
    let hours = (total_secs / 3600) % 24;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Renders the status bar at the bottom.
fn render_status_bar(state: &LogCorrelationState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let style = if state.is_disabled() {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };

    let active_name = if !state.streams.is_empty() {
        &state.streams[state.active_stream()].name
    } else {
        "None"
    };

    let filter_text = if !state.streams.is_empty() {
        let active = &state.streams[state.active_stream()];
        if active.filter.is_empty() {
            String::new()
        } else {
            format!(" [{}]", active.filter)
        }
    } else {
        String::new()
    };

    let sync_label = if state.sync_scroll() { "ON" } else { "OFF" };

    let status = format!(
        " Active: {}{}  Sync: {}",
        active_name, filter_text, sync_label
    );

    let paragraph = Paragraph::new(status).style(style);
    frame.render_widget(paragraph, area);
}
