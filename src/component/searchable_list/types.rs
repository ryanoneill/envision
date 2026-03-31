//! Types for the SearchableList component.
//!
//! Contains [`SearchableListMessage`], [`SearchableListOutput`], and
//! the internal [`Focus`] enum.

/// Messages that can be sent to a SearchableList.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchableListMessage {
    /// The filter text changed.
    FilterChanged(String),
    /// A character was typed (forwarded to the filter field).
    FilterChar(char),
    /// Delete the character before the cursor in the filter.
    FilterBackspace,
    /// Clear the filter text.
    FilterClear,
    /// Move selection up in the list.
    Up,
    /// Move selection down in the list.
    Down,
    /// Move selection to the first item.
    First,
    /// Move selection to the last item.
    Last,
    /// Move selection up by a page.
    PageUp(usize),
    /// Move selection down by a page.
    PageDown(usize),
    /// Select the current item (triggers output).
    Select,
    /// Switch focus between filter and list.
    ToggleFocus,
}

/// Output messages from a SearchableList.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchableListOutput<T: Clone> {
    /// An item was selected (e.g., Enter pressed while list is focused).
    Selected(T),
    /// The selection changed to a new filtered index.
    SelectionChanged(usize),
    /// The filter text changed.
    FilterChanged(String),
}

/// Identifies which sub-component has focus within the SearchableList.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub(in crate::component) enum Focus {
    /// The filter input field has focus.
    Filter,
    /// The selectable list has focus.
    List,
}
