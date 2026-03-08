//! A data table component with row selection and column sorting.
//!
//! `Table` provides a tabular data display with keyboard navigation,
//! row selection, and column sorting capabilities.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Column, Component, Focusable, SortDirection, Table, TableMessage, TableOutput,
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

mod types;

pub use types::{Column, SortDirection, TableMessage, TableOutput, TableRow};

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row};

use super::{Component, Disableable, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

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
    sort: Option<(usize, SortDirection)>,
    display_order: Vec<usize>,
    focused: bool,
    disabled: bool,
    filter_text: String,
}

impl<T: TableRow + PartialEq> PartialEq for TableState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows
            && self.columns == other.columns
            && self.selected == other.selected
            && self.sort == other.sort
            && self.display_order == other.display_order
            && self.focused == other.focused
            && self.disabled == other.disabled
            && self.filter_text == other.filter_text
    }
}

impl<T: TableRow> Default for TableState<T> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            columns: Vec::new(),
            selected: None,
            sort: None,
            display_order: Vec::new(),
            focused: false,
            disabled: false,
            filter_text: String::new(),
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
        Self {
            rows,
            columns,
            selected,
            sort: None,
            display_order,
            focused: false,
            disabled: false,
            filter_text: String::new(),
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
        Self {
            rows,
            columns,
            selected,
            sort: None,
            display_order,
            focused: false,
            disabled: false,
            filter_text: String::new(),
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

    /// Returns the current sort configuration.
    ///
    /// Returns `None` if no sort is applied.
    pub fn sort(&self) -> Option<(usize, SortDirection)> {
        self.sort
    }

    /// Returns the number of rows.
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
        self.sort = None;

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

    /// Returns `true` if the table is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// Disabled tables do not respond to messages.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the disabled state (builder method).
    ///
    /// Disabled tables do not respond to messages.
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
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Length(10))],
    /// ).with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the current filter text.
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Returns the number of rows visible after filtering.
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

        // Sort
        if let Some((col, direction)) = self.sort {
            self.display_order.sort_by(|&a, &b| {
                let cells_a = self.rows[a].cells();
                let cells_b = self.rows[b].cells();
                let cmp = cells_a.get(col).cmp(&cells_b.get(col));
                match direction {
                    SortDirection::Ascending => cmp,
                    SortDirection::Descending => cmp.reverse(),
                }
            });
        }

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
    /// Returns true if the table is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Maps an input event to a table message.
    pub fn handle_event(&self, event: &Event) -> Option<TableMessage> {
        Table::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<TableOutput<T>> {
        Table::dispatch_event(self, event)
    }

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

                    // Toggle sort: None -> Asc -> Desc -> None
                    let new_sort = match state.sort {
                        Some((c, SortDirection::Ascending)) if c == col => {
                            Some((col, SortDirection::Descending))
                        }
                        Some((c, SortDirection::Descending)) if c == col => None,
                        _ => Some((col, SortDirection::Ascending)),
                    };

                    state.sort = new_sort;
                    state.rebuild_display_order();

                    return match new_sort {
                        Some((column, direction)) => {
                            Some(TableOutput::Sorted { column, direction })
                        }
                        None => Some(TableOutput::SortCleared),
                    };
                }
            }
            TableMessage::ClearSort => {
                if state.sort.is_some() {
                    state.sort = None;
                    state.rebuild_display_order();
                    return Some(TableOutput::SortCleared);
                }
            }
            TableMessage::SetFilter(_) | TableMessage::ClearFilter => {
                unreachable!("handled above")
            }
        }

        None
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(TableMessage::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(TableMessage::Down),
                KeyCode::Home => Some(TableMessage::First),
                KeyCode::End => Some(TableMessage::Last),
                KeyCode::Enter => Some(TableMessage::Select),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        crate::annotation::with_registry(|reg| {
            let mut ann = crate::annotation::Annotation::table("table")
                .with_focus(state.focused)
                .with_disabled(state.disabled);
            if let Some(idx) = state.selected {
                ann = ann.with_selected(true).with_value(idx.to_string());
            }
            reg.register(area, ann);
        });

        // Build header row with sort indicators
        let header_cells: Vec<Cell> = state
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let mut text = col.header().to_string();
                if let Some((sort_col, dir)) = state.sort {
                    if sort_col == i {
                        text.push_str(match dir {
                            SortDirection::Ascending => " ↑",
                            SortDirection::Descending => " ↓",
                        });
                    }
                }
                Cell::from(text)
            })
            .collect();

        let header_style = if state.disabled {
            theme.disabled_style()
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };

        let header = Row::new(header_cells).style(header_style).bottom_margin(1);

        // Build data rows using display_order
        let rows: Vec<Row> = state
            .display_order
            .iter()
            .map(|&idx| {
                let cells: Vec<Cell> = state.rows[idx]
                    .cells()
                    .into_iter()
                    .map(Cell::from)
                    .collect();
                Row::new(cells)
            })
            .collect();

        let widths: Vec<Constraint> = state.columns.iter().map(|c| c.width()).collect();

        let border_style = if state.focused && !state.disabled {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let row_highlight_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.selected_highlight_style(state.focused)
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
    }
}

impl<T: TableRow + 'static> Focusable for Table<T> {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl<T: TableRow + 'static> Disableable for Table<T> {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;
