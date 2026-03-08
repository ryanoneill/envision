//! A data grid with inline cell editing.
//!
//! `DataGrid` wraps [`Table`](super::Table) adding column navigation and
//! inline cell editing. Press Enter to edit a cell, Escape to cancel, and
//! Enter again to confirm the edit.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, DataGrid, DataGridState,
//!     DataGridMessage, DataGridOutput, TableRow, Column,
//! };
//! use ratatui::layout::Constraint;
//!
//! #[derive(Clone, Debug)]
//! struct Person { name: String, age: String }
//!
//! impl TableRow for Person {
//!     fn cells(&self) -> Vec<String> {
//!         vec![self.name.clone(), self.age.clone()]
//!     }
//! }
//!
//! let rows = vec![
//!     Person { name: "Alice".into(), age: "30".into() },
//!     Person { name: "Bob".into(), age: "25".into() },
//! ];
//! let columns = vec![
//!     Column::new("Name", Constraint::Min(10)),
//!     Column::new("Age", Constraint::Min(5)),
//! ];
//! let mut state = DataGridState::new(rows, columns);
//! DataGrid::set_focused(&mut state, true);
//!
//! assert_eq!(state.selected_column(), 0);
//! assert!(!state.is_editing());
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Row, Table as RatatuiTable};

use super::{Column, Component, Focusable, InputFieldMessage, InputFieldState, TableRow};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Messages that can be sent to a DataGrid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataGridMessage {
    /// Move selection up by one row.
    Up,
    /// Move selection down by one row.
    Down,
    /// Move selection to the first row.
    First,
    /// Move selection to the last row.
    Last,
    /// Move the column cursor left.
    Left,
    /// Move the column cursor right.
    Right,
    /// Start editing the current cell, or confirm the edit.
    Enter,
    /// Cancel the current edit.
    Cancel,
    /// Type a character while editing.
    Input(char),
    /// Delete the character before the cursor while editing.
    Backspace,
    /// Delete the character after the cursor while editing.
    Delete,
    /// Move cursor to the start of the cell while editing.
    Home,
    /// Move cursor to the end of the cell while editing.
    End,
}

/// Output messages from a DataGrid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataGridOutput<T: Clone> {
    /// A cell was edited. Contains the row index, column index, and new value.
    CellEdited {
        /// The row index of the edited cell.
        row: usize,
        /// The column index of the edited cell.
        column: usize,
        /// The new value as a string.
        value: String,
    },
    /// A row was selected (Enter pressed when not editing).
    Selected(T),
    /// The row selection changed.
    SelectionChanged(usize),
    /// The column cursor moved.
    ColumnChanged(usize),
    /// An edit was cancelled.
    EditCancelled,
}

/// State for a DataGrid component.
///
/// Contains the table data, column cursor, and inline editor state.
#[derive(Clone, Debug)]
pub struct DataGridState<T: TableRow> {
    /// The rows of data.
    rows: Vec<T>,
    /// Column definitions.
    columns: Vec<Column>,
    /// Currently selected row index.
    selected_row: Option<usize>,
    /// Currently selected column index.
    selected_column: usize,
    /// Whether the cell editor is active.
    editing: bool,
    /// The inline editor state.
    editor: InputFieldState,
    /// Value before editing started (for cancel).
    original_value: String,
    /// Whether the overall component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
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
        }
    }

    /// Returns the rows.
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Returns the columns.
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
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns the number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns true if the grid has no rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Sets the rows, resetting selection and cancelling any edit.
    pub fn set_rows(&mut self, rows: Vec<T>) {
        self.editing = false;
        self.rows = rows;
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
    fn start_editing(&mut self) {
        if let Some(cell_value) = self.current_cell_value() {
            self.original_value = cell_value.clone();
            self.editor.set_value(&cell_value);
            self.editor.set_focused(true);
            self.editing = true;
        }
    }

    /// Cancels the current edit, restoring the original value.
    fn cancel_editing(&mut self) {
        self.editing = false;
        self.editor.set_focused(false);
    }
}

/// A data grid with inline cell editing.
///
/// Extends table functionality with column navigation (Left/Right)
/// and inline cell editing (Enter to edit, Escape to cancel).
///
/// # Type Parameters
///
/// - `T`: The row type. Must implement [`TableRow`] and `Clone`.
///
/// # Navigation
///
/// - `Up` / `Down` / `j` / `k` — Move row selection
/// - `Left` / `Right` / `h` / `l` — Move column cursor
/// - `Home` / `End` — First/last row (or cursor position when editing)
/// - `Enter` — Start editing or confirm edit
/// - `Escape` — Cancel edit
///
/// # Editing
///
/// When Enter is pressed on a cell, the editor opens with the cell's
/// current value. The user types to modify, then presses Enter to
/// confirm or Escape to cancel. On confirm, a `CellEdited` output
/// is emitted. The parent is responsible for updating the row data.
pub struct DataGrid<T: Clone>(PhantomData<T>);

impl<T: TableRow + 'static> Component for DataGrid<T> {
    type State = DataGridState<T>;
    type Message = DataGridMessage;
    type Output = DataGridOutput<T>;

    fn init() -> Self::State {
        DataGridState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        if let Some(key) = event.as_key() {
            if state.editing {
                // Editing mode key bindings
                match key.code {
                    KeyCode::Enter => Some(DataGridMessage::Enter),
                    KeyCode::Esc => Some(DataGridMessage::Cancel),
                    KeyCode::Char(c) => Some(DataGridMessage::Input(c)),
                    KeyCode::Backspace => Some(DataGridMessage::Backspace),
                    KeyCode::Delete => Some(DataGridMessage::Delete),
                    KeyCode::Home => Some(DataGridMessage::Home),
                    KeyCode::End => Some(DataGridMessage::End),
                    _ => None,
                }
            } else {
                // Navigation mode key bindings
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => Some(DataGridMessage::Up),
                    KeyCode::Down | KeyCode::Char('j') => Some(DataGridMessage::Down),
                    KeyCode::Left | KeyCode::Char('h') => Some(DataGridMessage::Left),
                    KeyCode::Right | KeyCode::Char('l') => Some(DataGridMessage::Right),
                    KeyCode::Home => Some(DataGridMessage::First),
                    KeyCode::End => Some(DataGridMessage::Last),
                    KeyCode::Enter => Some(DataGridMessage::Enter),
                    KeyCode::Esc => Some(DataGridMessage::Cancel),
                    _ => None,
                }
            }
        } else {
            None
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled || state.rows.is_empty() {
            return None;
        }

        if state.editing {
            // Editing mode
            match msg {
                DataGridMessage::Enter => {
                    // Confirm edit
                    let value = state.editor.value().to_string();
                    let row = state.selected_row.unwrap_or(0);
                    let col = state.selected_column;
                    state.cancel_editing();
                    Some(DataGridOutput::CellEdited {
                        row,
                        column: col,
                        value,
                    })
                }
                DataGridMessage::Cancel => {
                    state.cancel_editing();
                    Some(DataGridOutput::EditCancelled)
                }
                DataGridMessage::Input(c) => {
                    state.editor.update(InputFieldMessage::Insert(c));
                    None
                }
                DataGridMessage::Backspace => {
                    state.editor.update(InputFieldMessage::Backspace);
                    None
                }
                DataGridMessage::Delete => {
                    state.editor.update(InputFieldMessage::Delete);
                    None
                }
                DataGridMessage::Home => {
                    state.editor.update(InputFieldMessage::Home);
                    None
                }
                DataGridMessage::End => {
                    state.editor.update(InputFieldMessage::End);
                    None
                }
                _ => None,
            }
        } else {
            // Navigation mode
            let len = state.rows.len();
            let current_row = state.selected_row.unwrap_or(0);
            let col_count = state.columns.len();

            match msg {
                DataGridMessage::Up => {
                    let new_index = current_row.saturating_sub(1);
                    if new_index != current_row {
                        state.selected_row = Some(new_index);
                        Some(DataGridOutput::SelectionChanged(new_index))
                    } else {
                        None
                    }
                }
                DataGridMessage::Down => {
                    let new_index = (current_row + 1).min(len - 1);
                    if new_index != current_row {
                        state.selected_row = Some(new_index);
                        Some(DataGridOutput::SelectionChanged(new_index))
                    } else {
                        None
                    }
                }
                DataGridMessage::First => {
                    if current_row != 0 {
                        state.selected_row = Some(0);
                        Some(DataGridOutput::SelectionChanged(0))
                    } else {
                        None
                    }
                }
                DataGridMessage::Last => {
                    let last = len - 1;
                    if current_row != last {
                        state.selected_row = Some(last);
                        Some(DataGridOutput::SelectionChanged(last))
                    } else {
                        None
                    }
                }
                DataGridMessage::Left => {
                    if col_count > 0 && state.selected_column > 0 {
                        state.selected_column -= 1;
                        Some(DataGridOutput::ColumnChanged(state.selected_column))
                    } else {
                        None
                    }
                }
                DataGridMessage::Right => {
                    if col_count > 0 && state.selected_column < col_count - 1 {
                        state.selected_column += 1;
                        Some(DataGridOutput::ColumnChanged(state.selected_column))
                    } else {
                        None
                    }
                }
                DataGridMessage::Enter => {
                    state.start_editing();
                    None
                }
                DataGridMessage::Cancel => None,
                _ => None,
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if state.columns.is_empty() {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::table("data_grid")
                    .with_focus(state.focused)
                    .with_disabled(state.disabled),
            );
        });

        let widths: Vec<Constraint> = state.columns.iter().map(|c| c.width()).collect();

        // Build header row
        let headers: Vec<String> = state
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                if !state.editing && state.focused && i == state.selected_column {
                    format!("[{}]", col.header())
                } else {
                    col.header().to_string()
                }
            })
            .collect();

        let header_style = if state.disabled {
            theme.disabled_style()
        } else {
            Style::default().add_modifier(Modifier::BOLD)
        };

        let header = Row::new(headers).style(header_style).bottom_margin(1);

        // Build data rows
        let rows: Vec<Row> = state
            .rows
            .iter()
            .enumerate()
            .map(|(row_idx, row)| {
                let cells = row.cells();
                let display_cells: Vec<String> = cells
                    .iter()
                    .enumerate()
                    .map(|(col_idx, cell)| {
                        if state.editing
                            && state.selected_row == Some(row_idx)
                            && col_idx == state.selected_column
                        {
                            // Show editor content for the cell being edited
                            state.editor.value().to_string()
                        } else {
                            cell.clone()
                        }
                    })
                    .collect();
                Row::new(display_cells)
            })
            .collect();

        let border_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let highlight_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.selected_highlight_style(state.focused)
        };

        let table = RatatuiTable::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .row_highlight_style(highlight_style)
            .highlight_symbol("> ");

        let mut table_state = ratatui::widgets::TableState::default();
        table_state.select(state.selected_row);
        frame.render_stateful_widget(table, area, &mut table_state);

        // Show cursor when editing
        if state.editing && state.focused {
            if let Some(row_idx) = state.selected_row {
                // Calculate cursor position for the edit cell
                // This is approximate — exact positioning depends on column widths
                let content_area = area.inner(Margin::new(1, 1));
                let col_areas = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(state.columns.iter().map(|c| c.width()).collect::<Vec<_>>())
                    .split(content_area);

                if let Some(col_area) = col_areas.get(state.selected_column) {
                    // +2 for header row and margin
                    let cursor_y = content_area.y + 2 + (row_idx as u16);
                    let cursor_x = col_area.x + state.editor.cursor_display_position() as u16;
                    if cursor_y < area.bottom() && cursor_x < col_area.right() {
                        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
                    }
                }
            }
        }
    }
}

impl<T: TableRow + 'static> Focusable for DataGrid<T> {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
