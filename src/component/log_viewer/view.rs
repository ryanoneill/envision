use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::{LogViewerState, StatusLogLevel};
use crate::component::RenderContext;

/// Renders the search bar area.
pub(super) fn render_search_bar(state: &LogViewerState, ctx: &mut RenderContext<'_, '_>) {
    let search_style = if ctx.disabled {
        ctx.theme.disabled_style()
    } else if state.is_search_focused() {
        ctx.theme.focused_style()
    } else {
        ctx.theme.normal_style()
    };

    // Build the search prefix with regex indicator
    let prefix = if state.use_regex() { "/r " } else { "/ " };

    let display = if state.search_text().is_empty() {
        if state.use_regex() {
            "/r Search (regex)...".to_string()
        } else {
            "/ Search...".to_string()
        }
    } else {
        format!("{}{}", prefix, state.search_value())
    };

    let paragraph = Paragraph::new(display).style(search_style);
    ctx.frame.render_widget(paragraph, ctx.area);

    // Show cursor when search is focused
    if ctx.focused && state.is_search_focused() && !ctx.disabled {
        let prefix_len = prefix.len() as u16;
        let cursor_x = ctx.area.x + prefix_len + state.search_cursor_position() as u16;
        if cursor_x < ctx.area.right() {
            ctx.frame
                .set_cursor_position(Position::new(cursor_x, ctx.area.y));
        }
    }
}

/// Renders the filter bar showing which severity levels are active.
pub(super) fn render_filter_bar(state: &LogViewerState, ctx: &mut RenderContext<'_, '_>) {
    let filter_style = if ctx.disabled {
        ctx.theme.disabled_style()
    } else {
        ctx.theme.normal_style()
    };

    let info_marker = if state.show_info() { "●" } else { "○" };
    let success_marker = if state.show_success() { "●" } else { "○" };
    let warning_marker = if state.show_warning() { "●" } else { "○" };
    let error_marker = if state.show_error() { "●" } else { "○" };

    let follow_indicator = if state.follow() { " FOLLOW" } else { "" };

    let spans = vec![
        Span::styled(
            format!("1:{} Info ", info_marker),
            if ctx.disabled {
                filter_style
            } else {
                Style::default().fg(StatusLogLevel::Info.color())
            },
        ),
        Span::styled(
            format!("2:{} Success ", success_marker),
            if ctx.disabled {
                filter_style
            } else {
                Style::default().fg(StatusLogLevel::Success.color())
            },
        ),
        Span::styled(
            format!("3:{} Warning ", warning_marker),
            if ctx.disabled {
                filter_style
            } else {
                Style::default().fg(StatusLogLevel::Warning.color())
            },
        ),
        Span::styled(
            format!("4:{} Error", error_marker),
            if ctx.disabled {
                filter_style
            } else {
                Style::default().fg(StatusLogLevel::Error.color())
            },
        ),
        Span::styled(
            follow_indicator,
            if ctx.disabled {
                filter_style
            } else {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            },
        ),
    ];

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    ctx.frame.render_widget(paragraph, ctx.area);
}

/// Renders the log entries area.
pub(super) fn render_log(state: &LogViewerState, ctx: &mut RenderContext<'_, '_>) {
    let visible = state.visible_entries();

    let border_style = if ctx.disabled {
        ctx.theme.disabled_style()
    } else if ctx.focused && !state.is_search_focused() {
        ctx.theme.focused_border_style()
    } else {
        ctx.theme.border_style()
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

    let inner = block.inner(ctx.area);
    ctx.frame.render_widget(block, ctx.area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let disabled = ctx.disabled;
    let items: Vec<ListItem> = visible
        .iter()
        .skip(state.scroll_offset())
        .take(inner.height as usize)
        .map(|entry| {
            let style = if disabled {
                ctx.theme.disabled_style()
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
            if !state.search_text().is_empty() && !disabled {
                let is_match = {
                    #[cfg(feature = "regex")]
                    {
                        if state.use_regex() {
                            regex::RegexBuilder::new(state.search_text())
                                .case_insensitive(true)
                                .build()
                                .map(|re| re.is_match(&text))
                                .unwrap_or_else(|_| {
                                    text.to_lowercase()
                                        .contains(&state.search_text().to_lowercase())
                                })
                        } else {
                            text.to_lowercase()
                                .contains(&state.search_text().to_lowercase())
                        }
                    }
                    #[cfg(not(feature = "regex"))]
                    {
                        text.to_lowercase()
                            .contains(&state.search_text().to_lowercase())
                    }
                };
                if is_match {
                    let style = style.add_modifier(Modifier::BOLD);
                    return ListItem::new(text).style(style);
                }
            }

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items);
    ctx.frame.render_widget(list, inner);

    // Render scrollbar if content exceeds viewport
    if visible.len() > inner.height as usize {
        let mut bar_scroll = crate::scroll::ScrollState::new(visible.len());
        bar_scroll.set_viewport_height(inner.height as usize);
        bar_scroll.set_offset(
            state
                .scroll_offset()
                .min(visible.len().saturating_sub(inner.height as usize)),
        );
        crate::scroll::render_scrollbar_inside_border(&bar_scroll, ctx.frame, ctx.area, ctx.theme);
    }
}
