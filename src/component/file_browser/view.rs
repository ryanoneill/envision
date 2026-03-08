//! Rendering logic for the [`FileBrowser`](super::FileBrowser) component.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::types::FileBrowserFocus;
use super::{format_size, FileBrowserState};
use crate::theme::Theme;

/// Renders the file browser component into the given frame area.
pub(super) fn render(state: &FileBrowserState, frame: &mut Frame, area: Rect, theme: &Theme) {
    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::new(crate::annotation::WidgetType::FileBrowser)
                .with_id("file_browser")
                .with_focus(state.is_focused())
                .with_disabled(state.is_disabled()),
        );
    });

    let border_style = if state.is_disabled() {
        theme.disabled_style()
    } else if state.is_focused() {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Layout: path bar (1 line) | filter (1 line if active) | listing
    let internal_focus = state.internal_focus();
    let has_filter =
        !state.filter_text().is_empty() || *internal_focus == FileBrowserFocus::Filter;

    let path_height = 1u16;
    let filter_height = if has_filter { 1u16 } else { 0u16 };
    let list_height = inner.height.saturating_sub(path_height + filter_height);

    let path_area = Rect::new(inner.x, inner.y, inner.width, path_height);
    let filter_area = if has_filter {
        Rect::new(inner.x, inner.y + path_height, inner.width, filter_height)
    } else {
        Rect::ZERO
    };
    let list_area = Rect::new(
        inner.x,
        inner.y + path_height + filter_height,
        inner.width,
        list_height,
    );

    // Render path bar
    let path_text = state.path_segments().join(" / ");
    let path_style = if *internal_focus == FileBrowserFocus::PathBar && state.is_focused() {
        theme.focused_style()
    } else {
        theme.info_style()
    };
    frame.render_widget(Paragraph::new(path_text).style(path_style), path_area);

    // Render filter
    if has_filter {
        let filter_display = format!("Filter: {}", state.filter_text());
        let filter_style = if *internal_focus == FileBrowserFocus::Filter && state.is_focused() {
            theme.focused_style()
        } else {
            theme.normal_style()
        };
        frame.render_widget(
            Paragraph::new(filter_display).style(filter_style),
            filter_area,
        );
    }

    // Render file listing
    let entries = state.entries();
    let selected_paths = state.selected_paths();
    let items: Vec<ListItem> = state
        .filtered_indices()
        .iter()
        .map(|&i| {
            let entry = &entries[i];
            let icon = if entry.is_dir() { "📁" } else { "📄" };
            let size_str = entry.size().map(format_size).unwrap_or_default();
            let selected_marker = if selected_paths.contains(&entry.path().to_string()) {
                "✓ "
            } else {
                "  "
            };
            let text = if size_str.is_empty() {
                format!("{}{} {}", selected_marker, icon, entry.name())
            } else {
                format!("{}{} {}  {}", selected_marker, icon, entry.name(), size_str)
            };
            ListItem::new(text)
        })
        .collect();

    let list_style = if *internal_focus == FileBrowserFocus::FileList && state.is_focused() {
        theme.focused_style()
    } else {
        theme.normal_style()
    };

    let highlight_style = theme.selected_style(state.is_focused());
    let list = List::new(items)
        .style(list_style)
        .highlight_style(highlight_style);

    let mut ls = state.list_state.clone();
    frame.render_stateful_widget(list, list_area, &mut ls);
}
