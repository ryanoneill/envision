use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::EventStreamState;
use crate::theme::Theme;

/// Renders the complete event stream component.
pub(super) fn render_event_stream(
    state: &EventStreamState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    if area.height < 3 {
        return;
    }

    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::container("event_stream")
                .with_focus(state.is_focused())
                .with_disabled(state.is_disabled()),
        );
    });

    // Layout: event list + status bar (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let list_area = chunks[0];
    let status_area = chunks[1];

    // Render event list
    render_event_list(state, frame, list_area, theme);

    // Render status bar (filter + level + auto-scroll indicator)
    render_status_bar(state, frame, status_area, theme);
}

/// Renders the event list area with a bordered block.
fn render_event_list(state: &EventStreamState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let visible = state.visible_events();

    let border_style = if state.is_disabled() {
        theme.disabled_style()
    } else if state.is_focused() && !state.is_search_focused() {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    // Build title with event counts
    let total = state.event_count();
    let showing = visible.len();
    if let Some(title) = state.title() {
        if showing < total {
            block = block.title(format!(
                " {} ({} events, showing {}) ",
                title, total, showing
            ));
        } else {
            block = block.title(format!(" {} ({} events) ", title, total));
        }
    } else if showing < total {
        block = block.title(format!(
            " Event Stream ({} events, showing {}) ",
            total, showing
        ));
    } else {
        block = block.title(format!(" Event Stream ({} events) ", total));
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Render header line
    let header_height = 1u16;
    let data_height = inner.height.saturating_sub(header_height);

    if inner.height >= 1 {
        let header_area = Rect::new(inner.x, inner.y, inner.width, 1);
        render_header(state, frame, header_area, theme);
    }

    if data_height == 0 {
        return;
    }

    let data_area = Rect::new(inner.x, inner.y + header_height, inner.width, data_height);

    // Compute effective scroll offset, clamped to valid range for the viewport.
    // This is needed because auto_scroll may set the offset before the
    // viewport height is known (it defaults to 0).
    let max_offset = visible.len().saturating_sub(data_height as usize);
    let effective_offset = if state.auto_scroll() {
        // When auto-scrolling, always show the latest events
        max_offset
    } else {
        state.scroll_offset().min(max_offset)
    };

    let items: Vec<ListItem> = visible
        .iter()
        .skip(effective_offset)
        .take(data_height as usize)
        .map(|event| render_event_row(state, event, inner.width as usize, theme))
        .collect();

    let list = List::new(items);
    frame.render_widget(list, data_area);

    // Render scrollbar if content exceeds viewport
    if visible.len() > data_height as usize {
        let mut bar_scroll = crate::scroll::ScrollState::new(visible.len());
        bar_scroll.set_viewport_height(data_height as usize);
        bar_scroll.set_offset(effective_offset);
        crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
    }
}

/// Renders the column header line.
fn render_header(state: &EventStreamState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let style = if state.is_disabled() {
        theme.disabled_style()
    } else {
        theme.normal_style().add_modifier(Modifier::BOLD)
    };

    let header = build_header_text(state);
    let paragraph = Paragraph::new(header).style(style);
    frame.render_widget(paragraph, area);
}

/// Builds the header text based on visible columns.
fn build_header_text(state: &EventStreamState) -> String {
    let mut parts = Vec::new();

    if state.show_timestamps() {
        parts.push(format!("{:<12}", "Time"));
    }
    if state.show_level() {
        parts.push(format!("{:<5}", "Lvl"));
    }
    if state.show_source() {
        parts.push(format!("{:<10}", "Source"));
    }

    parts.push("Message".to_string());

    for col in state.visible_columns() {
        parts.push(col.clone());
    }

    parts.join(" ")
}

/// Renders a single event row as a `ListItem`.
fn render_event_row<'a>(
    state: &EventStreamState,
    event: &super::StreamEvent,
    _max_width: usize,
    theme: &Theme,
) -> ListItem<'a> {
    let level_color = event.level.color();
    let style = if state.is_disabled() {
        theme.disabled_style()
    } else {
        Style::default().fg(level_color)
    };

    let mut parts = Vec::new();

    if state.show_timestamps() {
        parts.push(format!("{:<12.1}", event.timestamp));
    }
    if state.show_level() {
        parts.push(format!("{:<5}", event.level.abbreviation()));
    }
    if state.show_source() {
        let source = event.source.as_deref().unwrap_or("-");
        parts.push(format!("{:<10}", truncate(source, 10)));
    }

    parts.push(event.message.clone());

    // Append visible column field values
    if !state.visible_columns().is_empty() {
        let field_parts: Vec<String> = state
            .visible_columns()
            .iter()
            .filter_map(|col| {
                event
                    .fields
                    .iter()
                    .find(|(k, _)| k == col)
                    .map(|(k, v)| format!("{}={}", k, v))
            })
            .collect();
        if !field_parts.is_empty() {
            parts.push(field_parts.join(" "));
        }
    } else if !event.fields.is_empty() {
        // Show all fields inline
        let field_parts: Vec<String> = event
            .fields
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        parts.push(field_parts.join(" "));
    }

    let text = parts.join(" ");

    // Highlight search matches
    if !state.filter_text().is_empty() && !state.is_disabled() {
        let text_lower = text.to_lowercase();
        let search_lower = state.filter_text().to_lowercase();
        if text_lower.contains(&search_lower) {
            let style = style.add_modifier(Modifier::BOLD);
            return ListItem::new(text).style(style);
        }
    }

    ListItem::new(text).style(style)
}

/// Renders the status bar at the bottom.
fn render_status_bar(state: &EventStreamState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let style = if state.is_disabled() {
        theme.disabled_style()
    } else if state.is_search_focused() {
        theme.focused_style()
    } else {
        theme.normal_style()
    };

    let mut spans = Vec::new();

    // Filter text display
    let filter_display = if state.is_search_focused() {
        format!("Filter: [{}]", state.search_value())
    } else if state.filter_text().is_empty() {
        "Filter: [/]".to_string()
    } else {
        format!("Filter: [{}]", state.filter_text())
    };
    spans.push(Span::styled(filter_display, style));

    spans.push(Span::raw("  "));

    // Level filter display
    let level_display = match state.level_filter() {
        Some(level) => format!("Level: >={}", level.abbreviation()),
        None => "Level: ALL".to_string(),
    };
    spans.push(Span::styled(level_display, style));

    spans.push(Span::raw("  "));

    // Auto-scroll indicator
    let auto_display = if state.auto_scroll() {
        "Auto: ON"
    } else {
        "Auto: OFF"
    };
    spans.push(Span::styled(auto_display, style));

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);

    // Show cursor when search is focused
    if state.is_focused() && state.is_search_focused() && !state.is_disabled() {
        // "Filter: [" is 9 chars, cursor is at that offset plus cursor position
        let cursor_x = area.x + 9 + state.search_cursor_position() as u16;
        if cursor_x < area.right() {
            frame.set_cursor_position(Position::new(cursor_x, area.y));
        }
    }
}

/// Truncates a string to at most `max_len` characters.
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}
