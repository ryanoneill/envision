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

use std::marker::PhantomData;

use ratatui::layout::Constraint;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row};

use super::{Component, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Trait for types that can be displayed as table rows.
///
/// Implement this trait for your data types to use them with `Table`.
///
/// # Example
///
/// ```rust
/// use envision::component::TableRow;
///
/// #[derive(Clone)]
/// struct Product {
///     name: String,
///     price: f64,
///     quantity: u32,
/// }
///
/// impl TableRow for Product {
///     fn cells(&self) -> Vec<String> {
///         vec![
///             self.name.clone(),
///             format!("${:.2}", self.price),
///             self.quantity.to_string(),
///         ]
///     }
/// }
/// ```
pub trait TableRow: Clone {
    /// Returns the cell values for this row.
    ///
    /// The order of values should match the order of columns
    /// defined in the table.
    fn cells(&self) -> Vec<String>;
}

/// Column definition for a table.
///
/// Columns define the header text, width, and whether the column
/// is sortable.
///
/// # Example
///
/// ```rust
/// use envision::component::Column;
/// use ratatui::layout::Constraint;
///
/// let col = Column::new("Name", Constraint::Length(20)).sortable();
/// assert_eq!(col.header(), "Name");
/// assert!(col.is_sortable());
/// ```
#[derive(Clone, Debug)]
pub struct Column {
    header: String,
    width: Constraint,
    sortable: bool,
}

impl Column {
    /// Creates a new column with the given header and width.
    ///
    /// The column is not sortable by default.
    pub fn new(header: impl Into<String>, width: Constraint) -> Self {
        Self {
            header: header.into(),
            width,
            sortable: false,
        }
    }

    /// Makes this column sortable.
    ///
    /// Sortable columns can be sorted by clicking/selecting the header
    /// or using `TableMessage::SortBy`.
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Returns the column header text.
    pub fn header(&self) -> &str {
        &self.header
    }

    /// Returns the column width constraint.
    pub fn width(&self) -> Constraint {
        self.width
    }

    /// Returns whether this column is sortable.
    pub fn is_sortable(&self) -> bool {
        self.sortable
    }
}

/// Sort direction for table columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SortDirection {
    /// Sort in ascending order (A-Z, 0-9).
    #[default]
    Ascending,
    /// Sort in descending order (Z-A, 9-0).
    Descending,
}

impl SortDirection {
    /// Returns the opposite sort direction.
    pub fn toggle(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

/// Messages that can be sent to a Table component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableMessage {
    /// Move selection up by one row.
    Up,
    /// Move selection down by one row.
    Down,
    /// Move selection to the first row.
    First,
    /// Move selection to the last row.
    Last,
    /// Move selection up by a page.
    PageUp(usize),
    /// Move selection down by a page.
    PageDown(usize),
    /// Confirm the current selection.
    Select,
    /// Sort by the given column index.
    ///
    /// If already sorted by this column, toggles direction.
    /// If sorted ascending, becomes descending.
    /// If sorted descending, clears the sort.
    SortBy(usize),
    /// Clear the current sort, returning to original order.
    ClearSort,
}

/// Output messages from a Table component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableOutput<T: Clone> {
    /// A row was selected (e.g., Enter pressed).
    Selected(T),
    /// The selection changed to a new row index.
    SelectionChanged(usize),
    /// The sort changed.
    Sorted {
        /// The column being sorted by.
        column: usize,
        /// The sort direction.
        direction: SortDirection,
    },
    /// Sort was cleared.
    SortCleared,
}

/// State for a Table component.
///
/// Holds the rows, columns, selection state, and sort configuration.
#[derive(Clone, Debug)]
pub struct TableState<T: TableRow> {
    rows: Vec<T>,
    columns: Vec<Column>,
    selected: Option<usize>,
    sort: Option<(usize, SortDirection)>,
    display_order: Vec<usize>,
    focused: bool,
    disabled: bool,
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
        }
    }

    /// Returns a reference to the rows.
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Returns a reference to the columns.
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Returns the currently selected display index.
    ///
    /// This is the index in the display order, not the original row index.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Returns a reference to the currently selected row.
    ///
    /// Returns `None` if no row is selected or the table is empty.
    pub fn selected_row(&self) -> Option<&T> {
        self.selected
            .and_then(|i| self.display_order.get(i))
            .and_then(|&idx| self.rows.get(idx))
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

    /// Sets the rows, resetting sort and adjusting selection.
    ///
    /// If there were rows selected, the selection is preserved if valid,
    /// otherwise clamped to the last row.
    pub fn set_rows(&mut self, rows: Vec<T>) {
        self.rows = rows;
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

    /// Applies the current sort to the display order.
    fn apply_sort(&mut self) {
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
        } else {
            // Reset to original order
            self.display_order = (0..self.rows.len()).collect();
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
        if state.disabled || state.rows.is_empty() {
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

                    // Get the currently selected row's original index
                    let selected_original = state
                        .selected
                        .and_then(|i| state.display_order.get(i).copied());

                    // Toggle sort: None -> Asc -> Desc -> None
                    let new_sort = match state.sort {
                        Some((c, SortDirection::Ascending)) if c == col => {
                            Some((col, SortDirection::Descending))
                        }
                        Some((c, SortDirection::Descending)) if c == col => None,
                        _ => Some((col, SortDirection::Ascending)),
                    };

                    state.sort = new_sort;
                    state.apply_sort();

                    // Restore selection to the same row
                    if let Some(orig) = selected_original {
                        state.selected = state.find_display_index(orig);
                    }

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
                    // Get the currently selected row's original index
                    let selected_original = state
                        .selected
                        .and_then(|i| state.display_order.get(i).copied());

                    state.sort = None;
                    state.apply_sort();

                    // Restore selection to the same row
                    if let Some(orig) = selected_original {
                        state.selected = state.find_display_index(orig);
                    }

                    return Some(TableOutput::SortCleared);
                }
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
        // Build header row with sort indicators
        let header_cells: Vec<Cell> = state
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let mut text = col.header.clone();
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

        let widths: Vec<Constraint> = state.columns.iter().map(|c| c.width).collect();

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

#[cfg(test)]
mod tests;
