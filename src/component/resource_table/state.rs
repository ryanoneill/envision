//! State for the `ResourceTable` component.

use super::{ResourceColumn, ResourceRow, RowStatus};
use crate::scroll::ScrollState;

/// State for the `ResourceTable` component.
///
/// # Examples
///
/// ```
/// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
/// use ratatui::layout::Constraint;
///
/// #[derive(Clone)]
/// struct Row { name: String }
/// impl ResourceRow for Row {
///     fn cells(&self) -> Vec<ResourceCell> { vec![ResourceCell::new(&self.name)] }
/// }
///
/// let state: ResourceTableState<Row> = ResourceTableState::new(vec![
///     ResourceColumn::new("NAME", Constraint::Length(20)),
/// ]);
/// assert_eq!(state.columns().len(), 1);
/// assert!(state.rows().is_empty());
/// ```
#[derive(Clone, Debug)]
pub struct ResourceTableState<T: ResourceRow> {
    pub(super) rows: Vec<T>,
    pub(super) columns: Vec<ResourceColumn>,
    pub(super) selected: Option<usize>,
    pub(super) scroll: ScrollState,
    pub(super) title: Option<String>,
}

impl<T: ResourceRow> PartialEq for ResourceTableState<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows
            && self.columns == other.columns
            && self.selected == other.selected
            && self.title == other.title
    }
}

impl<T: ResourceRow> Default for ResourceTableState<T> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            columns: Vec::new(),
            selected: None,
            scroll: ScrollState::new(0),
            title: None,
        }
    }
}

impl<T: ResourceRow> ResourceTableState<T> {
    /// Creates a new ResourceTableState with the given columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let state: ResourceTableState<R> = ResourceTableState::new(vec![
    ///     ResourceColumn::new("A", Constraint::Length(10)),
    /// ]);
    /// assert_eq!(state.columns().len(), 1);
    /// ```
    pub fn new(columns: Vec<ResourceColumn>) -> Self {
        Self {
            rows: Vec::new(),
            columns,
            selected: None,
            scroll: ScrollState::new(0),
            title: None,
        }
    }

    /// Sets the rows during construction.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let state = ResourceTableState::new(vec![ResourceColumn::new("A", Constraint::Length(5))])
    ///     .with_rows(vec![R, R, R]);
    /// assert_eq!(state.rows().len(), 3);
    /// ```
    pub fn with_rows(mut self, rows: Vec<T>) -> Self {
        self.scroll = ScrollState::new(rows.len());
        self.rows = rows;
        self
    }

    /// Sets the title shown in the border.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let state: ResourceTableState<R> = ResourceTableState::new(vec![])
    ///     .with_title("Pods");
    /// assert_eq!(state.title(), Some("Pods"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the initial selected index.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let state = ResourceTableState::new(vec![ResourceColumn::new("A", Constraint::Length(5))])
    ///     .with_rows(vec![R, R, R])
    ///     .with_selected(Some(1));
    /// assert_eq!(state.selected(), Some(1));
    /// ```
    pub fn with_selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    /// Returns the rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let state = ResourceTableState::new(vec![ResourceColumn::new("A", Constraint::Length(5))])
    ///     .with_rows(vec![R, R]);
    /// assert_eq!(state.rows().len(), 2);
    /// ```
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Returns the columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let state: ResourceTableState<R> = ResourceTableState::new(vec![
    ///     ResourceColumn::new("NAME", Constraint::Length(20)),
    ///     ResourceColumn::new("STATUS", Constraint::Length(15)),
    /// ]);
    /// assert_eq!(state.columns().len(), 2);
    /// ```
    pub fn columns(&self) -> &[ResourceColumn] {
        &self.columns
    }

    /// Returns the selected row index.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let state = ResourceTableState::new(vec![ResourceColumn::new("A", Constraint::Length(5))])
    ///     .with_rows(vec![R, R, R])
    ///     .with_selected(Some(1));
    /// assert_eq!(state.selected(), Some(1));
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Returns the title, if set.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceTableState, ResourceRow, ResourceCell};
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let state: ResourceTableState<R> = ResourceTableState::new(vec![])
    ///     .with_title("Pods");
    /// assert_eq!(state.title(), Some("Pods"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the selected row, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone, PartialEq, Debug)]
    /// struct R(u32);
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![ResourceCell::new(self.0.to_string())] }
    /// }
    ///
    /// let state = ResourceTableState::new(vec![ResourceColumn::new("A", Constraint::Length(5))])
    ///     .with_rows(vec![R(1), R(2), R(3)])
    ///     .with_selected(Some(1));
    /// assert_eq!(state.selected_row(), Some(&R(2)));
    /// ```
    pub fn selected_row(&self) -> Option<&T> {
        self.selected.and_then(|i| self.rows.get(i))
    }

    /// Replaces all rows. Preserves the selected row when possible by
    /// matching the previous index.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let mut state: ResourceTableState<R> = ResourceTableState::new(vec![]);
    /// state.set_rows(vec![R, R, R]);
    /// assert_eq!(state.rows().len(), 3);
    /// ```
    pub fn set_rows(&mut self, rows: Vec<T>) {
        let new_len = rows.len();
        self.rows = rows;
        self.scroll.set_content_length(new_len);
        if let Some(sel) = self.selected {
            if sel >= new_len {
                self.selected = if new_len == 0 {
                    None
                } else {
                    Some(new_len - 1)
                };
            }
        }
    }

    /// Sets the selected row index.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceColumn, ResourceTableState, ResourceRow, ResourceCell};
    /// use ratatui::layout::Constraint;
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let mut state = ResourceTableState::new(vec![ResourceColumn::new("A", Constraint::Length(5))])
    ///     .with_rows(vec![R, R, R]);
    /// state.set_selected(Some(2));
    /// assert_eq!(state.selected(), Some(2));
    /// ```
    pub fn set_selected(&mut self, selected: Option<usize>) {
        if let Some(idx) = selected {
            if idx < self.rows.len() {
                self.selected = Some(idx);
                self.scroll.ensure_visible(idx);
                return;
            }
        }
        self.selected = None;
    }

    /// Sets the title.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{ResourceTableState, ResourceRow, ResourceCell};
    ///
    /// #[derive(Clone)]
    /// struct R;
    /// impl ResourceRow for R {
    ///     fn cells(&self) -> Vec<ResourceCell> { vec![] }
    /// }
    ///
    /// let mut state: ResourceTableState<R> = ResourceTableState::new(vec![]);
    /// state.set_title(Some("Updated".to_string()));
    /// assert_eq!(state.title(), Some("Updated"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns whether any row has a non-None status (controls status column visibility).
    pub(crate) fn has_status_column(&self) -> bool {
        self.rows
            .iter()
            .any(|r| !matches!(r.status(), RowStatus::None))
    }

    /// Returns the scroll state (for rendering).
    pub(crate) fn scroll(&self) -> &ScrollState {
        &self.scroll
    }
}
