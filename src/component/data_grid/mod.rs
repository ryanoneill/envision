//! A data grid with inline cell editing.
//!
//! [`DataGrid<T>`] wraps [`Table`](super::Table) adding column navigation and
//! inline cell editing. Press Enter to edit a cell, Escape to cancel, and
//! Enter again to confirm the edit. State is stored in [`DataGridState<T>`],
//! updated via [`DataGridMessage`], and produces [`DataGridOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, DataGrid, DataGridState,
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
//!
//! assert_eq!(state.selected_column(), 0);
//! assert!(!state.is_editing());
//! ```

mod state;

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Row, Table as RatatuiTable};

use super::{
    Column, Component, EventContext, InputFieldMessage, InputFieldState, RenderContext, TableRow,
};
use crate::input::{Event, Key};
use crate::scroll::ScrollState;

/// Messages that can be sent to a DataGrid.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
    /// Hide a column by index.
    HideColumn(usize),
    /// Show a column by index.
    ShowColumn(usize),
    /// Toggle column visibility by index.
    ToggleColumn(usize),
}

/// Output messages from a DataGrid.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
    /// A column was hidden.
    ColumnHidden(usize),
    /// A column was shown.
    ColumnShown(usize),
}

/// State for a DataGrid component.
///
/// Contains the table data, column cursor, and inline editor state.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
    #[cfg_attr(feature = "serialization", serde(skip))]
    editor: InputFieldState,
    /// Value before editing started (for cancel).
    original_value: String,
    /// Scroll state for scrollbar rendering.
    #[cfg_attr(feature = "serialization", serde(skip))]
    scroll: ScrollState,
}

impl<T: TableRow + PartialEq> PartialEq for DataGridState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows
            && self.columns == other.columns
            && self.selected_row == other.selected_row
            && self.selected_column == other.selected_column
            && self.editing == other.editing
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
            scroll: ScrollState::default(),
        }
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

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        if let Some(key) = event.as_key() {
            if state.editing {
                // Editing mode key bindings
                match key.code {
                    Key::Enter => Some(DataGridMessage::Enter),
                    Key::Esc => Some(DataGridMessage::Cancel),
                    Key::Char(_) => key.raw_char.map(DataGridMessage::Input),
                    Key::Backspace => Some(DataGridMessage::Backspace),
                    Key::Delete => Some(DataGridMessage::Delete),
                    Key::Home => Some(DataGridMessage::Home),
                    Key::End => Some(DataGridMessage::End),
                    _ => None,
                }
            } else {
                // Navigation mode key bindings
                match key.code {
                    Key::Up | Key::Char('k') => Some(DataGridMessage::Up),
                    Key::Down | Key::Char('j') => Some(DataGridMessage::Down),
                    Key::Left | Key::Char('h') => Some(DataGridMessage::Left),
                    Key::Right | Key::Char('l') => Some(DataGridMessage::Right),
                    Key::Home => Some(DataGridMessage::First),
                    Key::End => Some(DataGridMessage::Last),
                    Key::Enter => Some(DataGridMessage::Enter),
                    Key::Esc => Some(DataGridMessage::Cancel),
                    _ => None,
                }
            }
        } else {
            None
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.rows.is_empty() {
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
                    // Skip hidden columns when navigating left
                    let mut new_col = state.selected_column;
                    while new_col > 0 {
                        new_col -= 1;
                        if state.columns.get(new_col).is_some_and(|c| c.is_visible()) {
                            state.selected_column = new_col;
                            return Some(DataGridOutput::ColumnChanged(new_col));
                        }
                    }
                    None
                }
                DataGridMessage::Right => {
                    // Skip hidden columns when navigating right
                    let mut new_col = state.selected_column;
                    while new_col < col_count.saturating_sub(1) {
                        new_col += 1;
                        if state.columns.get(new_col).is_some_and(|c| c.is_visible()) {
                            state.selected_column = new_col;
                            return Some(DataGridOutput::ColumnChanged(new_col));
                        }
                    }
                    None
                }
                DataGridMessage::Enter => {
                    // Only allow editing if the current column is editable
                    if state
                        .columns
                        .get(state.selected_column)
                        .is_some_and(|c| c.is_editable())
                    {
                        state.start_editing();
                    }
                    None
                }
                DataGridMessage::Cancel => None,
                DataGridMessage::HideColumn(idx) => {
                    if let Some(col) = state.columns.get_mut(idx) {
                        col.set_visible(false);
                        Some(DataGridOutput::ColumnHidden(idx))
                    } else {
                        None
                    }
                }
                DataGridMessage::ShowColumn(idx) => {
                    if let Some(col) = state.columns.get_mut(idx) {
                        col.set_visible(true);
                        Some(DataGridOutput::ColumnShown(idx))
                    } else {
                        None
                    }
                }
                DataGridMessage::ToggleColumn(idx) => {
                    if let Some(col) = state.columns.get_mut(idx) {
                        let was_visible = col.is_visible();
                        col.set_visible(!was_visible);
                        if was_visible {
                            Some(DataGridOutput::ColumnHidden(idx))
                        } else {
                            Some(DataGridOutput::ColumnShown(idx))
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if state.columns.is_empty() {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::table("data_grid")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let widths: Vec<Constraint> = state
            .columns
            .iter()
            .map(|c| {
                if c.is_visible() {
                    c.width()
                } else {
                    Constraint::Length(0)
                }
            })
            .collect();

        // Build header row
        let headers: Vec<String> = state
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                if !state.editing && ctx.focused && i == state.selected_column {
                    format!("[{}]", col.header())
                } else {
                    col.header().to_string()
                }
            })
            .collect();

        let header_style = if ctx.disabled {
            ctx.theme.disabled_style()
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

        let border_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else if ctx.focused {
            ctx.theme.focused_border_style()
        } else {
            ctx.theme.border_style()
        };

        let highlight_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else {
            ctx.theme.selected_highlight_style(ctx.focused)
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
        ctx.frame
            .render_stateful_widget(table, ctx.area, &mut table_state);

        // Render scrollbar by mirroring the offset from ratatui's TableState
        let inner = ctx.area.inner(Margin::new(1, 1));
        // Viewport for data rows: inner height minus header row (1) and bottom margin (1)
        let data_viewport = (inner.height as usize).saturating_sub(2);
        if data_viewport > 0 && state.rows.len() > data_viewport {
            let mut bar_scroll = ScrollState::new(state.rows.len());
            bar_scroll.set_viewport_height(data_viewport);
            bar_scroll.set_offset(table_state.offset());
            crate::scroll::render_scrollbar_inside_border(
                &bar_scroll,
                ctx.frame,
                ctx.area,
                ctx.theme,
            );
        }

        // Show cursor when editing
        if state.editing && ctx.focused {
            if let Some(row_idx) = state.selected_row {
                // Calculate cursor position for the edit cell
                // This is approximate — exact positioning depends on column widths
                let content_area = ctx.area.inner(Margin::new(1, 1));
                let col_areas = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(state.columns.iter().map(|c| c.width()).collect::<Vec<_>>())
                    .split(content_area);

                if let Some(col_area) = col_areas.get(state.selected_column) {
                    // +2 for header row and margin
                    let cursor_y = content_area.y + 2 + (row_idx as u16);
                    let cursor_x = col_area.x + state.editor.cursor_display_position() as u16;
                    if cursor_y < ctx.area.bottom() && cursor_x < col_area.right() {
                        ctx.frame
                            .set_cursor_position(Position::new(cursor_x, cursor_y));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
