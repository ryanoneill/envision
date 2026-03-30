//! Rendering functions for the LoadingList component.
//!
//! Extracted from the main loading_list module to keep file sizes manageable.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use super::*;

/// Renders the loading list into the given frame area.
pub(super) fn render_loading_list<T: Clone>(
    state: &LoadingListState<T>,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    crate::annotation::with_registry(|reg| {
        let mut ann = crate::annotation::Annotation::loading_list("loading_list")
            .with_focus(state.focused)
            .with_disabled(state.disabled);
        if let Some(idx) = state.selected {
            ann = ann.with_selected(true).with_value(idx.to_string());
        }
        reg.register(area, ann);
    });

    let block = if let Some(title) = &state.title {
        Block::default().borders(Borders::ALL).title(title.as_str())
    } else {
        Block::default().borders(Borders::ALL)
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.items.is_empty() || inner.height == 0 {
        return;
    }

    // Construct a local ScrollState for virtual scrolling and scrollbar
    let mut bar_scroll = ScrollState::new(state.items.len());
    bar_scroll.set_viewport_height(inner.height as usize);
    if let Some(sel) = state.selected {
        bar_scroll.ensure_visible(sel);
    }
    let range = bar_scroll.visible_range();

    let items: Vec<ListItem> = state.items[range.clone()]
        .iter()
        .enumerate()
        .map(|(view_idx, item)| {
            let actual_idx = range.start + view_idx;
            let is_selected = state.selected == Some(actual_idx);
            let select_marker = if is_selected { "\u{25b8}" } else { " " };

            let content = if state.show_indicators {
                let state_symbol = item.state.symbol(state.spinner_frame);

                if let ItemState::Error(msg) = &item.state {
                    format!(
                        "{} {} {} - Error: {}",
                        select_marker, state_symbol, item.label, msg
                    )
                } else {
                    format!("{} {} {}", select_marker, state_symbol, item.label)
                }
            } else if let ItemState::Error(msg) = &item.state {
                format!("{} {} - Error: {}", select_marker, item.label, msg)
            } else {
                format!("{} {}", select_marker, item.label)
            };

            let style = if is_selected {
                theme.focused_bold_style()
            } else {
                item.state.style(theme)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);

    // Render scrollbar when content exceeds viewport
    crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
}
