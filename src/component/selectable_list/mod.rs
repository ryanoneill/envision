//! A generic selectable list component with keyboard navigation.
//!
//! `SelectableList` provides a scrollable list of items with selection
//! tracking and keyboard navigation (vim-style and arrow keys).
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Focusable, SelectableListMessage, SelectableList, SelectableListState};
//!
//! // Create a list of items
//! let mut state = SelectableList::<String>::init();
//! state.set_items(vec!["Item 1".into(), "Item 2".into(), "Item 3".into()]);
//!
//! // Navigate down
//! SelectableList::<String>::update(&mut state, SelectableListMessage::Down);
//! assert_eq!(state.selected_index(), Some(1));
//!
//! // Get selected item
//! assert_eq!(state.selected_item(), Some(&"Item 2".into()));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use super::{Component, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Messages that can be sent to a SelectableList.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectableListMessage {
    /// Move selection up by one.
    Up,
    /// Move selection down by one.
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
    /// Set the filter text for searching items.
    SetFilter(String),
    /// Clear the filter text.
    ClearFilter,
}

/// Output messages from a SelectableList.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectableListOutput<T: Clone> {
    /// An item was selected (e.g., Enter pressed).
    Selected(T),
    /// The selection changed to a new index (original item index).
    SelectionChanged(usize),
    /// The filter text changed.
    FilterChanged(String),
}

/// State for a SelectableList component.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SelectableListState<T: Clone> {
    items: Vec<T>,
    #[cfg_attr(feature = "serialization", serde(skip))]
    list_state: ListState,
    focused: bool,
    disabled: bool,
    filter_text: String,
    filtered_indices: Vec<usize>,
}

impl<T: Clone + PartialEq> PartialEq for SelectableListState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
            && self.list_state.selected() == other.list_state.selected()
            && self.focused == other.focused
            && self.disabled == other.disabled
            && self.filter_text == other.filter_text
    }
}

impl<T: Clone> Default for SelectableListState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            list_state: ListState::default(),
            focused: false,
            disabled: false,
            filter_text: String::new(),
            filtered_indices: Vec::new(),
        }
    }
}

impl<T: Clone> SelectableListState<T> {
    /// Creates a new state with the given items.
    ///
    /// If the items list is non-empty, the first item is selected.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = SelectableListState::new(vec!["apple", "banana", "cherry"]);
    /// assert_eq!(state.selected_index(), Some(0));
    /// assert_eq!(state.len(), 3);
    /// ```
    pub fn new(items: Vec<T>) -> Self {
        Self::with_items(items)
    }

    /// Creates a new state with the given items.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = SelectableListState::with_items(vec![1, 2, 3]);
    /// assert_eq!(state.selected_item(), Some(&1));
    /// ```
    pub fn with_items(items: Vec<T>) -> Self {
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
        let mut state = Self {
            items,
            list_state: ListState::default(),
            focused: false,
            disabled: false,
            filter_text: String::new(),
            filtered_indices,
        };
        if !state.items.is_empty() {
            state.list_state.select(Some(0));
        }
        state
    }

    /// Sets the initially selected index (builder method).
    ///
    /// The index is clamped to the valid range. Has no effect on empty lists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SelectableListState;
    ///
    /// let state = SelectableListState::new(vec!["A", "B", "C"]).with_selected(1);
    /// assert_eq!(state.selected_index(), Some(1));
    /// assert_eq!(state.selected_item(), Some(&"B"));
    /// ```
    pub fn with_selected(mut self, index: usize) -> Self {
        if self.items.is_empty() {
            return self;
        }
        let clamped = index.min(self.items.len() - 1);
        if let Some(filtered_pos) = self.filtered_indices.iter().position(|&fi| fi == clamped) {
            self.list_state.select(Some(filtered_pos));
        }
        self
    }

    /// Returns a reference to the items.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = SelectableListState::new(vec!["a", "b", "c"]);
    /// assert_eq!(state.items(), &["a", "b", "c"]);
    /// ```
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Sets the items, clearing any active filter and resetting selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut state = SelectableListState::new(vec!["old"]);
    /// state.set_items(vec!["new1", "new2"]);
    /// assert_eq!(state.items(), &["new1", "new2"]);
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.filter_text.clear();
        self.filtered_indices = (0..self.items.len()).collect();
        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
        } else {
            let current = self.list_state.selected().unwrap_or(0);
            let new_index = current.min(self.filtered_indices.len().saturating_sub(1));
            self.list_state.select(Some(new_index));
        }
    }

    /// Returns the currently selected index in the original items list.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = SelectableListState::new(vec!["a", "b", "c"]);
    /// assert_eq!(state.selected_index(), Some(0));
    ///
    /// let empty: SelectableListState<String> = SelectableListState::new(vec![]);
    /// assert_eq!(empty.selected_index(), None);
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.list_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i).copied())
    }

    /// Returns a reference to the currently selected item.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = SelectableListState::new(vec!["a", "b", "c"]);
    /// assert_eq!(state.selected_item(), Some(&"a"));
    /// ```
    pub fn selected_item(&self) -> Option<&T> {
        self.selected_index().and_then(|i| self.items.get(i))
    }

    /// Selects the item at the given index in the original items list.
    ///
    /// If the item is filtered out, the selection is unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut state = SelectableListState::new(vec!["a", "b", "c"]);
    /// state.select(Some(2));
    /// assert_eq!(state.selected_index(), Some(2));
    /// assert_eq!(state.selected_item(), Some(&"c"));
    /// ```
    pub fn select(&mut self, index: Option<usize>) {
        match index {
            Some(i) if i < self.items.len() => {
                if let Some(filtered_pos) = self.filtered_indices.iter().position(|&fi| fi == i) {
                    self.list_state.select(Some(filtered_pos));
                }
            }
            Some(_) => {} // Index out of bounds, ignore
            None => self.list_state.select(None),
        }
    }

    /// Sets the selected index.
    ///
    /// The index is clamped to the valid range. Has no effect on empty lists.
    /// If the item at the given index is filtered out, the selection is unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut state = SelectableListState::new(vec!["a", "b", "c"]);
    /// state.set_selected(2);
    /// assert_eq!(state.selected_index(), Some(2));
    /// assert_eq!(state.selected_item(), Some(&"c"));
    /// ```
    pub fn set_selected(&mut self, index: usize) {
        if self.items.is_empty() {
            return;
        }
        let clamped = index.min(self.items.len() - 1);
        if let Some(filtered_pos) = self.filtered_indices.iter().position(|&fi| fi == clamped) {
            self.list_state.select(Some(filtered_pos));
        }
    }

    /// Returns true if the list is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let empty: SelectableListState<i32> = SelectableListState::new(vec![]);
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = SelectableListState::new(vec![1]);
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items in the list.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = SelectableListState::new(vec!["a", "b", "c"]);
    /// assert_eq!(state.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns the current filter text.
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Returns the number of items visible after filtering.
    pub fn visible_count(&self) -> usize {
        self.filtered_indices.len()
    }
}

impl<T: Clone + std::fmt::Display + 'static> SelectableListState<T> {
    /// Returns true if the selectable list is focused.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = SelectableListState::new(vec!["a", "b"]);
    /// assert!(!state.is_focused());
    /// ```
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut state = SelectableListState::new(vec!["a", "b"]);
    /// state.set_focused(true);
    /// assert!(state.is_focused());
    /// ```
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the selectable list is disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = SelectableListState::new(vec!["a"]);
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut state = SelectableListState::new(vec!["a"]);
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the disabled state using builder pattern.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the filter text for case-insensitive substring matching.
    ///
    /// Items whose `Display` output contains the filter text (case-insensitive)
    /// are shown. Selection is preserved if the selected item remains visible,
    /// otherwise it moves to the first visible item.
    pub fn set_filter_text(&mut self, text: &str) {
        self.filter_text = text.to_string();
        self.apply_filter();
    }

    /// Clears the filter, showing all items.
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.apply_filter();
    }

    /// Recomputes filtered_indices based on the current filter_text.
    fn apply_filter(&mut self) {
        let previously_selected = self.selected_index();

        if self.filter_text.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_indices = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| format!("{}", item).to_lowercase().contains(&filter_lower))
                .map(|(i, _)| i)
                .collect();
        }

        // Try to preserve the previously selected item
        if let Some(prev_idx) = previously_selected {
            if let Some(new_pos) = self.filtered_indices.iter().position(|&i| i == prev_idx) {
                self.list_state.select(Some(new_pos));
                return;
            }
        }

        // Otherwise, select first visible item or none
        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    /// Maps an input event to a selectable list message.
    pub fn handle_event(&self, event: &Event) -> Option<SelectableListMessage> {
        SelectableList::<T>::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<SelectableListOutput<T>> {
        SelectableList::<T>::dispatch_event(self, event)
    }

    /// Updates the selectable list state with a message, returning any output.
    pub fn update(&mut self, msg: SelectableListMessage) -> Option<SelectableListOutput<T>> {
        SelectableList::<T>::update(self, msg)
    }
}

/// A generic selectable list component.
///
/// This component provides a scrollable list with keyboard navigation.
/// It's generic over the item type `T`, which must be `Clone`.
///
/// # Navigation
///
/// - `Up` / `Down` - Move selection by one
/// - `First` / `Last` - Jump to beginning/end
/// - `PageUp` / `PageDown` - Move by page size
/// - `Select` - Emit the selected item
pub struct SelectableList<T: Clone>(std::marker::PhantomData<T>);

impl<T: Clone + std::fmt::Display + 'static> Component for SelectableList<T> {
    type State = SelectableListState<T>;
    type Message = SelectableListMessage;
    type Output = SelectableListOutput<T>;

    fn init() -> Self::State {
        SelectableListState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(SelectableListMessage::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(SelectableListMessage::Down),
                KeyCode::Home | KeyCode::Char('g') => Some(SelectableListMessage::First),
                KeyCode::End | KeyCode::Char('G') => Some(SelectableListMessage::Last),
                KeyCode::Enter => Some(SelectableListMessage::Select),
                KeyCode::PageUp => Some(SelectableListMessage::PageUp(10)),
                KeyCode::PageDown => Some(SelectableListMessage::PageDown(10)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            SelectableListMessage::SetFilter(text) => {
                state.set_filter_text(&text);
                return Some(SelectableListOutput::FilterChanged(text));
            }
            SelectableListMessage::ClearFilter => {
                state.clear_filter();
                return Some(SelectableListOutput::FilterChanged(String::new()));
            }
            _ => {}
        }

        if state.disabled || state.filtered_indices.is_empty() {
            return None;
        }

        let len = state.filtered_indices.len();
        let current = state.list_state.selected().unwrap_or(0);

        match msg {
            SelectableListMessage::Up => {
                let new_index = current.saturating_sub(1);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    let orig = state.filtered_indices[new_index];
                    return Some(SelectableListOutput::SelectionChanged(orig));
                }
            }
            SelectableListMessage::Down => {
                let new_index = (current + 1).min(len - 1);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    let orig = state.filtered_indices[new_index];
                    return Some(SelectableListOutput::SelectionChanged(orig));
                }
            }
            SelectableListMessage::First => {
                if current != 0 {
                    state.list_state.select(Some(0));
                    let orig = state.filtered_indices[0];
                    return Some(SelectableListOutput::SelectionChanged(orig));
                }
            }
            SelectableListMessage::Last => {
                let last = len - 1;
                if current != last {
                    state.list_state.select(Some(last));
                    let orig = state.filtered_indices[last];
                    return Some(SelectableListOutput::SelectionChanged(orig));
                }
            }
            SelectableListMessage::PageUp(page_size) => {
                let new_index = current.saturating_sub(page_size);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    let orig = state.filtered_indices[new_index];
                    return Some(SelectableListOutput::SelectionChanged(orig));
                }
            }
            SelectableListMessage::PageDown(page_size) => {
                let new_index = (current + page_size).min(len - 1);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    let orig = state.filtered_indices[new_index];
                    return Some(SelectableListOutput::SelectionChanged(orig));
                }
            }
            SelectableListMessage::Select => {
                let orig = state.filtered_indices[current];
                if let Some(item) = state.items.get(orig).cloned() {
                    return Some(SelectableListOutput::Selected(item));
                }
            }
            SelectableListMessage::SetFilter(_) | SelectableListMessage::ClearFilter => {
                unreachable!("handled above")
            }
        }

        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        crate::annotation::with_registry(|reg| {
            let mut ann = crate::annotation::Annotation::list("selectable_list")
                .with_focus(state.focused)
                .with_disabled(state.disabled);
            if let Some(idx) = state.selected_index() {
                ann = ann.with_selected(true).with_value(idx.to_string());
            }
            reg.register(area, ann);
        });

        let items: Vec<ListItem> = state
            .filtered_indices
            .iter()
            .map(|&idx| ListItem::new(format!("{}", state.items[idx])))
            .collect();

        let highlight_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.selected_highlight_style(state.focused)
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        // We need to clone the state for rendering since StatefulWidget needs &mut
        let mut list_state = state.list_state.clone();
        frame.render_stateful_widget(list, area, &mut list_state);
    }
}

impl<T: Clone + std::fmt::Display + 'static> Focusable for SelectableList<T> {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
