//! Rendering helpers for the CommandPalette component.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use super::CommandPaletteState;
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Renders the command palette component as a centered overlay.
pub(super) fn render_command_palette(
    state: &CommandPaletteState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    if !state.visible {
        return;
    }

    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::command_palette("command_palette")
                .with_focus(focused)
                .with_disabled(disabled)
                .with_expanded(state.visible),
        );
    });

    let visible_count = state.filtered_indices.len().min(state.max_visible);
    // Height = 3 (input with borders) + 1 (separator) + visible_count (items) + 1 (bottom border)
    let palette_height = (3 + 1 + visible_count + 1) as u16;
    let palette_height = palette_height.min(area.height);

    // Width: use ~60% of the available area, minimum 30, maximum 80
    let palette_width = area.width.saturating_mul(6) / 10;
    let palette_width = palette_width.clamp(30, 80).min(area.width);

    // Center the palette in the available area
    let x = area.x + (area.width.saturating_sub(palette_width)) / 2;
    let y = area.y + area.height.saturating_sub(palette_height) / 4; // slightly above center

    let palette_area = Rect {
        x,
        y,
        width: palette_width,
        height: palette_height,
    };

    // Clear the area behind the palette
    frame.render_widget(Clear, palette_area);

    let border_style = if focused && !disabled {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    // Build the outer block with title
    let title_text = state.title.as_deref().unwrap_or("Command Palette");
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(
            format!(" {} ", title_text),
            theme.normal_style(),
        ));

    let inner = outer_block.inner(palette_area);
    frame.render_widget(outer_block, palette_area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Split inner area: input line, separator, items
    let input_height = 1u16;
    let separator_height = 1u16;

    let input_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: input_height.min(inner.height),
    };

    // Render input line: "> query_" or "> placeholder"
    let input_content = if state.query.is_empty() {
        Line::from(vec![
            Span::styled("> ", theme.normal_style()),
            Span::styled(&state.placeholder, theme.placeholder_style()),
        ])
    } else {
        Line::from(vec![
            Span::styled("> ", theme.normal_style()),
            Span::styled(&state.query, theme.normal_style()),
        ])
    };
    let input_widget = Paragraph::new(input_content);
    frame.render_widget(input_widget, input_area);

    // Cursor position
    if focused && !disabled {
        let cursor_x = input_area.x + 2 + state.query.len() as u16;
        let cursor_y = input_area.y;
        if cursor_x < input_area.x + input_area.width {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }

    // Separator line
    let remaining_after_input = inner.height.saturating_sub(input_height);
    if remaining_after_input == 0 {
        return;
    }

    let separator_area = Rect {
        x: palette_area.x,
        y: inner.y + input_height,
        width: palette_area.width,
        height: separator_height.min(remaining_after_input),
    };

    let separator_line = "─".repeat(palette_area.width as usize);
    let separator = Paragraph::new(separator_line).style(border_style);
    frame.render_widget(separator, separator_area);

    // Items area
    let items_y = inner.y + input_height + separator_height;
    let items_height = inner.height.saturating_sub(input_height + separator_height);
    if items_height == 0 {
        return;
    }

    let items_area = Rect {
        x: inner.x,
        y: items_y,
        width: inner.width,
        height: items_height,
    };

    if state.filtered_indices.is_empty() {
        let no_match = Paragraph::new("  No matches").style(theme.placeholder_style());
        frame.render_widget(no_match, items_area);
        return;
    }

    // Determine the scroll window
    let total = state.filtered_indices.len();
    let viewport = items_height as usize;
    let selected = state.selected.unwrap_or(0);

    // Calculate offset to keep selected item in view
    let offset = if selected < viewport {
        0
    } else {
        selected.saturating_sub(viewport - 1)
    };

    let visible_range = offset..total.min(offset + viewport);

    let list_items: Vec<ListItem> = state
        .filtered_indices
        .iter()
        .enumerate()
        .skip(visible_range.start)
        .take(visible_range.end - visible_range.start)
        .map(|(fi, &item_idx)| {
            let item = &state.items[item_idx];
            let is_selected = state.selected == Some(fi);
            let prefix = if is_selected { "\u{25b8} " } else { "  " };

            let available_width = items_area.width as usize;

            // Build the line with label and optional shortcut right-aligned
            let label_part = format!("{}{}", prefix, item.label);

            let line = if let Some(ref shortcut) = item.shortcut {
                let shortcut_len = shortcut.len();
                let label_display_len = label_part.len();
                // At least 2 spaces between label and shortcut
                let gap = available_width
                    .saturating_sub(label_display_len)
                    .saturating_sub(shortcut_len);
                if gap >= 2 {
                    Line::from(vec![
                        Span::raw(label_part),
                        Span::raw(" ".repeat(gap)),
                        Span::styled(shortcut, theme.placeholder_style()),
                    ])
                } else {
                    Line::from(label_part)
                }
            } else {
                Line::from(label_part)
            };

            let style = if is_selected {
                theme.selected_style(focused)
            } else {
                theme.normal_style()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(list_items);
    frame.render_widget(list, items_area);

    // Scrollbar if content exceeds viewport
    if total > viewport {
        let mut bar_scroll = ScrollState::new(total);
        bar_scroll.set_viewport_height(viewport);
        bar_scroll.set_offset(offset);
        // Render scrollbar on the right edge of the palette area
        crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, palette_area, theme);
    }
}
