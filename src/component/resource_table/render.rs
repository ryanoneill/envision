//! Rendering for the ResourceTable component.

use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table as RatatuiTable};

use super::{ResourceRow, ResourceTableState};
use crate::component::context::RenderContext;
use crate::scroll::render_scrollbar_inside_border;

/// Renders a ResourceTable.
pub(super) fn render<T: ResourceRow>(
    state: &ResourceTableState<T>,
    ctx: &mut RenderContext<'_, '_>,
) {
    let disabled = ctx.disabled;
    let focused = ctx.focused;

    // Border
    let border_style = if disabled {
        ctx.theme.disabled_style()
    } else if focused {
        ctx.theme.focused_border_style()
    } else {
        ctx.theme.border_style()
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    if let Some(title) = state.title() {
        block = block.title(format!(" {} ", title));
    }

    let inner = block.inner(ctx.area);
    ctx.frame.render_widget(block, ctx.area);

    if inner.width < 3 || inner.height < 1 {
        return;
    }

    if state.rows().is_empty() {
        let msg = Paragraph::new("(no rows)").style(ctx.theme.disabled_style());
        ctx.frame.render_widget(msg, inner);
        return;
    }

    // Update scroll viewport based on visible height (minus header row)
    let content_height = inner.height.saturating_sub(1);
    let mut scroll = state.scroll().clone();
    scroll.set_viewport_height(content_height as usize);
    scroll.set_content_length(state.rows().len());
    if let Some(sel) = state.selected() {
        scroll.ensure_visible(sel);
    }

    // Determine columns with optional status column prepended
    let has_status = state.has_status_column();
    let status_width = if has_status { 2 } else { 0 };

    // Build ratatui column widths
    let mut widths: Vec<Constraint> = Vec::new();
    if has_status {
        widths.push(Constraint::Length(status_width));
    }
    for col in state.columns() {
        widths.push(col.width());
    }

    // Build header row
    let header_style = if disabled {
        ctx.theme.disabled_style()
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    let mut header_cells: Vec<Cell> = Vec::new();
    if has_status {
        header_cells.push(Cell::from(""));
    }
    for col in state.columns() {
        header_cells.push(Cell::from(col.header().to_string()).style(header_style));
    }
    let header = Row::new(header_cells).style(header_style);

    // Build data rows from visible range
    let visible_range = scroll.visible_range();
    let rows: Vec<Row> = state
        .rows()
        .iter()
        .enumerate()
        .skip(visible_range.start)
        .take(visible_range.end.saturating_sub(visible_range.start))
        .map(|(idx, row)| build_row(idx, row, has_status, disabled))
        .collect();

    // Row highlight for selection (relative to visible window)
    let row_highlight_style = if disabled {
        ctx.theme.disabled_style()
    } else if focused {
        ctx.theme.focused_style().add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    // Compute the selected index within the visible slice
    let mut table_state = ratatui::widgets::TableState::default();
    if let Some(sel) = state.selected() {
        if sel >= visible_range.start && sel < visible_range.end {
            table_state.select(Some(sel - visible_range.start));
        }
    }

    let table = RatatuiTable::new(rows, widths)
        .header(header)
        .row_highlight_style(row_highlight_style);

    ctx.frame
        .render_stateful_widget(table, inner, &mut table_state);

    // Scrollbar
    if scroll.can_scroll() {
        render_scrollbar_inside_border(&scroll, ctx.frame, ctx.area, ctx.theme);
    }
}

/// Builds a single row (header or data) with per-cell styling.
fn build_row<T: ResourceRow>(
    _idx: usize,
    row: &T,
    has_status: bool,
    disabled: bool,
) -> Row<'static> {
    let mut cells: Vec<Cell<'static>> = Vec::new();

    // Status indicator column
    if has_status {
        match row.status().indicator() {
            Some((symbol, color)) => {
                let style = if disabled {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(color)
                };
                cells.push(Cell::from(symbol.to_string()).style(style));
            }
            None => cells.push(Cell::from("")),
        }
    }

    // Data cells
    for cell in row.cells() {
        let style = if disabled {
            Style::default().fg(Color::DarkGray)
        } else {
            cell.style().to_style()
        };
        cells.push(Cell::from(cell.text().to_string()).style(style));
    }

    Row::new(cells)
}
