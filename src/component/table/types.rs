//! Types for the table component.

use std::cmp::Ordering;
use std::sync::Arc;

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
/// A type alias for custom sort comparator functions.
///
/// Comparators receive two string cell values and return an [`Ordering`].
/// Use [`numeric_comparator`] or [`date_comparator`] for common cases,
/// or provide your own closure.
pub type SortComparator = Arc<dyn Fn(&str, &str) -> Ordering + Send + Sync>;

/// Column definition for a table.
///
/// Columns define the header text, width, whether the column
/// is sortable, and an optional custom sort comparator.
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
#[derive(Clone)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Column {
    header: String,
    #[cfg_attr(feature = "serialization", serde(skip))]
    width: Constraint,
    sortable: bool,
    editable: bool,
    visible: bool,
    default_sort: SortDirection,
    #[cfg_attr(feature = "serialization", serde(skip))]
    comparator: Option<SortComparator>,
}

impl std::fmt::Debug for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Column")
            .field("header", &self.header)
            .field("width", &self.width)
            .field("sortable", &self.sortable)
            .field("editable", &self.editable)
            .field("visible", &self.visible)
            .field("default_sort", &self.default_sort)
            .field("comparator", &self.comparator.as_ref().map(|_| ".."))
            .finish()
    }
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
            && self.width == other.width
            && self.sortable == other.sortable
            && self.editable == other.editable
            && self.visible == other.visible
            && self.default_sort == other.default_sort
        // comparator is not compared (function equality is not meaningful)
    }
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
            editable: true,
            visible: true,
            default_sort: SortDirection::Ascending,
            comparator: None,
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = Column::new("Name", Constraint::Length(20));
    /// assert_eq!(col.header(), "Name");
    /// ```
    pub fn header(&self) -> &str {
        &self.header
    }

    /// Returns the column width constraint.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = Column::new("Price", Constraint::Length(10));
    /// assert_eq!(col.width(), Constraint::Length(10));
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = Column::new("Name", Constraint::Length(20));
    /// assert!(!col.is_sortable());
    ///
    /// let sortable = col.sortable();
    /// assert!(sortable.is_sortable());
    /// ```
    pub fn is_sortable(&self) -> bool {
        self.sortable
    }

    /// Sets whether this column is editable (builder pattern).
    ///
    /// Columns are editable by default. Set to `false` to make a column
    /// read-only in a `DataGrid`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = Column::new("ID", Constraint::Length(10)).with_editable(false);
    /// assert!(!col.is_editable());
    /// ```
    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// Returns whether this column is editable.
    ///
    /// Defaults to `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = Column::new("Name", Constraint::Length(10));
    /// assert!(col.is_editable());
    ///
    /// let read_only = col.with_editable(false);
    /// assert!(!read_only.is_editable());
    /// ```
    pub fn is_editable(&self) -> bool {
        self.editable
    }

    /// Sets whether this column is visible (builder pattern).
    ///
    /// Columns are visible by default. Set to `false` to hide a column
    /// from rendering while preserving its data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = Column::new("Internal", Constraint::Length(10)).with_visible(false);
    /// assert!(!col.is_visible());
    /// ```
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Returns whether this column is visible.
    ///
    /// Defaults to `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = Column::new("Name", Constraint::Length(10));
    /// assert!(col.is_visible());
    ///
    /// let hidden = col.with_visible(false);
    /// assert!(!hidden.is_visible());
    /// ```
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets column visibility.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let mut col = Column::new("Internal", Constraint::Length(10));
    /// assert!(col.is_visible());
    /// col.set_visible(false);
    /// assert!(!col.is_visible());
    /// ```
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Sets whether this column is editable.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let mut col = Column::new("Name", Constraint::Length(10));
    /// assert!(col.is_editable());
    /// col.set_editable(false);
    /// assert!(!col.is_editable());
    /// ```
    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
    }

    /// Declares this column's natural sort direction. `SortToggle` and
    /// `AddSortToggle` use this when activating the column for the first
    /// time. Default: `Ascending`.
    ///
    /// Use `Descending` for columns where bigger-is-worse (latency,
    /// regression delta, error count).
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::{Column, SortDirection};
    /// use ratatui::layout::Constraint;
    ///
    /// let c = Column::new("delta", Constraint::Length(10))
    ///     .with_default_sort(SortDirection::Descending);
    /// assert_eq!(c.default_sort(), SortDirection::Descending);
    /// ```
    pub fn with_default_sort(mut self, dir: SortDirection) -> Self {
        self.default_sort = dir;
        self
    }

    /// Returns the column's natural sort direction.
    pub fn default_sort(&self) -> SortDirection {
        self.default_sort
    }

    /// Sets a custom sort comparator for this column.
    ///
    /// The comparator receives two cell values as `&str` and returns an
    /// [`Ordering`]. When sorting, this comparator is used instead of
    /// the default lexicographic comparison.
    ///
    /// Setting a comparator also makes the column sortable.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Column, numeric_comparator};
    ///
    /// let col = Column::fixed("Price", 10)
    ///     .with_comparator(numeric_comparator());
    /// assert!(col.is_sortable());
    /// assert!(col.comparator().is_some());
    /// ```
    pub fn with_comparator(mut self, comparator: SortComparator) -> Self {
        self.comparator = Some(comparator);
        self.sortable = true;
        self
    }

    /// Returns the custom comparator for this column, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Column, numeric_comparator};
    ///
    /// let col = Column::fixed("Name", 10);
    /// assert!(col.comparator().is_none());
    ///
    /// let numeric = Column::fixed("Price", 10).with_comparator(numeric_comparator());
    /// assert!(numeric.comparator().is_some());
    /// ```
    pub fn comparator(&self) -> Option<&SortComparator> {
        self.comparator.as_ref()
    }

    /// Sets the width of this column (builder method).
    ///
    /// This is useful for column resizing operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::Column;
    /// use ratatui::layout::Constraint;
    ///
    /// let mut col = Column::fixed("Name", 10);
    /// assert_eq!(col.width(), Constraint::Length(10));
    /// col.set_width(Constraint::Length(20));
    /// assert_eq!(col.width(), Constraint::Length(20));
    /// ```
    pub fn set_width(&mut self, width: Constraint) {
        self.width = width;
    }
}

/// Returns a comparator that sorts cell values as numbers.
///
/// Values that cannot be parsed as `f64` sort after valid numbers.
/// Two unparseable values are compared lexicographically.
///
/// # Example
///
/// ```rust
/// use envision::component::numeric_comparator;
/// use std::cmp::Ordering;
///
/// let cmp = numeric_comparator();
/// assert_eq!(cmp("2", "10"), Ordering::Less);
/// assert_eq!(cmp("10", "2"), Ordering::Greater);
/// assert_eq!(cmp("abc", "10"), Ordering::Greater);
/// ```
pub fn numeric_comparator() -> SortComparator {
    Arc::new(|a: &str, b: &str| {
        let pa = a.parse::<f64>();
        let pb = b.parse::<f64>();
        match (pa, pb) {
            (Ok(va), Ok(vb)) => va.partial_cmp(&vb).unwrap_or(Ordering::Equal),
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
            (Err(_), Err(_)) => a.cmp(b),
        }
    })
}

/// Returns a comparator that sorts cell values as dates in `YYYY-MM-DD` format.
///
/// Values that do not match the `YYYY-MM-DD` pattern sort after valid dates.
/// Two unparseable values are compared lexicographically.
///
/// # Example
///
/// ```rust
/// use envision::component::date_comparator;
/// use std::cmp::Ordering;
///
/// let cmp = date_comparator();
/// assert_eq!(cmp("2024-01-15", "2024-02-01"), Ordering::Less);
/// assert_eq!(cmp("2024-02-01", "2024-01-15"), Ordering::Greater);
/// ```
pub fn date_comparator() -> SortComparator {
    Arc::new(|a: &str, b: &str| {
        let pa = parse_date(a);
        let pb = parse_date(b);
        match (pa, pb) {
            (Some(va), Some(vb)) => va.cmp(&vb),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => a.cmp(b),
        }
    })
}

/// Parses a `YYYY-MM-DD` date string into a comparable tuple.
fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year = parts[0].parse::<i32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let day = parts[2].parse::<u32>().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some((year, month, day))
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

/// Pair of (column index, direction) for declarative initial sort.
///
/// Used with `TableState::with_initial_sorts` to bootstrap the table
/// into a sorted state on frame 1.
///
/// # Example
///
/// ```
/// use envision::component::{InitialSort, SortDirection};
/// let s = InitialSort { column: 4, direction: SortDirection::Descending };
/// assert_eq!(s.column, 4);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct InitialSort {
    /// Index of the column to sort by.
    pub column: usize,
    /// Sort direction to apply.
    pub direction: SortDirection,
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
    /// Sort by the given column index (primary sort).
    ///
    /// If already the primary sort column, toggles direction.
    /// If sorted ascending, becomes descending.
    /// If sorted descending, clears the sort.
    /// If a different column, replaces all sorts with this column ascending.
    SortBy(usize),
    /// Add a column to the sort stack as a tiebreaker.
    ///
    /// If the column is already in the sort stack, toggles its direction.
    /// If not present, adds it as the lowest-priority sort.
    AddSort(usize),
    /// Clear all sorts, returning to original order.
    ClearSort,
    /// Increase the width of the column at the given index.
    IncreaseColumnWidth(usize),
    /// Decrease the width of the column at the given index.
    ///
    /// The width will not go below the minimum of 3 characters.
    DecreaseColumnWidth(usize),
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
    /// A column was resized.
    ColumnResized {
        /// The column that was resized.
        column: usize,
        /// The new width of the column.
        width: u16,
    },
}
