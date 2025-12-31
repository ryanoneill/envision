//! A generic selectable list component with keyboard navigation.
//!
//! `SelectableList` provides a scrollable list of items with selection
//! tracking and keyboard navigation (vim-style and arrow keys).
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Focusable, ListMessage, SelectableList, SelectableListState};
//!
//! // Create a list of items
//! let mut state = SelectableList::<String>::init();
//! state.set_items(vec!["Item 1".into(), "Item 2".into(), "Item 3".into()]);
//!
//! // Navigate down
//! SelectableList::<String>::update(&mut state, ListMessage::Down);
//! assert_eq!(state.selected_index(), Some(1));
//!
//! // Get selected item
//! assert_eq!(state.selected_item(), Some(&"Item 2".into()));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use super::{Component, Focusable};

/// Messages that can be sent to a SelectableList.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListMessage {
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
pub enum ListOutput<T: Clone> {
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
    type Message = ListMessage;
    type Output = ListOutput<T>;

    fn init() -> Self::State {
        SelectableListState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.items.is_empty() {
            return None;
        }

        let len = state.items.len();
        let current = state.list_state.selected().unwrap_or(0);

        match msg {
            ListMessage::Up => {
                let new_index = current.saturating_sub(1);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    return Some(ListOutput::SelectionChanged(new_index));
                }
            }
            ListMessage::Down => {
                let new_index = (current + 1).min(len - 1);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    return Some(ListOutput::SelectionChanged(new_index));
                }
            }
            ListMessage::First => {
                if current != 0 {
                    state.list_state.select(Some(0));
                    return Some(ListOutput::SelectionChanged(0));
                }
            }
            ListMessage::Last => {
                let last = len - 1;
                if current != last {
                    state.list_state.select(Some(last));
                    return Some(ListOutput::SelectionChanged(last));
                }
            }
            ListMessage::PageUp(page_size) => {
                let new_index = current.saturating_sub(page_size);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    return Some(ListOutput::SelectionChanged(new_index));
                }
            }
            ListMessage::PageDown(page_size) => {
                let new_index = (current + page_size).min(len - 1);
                if new_index != current {
                    state.list_state.select(Some(new_index));
                    return Some(ListOutput::SelectionChanged(new_index));
                }
            }
            ListMessage::Select => {
                if let Some(item) = state.items.get(current).cloned() {
                    return Some(ListOutput::Selected(item));
                }
            }
        }

        None
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        // Default view uses Display trait - users can implement custom rendering
        let items: Vec<ListItem> = state
            .items
            .iter()
            .map(|item| ListItem::new(format!("{}", item)))
            .collect();

        let highlight_style = if state.focused {
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
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
mod tests {
    use super::*;

    #[test]
    fn test_init_empty() {
        let state = SelectableList::<String>::init();
        assert!(state.is_empty());
        assert_eq!(state.selected_index(), None);
        assert_eq!(state.selected_item(), None);
    }

    #[test]
    fn test_with_items() {
        let state = SelectableListState::with_items(vec!["a", "b", "c"]);
        assert_eq!(state.len(), 3);
        assert_eq!(state.selected_index(), Some(0));
        assert_eq!(state.selected_item(), Some(&"a"));
    }

    #[test]
    fn test_set_items() {
        let mut state = SelectableList::<String>::init();
        state.set_items(vec!["x".into(), "y".into()]);
        assert_eq!(state.len(), 2);
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_set_items_preserves_selection() {
        let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
        state.select(Some(1));
        state.set_items(vec!["x", "y", "z", "w"]);
        // Selection should be preserved at index 1
        assert_eq!(state.selected_index(), Some(1));
    }

    #[test]
    fn test_set_items_clamps_selection() {
        let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
        state.select(Some(2));
        state.set_items(vec!["x"]); // Only one item now
                                    // Selection should be clamped to last valid index
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_navigate_down() {
        let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);

        let output = SelectableList::<&str>::update(&mut state, ListMessage::Down);
        assert_eq!(state.selected_index(), Some(1));
        assert_eq!(output, Some(ListOutput::SelectionChanged(1)));

        let output = SelectableList::<&str>::update(&mut state, ListMessage::Down);
        assert_eq!(state.selected_index(), Some(2));
        assert_eq!(output, Some(ListOutput::SelectionChanged(2)));

        // At the end, should stay at last item
        let output = SelectableList::<&str>::update(&mut state, ListMessage::Down);
        assert_eq!(state.selected_index(), Some(2));
        assert_eq!(output, None);
    }

    #[test]
    fn test_navigate_up() {
        let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
        state.select(Some(2));

        let output = SelectableList::<&str>::update(&mut state, ListMessage::Up);
        assert_eq!(state.selected_index(), Some(1));
        assert_eq!(output, Some(ListOutput::SelectionChanged(1)));

        let output = SelectableList::<&str>::update(&mut state, ListMessage::Up);
        assert_eq!(state.selected_index(), Some(0));
        assert_eq!(output, Some(ListOutput::SelectionChanged(0)));

        // At the beginning, should stay at first item
        let output = SelectableList::<&str>::update(&mut state, ListMessage::Up);
        assert_eq!(state.selected_index(), Some(0));
        assert_eq!(output, None);
    }

    #[test]
    fn test_navigate_first_last() {
        let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e"]);
        state.select(Some(2));

        let output = SelectableList::<&str>::update(&mut state, ListMessage::Last);
        assert_eq!(state.selected_index(), Some(4));
        assert_eq!(output, Some(ListOutput::SelectionChanged(4)));

        let output = SelectableList::<&str>::update(&mut state, ListMessage::First);
        assert_eq!(state.selected_index(), Some(0));
        assert_eq!(output, Some(ListOutput::SelectionChanged(0)));
    }

    #[test]
    fn test_page_navigation() {
        let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e", "f", "g"]);

        let output = SelectableList::<&str>::update(&mut state, ListMessage::PageDown(3));
        assert_eq!(state.selected_index(), Some(3));
        assert_eq!(output, Some(ListOutput::SelectionChanged(3)));

        let output = SelectableList::<&str>::update(&mut state, ListMessage::PageDown(10));
        assert_eq!(state.selected_index(), Some(6)); // Clamped to last
        assert_eq!(output, Some(ListOutput::SelectionChanged(6)));

        let output = SelectableList::<&str>::update(&mut state, ListMessage::PageUp(4));
        assert_eq!(state.selected_index(), Some(2));
        assert_eq!(output, Some(ListOutput::SelectionChanged(2)));
    }

    #[test]
    fn test_select() {
        let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
        state.select(Some(1));

        let output = SelectableList::<&str>::update(&mut state, ListMessage::Select);
        assert_eq!(output, Some(ListOutput::Selected("b")));
    }

    #[test]
    fn test_empty_list_navigation() {
        let mut state = SelectableList::<String>::init();

        assert_eq!(
            SelectableList::<String>::update(&mut state, ListMessage::Down),
            None
        );
        assert_eq!(
            SelectableList::<String>::update(&mut state, ListMessage::Up),
            None
        );
        assert_eq!(
            SelectableList::<String>::update(&mut state, ListMessage::Select),
            None
        );
    }

    #[test]
    fn test_focusable() {
        let mut state = SelectableList::<String>::init();

        assert!(!SelectableList::<String>::is_focused(&state));

        SelectableList::<String>::set_focused(&mut state, true);
        assert!(SelectableList::<String>::is_focused(&state));

        SelectableList::<String>::blur(&mut state);
        assert!(!SelectableList::<String>::is_focused(&state));
    }

    #[test]
    fn test_select_method() {
        let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);

        state.select(Some(2));
        assert_eq!(state.selected_index(), Some(2));

        state.select(None);
        assert_eq!(state.selected_index(), None);

        // Out of bounds should be ignored
        state.select(Some(0));
        state.select(Some(100));
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_items_accessor() {
        let state = SelectableListState::with_items(vec![1, 2, 3]);
        assert_eq!(state.items(), &[1, 2, 3]);
    }

    #[test]
    fn test_view() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = SelectableListState::with_items(vec!["Item 1", "Item 2", "Item 3"]);
        state.focused = true;

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                SelectableList::<&str>::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
        assert!(output.contains("Item 3"));
    }
}
