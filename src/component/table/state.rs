//! TableState constructors, builders, accessors, setters, and instance methods.
//!
//! Extracted from the main table module to keep file sizes manageable.

use super::{Column, SortDirection, Table, TableMessage, TableOutput, TableRow, TableState};
use crate::component::Component;
use crate::scroll::ScrollState;

impl<T: TableRow> TableState<T> {
    /// Creates a new table state with the given rows and columns.
    ///
    /// If there are rows, the first row is selected by default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    ///
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> {
    ///         vec![Cell::new(&self.name)]
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = TableState::with_selected(
    ///     vec![Item { name: "A".into() }, Item { name: "B".into() }],
    ///     vec![Column::new("Name", Constraint::Length(10))],
    ///     1,
    /// );
    /// assert_eq!(state.selected_index(), Some(1));
    /// ```
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
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
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
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = TableState::new(
    ///     vec![Item { name: "Alice".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Alias for [`selected_index()`](Self::selected_index).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = TableState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// assert_eq!(state.selected(), Some(0));
    /// ```
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
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let state = TableState::new(
    ///     vec![Item { name: "First".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// assert_eq!(state.selected_item().unwrap().name, "First");
    /// ```
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
    /// use envision::component::{Cell, Column, Table, TableRow, TableState, TableMessage, SortDirection, Component};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, Table, TableMessage, TableRow, TableState, Component};
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let mut state = TableState::new(
    ///     vec![Item { name: "B".into() }, Item { name: "A".into() }],
    ///     vec![Column::fixed("Name", 10).sortable()],
    /// );
    /// Table::<Item>::update(&mut state, TableMessage::SortBy(0));
    /// assert_eq!(state.sort_columns().len(), 1);
    /// ```
    pub fn sort_columns(&self) -> &[(usize, SortDirection)] {
        &self.sort_columns
    }

    /// Returns the number of rows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let empty: TableState<Item> = TableState::default();
    /// assert!(empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Sets the rows, clearing filter and sort, and adjusting selection.
    ///
    /// If there were rows selected, the selection is preserved if valid,
    /// otherwise clamped to the last row.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let mut state = TableState::new(
    ///     vec![Item { name: "A".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// state.set_rows(vec![Item { name: "X".into() }, Item { name: "Y".into() }]);
    /// assert_eq!(state.len(), 2);
    /// ```
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
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
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
    /// use envision::component::{Cell, Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
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
    /// use envision::component::{Cell, Column, TableRow, TableState};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let mut state = TableState::new(
    ///     vec![Item { name: "Alice".into() }, Item { name: "Bob".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// state.set_filter_text("ali");
    /// assert_eq!(state.visible_count(), 1);
    /// ```
    pub fn set_filter_text(&mut self, text: &str) {
        self.filter_text = text.to_string();
        self.rebuild_display_order();
    }

    /// Clears the filter, showing all rows.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState};
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let mut state = TableState::new(
    ///     vec![Item { name: "Alice".into() }, Item { name: "Bob".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// state.set_filter_text("Alice");
    /// assert_eq!(state.visible_count(), 1);
    /// state.clear_filter();
    /// assert_eq!(state.visible_count(), 2);
    /// ```
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.rebuild_display_order();
    }

    /// Rebuilds the display order by applying filter, then sort.
    pub(super) fn rebuild_display_order(&mut self) {
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
                        .any(|cell| cell.text().to_lowercase().contains(&filter_lower))
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
                    let val_a = cells_a.get(col).map(|c| c.text());
                    let val_b = cells_b.get(col).map(|c| c.text());
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Cell, Column, TableRow, TableState, TableMessage, TableOutput};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Item { name: String }
    /// impl TableRow for Item {
    ///     fn cells(&self) -> Vec<Cell> { vec![Cell::new(&self.name)] }
    /// }
    ///
    /// let mut state = TableState::new(
    ///     vec![Item { name: "A".into() }, Item { name: "B".into() }],
    ///     vec![Column::fixed("Name", 10)],
    /// );
    /// let output = state.update(TableMessage::Down);
    /// assert_eq!(output, Some(TableOutput::SelectionChanged(1)));
    /// ```
    pub fn update(&mut self, msg: TableMessage) -> Option<TableOutput<T>> {
        Table::update(self, msg)
    }
}
