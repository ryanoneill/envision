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
                        Some((column, direction)) => Some(TableOutput::Sorted { column, direction }),
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
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
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };

        let header = Row::new(header_cells)
            .style(header_style)
            .bottom_margin(1);

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
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let row_highlight_style = if state.disabled {
            Style::default().bg(Color::DarkGray)
        } else if state.focused {
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray)
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
mod tests {
    use super::*;

    // Test row type
    #[derive(Clone, Debug, PartialEq)]
    struct TestRow {
        name: String,
        value: String,
    }

    impl TestRow {
        fn new(name: &str, value: &str) -> Self {
            Self {
                name: name.into(),
                value: value.into(),
            }
        }
    }

    impl TableRow for TestRow {
        fn cells(&self) -> Vec<String> {
            vec![self.name.clone(), self.value.clone()]
        }
    }

    fn test_columns() -> Vec<Column> {
        vec![
            Column::new("Name", Constraint::Length(10)).sortable(),
            Column::new("Value", Constraint::Length(10)).sortable(),
        ]
    }

    fn test_rows() -> Vec<TestRow> {
        vec![
            TestRow::new("Charlie", "30"),
            TestRow::new("Alice", "10"),
            TestRow::new("Bob", "20"),
        ]
    }

    // TableRow Trait Tests

    #[test]
    fn test_tablerow_impl() {
        let row = TestRow::new("Test", "123");
        assert_eq!(row.cells(), vec!["Test", "123"]);
    }

    #[test]
    fn test_tablerow_empty_cells() {
        #[derive(Clone)]
        struct EmptyRow;

        impl TableRow for EmptyRow {
            fn cells(&self) -> Vec<String> {
                vec![]
            }
        }

        let row = EmptyRow;
        assert!(row.cells().is_empty());
    }

    // Column Tests

    #[test]
    fn test_column_new() {
        let col = Column::new("Header", Constraint::Length(15));
        assert_eq!(col.header(), "Header");
        assert!(!col.is_sortable());
    }

    #[test]
    fn test_column_sortable() {
        let col = Column::new("Header", Constraint::Length(15)).sortable();
        assert!(col.is_sortable());
    }

    #[test]
    fn test_column_clone() {
        let col = Column::new("Header", Constraint::Length(15)).sortable();
        let cloned = col.clone();
        assert_eq!(cloned.header(), "Header");
        assert!(cloned.is_sortable());
    }

    #[test]
    fn test_column_width() {
        let col = Column::new("Header", Constraint::Percentage(50));
        assert_eq!(col.width(), Constraint::Percentage(50));
    }

    // SortDirection Tests

    #[test]
    fn test_sort_direction_toggle() {
        assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
        assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_sort_direction_default() {
        let dir: SortDirection = Default::default();
        assert_eq!(dir, SortDirection::Ascending);
    }

    // State Creation Tests

    #[test]
    fn test_new() {
        let state = TableState::new(test_rows(), test_columns());
        assert_eq!(state.len(), 3);
        assert_eq!(state.selected_index(), Some(0));
        assert!(state.sort().is_none());
    }

    #[test]
    fn test_new_empty() {
        let state: TableState<TestRow> = TableState::new(vec![], test_columns());
        assert!(state.is_empty());
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn test_with_selected() {
        let state = TableState::with_selected(test_rows(), test_columns(), 2);
        assert_eq!(state.selected_index(), Some(2));
    }

    #[test]
    fn test_with_selected_clamps() {
        let state = TableState::with_selected(test_rows(), test_columns(), 100);
        assert_eq!(state.selected_index(), Some(2)); // Clamped to last
    }

    #[test]
    fn test_default() {
        let state: TableState<TestRow> = TableState::default();
        assert!(state.is_empty());
        assert_eq!(state.selected_index(), None);
        assert!(state.columns().is_empty());
    }

    // Accessors Tests

    #[test]
    fn test_rows_accessor() {
        let state = TableState::new(test_rows(), test_columns());
        assert_eq!(state.rows().len(), 3);
    }

    #[test]
    fn test_columns_accessor() {
        let state = TableState::new(test_rows(), test_columns());
        assert_eq!(state.columns().len(), 2);
    }

    #[test]
    fn test_selected_index() {
        let state = TableState::with_selected(test_rows(), test_columns(), 1);
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn test_selected_row() {
        let state = TableState::with_selected(test_rows(), test_columns(), 1);
        let row = state.selected_row().unwrap();
        assert_eq!(row.name, "Alice");
    }

    #[test]
    fn test_sort() {
        let state = TableState::new(test_rows(), test_columns());
        assert!(state.sort().is_none());
    }

    #[test]
    fn test_len() {
        let state = TableState::new(test_rows(), test_columns());
        assert_eq!(state.len(), 3);
    }

    #[test]
    fn test_is_empty() {
        let empty: TableState<TestRow> = TableState::new(vec![], vec![]);
        assert!(empty.is_empty());

        let not_empty = TableState::new(test_rows(), test_columns());
        assert!(!not_empty.is_empty());
    }

    // Mutators Tests

    #[test]
    fn test_set_rows() {
        let mut state = TableState::new(test_rows(), test_columns());
        state.set_rows(vec![TestRow::new("New", "1")]);
        assert_eq!(state.len(), 1);
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_set_rows_preserves_selection() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
        state.set_rows(vec![
            TestRow::new("A", "1"),
            TestRow::new("B", "2"),
            TestRow::new("C", "3"),
        ]);
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn test_set_rows_clamps_selection() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
        state.set_rows(vec![TestRow::new("A", "1")]);
        assert_eq!(state.selected_index(), Some(0)); // Clamped
    }

    #[test]
    fn test_set_selected() {
        let mut state = TableState::new(test_rows(), test_columns());
        state.set_selected(Some(2));
        assert_eq!(state.selected_index(), Some(2));

        state.set_selected(None);
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn test_disabled_accessors() {
        let mut state = TableState::new(test_rows(), test_columns());
        assert!(!state.is_disabled());

        state.set_disabled(true);
        assert!(state.is_disabled());

        state.set_disabled(false);
        assert!(!state.is_disabled());
    }

    // Navigation Tests

    #[test]
    fn test_down() {
        let mut state = TableState::new(test_rows(), test_columns());
        let output = Table::<TestRow>::update(&mut state, TableMessage::Down);
        assert_eq!(output, Some(TableOutput::SelectionChanged(1)));
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn test_down_at_last() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
        let output = Table::<TestRow>::update(&mut state, TableMessage::Down);
        assert_eq!(output, None);
        assert_eq!(state.selected_index(), Some(2));
    }

    #[test]
    fn test_up() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
        let output = Table::<TestRow>::update(&mut state, TableMessage::Up);
        assert_eq!(output, Some(TableOutput::SelectionChanged(0)));
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_up_at_first() {
        let mut state = TableState::new(test_rows(), test_columns());
        let output = Table::<TestRow>::update(&mut state, TableMessage::Up);
        assert_eq!(output, None);
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_first() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
        let output = Table::<TestRow>::update(&mut state, TableMessage::First);
        assert_eq!(output, Some(TableOutput::SelectionChanged(0)));
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_first_already_first() {
        let mut state = TableState::new(test_rows(), test_columns());
        let output = Table::<TestRow>::update(&mut state, TableMessage::First);
        assert_eq!(output, None);
    }

    #[test]
    fn test_last() {
        let mut state = TableState::new(test_rows(), test_columns());
        let output = Table::<TestRow>::update(&mut state, TableMessage::Last);
        assert_eq!(output, Some(TableOutput::SelectionChanged(2)));
        assert_eq!(state.selected_index(), Some(2));
    }

    #[test]
    fn test_last_already_last() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
        let output = Table::<TestRow>::update(&mut state, TableMessage::Last);
        assert_eq!(output, None);
    }

    #[test]
    fn test_page_down() {
        let mut state = TableState::new(test_rows(), test_columns());
        let output = Table::<TestRow>::update(&mut state, TableMessage::PageDown(2));
        assert_eq!(output, Some(TableOutput::SelectionChanged(2)));
    }

    #[test]
    fn test_page_up() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 2);
        let output = Table::<TestRow>::update(&mut state, TableMessage::PageUp(2));
        assert_eq!(output, Some(TableOutput::SelectionChanged(0)));
    }

    #[test]
    fn test_select() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
        let output = Table::<TestRow>::update(&mut state, TableMessage::Select);
        assert_eq!(
            output,
            Some(TableOutput::Selected(TestRow::new("Alice", "10")))
        );
    }

    #[test]
    fn test_empty_navigation() {
        let mut state: TableState<TestRow> = TableState::new(vec![], test_columns());

        assert_eq!(
            Table::<TestRow>::update(&mut state, TableMessage::Down),
            None
        );
        assert_eq!(Table::<TestRow>::update(&mut state, TableMessage::Up), None);
        assert_eq!(
            Table::<TestRow>::update(&mut state, TableMessage::Select),
            None
        );
    }

    // Sorting Tests

    #[test]
    fn test_sort_ascending() {
        let mut state = TableState::new(test_rows(), test_columns());
        let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
        assert_eq!(
            output,
            Some(TableOutput::Sorted {
                column: 0,
                direction: SortDirection::Ascending,
            })
        );

        // Check order: Alice, Bob, Charlie
        assert_eq!(state.rows()[state.display_order[0]].name, "Alice");
        assert_eq!(state.rows()[state.display_order[1]].name, "Bob");
        assert_eq!(state.rows()[state.display_order[2]].name, "Charlie");
    }

    #[test]
    fn test_sort_descending() {
        let mut state = TableState::new(test_rows(), test_columns());
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Ascending
        let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Descending
        assert_eq!(
            output,
            Some(TableOutput::Sorted {
                column: 0,
                direction: SortDirection::Descending,
            })
        );

        // Check order: Charlie, Bob, Alice
        assert_eq!(state.rows()[state.display_order[0]].name, "Charlie");
        assert_eq!(state.rows()[state.display_order[1]].name, "Bob");
        assert_eq!(state.rows()[state.display_order[2]].name, "Alice");
    }

    #[test]
    fn test_sort_clear() {
        let mut state = TableState::new(test_rows(), test_columns());
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Ascending
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Descending
        let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Clear
        assert_eq!(output, Some(TableOutput::SortCleared));
        assert!(state.sort().is_none());

        // Back to original order: Charlie, Alice, Bob
        assert_eq!(state.rows()[state.display_order[0]].name, "Charlie");
        assert_eq!(state.rows()[state.display_order[1]].name, "Alice");
        assert_eq!(state.rows()[state.display_order[2]].name, "Bob");
    }

    #[test]
    fn test_sort_unsortable_column() {
        let columns = vec![
            Column::new("Name", Constraint::Length(10)), // Not sortable
            Column::new("Value", Constraint::Length(10)).sortable(),
        ];
        let mut state = TableState::new(test_rows(), columns);
        let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
        assert_eq!(output, None);
    }

    #[test]
    fn test_sort_preserves_selection() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
        // Initially selected: Alice (index 1 in original order)

        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)); // Sort ascending

        // After sort, Alice should still be selected but at a different display index
        let selected = state.selected_row().unwrap();
        assert_eq!(selected.name, "Alice");
    }

    #[test]
    fn test_sort_numeric_strings() {
        // Numeric strings sort lexicographically, not numerically
        let rows = vec![
            TestRow::new("Item", "9"),
            TestRow::new("Item", "10"),
            TestRow::new("Item", "2"),
        ];
        let columns = vec![
            Column::new("Name", Constraint::Length(10)),
            Column::new("Value", Constraint::Length(10)).sortable(),
        ];
        let mut state = TableState::new(rows, columns);

        Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));

        // Lexicographic: "10" < "2" < "9"
        assert_eq!(state.rows()[state.display_order[0]].value, "10");
        assert_eq!(state.rows()[state.display_order[1]].value, "2");
        assert_eq!(state.rows()[state.display_order[2]].value, "9");
    }

    #[test]
    fn test_clear_sort() {
        let mut state = TableState::new(test_rows(), test_columns());
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
        assert!(state.sort().is_some());

        let output = Table::<TestRow>::update(&mut state, TableMessage::ClearSort);
        assert_eq!(output, Some(TableOutput::SortCleared));
        assert!(state.sort().is_none());
    }

    #[test]
    fn test_clear_sort_when_not_sorted() {
        let mut state = TableState::new(test_rows(), test_columns());
        let output = Table::<TestRow>::update(&mut state, TableMessage::ClearSort);
        assert_eq!(output, None);
    }

    #[test]
    fn test_sort_different_column() {
        let mut state = TableState::new(test_rows(), test_columns());

        // Sort by column 0
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
        assert_eq!(state.sort(), Some((0, SortDirection::Ascending)));

        // Sort by column 1 - should reset to ascending on new column
        let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(1));
        assert_eq!(
            output,
            Some(TableOutput::Sorted {
                column: 1,
                direction: SortDirection::Ascending,
            })
        );
    }

    // Disabled State Tests

    #[test]
    fn test_disabled() {
        let mut state = TableState::new(test_rows(), test_columns());
        state.set_disabled(true);

        assert_eq!(
            Table::<TestRow>::update(&mut state, TableMessage::Down),
            None
        );
        assert_eq!(Table::<TestRow>::update(&mut state, TableMessage::Up), None);
        assert_eq!(
            Table::<TestRow>::update(&mut state, TableMessage::Select),
            None
        );
        assert_eq!(
            Table::<TestRow>::update(&mut state, TableMessage::SortBy(0)),
            None
        );
    }

    // Focus Tests

    #[test]
    fn test_focusable() {
        let mut state = TableState::new(test_rows(), test_columns());
        assert!(!Table::<TestRow>::is_focused(&state));

        Table::<TestRow>::set_focused(&mut state, true);
        assert!(Table::<TestRow>::is_focused(&state));

        Table::<TestRow>::blur(&mut state);
        assert!(!Table::<TestRow>::is_focused(&state));

        Table::<TestRow>::focus(&mut state);
        assert!(Table::<TestRow>::is_focused(&state));
    }

    // View Tests

    #[test]
    fn test_view_renders() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = TableState::new(test_rows(), test_columns());

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Table::<TestRow>::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Name"));
        assert!(output.contains("Value"));
        assert!(output.contains("Charlie"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
    }

    #[test]
    fn test_view_with_header() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = TableState::new(test_rows(), test_columns());

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Table::<TestRow>::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Name"));
        assert!(output.contains("Value"));
    }

    #[test]
    fn test_view_with_sort_indicator() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = TableState::new(test_rows(), test_columns());
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Table::<TestRow>::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("↑")); // Ascending indicator
    }

    #[test]
    fn test_view_focused() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = TableState::new(test_rows(), test_columns());
        state.focused = true;

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Table::<TestRow>::view(&state, frame, frame.area());
            })
            .unwrap();

        // Should render without panicking
        let _output = terminal.backend().to_string();
    }

    #[test]
    fn test_view_disabled() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = TableState::new(test_rows(), test_columns());
        state.disabled = true;

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Table::<TestRow>::view(&state, frame, frame.area());
            })
            .unwrap();

        // Should render without panicking
        let _output = terminal.backend().to_string();
    }

    #[test]
    fn test_view_empty() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state: TableState<TestRow> = TableState::new(vec![], test_columns());

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Table::<TestRow>::view(&state, frame, frame.area());
            })
            .unwrap();

        // Should render without panicking
        let output = terminal.backend().to_string();
        assert!(output.contains("Name")); // Headers still shown
    }

    // Integration Tests

    #[test]
    fn test_clone() {
        let mut state = TableState::with_selected(test_rows(), test_columns(), 1);
        state.focused = true;
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

        let cloned = state.clone();
        assert_eq!(cloned.selected_index(), Some(0)); // Alice is now at position 0 after sort
        assert!(cloned.focused);
        assert!(cloned.sort().is_some());
    }

    #[test]
    fn test_init() {
        let state: TableState<TestRow> = Table::<TestRow>::init();
        assert!(state.is_empty());
        assert!(!state.focused);
        assert!(!state.disabled);
    }

    #[test]
    fn test_full_workflow() {
        let mut state = TableState::new(test_rows(), test_columns());
        Table::<TestRow>::set_focused(&mut state, true);

        // Navigate
        Table::<TestRow>::update(&mut state, TableMessage::Down);
        Table::<TestRow>::update(&mut state, TableMessage::Down);
        assert_eq!(state.selected_index(), Some(2));

        // Sort
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));
        // Selection should follow the row, not the position

        // Navigate after sort
        Table::<TestRow>::update(&mut state, TableMessage::First);
        assert_eq!(state.selected_row().unwrap().name, "Alice");

        // Select
        let output = Table::<TestRow>::update(&mut state, TableMessage::Select);
        assert_eq!(
            output,
            Some(TableOutput::Selected(TestRow::new("Alice", "10")))
        );
    }

    #[test]
    fn test_navigation_with_sort() {
        let mut state = TableState::new(test_rows(), test_columns());

        // Initially selected: Charlie (position 0 in original order)

        // Sort ascending by name
        Table::<TestRow>::update(&mut state, TableMessage::SortBy(0));

        // Now display order is: Alice, Bob, Charlie
        // But selection is preserved on the same ROW (Charlie), now at position 2
        assert_eq!(state.selected_row().unwrap().name, "Charlie");
        assert_eq!(state.selected_index(), Some(2));

        // Navigate to first to get to Alice
        Table::<TestRow>::update(&mut state, TableMessage::First);
        assert_eq!(state.selected_row().unwrap().name, "Alice");

        Table::<TestRow>::update(&mut state, TableMessage::Down);
        assert_eq!(state.selected_row().unwrap().name, "Bob");

        Table::<TestRow>::update(&mut state, TableMessage::Down);
        assert_eq!(state.selected_row().unwrap().name, "Charlie");
    }

    #[test]
    fn test_sort_out_of_bounds_column() {
        let mut state = TableState::new(test_rows(), test_columns());
        let output = Table::<TestRow>::update(&mut state, TableMessage::SortBy(99));
        assert_eq!(output, None);
    }

    #[test]
    fn test_page_navigation_bounds() {
        let mut state = TableState::new(test_rows(), test_columns());

        // PageDown beyond end
        let output = Table::<TestRow>::update(&mut state, TableMessage::PageDown(100));
        assert_eq!(output, Some(TableOutput::SelectionChanged(2)));

        // PageUp beyond start
        let output = Table::<TestRow>::update(&mut state, TableMessage::PageUp(100));
        assert_eq!(output, Some(TableOutput::SelectionChanged(0)));
    }
}
