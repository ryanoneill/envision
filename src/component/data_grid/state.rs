//! State type for the DataGrid component.

use std::sync::Arc;

use super::{
    Column, Component, DataGrid, DataGridMessage, DataGridOutput, InputFieldState, TableRow,
};
use crate::input::Event;
use crate::scroll::ScrollState;

/// A cell validator function.
///
/// Returns `Ok(())` if the value is valid, or `Err(message)` with an error
/// message if validation fails.
pub type CellValidator = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// State for a DataGrid component.
///
/// Contains the table data, column cursor, and inline editor state.
#[derive(Clone)]
pub struct DataGridState<T: TableRow> {
    /// The rows of data.
    pub(super) rows: Vec<T>,
    /// Column definitions.
    pub(super) columns: Vec<Column>,
    /// Currently selected row index.
    pub(super) selected_row: Option<usize>,
    /// Currently selected column index.
    pub(super) selected_column: usize,
    /// Whether the cell editor is active.
    pub(super) editing: bool,
    /// The inline editor state.
    pub(super) editor: InputFieldState,
    /// Value before editing started (for cancel).
    pub(super) original_value: String,
    /// Whether the overall component is focused.
    pub(super) focused: bool,
    /// Whether the component is disabled.
    pub(super) disabled: bool,
    /// Scroll state for scrollbar rendering.
    pub(super) scroll: ScrollState,
    /// Per-column validators, indexed by column index.
    pub(super) validators: Vec<Option<CellValidator>>,
    /// The last validation error, if any.
    pub(super) last_validation_error: Option<String>,
}

impl<T: TableRow + std::fmt::Debug> std::fmt::Debug for DataGridState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataGridState")
            .field("rows", &self.rows)
            .field("columns", &self.columns)
            .field("selected_row", &self.selected_row)
            .field("selected_column", &self.selected_column)
            .field("editing", &self.editing)
            .field("editor", &self.editor)
            .field("original_value", &self.original_value)
            .field("focused", &self.focused)
            .field("disabled", &self.disabled)
            .field("scroll", &self.scroll)
            .field(
                "validators",
                &format!("[{} validators]", self.validators.len()),
            )
            .field("last_validation_error", &self.last_validation_error)
            .finish()
    }
}

impl<T: TableRow + PartialEq> PartialEq for DataGridState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows
            && self.columns == other.columns
            && self.selected_row == other.selected_row
            && self.selected_column == other.selected_column
            && self.editing == other.editing
            && self.focused == other.focused
            && self.disabled == other.disabled
    }
}

impl<T: TableRow> Default for DataGridState<T> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            columns: Vec::new(),
            selected_row: None,
            selected_column: 0,
            editing: false,
            editor: InputFieldState::new(),
            original_value: String::new(),
            focused: false,
            disabled: false,
            scroll: ScrollState::default(),
            validators: Vec::new(),
            last_validation_error: None,
        }
    }
}

impl<T: TableRow> DataGridState<T> {
    /// Creates a new data grid with the given rows and columns.
    ///
    /// The first row is selected by default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
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
        let validator_count = columns.len();
        Self {
            rows,
            columns,
            selected_row,
            selected_column: 0,
            editing: false,
            editor: InputFieldState::new(),
            original_value: String::new(),
            focused: false,
            disabled: false,
            scroll,
            validators: vec![None; validator_count],
            last_validation_error: None,
        }
    }

    /// Returns the rows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
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
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
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
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_row
    }

    /// Alias for [`selected_index()`](Self::selected_index).
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }

    /// Returns a reference to the currently selected row.
    pub fn selected_row(&self) -> Option<&T> {
        self.selected_row.and_then(|i| self.rows.get(i))
    }

    /// Returns a reference to the currently selected item.
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
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
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
    pub fn selected_column(&self) -> usize {
        self.selected_column
    }

    /// Returns true if a cell is currently being edited.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
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
    pub fn editor_value(&self) -> &str {
        self.editor.value()
    }

    /// Returns the value of the currently selected cell.
    pub fn current_cell_value(&self) -> Option<String> {
        self.selected_row
            .and_then(|ri| self.rows.get(ri))
            .map(|row| {
                let cells = row.cells();
                cells.get(self.selected_column).cloned().unwrap_or_default()
            })
    }

    /// Returns the number of rows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
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
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
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
    /// use envision::component::{DataGridState, TableRow, Column};
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let state: DataGridState<Item> = DataGridState::default();
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Sets a validator for the given column index.
    ///
    /// The validator is called when an edit is confirmed. If it returns
    /// `Err(message)`, the edit is rejected and a `ValidationFailed` output
    /// is emitted.
    ///
    /// # Panics
    ///
    /// Panics if `col_index` is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    /// use std::sync::Arc;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String, age: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone(), self.age.clone()] }
    /// }
    ///
    /// let mut state = DataGridState::new(
    ///     vec![Item { name: "A".into(), age: "30".into() }],
    ///     vec![
    ///         Column::new("Name", Constraint::Min(10)),
    ///         Column::new("Age", Constraint::Min(5)),
    ///     ],
    /// );
    /// state.set_validator(1, Arc::new(|val: &str| {
    ///     val.parse::<u32>().map(|_| ()).map_err(|_| "Must be a number".to_string())
    /// }));
    /// ```
    pub fn set_validator(&mut self, col_index: usize, validator: CellValidator) {
        assert!(
            col_index < self.columns.len(),
            "Column index {col_index} out of bounds ({})",
            self.columns.len()
        );
        if self.validators.len() <= col_index {
            self.validators.resize_with(col_index + 1, || None);
        }
        self.validators[col_index] = Some(validator);
    }

    /// Sets a validator for the given column index (builder pattern).
    ///
    /// # Panics
    ///
    /// Panics if `col_index` is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    /// use std::sync::Arc;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String, age: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone(), self.age.clone()] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into(), age: "30".into() }],
    ///     vec![
    ///         Column::new("Name", Constraint::Min(10)),
    ///         Column::new("Age", Constraint::Min(5)),
    ///     ],
    /// ).with_validator(1, Arc::new(|val: &str| {
    ///     val.parse::<u32>().map(|_| ()).map_err(|_| "Must be a number".to_string())
    /// }));
    /// ```
    pub fn with_validator(mut self, col_index: usize, validator: CellValidator) -> Self {
        self.set_validator(col_index, validator);
        self
    }

    /// Returns the last validation error, if any.
    ///
    /// This is set when an edit is rejected by a validator and cleared
    /// when editing starts or a successful edit is confirmed.
    pub fn last_validation_error(&self) -> Option<&str> {
        self.last_validation_error.as_deref()
    }

    /// Clears the last validation error.
    pub fn clear_validation_error(&mut self) {
        self.last_validation_error = None;
    }

    /// Returns the number of visible columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DataGridState, TableRow, Column};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<String> { vec![self.name.clone()] }
    /// }
    ///
    /// let state = DataGridState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![
    ///         Column::new("Name", Constraint::Min(10)),
    ///         Column::new("Hidden", Constraint::Min(5)).with_visible(false),
    ///     ],
    /// );
    /// assert_eq!(state.visible_column_count(), 1);
    /// ```
    pub fn visible_column_count(&self) -> usize {
        self.columns.iter().filter(|c| c.is_visible()).count()
    }

    /// Returns indices of all visible columns.
    pub(super) fn visible_column_indices(&self) -> Vec<usize> {
        self.columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_visible())
            .map(|(i, _)| i)
            .collect()
    }

    /// Sets the rows, resetting selection and cancelling any edit.
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
    /// Returns true if the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the disabled state (builder pattern).
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Maps an input event to a data grid message.
    pub fn handle_event(&self, event: &Event) -> Option<DataGridMessage> {
        DataGrid::<T>::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<DataGridOutput<T>> {
        DataGrid::<T>::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: DataGridMessage) -> Option<DataGridOutput<T>> {
        DataGrid::<T>::update(self, msg)
    }

    /// Starts editing the current cell.
    ///
    /// Returns `false` if the current column is not editable.
    pub(super) fn start_editing(&mut self) -> bool {
        if let Some(col) = self.columns.get(self.selected_column) {
            if !col.is_editable() {
                return false;
            }
        }
        if let Some(cell_value) = self.current_cell_value() {
            self.original_value = cell_value.clone();
            self.editor.set_value(&cell_value);
            self.editor.set_focused(true);
            self.editing = true;
            self.last_validation_error = None;
            true
        } else {
            false
        }
    }

    /// Cancels the current edit, restoring the original value.
    pub(super) fn cancel_editing(&mut self) {
        self.editing = false;
        self.editor.set_focused(false);
    }
}
