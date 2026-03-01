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
}

/// Output messages from a SelectableList.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectableListOutput<T: Clone> {
    /// An item was selected (e.g., Enter pressed).
    Selected(T),
    /// The selection changed to a new index.
    SelectionChanged(usize),
}

/// State for a SelectableList component.
#[derive(Clone, Debug)]
pub struct SelectableListState<T: Clone> {
    items: Vec<T>,
    list_state: ListState,
    focused: bool,
}

impl<T: Clone> Default for SelectableListState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            list_state: ListState::default(),
            focused: false,
        }
    }
}

impl<T: Clone> SelectableListState<T> {
    /// Creates a new state with the given items.
    ///
    /// If the items list is non-empty, the first item is selected.
    pub fn new(items: Vec<T>) -> Self {
        Self::with_items(items)
    }

    /// Creates a new state with the given items.
    pub fn with_items(items: Vec<T>) -> Self {
        let mut state = Self {
            items,
            list_state: ListState::default(),
            focused: false,
        };
        if !state.items.is_empty() {
            state.list_state.select(Some(0));
        }
        state
    }

    /// Returns a reference to the items.
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Sets the items, resetting selection to the first item if any.
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        if self.items.is_empty() {
            self.list_state.select(None);
        } else {
            let current = self.list_state.selected().unwrap_or(0);
            let new_index = current.min(self.items.len().saturating_sub(1));
            self.list_state.select(Some(new_index));
        }
    }

    /// Returns the currently selected index.
    pub fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// Returns a reference to the currently selected item.
    pub fn selected_item(&self) -> Option<&T> {
        self.list_state.selected().and_then(|i| self.items.get(i))
    }

    /// Selects the item at the given index.
    pub fn select(&mut self, index: Option<usize>) {
        match index {
            Some(i) if i < self.items.len() => self.list_state.select(Some(i)),
            Some(_) => {} // Index out of bounds, ignore
            None => self.list_state.select(None),
        }
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items in the list.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns a mutable reference to the internal ListState for rendering.
    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }
}

impl<T: Clone + std::fmt::Display + 'static> SelectableListState<T> {
    /// Returns true if the selectable list is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
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
        if !state.focused {
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
        if state.items.is_empty() {
            return None;
        }

        let len = state.items.len();
        let current = state.list_state.selected().unwrap_or(0);

        match msg {
            SelectableListMessage::Up => {
                let new_index = current.saturating_sub(1);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    return Some(SelectableListOutput::SelectionChanged(new_index));
                }
            }
            SelectableListMessage::Down => {
                let new_index = (current + 1).min(len - 1);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    return Some(SelectableListOutput::SelectionChanged(new_index));
                }
            }
            SelectableListMessage::First => {
                if current != 0 {
                    state.list_state.select(Some(0));
                    return Some(SelectableListOutput::SelectionChanged(0));
                }
            }
            SelectableListMessage::Last => {
                let last = len - 1;
                if current != last {
                    state.list_state.select(Some(last));
                    return Some(SelectableListOutput::SelectionChanged(last));
                }
            }
            SelectableListMessage::PageUp(page_size) => {
                let new_index = current.saturating_sub(page_size);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    return Some(SelectableListOutput::SelectionChanged(new_index));
                }
            }
            SelectableListMessage::PageDown(page_size) => {
                let new_index = (current + page_size).min(len - 1);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    return Some(SelectableListOutput::SelectionChanged(new_index));
                }
            }
            SelectableListMessage::Select => {
                if let Some(item) = state.items.get(current).cloned() {
                    return Some(SelectableListOutput::Selected(item));
                }
            }
        }

        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Default view uses Display trait - users can implement custom rendering
        let items: Vec<ListItem> = state
            .items
            .iter()
            .map(|item| ListItem::new(format!("{}", item)))
            .collect();

        let highlight_style = theme.selected_highlight_style(state.focused);

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
