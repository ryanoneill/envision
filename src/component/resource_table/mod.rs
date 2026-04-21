//! A table component with per-cell styling, age formatting, and row status indicators.
//!
//! `ResourceTable` extends the basic `Table` pattern with rich semantics:
//! cells can be styled by health (success/warning/error), durations are
//! auto-formatted as ages ("3m12s"), and rows can carry status indicators
//! (a colored dot in the leftmost column).
//!
//! This is the foundation for K8s-style resource lists where engineers need
//! to see status at a glance without reading every cell.
//!
//! # Examples
//!
//! ```
//! use envision::component::resource_table::{
//!     ResourceCell, ResourceColumn, ResourceRow, ResourceTableState, RowStatus,
//! };
//! use ratatui::layout::Constraint;
//!
//! #[derive(Clone)]
//! struct PodRow {
//!     name: String,
//!     status: String,
//! }
//!
//! impl ResourceRow for PodRow {
//!     fn cells(&self) -> Vec<ResourceCell> {
//!         vec![
//!             ResourceCell::new(&self.name),
//!             if self.status == "Running" {
//!                 ResourceCell::success(&self.status)
//!             } else {
//!                 ResourceCell::error(&self.status)
//!             },
//!         ]
//!     }
//!
//!     fn status(&self) -> RowStatus {
//!         if self.status == "Running" { RowStatus::Healthy } else { RowStatus::Error }
//!     }
//! }
//!
//! let state = ResourceTableState::new(vec![
//!     ResourceColumn::new("NAME", Constraint::Length(20)),
//!     ResourceColumn::new("STATUS", Constraint::Length(20)),
//! ])
//! .with_rows(vec![
//!     PodRow { name: "nginx-abc".to_string(), status: "Running".to_string() },
//!     PodRow { name: "nginx-xyz".to_string(), status: "CrashLoopBackOff".to_string() },
//! ]);
//!
//! assert_eq!(state.rows().len(), 2);
//! ```

use std::time::Duration;

use ratatui::layout::{Alignment, Constraint};
use ratatui::style::{Color, Style};

use crate::component::Component;
use crate::component::context::{EventContext, RenderContext};
use crate::input::{Event, Key};

mod render;
mod state;

#[cfg(test)]
mod tests;

pub use state::ResourceTableState;

// ---------------------------------------------------------------------------
// CellStyle
// ---------------------------------------------------------------------------

/// Semantic styling for a `ResourceCell`.
///
/// Maps to a ratatui `Style` at render time using the theme's colors.
///
/// # Examples
///
/// ```
/// use envision::component::resource_table::CellStyle;
///
/// let style = CellStyle::default();
/// assert_eq!(style, CellStyle::Default);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum CellStyle {
    /// No special styling — uses theme default.
    #[default]
    #[cfg_attr(feature = "serialization", serde(rename = "default"))]
    Default,
    /// Green — indicates a healthy/passing state.
    Success,
    /// Yellow — indicates a warning state.
    Warning,
    /// Red — indicates an error state.
    Error,
    /// Dark gray — for de-emphasized text like ages or counts.
    Muted,
    /// Custom ratatui style.
    Custom(#[cfg_attr(feature = "serialization", serde(skip))] Style),
}

impl CellStyle {
    /// Converts the semantic style into a ratatui `Style`.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::CellStyle;
    /// use ratatui::style::Color;
    ///
    /// let style = CellStyle::Success.to_style();
    /// assert_eq!(style.fg, Some(Color::Green));
    /// ```
    pub fn to_style(&self) -> Style {
        match self {
            CellStyle::Default => Style::default(),
            CellStyle::Success => Style::default().fg(Color::Green),
            CellStyle::Warning => Style::default().fg(Color::Yellow),
            CellStyle::Error => Style::default().fg(Color::Red),
            CellStyle::Muted => Style::default().fg(Color::DarkGray),
            CellStyle::Custom(style) => *style,
        }
    }
}

// ---------------------------------------------------------------------------
// ResourceCell
// ---------------------------------------------------------------------------

/// A single styled cell in a `ResourceTable`.
///
/// Construct with `new()` for plain text or with `success()`, `warning()`,
/// `error()`, `muted()` for semantic colors. Use `age()` for auto-formatted
/// duration values.
///
/// # Examples
///
/// ```
/// use envision::component::resource_table::{CellStyle, ResourceCell};
/// use std::time::Duration;
///
/// let healthy = ResourceCell::success("Running");
/// assert_eq!(healthy.text(), "Running");
/// assert_eq!(healthy.style(), &CellStyle::Success);
///
/// let age = ResourceCell::age(Duration::from_secs(192));
/// assert_eq!(age.text(), "3m12s");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ResourceCell {
    text: String,
    style: CellStyle,
}

impl ResourceCell {
    /// Creates a new cell with default styling.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::ResourceCell;
    ///
    /// let cell = ResourceCell::new("hello");
    /// assert_eq!(cell.text(), "hello");
    /// ```
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Default,
        }
    }

    /// Creates a green (success) cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{CellStyle, ResourceCell};
    ///
    /// let cell = ResourceCell::success("OK");
    /// assert_eq!(cell.style(), &CellStyle::Success);
    /// ```
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Success,
        }
    }

    /// Creates a yellow (warning) cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{CellStyle, ResourceCell};
    ///
    /// let cell = ResourceCell::warning("Degraded");
    /// assert_eq!(cell.style(), &CellStyle::Warning);
    /// ```
    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Warning,
        }
    }

    /// Creates a red (error) cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{CellStyle, ResourceCell};
    ///
    /// let cell = ResourceCell::error("CrashLoopBackOff");
    /// assert_eq!(cell.style(), &CellStyle::Error);
    /// ```
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Error,
        }
    }

    /// Creates a dark-gray (muted) cell.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{CellStyle, ResourceCell};
    ///
    /// let cell = ResourceCell::muted("(none)");
    /// assert_eq!(cell.style(), &CellStyle::Muted);
    /// ```
    pub fn muted(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Muted,
        }
    }

    /// Creates a cell with a custom ratatui style.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{CellStyle, ResourceCell};
    /// use ratatui::style::{Color, Style};
    ///
    /// let style = Style::default().fg(Color::Cyan);
    /// let cell = ResourceCell::styled("custom", style);
    /// assert_eq!(cell.style(), &CellStyle::Custom(style));
    /// ```
    pub fn styled(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Custom(style),
        }
    }

    /// Creates a muted cell containing a formatted duration ("3m12s").
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::ResourceCell;
    /// use std::time::Duration;
    ///
    /// assert_eq!(ResourceCell::age(Duration::from_secs(45)).text(), "45s");
    /// assert_eq!(ResourceCell::age(Duration::from_secs(192)).text(), "3m12s");
    /// assert_eq!(ResourceCell::age(Duration::from_secs(8100)).text(), "2h15m");
    /// assert_eq!(ResourceCell::age(Duration::from_secs(360_000)).text(), "4d4h");
    /// ```
    pub fn age(duration: Duration) -> Self {
        Self {
            text: format_age(duration),
            style: CellStyle::Muted,
        }
    }

    /// Returns the cell's text.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::ResourceCell;
    ///
    /// let cell = ResourceCell::new("hello");
    /// assert_eq!(cell.text(), "hello");
    /// ```
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the cell's style.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::{CellStyle, ResourceCell};
    ///
    /// let cell = ResourceCell::new("hello");
    /// assert_eq!(cell.style(), &CellStyle::Default);
    /// ```
    pub fn style(&self) -> &CellStyle {
        &self.style
    }
}

/// Formats a duration as a compact age string.
pub(crate) fn format_age(d: Duration) -> String {
    let total_secs = d.as_secs();
    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        let m = total_secs / 60;
        let s = total_secs % 60;
        if s == 0 {
            format!("{}m", m)
        } else {
            format!("{}m{}s", m, s)
        }
    } else if total_secs < 86_400 {
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        if m == 0 {
            format!("{}h", h)
        } else {
            format!("{}h{}m", h, m)
        }
    } else {
        let d = total_secs / 86_400;
        let h = (total_secs % 86_400) / 3600;
        if h == 0 {
            format!("{}d", d)
        } else {
            format!("{}d{}h", d, h)
        }
    }
}

// ---------------------------------------------------------------------------
// RowStatus
// ---------------------------------------------------------------------------

/// Row-level status indicator displayed in the leftmost column.
///
/// # Examples
///
/// ```
/// use envision::component::resource_table::RowStatus;
///
/// let status = RowStatus::default();
/// assert_eq!(status, RowStatus::None);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum RowStatus {
    /// No indicator (the status column is hidden if no row has one).
    #[default]
    #[cfg_attr(feature = "serialization", serde(rename = "none"))]
    None,
    /// Green ● — healthy.
    Healthy,
    /// Yellow ▲ — warning.
    Warning,
    /// Red ✖ — error.
    Error,
    /// Gray ? — unknown.
    Unknown,
    /// Custom symbol and color.
    Custom {
        /// The character to display.
        symbol: &'static str,
        /// The color of the symbol.
        color: Color,
    },
}

impl RowStatus {
    /// Returns the symbol and color for this status, or None for `RowStatus::None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::RowStatus;
    /// use ratatui::style::Color;
    ///
    /// let healthy = RowStatus::Healthy;
    /// let (symbol, color) = healthy.indicator().unwrap();
    /// assert_eq!(symbol, "\u{25cf}");
    /// assert_eq!(color, Color::Green);
    /// ```
    pub fn indicator(&self) -> Option<(&str, Color)> {
        match self {
            RowStatus::None => None,
            RowStatus::Healthy => Some(("\u{25cf}", Color::Green)), // ●
            RowStatus::Warning => Some(("\u{25b2}", Color::Yellow)), // ▲
            RowStatus::Error => Some(("\u{2716}", Color::Red)),     // ✖
            RowStatus::Unknown => Some(("?", Color::DarkGray)),
            RowStatus::Custom { symbol, color } => Some((symbol, *color)),
        }
    }
}

// ---------------------------------------------------------------------------
// ResourceColumn
// ---------------------------------------------------------------------------

/// A column definition for a `ResourceTable`.
///
/// # Examples
///
/// ```
/// use envision::component::resource_table::ResourceColumn;
/// use ratatui::layout::{Alignment, Constraint};
///
/// let col = ResourceColumn::new("NAME", Constraint::Length(20))
///     .with_alignment(Alignment::Left);
/// assert_eq!(col.header(), "NAME");
/// assert_eq!(col.alignment(), Alignment::Left);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceColumn {
    header: String,
    width: Constraint,
    alignment: Alignment,
}

impl ResourceColumn {
    /// Creates a new column with the given header and width.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::ResourceColumn;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = ResourceColumn::new("STATUS", Constraint::Length(15));
    /// assert_eq!(col.header(), "STATUS");
    /// ```
    pub fn new(header: impl Into<String>, width: Constraint) -> Self {
        Self {
            header: header.into(),
            width,
            alignment: Alignment::Left,
        }
    }

    /// Sets the column alignment.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::ResourceColumn;
    /// use ratatui::layout::{Alignment, Constraint};
    ///
    /// let col = ResourceColumn::new("AGE", Constraint::Length(8))
    ///     .with_alignment(Alignment::Right);
    /// assert_eq!(col.alignment(), Alignment::Right);
    /// ```
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Returns the column header text.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::ResourceColumn;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = ResourceColumn::new("NAME", Constraint::Length(20));
    /// assert_eq!(col.header(), "NAME");
    /// ```
    pub fn header(&self) -> &str {
        &self.header
    }

    /// Returns the column width constraint.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::ResourceColumn;
    /// use ratatui::layout::Constraint;
    ///
    /// let col = ResourceColumn::new("X", Constraint::Length(10));
    /// assert_eq!(col.width(), Constraint::Length(10));
    /// ```
    pub fn width(&self) -> Constraint {
        self.width
    }

    /// Returns the column alignment.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_table::ResourceColumn;
    /// use ratatui::layout::{Alignment, Constraint};
    ///
    /// let col = ResourceColumn::new("X", Constraint::Length(10));
    /// assert_eq!(col.alignment(), Alignment::Left);
    /// ```
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }
}

// ---------------------------------------------------------------------------
// ResourceRow trait
// ---------------------------------------------------------------------------

/// A row in a `ResourceTable` with per-cell styling and an optional status.
///
/// Implement this trait on your domain type (Pod, Service, Deployment, etc.)
/// to convert it into displayable cells.
///
/// # Examples
///
/// ```
/// use envision::component::resource_table::{ResourceCell, ResourceRow, RowStatus};
///
/// #[derive(Clone)]
/// struct Service {
///     name: String,
///     healthy: bool,
/// }
///
/// impl ResourceRow for Service {
///     fn cells(&self) -> Vec<ResourceCell> {
///         vec![
///             ResourceCell::new(&self.name),
///             if self.healthy {
///                 ResourceCell::success("UP")
///             } else {
///                 ResourceCell::error("DOWN")
///             },
///         ]
///     }
///
///     fn status(&self) -> RowStatus {
///         if self.healthy { RowStatus::Healthy } else { RowStatus::Error }
///     }
/// }
/// ```
pub trait ResourceRow: Clone {
    /// Returns the styled cells for this row, in column order.
    fn cells(&self) -> Vec<ResourceCell>;

    /// Returns the row-level status indicator.
    ///
    /// Default implementation returns `RowStatus::None`. Override to enable
    /// the status column.
    fn status(&self) -> RowStatus {
        RowStatus::None
    }
}

// ---------------------------------------------------------------------------
// Message and Output
// ---------------------------------------------------------------------------

/// Messages for the `ResourceTable` component.
///
/// # Examples
///
/// ```
/// use envision::component::resource_table::ResourceTableMessage;
///
/// let msg: ResourceTableMessage<()> = ResourceTableMessage::Down;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceTableMessage<T: Clone> {
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Move selection to the first row.
    First,
    /// Move selection to the last row.
    Last,
    /// Move selection up by N rows.
    PageUp(usize),
    /// Move selection down by N rows.
    PageDown(usize),
    /// Confirm/activate the selected row.
    Select,
    /// Replace all rows.
    SetRows(Vec<T>),
}

/// Outputs emitted by the `ResourceTable` component.
///
/// # Examples
///
/// ```
/// use envision::component::resource_table::ResourceTableOutput;
///
/// let _: ResourceTableOutput<()> = ResourceTableOutput::SelectionChanged(0);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceTableOutput<T: Clone> {
    /// A row was selected (Enter pressed).
    Selected(T),
    /// The selection moved to a new index.
    SelectionChanged(usize),
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// The ResourceTable component.
///
/// Renders rows of structured data with per-cell styling, age formatting,
/// and row status indicators.
pub struct ResourceTable<T: ResourceRow>(std::marker::PhantomData<T>);

impl<T: ResourceRow + 'static> Component for ResourceTable<T> {
    type State = ResourceTableState<T>;
    type Message = ResourceTableMessage<T>;
    type Output = ResourceTableOutput<T>;

    fn init() -> Self::State {
        ResourceTableState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        if let Event::Key(key) = event {
            return match key.code {
                Key::Up | Key::Char('k') => Some(ResourceTableMessage::Up),
                Key::Down | Key::Char('j') => Some(ResourceTableMessage::Down),
                Key::Home | Key::Char('g') => Some(ResourceTableMessage::First),
                Key::End | Key::Char('G') => Some(ResourceTableMessage::Last),
                Key::PageUp => Some(ResourceTableMessage::PageUp(10)),
                Key::PageDown => Some(ResourceTableMessage::PageDown(10)),
                Key::Enter => Some(ResourceTableMessage::Select),
                _ => None,
            };
        }
        None
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ResourceTableMessage::Up => {
                if state.rows.is_empty() {
                    return None;
                }
                let new_idx = match state.selected {
                    None => 0,
                    Some(0) => 0,
                    Some(i) => i - 1,
                };
                if Some(new_idx) != state.selected {
                    state.selected = Some(new_idx);
                    state.scroll.ensure_visible(new_idx);
                    return Some(ResourceTableOutput::SelectionChanged(new_idx));
                }
                None
            }
            ResourceTableMessage::Down => {
                if state.rows.is_empty() {
                    return None;
                }
                let new_idx = match state.selected {
                    None => 0,
                    Some(i) if i + 1 < state.rows.len() => i + 1,
                    Some(i) => i,
                };
                if Some(new_idx) != state.selected {
                    state.selected = Some(new_idx);
                    state.scroll.ensure_visible(new_idx);
                    return Some(ResourceTableOutput::SelectionChanged(new_idx));
                }
                None
            }
            ResourceTableMessage::First => {
                if state.rows.is_empty() {
                    return None;
                }
                state.selected = Some(0);
                state.scroll.ensure_visible(0);
                Some(ResourceTableOutput::SelectionChanged(0))
            }
            ResourceTableMessage::Last => {
                if state.rows.is_empty() {
                    return None;
                }
                let last = state.rows.len() - 1;
                state.selected = Some(last);
                state.scroll.ensure_visible(last);
                Some(ResourceTableOutput::SelectionChanged(last))
            }
            ResourceTableMessage::PageUp(n) => {
                if state.rows.is_empty() {
                    return None;
                }
                let new_idx = state.selected.unwrap_or(0).saturating_sub(n);
                if Some(new_idx) != state.selected {
                    state.selected = Some(new_idx);
                    state.scroll.ensure_visible(new_idx);
                    return Some(ResourceTableOutput::SelectionChanged(new_idx));
                }
                None
            }
            ResourceTableMessage::PageDown(n) => {
                if state.rows.is_empty() {
                    return None;
                }
                let max = state.rows.len() - 1;
                let new_idx = state.selected.unwrap_or(0).saturating_add(n).min(max);
                if Some(new_idx) != state.selected {
                    state.selected = Some(new_idx);
                    state.scroll.ensure_visible(new_idx);
                    return Some(ResourceTableOutput::SelectionChanged(new_idx));
                }
                None
            }
            ResourceTableMessage::Select => state
                .selected_row()
                .cloned()
                .map(ResourceTableOutput::Selected),
            ResourceTableMessage::SetRows(rows) => {
                state.set_rows(rows);
                None
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        render::render(state, ctx);
    }
}
