//! A data table component with row selection and column sorting.
//!
//! [`Table<T>`] provides a tabular data display with keyboard navigation,
//! row selection, and column sorting capabilities. State is stored in
//! [`TableState<T>`], updated via [`TableMessage`], and produces [`TableOutput`].
//!
//!
//! See also [`DataGrid`](super::DataGrid) for a table with inline cell editing.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Cell, Column, Component, SortDirection, Table, TableMessage, TableOutput,
//!     TableRow, TableState,
//! };
//! use ratatui::layout::Constraint;
//!
//! // Define your row type
//! #[derive(Clone, Debug, PartialEq)]
//! struct User {
//!     name: String,
//!     email: String,
//! }
//!
//! impl TableRow for User {
//!     fn cells(&self) -> Vec<Cell> {
//!         vec![Cell::new(&self.name), Cell::new(&self.email)]
//!     }
//! }
//!
//! // Create table state
//! let users = vec![
//!     User { name: "Alice".into(), email: "alice@example.com".into() },
//!     User { name: "Bob".into(), email: "bob@example.com".into() },
//! ];
//!
//! let columns = vec![
//!     Column::new("Name", Constraint::Length(15)).sortable(),
//!     Column::new("Email", Constraint::Length(25)),
//! ];
//!
//! let mut state = TableState::new(users, columns);
//!
//! // Navigate down
//! let output = Table::<User>::update(&mut state, TableMessage::Down);
//! assert_eq!(output, Some(TableOutput::SelectionChanged(1)));
//!
//! // Sort by name column
//! let output = Table::<User>::update(&mut state, TableMessage::SortAsc(0));
//! assert_eq!(output, Some(TableOutput::Sorted {
//!     column: 0,
//!     direction: SortDirection::Ascending,
//! }));
//! ```

mod render;
mod state;
mod types;

pub use types::{Column, InitialSort, SortDirection, TableMessage, TableOutput, TableRow};

use std::collections::HashSet;
use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Minimum column width in characters for column resizing.
const MIN_COLUMN_WIDTH: u16 = 3;

/// State for a Table component.
///
/// Holds the rows, columns, selection state, and sort configuration.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TableState<T: TableRow> {
    rows: Vec<T>,
    columns: Vec<Column>,
    selected: Option<usize>,
    sort_columns: Vec<(usize, SortDirection)>,
    display_order: Vec<usize>,
    filter_text: String,
    #[cfg_attr(feature = "serialization", serde(skip))]
    scroll: ScrollState,
    /// Dedup keys for cross-variant `SortKey` warnings: column indices
    /// that already emitted a warning during the current render pass.
    /// Cleared at the start of each `rebuild_display_order` call so
    /// each pass emits at most one warning per affected column.
    ///
    /// Runtime-only state; not part of logical equality and not
    /// serialized.
    #[cfg_attr(feature = "serialization", serde(skip))]
    cross_variant_warned_cols: HashSet<usize>,
}

impl<T: TableRow + PartialEq> PartialEq for TableState<T> {
    fn eq(&self, other: &Self) -> bool {
        // `cross_variant_warned_cols` is intentionally excluded — it's
        // transient, render-pass-scoped diagnostics state, not part of
        // the logical equality of the table.
        self.rows == other.rows
            && self.columns == other.columns
            && self.selected == other.selected
            && self.sort_columns == other.sort_columns
            && self.display_order == other.display_order
            && self.filter_text == other.filter_text
    }
}

impl<T: TableRow> Default for TableState<T> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            columns: Vec::new(),
            selected: None,
            sort_columns: Vec::new(),
            display_order: Vec::new(),
            filter_text: String::new(),
            scroll: ScrollState::default(),
            cross_variant_warned_cols: HashSet::new(),
        }
    }
}

/// A data table component with row selection and column sorting.
///
/// `Table` displays tabular data with support for keyboard navigation,
/// single row selection, and column sorting. It uses a generic row type
/// that implements the [`TableRow`] trait.
///
/// # Type Parameters
///
/// - `T`: The row data type. Must implement [`TableRow`] and `Clone`.
///
/// # Navigation
///
/// - `Up` / `Down` - Move selection by one row
/// - `First` / `Last` - Jump to beginning/end
/// - `PageUp` / `PageDown` - Move by page size
/// - `Select` - Confirm the current selection
/// - `SortAsc(column)` / `SortDesc(column)` / `SortToggle(column)` - Sort by the given column
/// - `SortClear` - Clear the current sort
///
/// # Sorting
///
/// `SortToggle` flips Ascending <-> Descending without clearing.
/// Only columns marked as `sortable()` can be sorted.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Cell, Column, Component, Table, TableMessage, TableRow, TableState,
/// };
/// use ratatui::layout::Constraint;
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// impl TableRow for Person {
///     fn cells(&self) -> Vec<Cell> {
///         vec![Cell::new(&self.name), Cell::uint(self.age as u64)]
///     }
/// }
///
/// let people = vec![
///     Person { name: "Alice".into(), age: 30 },
///     Person { name: "Bob".into(), age: 25 },
/// ];
///
/// let columns = vec![
///     Column::new("Name", Constraint::Length(15)).sortable(),
///     Column::new("Age", Constraint::Length(5)).sortable(),
/// ];
///
/// let mut state = TableState::new(people, columns);
///
/// // Navigate and select
/// Table::<Person>::update(&mut state, TableMessage::Down);
/// let output = Table::<Person>::update(&mut state, TableMessage::Select);
/// // output is Some(TableOutput::Selected(Person { name: "Bob", age: 25 }))
/// ```
pub struct Table<T: TableRow>(PhantomData<T>);

impl<T: TableRow + 'static> Component for Table<T> {
    type State = TableState<T>;
    type Message = TableMessage;
    type Output = TableOutput<T>;

    fn init() -> Self::State {
        TableState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            TableMessage::SetFilter(ref text) => {
                state.set_filter_text(text);
                return Some(TableOutput::FilterChanged(text.clone()));
            }
            TableMessage::ClearFilter => {
                state.clear_filter();
                return Some(TableOutput::FilterChanged(String::new()));
            }
            _ => {}
        }

        if state.display_order.is_empty() {
            return None;
        }

        let len = state.display_order.len();
        let current = state.selected.unwrap_or(0);

        match msg {
            TableMessage::Up => {
                if current > 0 {
                    let new_index = current - 1;
                    state.selected = Some(new_index);
                    return Some(TableOutput::SelectionChanged(new_index));
                }
            }
            TableMessage::Down => {
                if current < len - 1 {
                    let new_index = current + 1;
                    state.selected = Some(new_index);
                    return Some(TableOutput::SelectionChanged(new_index));
                }
            }
            TableMessage::First => {
                if current != 0 {
                    state.selected = Some(0);
                    return Some(TableOutput::SelectionChanged(0));
                }
            }
            TableMessage::Last => {
                let last = len - 1;
                if current != last {
                    state.selected = Some(last);
                    return Some(TableOutput::SelectionChanged(last));
                }
            }
            TableMessage::PageUp(page_size) => {
                let new_index = current.saturating_sub(page_size);
                if new_index != current {
                    state.selected = Some(new_index);
                    return Some(TableOutput::SelectionChanged(new_index));
                }
            }
            TableMessage::PageDown(page_size) => {
                let new_index = (current + page_size).min(len - 1);
                if new_index != current {
                    state.selected = Some(new_index);
                    return Some(TableOutput::SelectionChanged(new_index));
                }
            }
            TableMessage::Select => {
                if let Some(row) = state.selected_row().cloned() {
                    return Some(TableOutput::Selected(row));
                }
            }
            TableMessage::IncreaseColumnWidth(col) => {
                if let Some(column) = state.columns.get_mut(col) {
                    if let Constraint::Length(w) = column.width() {
                        let new_width = w.saturating_add(1);
                        column.set_width(Constraint::Length(new_width));
                        return Some(TableOutput::ColumnResized {
                            column: col,
                            width: new_width,
                        });
                    }
                }
            }
            TableMessage::DecreaseColumnWidth(col) => {
                if let Some(column) = state.columns.get_mut(col) {
                    if let Constraint::Length(w) = column.width() {
                        let new_width = w.saturating_sub(1).max(MIN_COLUMN_WIDTH);
                        if new_width != w {
                            column.set_width(Constraint::Length(new_width));
                            return Some(TableOutput::ColumnResized {
                                column: col,
                                width: new_width,
                            });
                        }
                    }
                }
            }
            TableMessage::SetFilter(_) | TableMessage::ClearFilter => {
                unreachable!("handled above")
            }
            TableMessage::SortAsc(col) => {
                if let Some(column) = state.columns.get(col) {
                    if !column.is_sortable() {
                        return None;
                    }
                    // Idempotent: already (col, Ascending) primary → no-op.
                    if state.sort_columns.first() == Some(&(col, SortDirection::Ascending)) {
                        return None;
                    }
                    state.sort_columns = vec![(col, SortDirection::Ascending)];
                    state.rebuild_display_order();
                    return Some(TableOutput::Sorted {
                        column: col,
                        direction: SortDirection::Ascending,
                    });
                }
            }
            TableMessage::SortDesc(col) => {
                if let Some(column) = state.columns.get(col) {
                    if !column.is_sortable() {
                        return None;
                    }
                    // Idempotent: already (col, Descending) primary → no-op.
                    if state.sort_columns.first() == Some(&(col, SortDirection::Descending)) {
                        return None;
                    }
                    state.sort_columns = vec![(col, SortDirection::Descending)];
                    state.rebuild_display_order();
                    return Some(TableOutput::Sorted {
                        column: col,
                        direction: SortDirection::Descending,
                    });
                }
            }
            TableMessage::SortToggle(col) => {
                if let Some(column) = state.columns.get(col) {
                    if !column.is_sortable() {
                        return None;
                    }
                    let primary = state.sort_columns.first().copied();
                    let new_dir = match primary {
                        Some((c, dir)) if c == col => dir.toggle(),
                        _ => column.default_sort(),
                    };
                    state.sort_columns = vec![(col, new_dir)];
                    state.rebuild_display_order();
                    return Some(TableOutput::Sorted {
                        column: col,
                        direction: new_dir,
                    });
                }
            }
            TableMessage::SortClear => {
                if !state.sort_columns.is_empty() {
                    state.sort_columns.clear();
                    state.rebuild_display_order();
                    return Some(TableOutput::SortCleared);
                }
            }
            TableMessage::RemoveSort(col) => {
                if let Some(pos) = state.sort_columns.iter().position(|&(c, _)| c == col) {
                    let was_primary = pos == 0;
                    state.sort_columns.remove(pos);
                    state.rebuild_display_order();
                    if state.sort_columns.is_empty() {
                        return Some(TableOutput::SortCleared);
                    }
                    if was_primary {
                        // Primary removed; next entry is promoted to primary.
                        let (next_col, next_dir) = state.sort_columns[0];
                        return Some(TableOutput::Sorted {
                            column: next_col,
                            direction: next_dir,
                        });
                    }
                    // Tiebreaker removed; primary unchanged → no observable
                    // change in primary sort, emit None.
                    return None;
                }
            }
            TableMessage::AddSortAsc(col) => {
                if let Some(column) = state.columns.get(col) {
                    if !column.is_sortable() {
                        return None;
                    }
                    if let Some(pos) = state.sort_columns.iter().position(|&(c, _)| c == col) {
                        // Idempotent: already Ascending in stack at this slot.
                        if state.sort_columns[pos].1 == SortDirection::Ascending {
                            return None;
                        }
                        // Replace direction in place; preserve stack position.
                        state.sort_columns[pos].1 = SortDirection::Ascending;
                    } else {
                        state.sort_columns.push((col, SortDirection::Ascending));
                    }
                    state.rebuild_display_order();
                    return Some(TableOutput::Sorted {
                        column: col,
                        direction: SortDirection::Ascending,
                    });
                }
            }
            TableMessage::AddSortDesc(col) => {
                if let Some(column) = state.columns.get(col) {
                    if !column.is_sortable() {
                        return None;
                    }
                    if let Some(pos) = state.sort_columns.iter().position(|&(c, _)| c == col) {
                        // Idempotent: already Descending in stack at this slot.
                        if state.sort_columns[pos].1 == SortDirection::Descending {
                            return None;
                        }
                        // Replace direction in place; preserve stack position.
                        state.sort_columns[pos].1 = SortDirection::Descending;
                    } else {
                        state.sort_columns.push((col, SortDirection::Descending));
                    }
                    state.rebuild_display_order();
                    return Some(TableOutput::Sorted {
                        column: col,
                        direction: SortDirection::Descending,
                    });
                }
            }
            TableMessage::AddSortToggle(col) => {
                if let Some(column) = state.columns.get(col) {
                    if !column.is_sortable() {
                        return None;
                    }
                    let new_dir =
                        if let Some(pos) = state.sort_columns.iter().position(|&(c, _)| c == col) {
                            let (_, dir) = state.sort_columns[pos];
                            let toggled = dir.toggle();
                            state.sort_columns[pos] = (col, toggled);
                            toggled
                        } else {
                            let dir = column.default_sort();
                            state.sort_columns.push((col, dir));
                            dir
                        };
                    state.rebuild_display_order();
                    return Some(TableOutput::Sorted {
                        column: col,
                        direction: new_dir,
                    });
                }
            }
        }

        None
    }

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            let has_shift = key.modifiers.shift();
            match key.code {
                Key::Up | Key::Char('k') => Some(TableMessage::Up),
                Key::Down | Key::Char('j') => Some(TableMessage::Down),
                Key::Home => Some(TableMessage::First),
                Key::End => Some(TableMessage::Last),
                Key::Enter if has_shift => {
                    // Shift+Enter adds the current primary sort column to the sort stack
                    // This is a no-op if there's no selection context for a column
                    None
                }
                Key::Enter => Some(TableMessage::Select),
                Key::Char('+') => {
                    // Increase the width of the currently selected column
                    // Uses the primary sort column index, or column 0 if no sort
                    let col = state.sort_columns.first().map(|&(c, _)| c).unwrap_or(0);
                    Some(TableMessage::IncreaseColumnWidth(col))
                }
                Key::Char('-') => {
                    // Decrease the width of the currently selected column
                    let col = state.sort_columns.first().map(|&(c, _)| c).unwrap_or(0);
                    Some(TableMessage::DecreaseColumnWidth(col))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        render::render_table(
            state,
            ctx.frame,
            ctx.area,
            ctx.theme,
            ctx.focused,
            ctx.disabled,
        );
    }
}

#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod multi_sort_tests;
#[cfg(test)]
mod resize_tests;
#[cfg(test)]
mod sort_proptests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;
