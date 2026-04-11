//! A list component with per-item loading and error states.
//!
//! [`LoadingList<T>`] extends the basic list pattern with loading indicators and
//! error states for each item. Useful for lists where items can be fetched
//! or processed asynchronously. State is stored in [`LoadingListState<T>`],
//! updated via [`LoadingListMessage`], and produces [`LoadingListOutput`].
//! Items are wrapped in [`LoadingItem<T>`].
//!
//!
//! See also [`SelectableList`](super::SelectableList) for a simpler list.
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

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode};
use crate::scroll::ScrollState;
use crate::theme::Theme;

mod render;

/// Loading state of an individual item.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ItemState;
    ///
    /// assert!(!ItemState::Ready.is_loading());
    /// assert!(ItemState::Loading.is_loading());
    /// ```
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns true if the item has an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ItemState;
    ///
    /// assert!(!ItemState::Ready.is_error());
    /// assert!(ItemState::Error("failed".into()).is_error());
    /// ```
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Returns true if the item is ready.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ItemState;
    ///
    /// assert!(ItemState::Ready.is_ready());
    /// assert!(!ItemState::Loading.is_ready());
    /// ```
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Returns the error message if in error state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::ItemState;
    ///
    /// let state = ItemState::Error("connection lost".into());
    /// assert_eq!(state.error_message(), Some("connection lost"));
    ///
    /// assert_eq!(ItemState::Ready.error_message(), None);
    /// ```
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
                // Braille dots animation matching SpinnerStyle::Dots
                const LOADING_FRAMES: [&str; 10] =
                    ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                LOADING_FRAMES[spinner_frame % LOADING_FRAMES.len()]
            }
            Self::Error(_) => "✗",
        }
    }

    /// Returns the style for this state using the theme.
    pub fn style(&self, theme: &Theme) -> Style {
        match self {
            Self::Ready => theme.normal_style(),
            Self::Loading => theme.warning_style(),
            Self::Error(_) => theme.error_style(),
        }
    }
}

/// A single item in the loading list.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct LoadingListItem<T: Clone> {
    /// The underlying data.
    data: T,
    /// Display label.
    label: String,
    /// Current loading state.
    state: ItemState,
}

impl<T: Clone + PartialEq> PartialEq for LoadingListItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data && self.label == other.label && self.state == other.state
    }
}

impl<T: Clone> LoadingListItem<T> {
    /// Creates a new item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListItem;
    ///
    /// let item = LoadingListItem::new("data", "Label");
    /// assert_eq!(item.label(), "Label");
    /// assert!(item.is_ready());
    /// ```
    pub fn new(data: T, label: impl Into<String>) -> Self {
        Self {
            data,
            label: label.into(),
            state: ItemState::Ready,
        }
    }

    /// Returns the underlying data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListItem;
    ///
    /// let item = LoadingListItem::new(42, "Answer");
    /// assert_eq!(*item.data(), 42);
    /// ```
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListItem;
    ///
    /// let mut item = LoadingListItem::new(42u32, "Answer");
    /// *item.data_mut() = 99;
    /// assert_eq!(*item.data(), 99);
    /// ```
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Returns the label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListItem;
    ///
    /// let item = LoadingListItem::new("x", "Display Name");
    /// assert_eq!(item.label(), "Display Name");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Returns the current state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ItemState, LoadingListItem};
    ///
    /// let item = LoadingListItem::new("x", "Item");
    /// assert_eq!(item.state(), &ItemState::Ready);
    /// ```
    pub fn state(&self) -> &ItemState {
        &self.state
    }

    /// Sets the state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{ItemState, LoadingListItem};
    ///
    /// let mut item = LoadingListItem::new("x", "Item");
    /// item.set_state(ItemState::Loading);
    /// assert!(item.is_loading());
    /// ```
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
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct LoadingListState<T: Clone> {
    /// All items.
    items: Vec<LoadingListItem<T>>,
    /// Currently selected index.
    selected: Option<usize>,
    /// Current spinner animation frame.
    spinner_frame: usize,
    /// Optional title.
    title: Option<String>,
    /// Whether to show loading indicators.
    show_indicators: bool,
    /// Scroll state for virtual scrolling and scrollbar rendering.
    #[cfg_attr(feature = "serialization", serde(skip))]
    scroll: ScrollState,
}

impl<T: Clone + PartialEq> PartialEq for LoadingListState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
            && self.selected == other.selected
            && self.spinner_frame == other.spinner_frame
            && self.title == other.title
            && self.show_indicators == other.show_indicators
    }
}

impl<T: Clone> Default for LoadingListState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            spinner_frame: 0,
            title: None,
            show_indicators: true,
            scroll: ScrollState::default(),
        }
    }
}

impl<T: Clone> LoadingListState<T> {
    /// Creates a new empty LoadingList state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let state = LoadingListState::<String>::new();
    /// assert!(state.is_empty());
    /// assert_eq!(state.len(), 0);
    /// ```
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
        let list_items: Vec<LoadingListItem<T>> = items
            .into_iter()
            .map(|data| {
                let label = label_fn(&data);
                LoadingListItem::new(data, label)
            })
            .collect();

        let scroll = ScrollState::new(list_items.len());

        Self {
            items: list_items,
            selected: None,
            spinner_frame: 0,
            title: None,
            show_indicators: true,
            scroll,
        }
    }

    /// Sets the initially selected index (builder method).
    ///
    /// The index is clamped to the valid range. Has no effect on empty lists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// #[derive(Clone)]
    /// struct Task { name: String }
    ///
    /// let state = LoadingListState::with_items(
    ///     vec![
    ///         Task { name: "Build".to_string() },
    ///         Task { name: "Test".to_string() },
    ///         Task { name: "Deploy".to_string() },
    ///     ],
    ///     |t| t.name.clone(),
    /// ).with_selected(1);
    /// assert_eq!(state.selected_index(), Some(1));
    /// ```
    pub fn with_selected(mut self, index: usize) -> Self {
        if self.items.is_empty() {
            return self;
        }
        self.selected = Some(index.min(self.items.len() - 1));
        self
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let state = LoadingListState::<String>::new()
    ///     .with_title("Tasks");
    /// assert_eq!(state.title(), Some("Tasks"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show loading indicators.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let state = LoadingListState::<String>::new()
    ///     .with_indicators(false);
    /// assert!(!state.show_indicators());
    /// ```
    pub fn with_indicators(mut self, show: bool) -> Self {
        self.show_indicators = show;
        self
    }

    /// Returns all items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let state = LoadingListState::with_items(
    ///     vec!["alpha".to_string(), "beta".to_string()],
    ///     |s| s.clone(),
    /// );
    /// assert_eq!(state.items().len(), 2);
    /// assert_eq!(state.items()[0].label(), "alpha");
    /// ```
    pub fn items(&self) -> &[LoadingListItem<T>] {
        &self.items
    }

    /// Returns a mutable reference to all items.
    pub fn items_mut(&mut self) -> &mut Vec<LoadingListItem<T>> {
        &mut self.items
    }

    /// Returns the number of items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let state = LoadingListState::with_items(
    ///     vec!["a".to_string(), "b".to_string()],
    ///     |s| s.clone(),
    /// );
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if there are no items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let state = LoadingListState::<String>::new();
    /// assert!(state.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the selected index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let state = LoadingListState::with_items(
    ///     vec!["a".to_string()],
    ///     |s| s.clone(),
    /// ).with_selected(0);
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Alias for [`selected_index()`](Self::selected_index).
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }

    /// Returns the selected item.
    ///
    /// Returns `None` if no item is selected.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// #[derive(Clone)]
    /// struct Task { name: String }
    ///
    /// let mut state = LoadingListState::with_items(
    ///     vec![Task { name: "Build".to_string() }],
    ///     |t| t.name.clone(),
    /// );
    /// assert!(state.selected_item().is_none());
    ///
    /// state.set_selected(Some(0));
    /// assert_eq!(state.selected_item().unwrap().label(), "Build");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// #[derive(Clone)]
    /// struct Task { name: String }
    ///
    /// let mut state = LoadingListState::with_items(
    ///     vec![Task { name: "Build".to_string() }],
    ///     |t| t.name.clone(),
    /// );
    /// state.set_loading(0);
    /// assert!(state.has_loading());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// #[derive(Clone)]
    /// struct Task { name: String }
    ///
    /// let mut state = LoadingListState::with_items(
    ///     vec![
    ///         Task { name: "A".to_string() },
    ///         Task { name: "B".to_string() },
    ///     ],
    ///     |t| t.name.clone(),
    /// );
    /// state.set_loading(0);
    /// state.set_loading(1);
    /// assert_eq!(state.loading_count(), 2);
    /// ```
    pub fn loading_count(&self) -> usize {
        self.items.iter().filter(|i| i.is_loading()).count()
    }

    /// Returns the number of items with errors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// #[derive(Clone)]
    /// struct Task { name: String }
    ///
    /// let mut state = LoadingListState::with_items(
    ///     vec![
    ///         Task { name: "A".to_string() },
    ///         Task { name: "B".to_string() },
    ///     ],
    ///     |t| t.name.clone(),
    /// );
    /// state.set_error(0, "failed");
    /// assert_eq!(state.error_count(), 1);
    /// ```
    pub fn error_count(&self) -> usize {
        self.items.iter().filter(|i| i.is_error()).count()
    }

    /// Returns true if any item is loading.
    pub fn has_loading(&self) -> bool {
        self.items.iter().any(|i| i.is_loading())
    }

    /// Returns true if any item has an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// #[derive(Clone)]
    /// struct Task { name: String }
    ///
    /// let mut state = LoadingListState::with_items(
    ///     vec![Task { name: "Build".to_string() }],
    ///     |t| t.name.clone(),
    /// );
    /// assert!(!state.has_errors());
    /// state.set_error(0, "Failed to build");
    /// assert!(state.has_errors());
    /// ```
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|i| i.is_error())
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let state = LoadingListState::<String>::new().with_title("Tasks");
    /// assert_eq!(state.title(), Some("Tasks"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let mut state = LoadingListState::<String>::new();
    /// state.set_title(Some("Downloads".to_string()));
    /// assert_eq!(state.title(), Some("Downloads"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns whether indicators are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let state = LoadingListState::<String>::new();
    /// assert!(state.show_indicators()); // enabled by default
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::LoadingListState;
    ///
    /// let mut state = LoadingListState::with_items(
    ///     vec!["a".to_string()],
    ///     |s| s.clone(),
    /// );
    /// assert_eq!(state.len(), 1);
    /// state.clear();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.items.clear();
        self.selected = None;
        self.scroll.set_content_length(0);
    }
}

impl<T: Clone + 'static> LoadingListState<T> {
    /// Updates the loading list state with a message, returning any output.
    pub fn update(&mut self, msg: LoadingListMessage<T>) -> Option<LoadingListOutput<T>> {
        LoadingList::update(self, msg)
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
                state.scroll.set_content_length(state.items.len());
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
                state.scroll.ensure_visible(new_index);
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
                state.scroll.ensure_visible(new_index);
                Some(LoadingListOutput::SelectionChanged(new_index))
            }

            LoadingListMessage::First => {
                if state.items.is_empty() {
                    return None;
                }

                state.selected = Some(0);
                state.scroll.ensure_visible(0);
                Some(LoadingListOutput::SelectionChanged(0))
            }

            LoadingListMessage::Last => {
                if state.items.is_empty() {
                    return None;
                }

                let last = state.items.len() - 1;
                state.selected = Some(last);
                state.scroll.ensure_visible(last);
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

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(LoadingListMessage::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(LoadingListMessage::Down),
                KeyCode::Enter => Some(LoadingListMessage::Select),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        render::render_loading_list(state, frame, area, theme, ctx.focused, ctx.disabled);
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
