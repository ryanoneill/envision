//! Rendering helpers for the SearchableList component.

use std::fmt::Display;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::{Focus, SearchableListState};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Renders the searchable list component.
///
/// `chrome_owned` signals that the parent has already drawn the outer
/// chrome for `area`. When true, the list panel's outer `Block` draw is
/// suppressed; the filter input retains its decorative border (the
/// filter is a functional input widget, not chrome around content).
pub(super) fn render_searchable_list<T: Clone + Display>(
    state: &SearchableListState<T>,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
    chrome_owned: bool,
) {
    crate::annotation::with_registry(|reg| {
        reg.open(
            area,
            crate::annotation::Annotation::new(crate::annotation::WidgetType::SearchableList)
                .with_id("searchable_list")
                .with_focus(focused)
                .with_disabled(disabled),
        );
    });

    // Split area: filter input on top (3 lines), list below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // Render filter input
    let filter_focused = focused && state.internal_focus == Focus::Filter;
    let filter_border_style = if disabled {
        theme.disabled_style()
    } else if filter_focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let filter_display = if state.filter_text.is_empty() {
        Span::styled(&state.placeholder, theme.disabled_style())
    } else {
        Span::styled(&state.filter_text, theme.normal_style())
    };

    let match_count = format!(" {}/{} ", state.filtered_indices.len(), state.items.len());
    let filter_block = Block::default()
        .borders(Borders::ALL)
        .border_style(filter_border_style)
        .title(Span::styled(" Filter ", theme.normal_style()))
        .title_bottom(Line::from(match_count).alignment(Alignment::Right));

    let filter_widget = Paragraph::new(Line::from(filter_display)).block(filter_block);
    frame.render_widget(filter_widget, chunks[0]);

    // Show cursor in filter when focused
    if filter_focused && !disabled {
        let cursor_x = chunks[0].x + 1 + state.filter_text.len() as u16;
        let cursor_y = chunks[0].y + 1;
        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
    }

    // Render filtered list
    let list_focused = focused && state.internal_focus == Focus::List;
    let list_border_style = if disabled {
        theme.disabled_style()
    } else if list_focused {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let items: Vec<ListItem> = state
        .filtered_indices
        .iter()
        .filter_map(|&i| state.items.get(i))
        .map(|item| ListItem::new(format!("{}", item)))
        .collect();

    let highlight_style = if disabled {
        theme.disabled_style()
    } else {
        theme.selected_highlight_style(list_focused)
    };

    let mut list_widget = List::new(items)
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    let list_inner = if chrome_owned {
        chunks[1]
    } else {
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(list_border_style);

        let inner = list_block.inner(chunks[1]);
        list_widget = list_widget.block(list_block);
        inner
    };

    let mut list_state = state.list_state.clone();
    frame.render_stateful_widget(list_widget, chunks[1], &mut list_state);

    // Render scrollbar when content exceeds viewport. In chrome-owned mode
    // the data already occupies the full list area (no border inset).
    if state.filtered_indices.len() > list_inner.height as usize {
        let mut bar_scroll = ScrollState::new(state.filtered_indices.len());
        bar_scroll.set_viewport_height(list_inner.height as usize);
        bar_scroll.set_offset(list_state.offset());
        if chrome_owned {
            crate::scroll::render_scrollbar(&bar_scroll, frame, chunks[1], theme);
        } else {
            crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, chunks[1], theme);
        }
    }

    crate::annotation::with_registry(|reg| {
        reg.close();
    });
}
