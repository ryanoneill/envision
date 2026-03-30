//! Types for the table component.

use ratatui::layout::Constraint;

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
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Column {
    header: String,
    #[cfg_attr(feature = "serialization", serde(skip))]
    width: Constraint,
    sortable: bool,
}

impl Column {
    /// Creates a new column with the given header and width.
    ///
    /// The column is not sortable by default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = Column::new("Name", Constraint::Length(20));
    /// assert_eq!(col.header(), "Name");
    /// assert!(!col.is_sortable());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = Column::new("Name", Constraint::Length(20)).sortable();
    /// assert!(col.is_sortable());
    /// ```
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

    /// Creates a column with a fixed width.
    ///
    /// This is a shorthand for `Column::new(header, Constraint::Length(width))`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    ///
    /// let col = Column::fixed("Name", 20);
    /// assert_eq!(col.header(), "Name");
    /// ```
    pub fn fixed(header: impl Into<String>, width: u16) -> Self {
        Self::new(header, Constraint::Length(width))
    }

    /// Creates a column with a minimum width.
    ///
    /// This is a shorthand for `Column::new(header, Constraint::Min(width))`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    ///
    /// let col = Column::min("Description", 10);
    /// ```
    pub fn min(header: impl Into<String>, width: u16) -> Self {
        Self::new(header, Constraint::Min(width))
    }

    /// Creates a column that takes a percentage of available width.
    ///
    /// This is a shorthand for `Column::new(header, Constraint::Percentage(percent))`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    ///
    /// let col = Column::percent("Status", 25);
    /// ```
    pub fn percent(header: impl Into<String>, percent: u16) -> Self {
        Self::new(header, Constraint::Percentage(percent))
    }

    /// Returns whether this column is sortable.
    pub fn is_sortable(&self) -> bool {
        self.sortable
    }
}

/// Sort direction for table columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum SortDirection {
    /// Sort in ascending order (A-Z, 0-9).
    #[default]
    Ascending,
    /// Sort in descending order (Z-A, 9-0).
    Descending,
}

impl SortDirection {
    /// Returns the opposite sort direction.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SortDirection;
    ///
    /// assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
    /// assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
    /// ```
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
    /// Set the filter text for searching rows.
    SetFilter(String),
    /// Clear the filter text.
    ClearFilter,
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
    /// The filter text changed.
    FilterChanged(String),
}
