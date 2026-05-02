//! DataGridState constructors, builders, accessors, setters, and instance methods.
//!
//! Extracted from the main data_grid module to keep file sizes manageable.

use super::{
    Column, DataGrid, DataGridMessage, DataGridOutput, DataGridState, InputFieldState, TableRow,
};
use crate::component::Component;
use crate::scroll::ScrollState;

impl<T: TableRow> DataGridState<T> {
    /// Creates a new data grid with the given rows and columns.
    ///
    /// The first row is selected by default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.selected_index(), Some(0));
    /// assert_eq!(state.selected_column(), 0);
    /// ```
    pub fn new(rows: Vec<T>, columns: Vec<Column>) -> Self {
        let selected_row = if rows.is_empty() { None } else { Some(0) };
        let scroll = ScrollState::new(rows.len());
        Self {
            rows,
            columns,
            selected_row,
            selected_column: 0,
            editing: false,
            editor: InputFieldState::new(),
            original_value: String::new(),
            scroll,
        }
    }

    /// Returns the rows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.rows().len(), 1);
    /// ```
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Returns the columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.columns().len(), 1);
    /// assert_eq!(state.columns()[0].header(), "Name");
    /// ```
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Returns the currently selected row index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, DataGridState, TableRow};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_row
    }

    /// Alias for [`selected_index()`](Self::selected_index).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.selected(), Some(0));
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }

    /// Returns a reference to the currently selected row.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "Alice".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.selected_row().unwrap().name, "Alice");
    /// ```
    pub fn selected_row(&self) -> Option<&T> {
        self.selected_row.and_then(|i| self.rows.get(i))
    }

    /// Returns a reference to the currently selected item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "Bob".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.selected_item().unwrap().name, "Bob");
    /// ```
    pub fn selected_item(&self) -> Option<&T> {
        self.selected_row()
    }

    /// Sets the selected row index.
    ///
    /// The index is clamped to the valid range. Has no effect on empty grids.
    /// Cancels any active edit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let mut state = DataGridState::new(
    ///     vec![Item { name: "A".into() }, Item { name: "B".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// state.set_selected(Some(1));
    /// assert_eq!(state.selected_index(), Some(1));
    /// ```
    pub fn set_selected(&mut self, index: Option<usize>) {
        match index {
            Some(i) => {
                if self.rows.is_empty() {
                    return;
                }
                self.editing = false;
                self.selected_row = Some(i.min(self.rows.len() - 1));
            }
            None => {
                self.editing = false;
                self.selected_row = None;
            }
        }
    }

    /// Returns the currently selected column index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, DataGridState, TableRow};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.selected_column(), 0);
    /// ```
    pub fn selected_column(&self) -> usize {
        self.selected_column
    }

    /// Returns true if a cell is currently being edited.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert!(!state.is_editing());
    /// ```
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Returns the current editor value (while editing).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.editor_value(), "");
    /// ```
    pub fn editor_value(&self) -> &str {
        self.editor.value()
    }

    /// Returns the value of the currently selected cell.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "Alice".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.current_cell_value(), Some("Alice".to_string()));
    /// ```
    pub fn current_cell_value(&self) -> Option<String> {
        self.selected_row
            .and_then(|ri| self.rows.get(ri))
            .map(|row| {
                let cells = row.cells();
                cells
                    .get(self.selected_column)
                    .map(|c| c.text().to_string())
                    .unwrap_or_default()
            })
    }

    /// Returns the number of rows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }, Item { name: "B".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// assert_eq!(state.row_count(), 2);
    /// ```
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns the number of columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![
    ///         Column::new("Name", Constraint::Min(10)),
    ///         Column::new("Value", Constraint::Min(5)),
    ///     ],
    /// );
    /// assert_eq!(state.column_count(), 2);
    /// ```
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns true if the grid has no rows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state: DataGridState<Item> = DataGridState::default();
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Sets the rows, resetting selection and cancelling any edit.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let mut state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// state.set_rows(vec![Item { name: "X".into() }, Item { name: "Y".into() }]);
    /// assert_eq!(state.row_count(), 2);
    /// ```
    pub fn set_rows(&mut self, rows: Vec<T>) {
        self.editing = false;
        self.rows = rows;
        self.scroll.set_content_length(self.rows.len());
        if self.rows.is_empty() {
            self.selected_row = None;
        } else {
            let current = self.selected_row.unwrap_or(0);
            self.selected_row = Some(current.min(self.rows.len() - 1));
        }
    }
}

impl<T: TableRow + 'static> DataGridState<T> {
    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, DataGridMessage, DataGridOutput, DataGridState, TableRow};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let mut state = DataGridState::new(
    ///     vec![Item { name: "A".into() }, Item { name: "B".into() }],
    ///     vec![Column::new("Name", Constraint::Min(10))],
    /// );
    /// let output = state.update(DataGridMessage::Down);
    /// assert_eq!(output, Some(DataGridOutput::SelectionChanged(1)));
    /// ```
    pub fn update(&mut self, msg: DataGridMessage) -> Option<DataGridOutput<T>> {
        DataGrid::<T>::update(self, msg)
    }

    /// Starts editing the current cell.
    pub(super) fn start_editing(&mut self) {
        if let Some(cell_value) = self.current_cell_value() {
            self.original_value = cell_value.clone();
            self.editor.set_value(&cell_value);
            self.editing = true;
        }
    }

    /// Cancels the current edit, restoring the original value.
    pub(super) fn cancel_editing(&mut self) {
        self.editing = false;
    }
}
