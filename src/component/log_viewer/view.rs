use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::{LogViewerState, StatusLogLevel};
use crate::theme::Theme;

/// Renders the search bar area.
pub(super) fn render_search_bar(
    state: &LogViewerState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let search_style = if state.is_disabled() {
        theme.disabled_style()
    } else if state.is_search_focused() {
        theme.focused_style()
    } else {
        theme.normal_style()
    };

    let display = if state.search_text().is_empty() {
        "/ Search...".to_string()
    } else {
        format!("/ {}", state.search_value())
    };

    let paragraph = Paragraph::new(display).style(search_style);
    frame.render_widget(paragraph, area);

    // Show cursor when search is focused
    if state.is_focused() && state.is_search_focused() && !state.is_disabled() {
        let cursor_x = area.x + 2 + state.search_cursor_position() as u16;
        if cursor_x < area.right() {
            frame.set_cursor_position(Position::new(cursor_x, area.y));
        }
    }
}

/// Renders the filter bar showing which severity levels are active.
pub(super) fn render_filter_bar(
    state: &LogViewerState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let filter_style = if state.is_disabled() {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };

    let info_marker = if state.show_info() { "●" } else { "○" };
    let success_marker = if state.show_success() { "●" } else { "○" };
    let warning_marker = if state.show_warning() { "●" } else { "○" };
    let error_marker = if state.show_error() { "●" } else { "○" };

    let spans = vec![
        Span::styled(
            format!("1:{} Info ", info_marker),
            if state.is_disabled() {
                filter_style
            } else {
                Style::default().fg(StatusLogLevel::Info.color())
            },
        ),
        Span::styled(
            format!("2:{} Success ", success_marker),
            if state.is_disabled() {
                filter_style
            } else {
                Style::default().fg(StatusLogLevel::Success.color())
            },
        ),
        Span::styled(
            format!("3:{} Warning ", warning_marker),
            if state.is_disabled() {
                filter_style
            } else {
                Style::default().fg(StatusLogLevel::Warning.color())
            },
        ),
        Span::styled(
            format!("4:{} Error", error_marker),
            if state.is_disabled() {
                filter_style
            } else {
                Style::default().fg(StatusLogLevel::Error.color())
            },
        ),
    ];

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

/// Renders the log entries area.
pub(super) fn render_log(state: &LogViewerState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let visible = state.visible_entries();

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

    if let Some(title) = state.title() {
        let match_count = visible.len();
        let total_count = state.len();
        if match_count < total_count {
            block = block.title(format!("{} ({}/{})", title, match_count, total_count));
        } else {
            block = block.title(format!("{} ({})", title, total_count));
        }
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let items: Vec<ListItem> = visible
        .iter()
        .skip(state.scroll_offset())
        .take(inner.height as usize)
        .map(|entry| {
            let style = if state.is_disabled() {
                theme.disabled_style()
            } else {
                Style::default().fg(entry.level().color())
            };

            let mut text = String::new();
            text.push_str(entry.level().prefix());
            text.push(' ');

            if state.show_timestamps() {
                if let Some(ts) = entry.timestamp() {
                    text.push_str(ts);
                    text.push(' ');
                }
            }

            text.push_str(entry.message());

            // Highlight search matches
            if !state.search_text().is_empty() && !state.is_disabled() {
                let msg_lower = text.to_lowercase();
                let search_lower = state.search_text().to_lowercase();
                if msg_lower.contains(&search_lower) {
                    let style = style.add_modifier(Modifier::BOLD);
                    return ListItem::new(text).style(style);
                }
            }

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);

    // Render scrollbar if content exceeds viewport
    if visible.len() > inner.height as usize {
        let mut bar_scroll = crate::scroll::ScrollState::new(visible.len());
        bar_scroll.set_viewport_height(inner.height as usize);
        bar_scroll.set_offset(
            state
                .scroll_offset()
                .min(visible.len().saturating_sub(inner.height as usize)),
        );
        crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
    }
}
