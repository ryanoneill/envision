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
//!     Column, Component, SortDirection, Table, TableMessage, TableOutput,
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
//!     fn cells(&self) -> Vec<String> {
//!         vec![self.name.clone(), self.email.clone()]
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
//! Table::set_focused(&mut state, true);
//!
//! // Navigate down
//! let output = Table::<User>::update(&mut state, TableMessage::Down);
//! assert_eq!(output, Some(TableOutput::SelectionChanged(1)));
//!
//! // Sort by name column
//! let output = Table::<User>::update(&mut state, TableMessage::SortBy(0));
//! assert_eq!(output, Some(TableOutput::Sorted {
//!     column: 0,
//!     direction: SortDirection::Ascending,
//! }));
//! ```

mod render;
mod types;

pub use types::{
    date_comparator, numeric_comparator, Column, SortComparator, SortDirection, TableMessage,
    TableOutput, TableRow,
};

use std::marker::PhantomData;

use ratatui::prelude::*;

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
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
}

impl<T: TableRow + PartialEq> PartialEq for TableState<T> {
    fn eq(&self, other: &Self) -> bool {
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
        }
    }
}

impl<T: TableRow> TableState<T> {
    /// Creates a new table state with the given rows and columns.
    ///
    /// If there are rows, the first row is selected by default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    ///
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> {
    ///         vec![self.name.clone()]
    ///     }
    /// }
    ///
    /// let state = TableState::new(
    ///     vec![Item { name: "A".into() }, Item { name: "B".into() }],
    ///     vec![Column::new("Name", Constraint::Length(10))],
    /// );
    /// assert_eq!(state.len(), 2);
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn new(rows: Vec<T>, columns: Vec<Column>) -> Self {
        let display_order: Vec<usize> = (0..rows.len()).collect();
        let selected = if rows.is_empty() { None } else { Some(0) };
        let scroll = ScrollState::new(display_order.len());
        Self {
            rows,
            columns,
            selected,
            sort_columns: Vec::new(),
            display_order,
            filter_text: String::new(),
            scroll,
        }
    }

    /// Creates a table state with a specific row selected.
    ///
    /// The index is clamped to the valid range.
    pub fn with_selected(rows: Vec<T>, columns: Vec<Column>, selected: usize) -> Self {
        let display_order: Vec<usize> = (0..rows.len()).collect();
        let selected = if rows.is_empty() {
            None
        } else {
            Some(selected.min(rows.len() - 1))
        };
        let scroll = ScrollState::new(display_order.len());
        Self {
            rows,
            columns,
            selected,
            sort_columns: Vec::new(),
            display_order,
            filter_text: String::new(),
            scroll,
        }
    }

    /// Returns a reference to the rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let state = TableState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// assert_eq!(state.rows().len(), 1);
    /// assert_eq!(state.rows()[0].name, "A");
    /// ```
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Returns a reference to the columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let state = TableState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// assert_eq!(state.columns().len(), 1);
    /// assert_eq!(state.columns()[0].header(), "Name");
    /// ```
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Returns the currently selected display index.
    ///
    /// This is the index in the display order, not the original row index.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Alias for [`selected_index()`](Self::selected_index).
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }

    /// Returns a reference to the currently selected row.
    ///
    /// Returns `None` if no row is selected or the table is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let state = TableState::new(
    ///     vec![Item { name: "first".into() }, Item { name: "second".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// assert_eq!(state.selected_row().unwrap().name, "first");
    /// ```
    pub fn selected_row(&self) -> Option<&T> {
        self.selected
            .and_then(|i| self.display_order.get(i))
            .and_then(|&idx| self.rows.get(idx))
    }

    /// Returns a reference to the currently selected item.
    ///
    /// This is an alias for [`selected_row()`](Self::selected_row) that provides a
    /// consistent accessor name across all selection-based components.
    pub fn selected_item(&self) -> Option<&T> {
        self.selected_row()
    }

    /// Returns the primary (highest-priority) sort column and direction.
    ///
    /// Returns `None` if no sort is applied. For multi-column sort,
    /// use [`sort_columns()`](Self::sort_columns).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Column, Table, TableRow, TableState, TableMessage, SortDirection, Component};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let mut state = TableState::new(
    ///     vec![Item { name: "B".into() }, Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Length(10)).sortable()],
    /// );
    /// assert_eq!(state.sort(), None);
    ///
    /// Table::<Item>::update(&mut state, TableMessage::SortBy(0));
    /// assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));
    /// ```
    pub fn sort(&self) -> Option<(usize, SortDirection)> {
        self.sort_columns.first().copied()
    }

    /// Returns all sort columns in priority order.
    ///
    /// The first element is the primary sort, the second is the
    /// first tiebreaker, and so on.
    pub fn sort_columns(&self) -> &[(usize, SortDirection)] {
        &self.sort_columns
    }

    /// Returns the number of rows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let state = TableState::new(
    ///     vec![Item { name: "A".into() }, Item { name: "B".into() }],
    ///     vec![Column::new("Name", Constraint::Length(10))],
    /// );
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns `true` if the table has no rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Sets the rows, clearing filter and sort, and adjusting selection.
    ///
    /// If there were rows selected, the selection is preserved if valid,
    /// otherwise clamped to the last row.
    pub fn set_rows(&mut self, rows: Vec<T>) {
        self.rows = rows;
        self.filter_text.clear();
        self.display_order = (0..self.rows.len()).collect();
        self.sort_columns.clear();
        self.scroll.set_content_length(self.display_order.len());

        if self.rows.is_empty() {
            self.selected = None;
        } else if let Some(sel) = self.selected {
            self.selected = Some(sel.min(self.rows.len() - 1));
        } else {
            self.selected = Some(0);
        }
    }

    /// Sets the selected row by display index.
    ///
    /// Pass `None` to clear the selection.
    /// Out of bounds indices are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let mut state = TableState::new(
    ///     vec![Item { name: "A".into() }, Item { name: "B".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// state.set_selected(Some(1));
    /// assert_eq!(state.selected_index(), Some(1));
    /// ```
    pub fn set_selected(&mut self, index: Option<usize>) {
        match index {
            Some(i) if i < self.display_order.len() => self.selected = Some(i),
            Some(_) => {} // Out of bounds, ignore
            None => self.selected = None,
        }
    }

    /// Returns the current filter text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let mut state = TableState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Length(10))],
    /// );
    /// assert_eq!(state.filter_text(), "");
    /// state.set_filter_text("A");
    /// assert_eq!(state.filter_text(), "A");
    /// ```
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Returns the number of rows visible after filtering.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let mut state = TableState::new(
    ///     vec![Item { name: "Alice".into() }, Item { name: "Bob".into() }],
    ///     vec![Column::new("Name", Constraint::Length(10))],
    /// );
    /// assert_eq!(state.visible_count(), 2);
    /// state.set_filter_text("Alice");
    /// assert_eq!(state.visible_count(), 1);
    /// ```
    pub fn visible_count(&self) -> usize {
        self.display_order.len()
    }

    /// Sets the filter text for case-insensitive substring matching on cell content.
    ///
    /// Rows where any cell contains the filter text (case-insensitive) are shown.
    /// Selection is preserved if the selected row remains visible.
    pub fn set_filter_text(&mut self, text: &str) {
        self.filter_text = text.to_string();
        self.rebuild_display_order();
    }

    /// Clears the filter, showing all rows.
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.rebuild_display_order();
    }

    /// Rebuilds the display order by applying filter, then sort.
    fn rebuild_display_order(&mut self) {
        let selected_original = self
            .selected
            .and_then(|i| self.display_order.get(i).copied());

        // Filter
        if self.filter_text.is_empty() {
            self.display_order = (0..self.rows.len()).collect();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.display_order = self
                .rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    row.cells()
                        .iter()
                        .any(|cell| cell.to_lowercase().contains(&filter_lower))
                })
                .map(|(i, _)| i)
                .collect();
        }

        // Multi-column sort
        if !self.sort_columns.is_empty() {
            let columns = &self.columns;
            let sort_spec: Vec<(usize, SortDirection)> = self.sort_columns.clone();
            self.display_order.sort_by(|&a, &b| {
                let cells_a = self.rows[a].cells();
                let cells_b = self.rows[b].cells();
                for &(col, direction) in &sort_spec {
                    let val_a = cells_a.get(col).map(|s| s.as_str());
                    let val_b = cells_b.get(col).map(|s| s.as_str());
                    let cmp = match (val_a, val_b) {
                        (Some(a_str), Some(b_str)) => {
                            if let Some(comparator) = columns.get(col).and_then(|c| c.comparator())
                            {
                                comparator(a_str, b_str)
                            } else {
                                a_str.cmp(b_str)
                            }
                        }
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    };
                    let ordered = match direction {
                        SortDirection::Ascending => cmp,
                        SortDirection::Descending => cmp.reverse(),
                    };
                    if ordered != std::cmp::Ordering::Equal {
                        return ordered;
                    }
                }
                std::cmp::Ordering::Equal
            });
        }

        self.scroll.set_content_length(self.display_order.len());

        // Preserve selection
        if let Some(orig) = selected_original {
            self.selected = self.find_display_index(orig);
        }
        if self.selected.is_none() && !self.display_order.is_empty() {
            self.selected = Some(0);
        }
    }

    /// Finds the display index of the given original row index.
    fn find_display_index(&self, original_index: usize) -> Option<usize> {
        self.display_order
            .iter()
            .position(|&idx| idx == original_index)
    }
}

impl<T: TableRow + 'static> TableState<T> {
    /// Updates the table state with a message, returning any output.
    pub fn update(&mut self, msg: TableMessage) -> Option<TableOutput<T>> {
        Table::update(self, msg)
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
/// - `SortBy(column)` - Sort by the given column
/// - `ClearSort` - Clear the current sort
///
/// # Sorting
///
/// Clicking the same column cycles through: Ascending -> Descending -> None.
/// Only columns marked as `sortable()` can be sorted.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Column, Component, Table, TableMessage, TableRow, TableState,
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
///     fn cells(&self) -> Vec<String> {
///         vec![self.name.clone(), self.age.to_string()]
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

        if state.disabled || state.display_order.is_empty() {
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
            TableMessage::SortBy(col) => {
                // Check if column exists and is sortable
                if let Some(column) = state.columns.get(col) {
                    if !column.is_sortable() {
                        return None;
                    }

                    // If this column is already the primary sort, toggle direction
                    // If primary ascending -> descending
                    // If primary descending -> clear all
                    // Otherwise, replace all sorts with this column ascending
                    let primary = state.sort_columns.first().copied();
                    match primary {
                        Some((c, SortDirection::Ascending)) if c == col => {
                            state.sort_columns = vec![(col, SortDirection::Descending)];
                            state.rebuild_display_order();
                            return Some(TableOutput::Sorted {
                                column: col,
                                direction: SortDirection::Descending,
                            });
                        }
                        Some((c, SortDirection::Descending)) if c == col => {
                            state.sort_columns.clear();
                            state.rebuild_display_order();
                            return Some(TableOutput::SortCleared);
                        }
                        _ => {
                            state.sort_columns = vec![(col, SortDirection::Ascending)];
                            state.rebuild_display_order();
                            return Some(TableOutput::Sorted {
                                column: col,
                                direction: SortDirection::Ascending,
                            });
                        }
                    }
                }
            }
            TableMessage::AddSort(col) => {
                // Check if column exists and is sortable
                if let Some(column) = state.columns.get(col) {
                    if !column.is_sortable() {
                        return None;
                    }

                    // If the column is already in the sort stack, toggle its direction
                    if let Some(pos) = state.sort_columns.iter().position(|&(c, _)| c == col) {
                        let (_, dir) = state.sort_columns[pos];
                        let new_dir = dir.toggle();
                        state.sort_columns[pos] = (col, new_dir);
                        state.rebuild_display_order();
                        return Some(TableOutput::Sorted {
                            column: col,
                            direction: new_dir,
                        });
                    }

                    // Otherwise, add it as a new tiebreaker
                    state.sort_columns.push((col, SortDirection::Ascending));
                    state.rebuild_display_order();
                    return Some(TableOutput::Sorted {
                        column: col,
                        direction: SortDirection::Ascending,
                    });
                }
            }
            TableMessage::ClearSort => {
                if !state.sort_columns.is_empty() {
                    state.sort_columns.clear();
                    state.rebuild_display_order();
                    return Some(TableOutput::SortCleared);
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
        }

        None
    }

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(TableMessage::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(TableMessage::Down),
                KeyCode::Home => Some(TableMessage::First),
                KeyCode::End => Some(TableMessage::Last),
                KeyCode::Enter if has_shift => {
                    // Shift+Enter adds the current primary sort column to the sort stack
                    // This is a no-op if there's no selection context for a column
                    None
                }
                KeyCode::Enter => Some(TableMessage::Select),
                KeyCode::Char('+') => {
                    // Increase the width of the currently selected column
                    // Uses the primary sort column index, or column 0 if no sort
                    let col = state.sort_columns.first().map(|&(c, _)| c).unwrap_or(0);
                    Some(TableMessage::IncreaseColumnWidth(col))
                }
                KeyCode::Char('-') => {
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        render::render_table(state, frame, area, theme, ctx.focused, ctx.disabled);
    }
}

#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod multi_sort_tests;
#[cfg(test)]
mod resize_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;
