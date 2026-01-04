//! A list component with per-item loading and error states.
//!
//! `LoadingList` extends the basic list pattern with loading indicators and
//! error states for each item. Useful for lists where items can be fetched
//! or processed asynchronously.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{LoadingList, LoadingListState, LoadingListMessage, ItemState, Component};
//!
//! #[derive(Clone, Debug)]
//! struct Book {
//!     id: String,
//!     title: String,
//! }
//!
//! let books = vec![
//!     Book { id: "1".to_string(), title: "Book One".to_string() },
//!     Book { id: "2".to_string(), title: "Book Two".to_string() },
//! ];
//!
//! let mut state = LoadingListState::with_items(books, |b| b.title.clone());
//!
//! // Set first item as loading
//! LoadingList::update(&mut state, LoadingListMessage::SetLoading(0));
//!
//! // Later, mark as ready or error
//! LoadingList::update(&mut state, LoadingListMessage::SetReady(0));
//! // Or: LoadingList::update(&mut state, LoadingListMessage::SetError { index: 0, message: "Failed".to_string() });
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use super::{Component, Focusable};

/// Loading state of an individual item.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ItemState {
    /// Item is ready (normal state).
    #[default]
    Ready,
    /// Item is currently loading.
    Loading,
    /// Item has an error.
    Error(String),
}

impl ItemState {
    /// Returns true if the item is loading.
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns true if the item has an error.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Returns true if the item is ready.
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Returns the error message if in error state.
    pub fn error_message(&self) -> Option<&str> {
        if let Self::Error(msg) = self {
            Some(msg)
        } else {
            None
        }
    }

    /// Returns the symbol for this state.
    pub fn symbol(&self, spinner_frame: usize) -> &'static str {
        match self {
            Self::Ready => " ",
            Self::Loading => {
                const FRAMES: [&str; 4] = ["⠋", "⠙", "⠹", "⠸"];
                FRAMES[spinner_frame % FRAMES.len()]
            }
            Self::Error(_) => "✗",
        }
    }

    /// Returns the color for this state.
    pub fn color(&self) -> Color {
        match self {
            Self::Ready => Color::Reset,
            Self::Loading => Color::Yellow,
            Self::Error(_) => Color::Red,
        }
    }
}

/// A single item in the loading list.
#[derive(Clone, Debug)]
pub struct LoadingListItem<T: Clone> {
    /// The underlying data.
    data: T,
    /// Display label.
    label: String,
    /// Current loading state.
    state: ItemState,
}

impl<T: Clone> LoadingListItem<T> {
    /// Creates a new item.
    pub fn new(data: T, label: impl Into<String>) -> Self {
        Self {
            data,
            label: label.into(),
            state: ItemState::Ready,
        }
    }

    /// Returns the underlying data.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the data.
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Returns the label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Returns the current state.
    pub fn state(&self) -> &ItemState {
        &self.state
    }

    /// Sets the state.
    pub fn set_state(&mut self, state: ItemState) {
        self.state = state;
    }

    /// Returns true if the item is loading.
    pub fn is_loading(&self) -> bool {
        self.state.is_loading()
    }

    /// Returns true if the item has an error.
    pub fn is_error(&self) -> bool {
        self.state.is_error()
    }

    /// Returns true if the item is ready.
    pub fn is_ready(&self) -> bool {
        self.state.is_ready()
    }
}

/// Messages for the LoadingList component.
#[derive(Clone, Debug, PartialEq)]
pub enum LoadingListMessage<T: Clone> {
    /// Set all items.
    SetItems(Vec<T>),
    /// Set an item's state to loading.
    SetLoading(usize),
    /// Set an item's state to ready.
    SetReady(usize),
    /// Set an item's state to error.
    SetError {
        /// Item index.
        index: usize,
        /// Error message.
        message: String,
    },
    /// Clear an item's error (set to ready).
    ClearError(usize),
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Move to first item.
    First,
    /// Move to last item.
    Last,
    /// Select the current item.
    Select,
    /// Tick animation (advances spinner frame).
    Tick,
}

/// Output messages from LoadingList.
#[derive(Clone, Debug, PartialEq)]
pub enum LoadingListOutput<T: Clone> {
    /// An item was selected.
    Selected(T),
    /// Selection changed.
    SelectionChanged(usize),
    /// An item's state changed.
    ItemStateChanged {
        /// Item index.
        index: usize,
        /// New state.
        state: ItemState,
    },
}

/// State for the LoadingList component.
#[derive(Clone, Debug)]
pub struct LoadingListState<T: Clone> {
    /// All items.
    items: Vec<LoadingListItem<T>>,
    /// Currently selected index.
    selected: Option<usize>,
    /// Whether the component is focused.
    focused: bool,
    /// Current spinner animation frame.
    spinner_frame: usize,
    /// Label extractor function (stored as a result of initial extraction).
    /// Note: We can't store the function, so labels are extracted at construction time.
    /// Optional title.
    title: Option<String>,
    /// Whether to show loading indicators.
    show_indicators: bool,
}

impl<T: Clone> Default for LoadingListState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            focused: false,
            spinner_frame: 0,
            title: None,
            show_indicators: true,
        }
    }
}

impl<T: Clone> LoadingListState<T> {
    /// Creates a new empty LoadingList state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a state with items, using a label extractor function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// #[derive(Clone)]
    /// struct Item { name: String }
    ///
    /// let items = vec![
    ///     Item { name: "One".to_string() },
    ///     Item { name: "Two".to_string() },
    /// ];
    ///
    /// let state = LoadingListState::with_items(items, |i| i.name.clone());
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn with_items<F>(items: Vec<T>, label_fn: F) -> Self
    where
        F: Fn(&T) -> String,
    {
        let list_items = items
            .into_iter()
            .map(|data| {
                let label = label_fn(&data);
                LoadingListItem::new(data, label)
            })
            .collect();

        Self {
            items: list_items,
            selected: None,
            focused: false,
            spinner_frame: 0,
            title: None,
            show_indicators: true,
        }
    }

    /// Sets the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show loading indicators.
    pub fn with_indicators(mut self, show: bool) -> Self {
        self.show_indicators = show;
        self
    }

    /// Returns all items.
    pub fn items(&self) -> &[LoadingListItem<T>] {
        &self.items
    }

    /// Returns a mutable reference to all items.
    pub fn items_mut(&mut self) -> &mut Vec<LoadingListItem<T>> {
        &mut self.items
    }

    /// Returns the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if there are no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the selected index.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Returns the selected item.
    pub fn selected_item(&self) -> Option<&LoadingListItem<T>> {
        self.selected.and_then(|i| self.items.get(i))
    }

    /// Returns the selected item's data.
    pub fn selected_data(&self) -> Option<&T> {
        self.selected_item().map(|item| item.data())
    }

    /// Sets the selected index.
    pub fn set_selected(&mut self, index: Option<usize>) {
        self.selected = index.map(|i| i.min(self.items.len().saturating_sub(1)));
    }

    /// Returns an item by index.
    pub fn get(&self, index: usize) -> Option<&LoadingListItem<T>> {
        self.items.get(index)
    }

    /// Returns a mutable item by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut LoadingListItem<T>> {
        self.items.get_mut(index)
    }

    /// Sets the loading state for an item.
    pub fn set_loading(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.state = ItemState::Loading;
        }
    }

    /// Sets the ready state for an item.
    pub fn set_ready(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.state = ItemState::Ready;
        }
    }

    /// Sets the error state for an item.
    pub fn set_error(&mut self, index: usize, message: impl Into<String>) {
        if let Some(item) = self.items.get_mut(index) {
            item.state = ItemState::Error(message.into());
        }
    }

    /// Returns the number of items currently loading.
    pub fn loading_count(&self) -> usize {
        self.items.iter().filter(|i| i.is_loading()).count()
    }

    /// Returns the number of items with errors.
    pub fn error_count(&self) -> usize {
        self.items.iter().filter(|i| i.is_error()).count()
    }

    /// Returns true if any item is loading.
    pub fn has_loading(&self) -> bool {
        self.items.iter().any(|i| i.is_loading())
    }

    /// Returns true if any item has an error.
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|i| i.is_error())
    }

    /// Returns the title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns whether indicators are shown.
    pub fn show_indicators(&self) -> bool {
        self.show_indicators
    }

    /// Sets whether to show indicators.
    pub fn set_show_indicators(&mut self, show: bool) {
        self.show_indicators = show;
    }

    /// Returns the current spinner frame.
    pub fn spinner_frame(&self) -> usize {
        self.spinner_frame
    }

    /// Clears all items.
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = None;
    }
}

/// A list component with per-item loading and error states.
///
/// # Visual Format
///
/// ```text
/// ┌─Items───────────────────────────┐
/// │   Item 1                        │
/// │ ⠙ Item 2 (loading)              │
/// │ ▸ Item 3 (selected)             │
/// │ ✗ Item 4 - Error: Failed        │
/// └─────────────────────────────────┘
/// ```
pub struct LoadingList<T: Clone>(std::marker::PhantomData<T>);

impl<T: Clone> Component for LoadingList<T> {
    type State = LoadingListState<T>;
    type Message = LoadingListMessage<T>;
    type Output = LoadingListOutput<T>;

    fn init() -> Self::State {
        LoadingListState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            LoadingListMessage::SetItems(items) => {
                // Convert items without a label function - uses Debug if available
                // In practice, users should use LoadingListState::with_items
                state.items = items
                    .into_iter()
                    .enumerate()
                    .map(|(i, data)| LoadingListItem::new(data, format!("Item {}", i + 1)))
                    .collect();
                state.selected = None;
                None
            }

            LoadingListMessage::SetLoading(index) => {
                if let Some(item) = state.items.get_mut(index) {
                    item.state = ItemState::Loading;
                    Some(LoadingListOutput::ItemStateChanged {
                        index,
                        state: ItemState::Loading,
                    })
                } else {
                    None
                }
            }

            LoadingListMessage::SetReady(index) => {
                if let Some(item) = state.items.get_mut(index) {
                    item.state = ItemState::Ready;
                    Some(LoadingListOutput::ItemStateChanged {
                        index,
                        state: ItemState::Ready,
                    })
                } else {
                    None
                }
            }

            LoadingListMessage::SetError { index, message } => {
                if let Some(item) = state.items.get_mut(index) {
                    let new_state = ItemState::Error(message.clone());
                    item.state = new_state.clone();
                    Some(LoadingListOutput::ItemStateChanged {
                        index,
                        state: new_state,
                    })
                } else {
                    None
                }
            }

            LoadingListMessage::ClearError(index) => {
                if let Some(item) = state.items.get_mut(index) {
                    if item.is_error() {
                        item.state = ItemState::Ready;
                        return Some(LoadingListOutput::ItemStateChanged {
                            index,
                            state: ItemState::Ready,
                        });
                    }
                }
                None
            }

            LoadingListMessage::Up => {
                if state.items.is_empty() {
                    return None;
                }

                let new_index = match state.selected {
                    Some(i) if i > 0 => i - 1,
                    Some(_) => state.items.len() - 1, // Wrap to bottom
                    None => state.items.len() - 1,
                };

                state.selected = Some(new_index);
                Some(LoadingListOutput::SelectionChanged(new_index))
            }

            LoadingListMessage::Down => {
                if state.items.is_empty() {
                    return None;
                }

                let new_index = match state.selected {
                    Some(i) if i < state.items.len() - 1 => i + 1,
                    Some(_) => 0, // Wrap to top
                    None => 0,
                };

                state.selected = Some(new_index);
                Some(LoadingListOutput::SelectionChanged(new_index))
            }

            LoadingListMessage::First => {
                if state.items.is_empty() {
                    return None;
                }

                state.selected = Some(0);
                Some(LoadingListOutput::SelectionChanged(0))
            }

            LoadingListMessage::Last => {
                if state.items.is_empty() {
                    return None;
                }

                let last = state.items.len() - 1;
                state.selected = Some(last);
                Some(LoadingListOutput::SelectionChanged(last))
            }

            LoadingListMessage::Select => {
                if let Some(index) = state.selected {
                    if let Some(item) = state.items.get(index) {
                        return Some(LoadingListOutput::Selected(item.data.clone()));
                    }
                }
                None
            }

            LoadingListMessage::Tick => {
                state.spinner_frame = (state.spinner_frame + 1) % 4;
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let block = if let Some(title) = &state.title {
            Block::default().borders(Borders::ALL).title(title.as_str())
        } else {
            Block::default().borders(Borders::ALL)
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if state.items.is_empty() || inner.height == 0 {
            return;
        }

        let items: Vec<ListItem> = state
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_selected = state.selected == Some(idx);
                let select_marker = if is_selected { "▸" } else { " " };

                let content = if state.show_indicators {
                    let state_symbol = item.state.symbol(state.spinner_frame);

                    if let ItemState::Error(msg) = &item.state {
                        format!(
                            "{} {} {} - Error: {}",
                            select_marker, state_symbol, item.label, msg
                        )
                    } else {
                        format!("{} {} {}", select_marker, state_symbol, item.label)
                    }
                } else if let ItemState::Error(msg) = &item.state {
                    format!("{} {} - Error: {}", select_marker, item.label, msg)
                } else {
                    format!("{} {}", select_marker, item.label)
                };

                let style = if is_selected {
                    Style::default().fg(Color::Yellow).bold()
                } else {
                    Style::default().fg(item.state.color())
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }
}

impl<T: Clone> Focusable for LoadingList<T> {
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
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    #[derive(Clone, Debug, PartialEq)]
    struct TestItem {
        id: u32,
        name: String,
    }

    fn make_items() -> Vec<TestItem> {
        vec![
            TestItem {
                id: 1,
                name: "Item One".to_string(),
            },
            TestItem {
                id: 2,
                name: "Item Two".to_string(),
            },
            TestItem {
                id: 3,
                name: "Item Three".to_string(),
            },
        ]
    }

    // ========================================
    // ItemState Tests
    // ========================================

    #[test]
    fn test_item_state_default() {
        let state = ItemState::default();
        assert!(state.is_ready());
    }

    #[test]
    fn test_item_state_is_loading() {
        let state = ItemState::Loading;
        assert!(state.is_loading());
        assert!(!state.is_ready());
        assert!(!state.is_error());
    }

    #[test]
    fn test_item_state_is_error() {
        let state = ItemState::Error("Test error".to_string());
        assert!(state.is_error());
        assert!(!state.is_loading());
        assert!(!state.is_ready());
    }

    #[test]
    fn test_item_state_error_message() {
        let state = ItemState::Error("Test error".to_string());
        assert_eq!(state.error_message(), Some("Test error"));

        let ready = ItemState::Ready;
        assert!(ready.error_message().is_none());
    }

    #[test]
    fn test_item_state_symbols() {
        assert_eq!(ItemState::Ready.symbol(0), " ");
        assert_eq!(ItemState::Error("".to_string()).symbol(0), "✗");
        // Loading has animated symbols
        assert!(!ItemState::Loading.symbol(0).is_empty());
    }

    #[test]
    fn test_item_state_colors() {
        assert_eq!(ItemState::Ready.color(), Color::Reset);
        assert_eq!(ItemState::Loading.color(), Color::Yellow);
        assert_eq!(ItemState::Error("".to_string()).color(), Color::Red);
    }

    // ========================================
    // LoadingListItem Tests
    // ========================================

    #[test]
    fn test_list_item_new() {
        let item = LoadingListItem::new("data", "Label");
        assert_eq!(item.data(), &"data");
        assert_eq!(item.label(), "Label");
        assert!(item.is_ready());
    }

    #[test]
    fn test_list_item_set_label() {
        let mut item = LoadingListItem::new("data", "Old");
        item.set_label("New");
        assert_eq!(item.label(), "New");
    }

    #[test]
    fn test_list_item_set_state() {
        let mut item = LoadingListItem::new("data", "Label");
        item.set_state(ItemState::Loading);
        assert!(item.is_loading());

        item.set_state(ItemState::Error("err".to_string()));
        assert!(item.is_error());
    }

    #[test]
    fn test_list_item_data_mut() {
        let mut item = LoadingListItem::new("original", "Label");
        *item.data_mut() = "modified";
        assert_eq!(item.data(), &"modified");
    }

    // ========================================
    // LoadingListState Tests
    // ========================================

    #[test]
    fn test_state_new() {
        let state: LoadingListState<String> = LoadingListState::new();
        assert!(state.is_empty());
        assert!(state.selected().is_none());
        assert!(state.show_indicators());
    }

    #[test]
    fn test_state_with_items() {
        let items = make_items();
        let state = LoadingListState::with_items(items, |i| i.name.clone());

        assert_eq!(state.len(), 3);
        assert_eq!(state.items()[0].label(), "Item One");
        assert_eq!(state.items()[1].label(), "Item Two");
        assert_eq!(state.items()[2].label(), "Item Three");
    }

    #[test]
    fn test_state_with_title() {
        let state: LoadingListState<String> = LoadingListState::new().with_title("My List");
        assert_eq!(state.title(), Some("My List"));
    }

    #[test]
    fn test_state_with_indicators() {
        let state: LoadingListState<String> = LoadingListState::new().with_indicators(false);
        assert!(!state.show_indicators());
    }

    #[test]
    fn test_state_set_loading() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        state.set_loading(0);
        assert!(state.items()[0].is_loading());
    }

    #[test]
    fn test_state_set_ready() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        state.set_loading(0);
        state.set_ready(0);
        assert!(state.items()[0].is_ready());
    }

    #[test]
    fn test_state_set_error() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        state.set_error(0, "Failed to load");
        assert!(state.items()[0].is_error());
        assert_eq!(
            state.items()[0].state().error_message(),
            Some("Failed to load")
        );
    }

    #[test]
    fn test_state_counts() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        state.set_loading(0);
        state.set_loading(1);
        state.set_error(2, "Error");

        assert_eq!(state.loading_count(), 2);
        assert_eq!(state.error_count(), 1);
        assert!(state.has_loading());
        assert!(state.has_errors());
    }

    #[test]
    fn test_state_selected() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        state.set_selected(Some(1));
        assert_eq!(state.selected(), Some(1));
        assert_eq!(state.selected_item().unwrap().label(), "Item Two");
        assert_eq!(state.selected_data().unwrap().id, 2);
    }

    #[test]
    fn test_state_selected_clamped() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        state.set_selected(Some(100)); // Too high
        assert_eq!(state.selected(), Some(2)); // Clamped to last
    }

    #[test]
    fn test_state_get() {
        let items = make_items();
        let state = LoadingListState::with_items(items, |i| i.name.clone());

        assert!(state.get(0).is_some());
        assert!(state.get(100).is_none());
    }

    #[test]
    fn test_state_clear() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_selected(Some(0));

        state.clear();
        assert!(state.is_empty());
        assert!(state.selected().is_none());
    }

    // ========================================
    // Component Tests
    // ========================================

    #[test]
    fn test_init() {
        let state: LoadingListState<String> = LoadingList::init();
        assert!(state.is_empty());
    }

    #[test]
    fn test_update_set_loading() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        let output = LoadingList::update(&mut state, LoadingListMessage::SetLoading(0));

        assert!(matches!(
            output,
            Some(LoadingListOutput::ItemStateChanged {
                index: 0,
                state: ItemState::Loading
            })
        ));
        assert!(state.items()[0].is_loading());
    }

    #[test]
    fn test_update_set_ready() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_loading(0);

        let output = LoadingList::update(&mut state, LoadingListMessage::SetReady(0));

        assert!(matches!(
            output,
            Some(LoadingListOutput::ItemStateChanged {
                index: 0,
                state: ItemState::Ready
            })
        ));
    }

    #[test]
    fn test_update_set_error() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        let output = LoadingList::update(
            &mut state,
            LoadingListMessage::SetError {
                index: 0,
                message: "Failed".to_string(),
            },
        );

        assert!(matches!(
            output,
            Some(LoadingListOutput::ItemStateChanged {
                index: 0,
                state: ItemState::Error(_)
            })
        ));
    }

    #[test]
    fn test_update_clear_error() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_error(0, "Error");

        let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(0));

        assert!(matches!(
            output,
            Some(LoadingListOutput::ItemStateChanged {
                index: 0,
                state: ItemState::Ready
            })
        ));
        assert!(state.items()[0].is_ready());
    }

    #[test]
    fn test_update_clear_error_not_error() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(0));
        assert!(output.is_none()); // Was already ready
    }

    // ========================================
    // Navigation Tests
    // ========================================

    #[test]
    fn test_update_down() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        LoadingList::update(&mut state, LoadingListMessage::Down);
        assert_eq!(state.selected(), Some(0));

        LoadingList::update(&mut state, LoadingListMessage::Down);
        assert_eq!(state.selected(), Some(1));
    }

    #[test]
    fn test_update_down_wrap() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_selected(Some(2)); // Last item

        LoadingList::update(&mut state, LoadingListMessage::Down);
        assert_eq!(state.selected(), Some(0)); // Wraps
    }

    #[test]
    fn test_update_up() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_selected(Some(2));

        LoadingList::update(&mut state, LoadingListMessage::Up);
        assert_eq!(state.selected(), Some(1));
    }

    #[test]
    fn test_update_up_wrap() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_selected(Some(0));

        LoadingList::update(&mut state, LoadingListMessage::Up);
        assert_eq!(state.selected(), Some(2)); // Wraps
    }

    #[test]
    fn test_update_first() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_selected(Some(2));

        LoadingList::update(&mut state, LoadingListMessage::First);
        assert_eq!(state.selected(), Some(0));
    }

    #[test]
    fn test_update_last() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        LoadingList::update(&mut state, LoadingListMessage::Last);
        assert_eq!(state.selected(), Some(2));
    }

    #[test]
    fn test_update_select() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items.clone(), |i| i.name.clone());
        state.set_selected(Some(1));

        let output = LoadingList::update(&mut state, LoadingListMessage::Select);

        assert!(matches!(output, Some(LoadingListOutput::Selected(item)) if item.id == 2));
    }

    #[test]
    fn test_update_select_nothing() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        let output = LoadingList::update(&mut state, LoadingListMessage::Select);
        assert!(output.is_none()); // Nothing selected
    }

    #[test]
    fn test_update_tick() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        assert_eq!(state.spinner_frame(), 0);
        LoadingList::update(&mut state, LoadingListMessage::Tick);
        assert_eq!(state.spinner_frame(), 1);
    }

    #[test]
    fn test_navigation_empty_list() {
        let mut state: LoadingListState<TestItem> = LoadingListState::new();

        let output = LoadingList::update(&mut state, LoadingListMessage::Down);
        assert!(output.is_none());

        let output = LoadingList::update(&mut state, LoadingListMessage::Up);
        assert!(output.is_none());
    }

    // ========================================
    // Focusable Tests
    // ========================================

    #[test]
    fn test_focusable() {
        let mut state: LoadingListState<String> = LoadingListState::new();
        assert!(!LoadingList::is_focused(&state));

        LoadingList::focus(&mut state);
        assert!(LoadingList::is_focused(&state));

        LoadingList::blur(&mut state);
        assert!(!LoadingList::is_focused(&state));
    }

    // ========================================
    // View Tests
    // ========================================

    #[test]
    fn test_view_empty() {
        let state: LoadingListState<String> = LoadingListState::new();
        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| LoadingList::view(&state, frame, frame.area()))
            .unwrap();

        // Should render border only
        let output = terminal.backend().to_string();
        assert!(output.contains("─") || output.contains("│"));
    }

    #[test]
    fn test_view_with_items() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_selected(Some(1));

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| LoadingList::view(&state, frame, frame.area()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Item One"));
        assert!(output.contains("Item Two"));
        assert!(output.contains("Item Three"));
        assert!(output.contains("▸")); // Selection marker
    }

    #[test]
    fn test_view_with_title() {
        let items = make_items();
        let state = LoadingListState::with_items(items, |i| i.name.clone()).with_title("My Items");

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| LoadingList::view(&state, frame, frame.area()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("My Items"));
    }

    #[test]
    fn test_view_with_error() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_error(0, "Connection failed");

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| LoadingList::view(&state, frame, frame.area()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Error"));
        assert!(output.contains("Connection failed"));
    }

    #[test]
    fn test_clone() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        state.set_selected(Some(1));

        let cloned = state.clone();
        assert_eq!(cloned.len(), 3);
        assert_eq!(cloned.selected(), Some(1));
    }

    // ========================================
    // Additional Coverage Tests
    // ========================================

    #[test]
    fn test_view_zero_size_area() {
        let items = make_items();
        let state = LoadingListState::with_items(items, |i| i.name.clone());
        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        // Test with zero width
        terminal
            .draw(|frame| {
                LoadingList::view(&state, frame, Rect::new(0, 0, 0, 10));
            })
            .unwrap();

        // Test with zero height
        terminal
            .draw(|frame| {
                LoadingList::view(&state, frame, Rect::new(0, 0, 60, 0));
            })
            .unwrap();
    }

    #[test]
    fn test_view_without_indicators() {
        let items = make_items();
        let mut state =
            LoadingListState::with_items(items, |i| i.name.clone()).with_indicators(false);
        state.set_selected(Some(0));

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| LoadingList::view(&state, frame, frame.area()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Item One"));
    }

    #[test]
    fn test_view_without_indicators_with_error() {
        let items = make_items();
        let mut state =
            LoadingListState::with_items(items, |i| i.name.clone()).with_indicators(false);
        state.set_error(0, "Failed");

        let backend = CaptureBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| LoadingList::view(&state, frame, frame.area()))
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Error"));
        assert!(output.contains("Failed"));
    }

    #[test]
    fn test_update_set_items() {
        let mut state: LoadingListState<TestItem> = LoadingListState::new();
        state.set_selected(Some(0));

        let items = make_items();
        LoadingList::update(&mut state, LoadingListMessage::SetItems(items));

        assert_eq!(state.len(), 3);
        assert!(state.selected().is_none()); // Selection cleared
        assert_eq!(state.items()[0].label(), "Item 1"); // Uses default labeling
    }

    #[test]
    fn test_update_invalid_index() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        // SetLoading with invalid index
        let output = LoadingList::update(&mut state, LoadingListMessage::SetLoading(100));
        assert!(output.is_none());

        // SetReady with invalid index
        let output = LoadingList::update(&mut state, LoadingListMessage::SetReady(100));
        assert!(output.is_none());

        // SetError with invalid index
        let output = LoadingList::update(
            &mut state,
            LoadingListMessage::SetError {
                index: 100,
                message: "Error".to_string(),
            },
        );
        assert!(output.is_none());

        // ClearError with invalid index
        let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(100));
        assert!(output.is_none());
    }

    #[test]
    fn test_up_no_selection() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        // No selection set

        let output = LoadingList::update(&mut state, LoadingListMessage::Up);
        assert_eq!(state.selected(), Some(2)); // Goes to last item
        assert!(matches!(
            output,
            Some(LoadingListOutput::SelectionChanged(2))
        ));
    }

    #[test]
    fn test_down_no_selection() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());
        // No selection set

        let output = LoadingList::update(&mut state, LoadingListMessage::Down);
        assert_eq!(state.selected(), Some(0)); // Goes to first item
        assert!(matches!(
            output,
            Some(LoadingListOutput::SelectionChanged(0))
        ));
    }

    #[test]
    fn test_first_empty_list() {
        let mut state: LoadingListState<TestItem> = LoadingListState::new();

        let output = LoadingList::update(&mut state, LoadingListMessage::First);
        assert!(output.is_none());
    }

    #[test]
    fn test_last_empty_list() {
        let mut state: LoadingListState<TestItem> = LoadingListState::new();

        let output = LoadingList::update(&mut state, LoadingListMessage::Last);
        assert!(output.is_none());
    }

    #[test]
    fn test_get_mut() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        if let Some(item) = state.get_mut(0) {
            item.set_label("Modified");
        }
        assert_eq!(state.items()[0].label(), "Modified");

        assert!(state.get_mut(100).is_none());
    }

    #[test]
    fn test_items_mut() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        state.items_mut().push(LoadingListItem::new(
            TestItem {
                id: 4,
                name: "Item Four".to_string(),
            },
            "Item Four",
        ));

        assert_eq!(state.len(), 4);
    }

    #[test]
    fn test_set_title() {
        let mut state: LoadingListState<String> = LoadingListState::new();
        assert!(state.title().is_none());

        state.set_title(Some("New Title".to_string()));
        assert_eq!(state.title(), Some("New Title"));

        state.set_title(None);
        assert!(state.title().is_none());
    }

    #[test]
    fn test_set_show_indicators() {
        let mut state: LoadingListState<String> = LoadingListState::new();
        assert!(state.show_indicators());

        state.set_show_indicators(false);
        assert!(!state.show_indicators());
    }

    #[test]
    fn test_set_loading_invalid_index() {
        let items = make_items();
        let mut state = LoadingListState::with_items(items, |i| i.name.clone());

        // This should not panic
        state.set_loading(100);
        state.set_ready(100);
        state.set_error(100, "Error");
    }

    #[test]
    fn test_spinner_animation_frames() {
        let state = ItemState::Loading;
        // Test all 4 spinner frames
        let frame0 = state.symbol(0);
        let frame1 = state.symbol(1);
        let frame2 = state.symbol(2);
        let frame3 = state.symbol(3);
        let frame4 = state.symbol(4); // Should wrap to frame 0

        assert_eq!(frame0, frame4);
        assert_ne!(frame0, frame1);
        assert_ne!(frame1, frame2);
        assert_ne!(frame2, frame3);
    }
}
