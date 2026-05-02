//! Rendering functions for the Table component.
//!
//! Extracted from the main table module to keep file sizes manageable.

use ratatui::prelude::*;
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Cell as RatatuiCell, Row};

use super::*;
use crate::component::cell::CellStyle;

/// Translates a semantic [`CellStyle`] into a concrete `ratatui::Style`.
///
/// `disabled` overrides everything (theme + per-cell semantics) with a
/// muted dark-gray foreground so a disabled table reads uniformly. Active
/// rendering maps the semantic variants to the theme's success/warning/
/// error styles where available, with a hardcoded dark-gray for `Muted`
/// (the theme does not expose a dedicated muted accessor at present).
fn cell_style_to_ratatui(style: &CellStyle, theme: &Theme, disabled: bool) -> Style {
    if disabled {
        return Style::default().fg(Color::DarkGray);
    }
    match style {
        CellStyle::Default => Style::default(),
        CellStyle::Success => theme.success_style(),
        CellStyle::Warning => theme.warning_style(),
        CellStyle::Error => theme.error_style(),
        CellStyle::Muted => Style::default().fg(Color::DarkGray),
        CellStyle::Custom(s) => *s,
    }
}

/// Renders the table into the given frame area.
pub(super) fn render_table<T: TableRow>(
    state: &TableState<T>,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    crate::annotation::with_registry(|reg| {
        let mut ann = crate::annotation::Annotation::table("table")
            .with_focus(focused)
            .with_disabled(disabled);
        if let Some(idx) = state.selected {
            ann = ann.with_selected(true).with_value(idx.to_string());
        }
        reg.register(area, ann);
    });

    let has_status = state.has_status_column();

    let header_style = if disabled {
        theme.disabled_style()
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    // Build header row with sort indicators, optionally prepending an
    // empty header cell for the status column.
    let mut header_cells: Vec<RatatuiCell> = Vec::new();
    if has_status {
        header_cells.push(RatatuiCell::from(""));
    }
    for (i, col) in state.columns.iter().enumerate() {
        let mut text = col.header().to_string();
        if let Some(pos) = state.sort_columns.iter().position(|&(c, _)| c == i) {
            let (_, dir) = state.sort_columns[pos];
            let arrow = match dir {
                SortDirection::Ascending => "\u{2191}",
                SortDirection::Descending => "\u{2193}",
            };
            if state.sort_columns.len() == 1 {
                // Single sort: just show arrow (backward compatible)
                text.push(' ');
                text.push_str(arrow);
            } else {
                // Multi-sort: show arrow with priority number
                text.push(' ');
                text.push_str(arrow);
                text.push_str(&(pos + 1).to_string());
            }
        }
        header_cells.push(RatatuiCell::from(text));
    }

    let header = Row::new(header_cells).style(header_style).bottom_margin(1);

    // Build data rows using display_order, applying per-cell styling and
    // optionally prepending the row-status indicator cell.
    let rows: Vec<Row> = state
        .display_order
        .iter()
        .map(|&idx| {
            let row = &state.rows[idx];
            let row_cells = row.cells();
            let mut cells: Vec<RatatuiCell> = Vec::with_capacity(row_cells.len() + 1);

            if has_status {
                match row.status().indicator() {
                    Some((symbol, color)) => {
                        let style = if disabled {
                            Style::default().fg(Color::DarkGray)
                        } else {
                            Style::default().fg(color)
                        };
                        cells.push(RatatuiCell::from(symbol.to_string()).style(style));
                    }
                    None => cells.push(RatatuiCell::from("")),
                }
            }

            for cell in row_cells {
                let style = cell_style_to_ratatui(cell.style(), theme, disabled);
                cells.push(RatatuiCell::from(cell.text().to_string()).style(style));
            }

            Row::new(cells)
        })
        .collect();

    let mut widths: Vec<Constraint> = Vec::new();
    if has_status {
        widths.push(Constraint::Length(2));
    }
    for col in state.columns.iter() {
        widths.push(col.width());
    }

    let border_style = if focused && !disabled {
        theme.focused_border_style()
    } else {
        theme.border_style()
    };

    let row_highlight_style = if disabled {
        theme.disabled_style()
    } else {
        theme.selected_highlight_style(focused)
    };

    let table = ratatui::widgets::Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .row_highlight_style(row_highlight_style)
        .highlight_symbol("> ");

    // Use TableState for stateful rendering
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(state.selected);
    frame.render_stateful_widget(table, area, &mut table_state);

    // Render scrollbar by mirroring the offset from ratatui's TableState
    let inner = area.inner(Margin::new(1, 1));
    // Viewport for data rows: inner height minus header row (1) and bottom margin (1)
    let data_viewport = (inner.height as usize).saturating_sub(2);
    if data_viewport > 0 && state.display_order.len() > data_viewport {
        let mut bar_scroll = ScrollState::new(state.display_order.len());
        bar_scroll.set_viewport_height(data_viewport);
        bar_scroll.set_offset(table_state.offset());
        crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
    }
}
