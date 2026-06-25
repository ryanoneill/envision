//! Rendering functions for the Table component.
//!
//! Extracted from the main table module to keep file sizes manageable.

use ratatui::prelude::*;
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Cell as RatatuiCell, Row};

use super::*;
use crate::component::cell::CellStyle;

/// Identifies columns whose declared lower-bound width constraint was
/// violated by the resolved layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ClippedColumn {
    pub idx: usize,
    pub declared: u16,
    pub resolved: u16,
    pub kind: ClipKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClipKind {
    Length,
    Min,
}

#[cfg(feature = "tracing")]
impl ClipKind {
    /// Renders as `"Length"` or `"Min"` for warning text.
    pub(crate) fn label(self) -> &'static str {
        match self {
            ClipKind::Length => "Length",
            ClipKind::Min => "Min",
        }
    }
}

/// Returns the set of columns whose declared lower-bound width
/// (`Length(n)` or `Min(n)`) was violated by the resolved layout.
///
/// `Length` declares an exact width that doubles as both upper and lower
/// bound; `Min` declares an explicit lower bound. Both are floors the
/// consumer wrote into their column declaration. `Percentage` is a share
/// of the resolved area with no absolute floor, so it is never flagged.
pub(crate) fn detect_clipped_columns(
    columns: &[Column],
    resolved_widths: &[u16],
) -> Vec<ClippedColumn> {
    use ratatui::layout::Constraint;

    columns
        .iter()
        .zip(resolved_widths.iter())
        .enumerate()
        .filter_map(|(idx, (col, &resolved))| {
            let (declared, kind) = match col.width() {
                Constraint::Length(n) if resolved < n => (n, ClipKind::Length),
                Constraint::Min(n) if resolved < n => (n, ClipKind::Min),
                _ => return None,
            };
            Some(ClippedColumn {
                idx,
                declared,
                resolved,
                kind,
            })
        })
        .collect()
}

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
        CellStyle::Severity(sev) => theme.severity_style(*sev),
    }
}

/// Renders the table into the given frame area.
///
/// `chrome_owned` signals that the parent has already drawn the outer
/// chrome (border, title, focus ring) for `area`. When true, the outer
/// `Block` draw is suppressed and data rendering proceeds against `area`
/// directly. The internal layout (header, rows, scrollbar) is unchanged.
pub(super) fn render_table<T: TableRow>(
    state: &TableState<T>,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
    chrome_owned: bool,
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

    // Best-effort clip-warning diagnostic: compute resolved column
    // widths via a one-shot Layout split that mirrors what ratatui
    // feeds the Table widget — full `widths` vec (incl. status
    // reservation) over the column-distribution area — then slice
    // off the status reservation before mapping back to user
    // columns. Detects Length/Min declared-floor violations, dedups
    // per (column index, area width) across the TableState's
    // lifetime, and emits a tracing warning on first detection.
    //
    // Skipped entirely when the table has no user columns.
    if !state.columns.is_empty() {
        // Mirror the full ratatui 0.29 Table width formula so detection
        // matches what the renderer actually distributes columns over.
        // Every term below corresponds to a row in the spec's "Canonical
        // reservation contract" table.
        //
        //   1. Border margin: when !chrome_owned the table is wrapped in
        //      Block::default().borders(Borders::ALL) below, which
        //      shrinks the inner rect by 1 cell on each side. When
        //      chrome_owned, `area` is already inner.
        //   2. Highlight-symbol reservation: the Table is constructed
        //      with .highlight_symbol("> ") (display width 2) and no
        //      explicit .highlight_spacing(...) call, so ratatui's
        //      default HighlightSpacing::WhenSelected applies — when
        //      state.selected.is_some(), ratatui reserves 2 cells from
        //      the column-distribution area before laying out columns.
        //   3. column_spacing: the Table is constructed with no explicit
        //      .column_spacing(...) call, so ratatui's default of 1 cell
        //      between columns applies. Layout::horizontal's default
        //      spacing is 0, so detection must opt into .spacing(1) to
        //      mirror what ratatui actually does — otherwise it
        //      over-distributes by (num_columns - 1) cells.
        //   4. has_status offset: see slicing comment below.
        //
        // Flex::Start is the default for both Table and Layout::horizontal
        // — no explicit .flex(...) call needed.
        const HIGHLIGHT_SYMBOL_WIDTH: u16 = 2; // matches "> " set at render.rs:153
        const COLUMN_SPACING: u16 = 1; // matches ratatui Table default; render.rs:150-153 sets no override
        let mut col_dist_area = if chrome_owned {
            area
        } else {
            area.inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            })
        };
        if state.selected.is_some() {
            col_dist_area.width = col_dist_area.width.saturating_sub(HIGHLIGHT_SYMBOL_WIDTH);
        }
        let resolved_rects = ratatui::layout::Layout::horizontal(widths.iter().copied())
            .spacing(COLUMN_SPACING)
            .split(col_dist_area);
        // Skip the status reservation when mapping back to user columns.
        // resolved_rects[has_status as usize..] aligns 1:1 with state.columns.
        let user_resolved: Vec<u16> = resolved_rects
            .iter()
            .skip(has_status as usize)
            .map(|r| r.width)
            .collect();
        let clipped = detect_clipped_columns(state.columns.as_slice(), &user_resolved);

        // Always track area width so resize re-arms detection, even
        // when the current render has no clipped columns.
        {
            let mut dedup = state.clip_warn_state.borrow_mut();
            if dedup.last_area_width != Some(area.width) {
                dedup.warned_cols.clear();
                dedup.last_area_width = Some(area.width);
            }
        }

        for clip in &clipped {
            let mut dedup = state.clip_warn_state.borrow_mut();
            if dedup.warned_cols.insert(clip.idx) {
                #[cfg(feature = "tracing")]
                tracing::warn!(
                    column_header = %state.columns[clip.idx].header(),
                    declared_kind = clip.kind.label(),
                    declared = clip.declared,
                    resolved = clip.resolved,
                    area_width = area.width,
                    "table column clipped: declared {}({}), resolved {} (table area {})",
                    clip.kind.label(),
                    clip.declared,
                    clip.resolved,
                    area.width,
                );
            }
        }
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

    let table_widget = ratatui::widgets::Table::new(rows, widths)
        .header(header)
        .row_highlight_style(row_highlight_style)
        .highlight_symbol("> ");

    let table_widget = if chrome_owned {
        table_widget
    } else {
        table_widget.block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        )
    };

    // Use TableState for stateful rendering
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(state.selected);
    frame.render_stateful_widget(table_widget, area, &mut table_state);

    // Render scrollbar. In chrome-owned mode the data already occupies the
    // full `area` (no border inset), so the scrollbar tracks `area` directly.
    // Otherwise it tracks the inset interior the outer Block carved out.
    let (inner, viewport_offset) = if chrome_owned {
        (area, 1) // header row only; no bottom border margin
    } else {
        (area.inner(Margin::new(1, 1)), 2) // header row + bottom margin
    };
    let data_viewport = (inner.height as usize).saturating_sub(viewport_offset);
    if data_viewport > 0 && state.display_order.len() > data_viewport {
        let mut bar_scroll = ScrollState::new(state.display_order.len());
        bar_scroll.set_viewport_height(data_viewport);
        bar_scroll.set_offset(table_state.offset());
        if chrome_owned {
            crate::scroll::render_scrollbar(&bar_scroll, frame, area, theme);
        } else {
            crate::scroll::render_scrollbar_inside_border(&bar_scroll, frame, area, theme);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Constraint;

    fn cols(widths: &[Constraint]) -> Vec<Column> {
        widths
            .iter()
            .enumerate()
            .map(|(i, &w)| Column::new(format!("col{i}"), w))
            .collect()
    }

    #[test]
    fn detect_clipped_columns_returns_empty_when_widths_fit() {
        let columns = cols(&[Constraint::Length(8), Constraint::Length(10)]);
        let resolved = vec![8, 10];
        assert!(detect_clipped_columns(&columns, &resolved).is_empty());
    }

    #[test]
    fn detect_clipped_columns_identifies_truncated_length_columns() {
        let columns = cols(&[Constraint::Length(20)]);
        let resolved = vec![10];
        let result = detect_clipped_columns(&columns, &resolved);
        assert_eq!(
            result,
            vec![ClippedColumn {
                idx: 0,
                declared: 20,
                resolved: 10,
                kind: ClipKind::Length,
            }]
        );
    }

    #[test]
    fn detect_clipped_columns_identifies_violated_min_constraints() {
        let columns = cols(&[Constraint::Min(20)]);
        let resolved = vec![10];
        let result = detect_clipped_columns(&columns, &resolved);
        assert_eq!(
            result,
            vec![ClippedColumn {
                idx: 0,
                declared: 20,
                resolved: 10,
                kind: ClipKind::Min,
            }]
        );
    }

    #[test]
    fn detect_clipped_columns_ignores_min_when_resolved_meets_floor() {
        let columns = cols(&[Constraint::Min(10)]);
        let resolved = vec![20];
        assert!(detect_clipped_columns(&columns, &resolved).is_empty());
    }

    #[test]
    fn detect_clipped_columns_ignores_percentage_constraints() {
        let columns = cols(&[Constraint::Percentage(50)]);
        let resolved = vec![5];
        assert!(detect_clipped_columns(&columns, &resolved).is_empty());
    }

    #[test]
    fn detect_clipped_columns_multiple_violations_mixed_kinds() {
        let columns = cols(&[
            Constraint::Length(20),
            Constraint::Min(20),
            Constraint::Percentage(50),
        ]);
        let resolved = vec![10, 10, 5];
        let result = detect_clipped_columns(&columns, &resolved);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].idx, 0);
        assert_eq!(result[0].kind, ClipKind::Length);
        assert_eq!(result[1].idx, 1);
        assert_eq!(result[1].kind, ClipKind::Min);
    }

    #[cfg(feature = "tracing")]
    #[test]
    fn clip_kind_label_returns_constraint_name() {
        assert_eq!(ClipKind::Length.label(), "Length");
        assert_eq!(ClipKind::Min.label(), "Min");
    }
}
