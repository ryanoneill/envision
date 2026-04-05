//! A searchable list component combining a text filter with a selectable list.
//!
//! [`SearchableList<T>`] composes an [`InputField`](super::InputField) and a
//! [`SelectableList`](super::SelectableList) into a single component. Typing
//! in the filter field narrows the visible items, and keyboard navigation
//! lets the user select from the filtered results. State is stored in
//! [`SearchableListState<T>`], updated via [`SearchableListMessage`], and
//! produces [`SearchableListOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, SearchableList, SearchableListState,
//!     SearchableListMessage, SearchableListOutput,
//! };
//!
//! let mut state = SearchableListState::new(vec![
//!     "Apple".to_string(),
//!     "Banana".to_string(),
//!     "Cherry".to_string(),
//!     "Date".to_string(),
//! ]);
//! SearchableList::set_focused(&mut state, true);
//!
//! // Type "an" to filter
//! SearchableList::update(&mut state, SearchableListMessage::FilterChanged("an".into()));
//! assert_eq!(state.filtered_items().len(), 1); // "Banana" matches
//!
//! // Select the current item
//! let output = SearchableList::update(&mut state, SearchableListMessage::Select);
//! assert!(matches!(output, Some(SearchableListOutput::Selected(_))));
//! ```

mod render;

use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Arc;

use ratatui::prelude::*;
use ratatui::widgets::ListState;

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// A matcher function that takes `(query, item_text)` and returns
/// `None` for no match or `Some(score)` for a ranked match (higher = better).
type MatcherFn = dyn Fn(&str, &str) -> Option<i64> + Send + Sync;

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
pub(super) enum Focus {
    /// The filter input field has focus.
    Filter,
    /// The selectable list has focus.
    List,
}

/// State for a SearchableList component.
///
/// Contains the full list of items, the current filter text, and the
/// filtered subset that is displayed. The filter is case-insensitive.
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SearchableListState<T: Clone> {
    /// All items (unfiltered).
    pub(super) items: Vec<T>,
    /// Indices into `items` that match the current filter.
    pub(super) filtered_indices: Vec<usize>,
    /// Current filter text.
    pub(super) filter_text: String,
    /// Index into `filtered_indices` for the currently selected item.
    pub(super) selected: Option<usize>,
    /// Ratatui list state for scroll tracking.
    #[cfg_attr(feature = "serialization", serde(skip))]
    pub(super) list_state: ListState,
    /// Scroll state for scrollbar rendering.
    #[cfg_attr(feature = "serialization", serde(skip))]
    pub(super) scroll: ScrollState,
    /// Which sub-component has internal focus.
    pub(super) internal_focus: Focus,
    /// Placeholder text for the filter input.
    pub(super) placeholder: String,
    /// Custom matcher function: `(query, item_text) -> Option<score>`.
    /// `None` means no match, `Some(score)` for ranked match (higher = better).
    /// Wrapped in `Arc` so that `SearchableListState` can derive `Clone`.
    #[cfg_attr(feature = "serialization", serde(skip))]
    pub(super) matcher: Option<Arc<MatcherFn>>,
}

impl<T: Clone> Clone for SearchableListState<T> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            filtered_indices: self.filtered_indices.clone(),
            filter_text: self.filter_text.clone(),
            selected: self.selected,
            list_state: self.list_state.clone(),
            scroll: self.scroll.clone(),
            internal_focus: self.internal_focus.clone(),
            placeholder: self.placeholder.clone(),
            matcher: self.matcher.clone(),
        }
    }
}

impl<T: Clone + std::fmt::Debug> std::fmt::Debug for SearchableListState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchableListState")
            .field("items", &self.items)
            .field("filtered_indices", &self.filtered_indices)
            .field("filter_text", &self.filter_text)
            .field("selected", &self.selected)
            .field("list_state", &self.list_state)
            .field("internal_focus", &self.internal_focus)
            .field("placeholder", &self.placeholder)
            .field("matcher", &self.matcher.as_ref().map(|_| "..."))
            .finish()
    }
}

impl<T: Clone + PartialEq> PartialEq for SearchableListState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
            && self.filtered_indices == other.filtered_indices
            && self.filter_text == other.filter_text
            && self.selected == other.selected
            && self.list_state.selected() == other.list_state.selected()
            && self.internal_focus == other.internal_focus
            && self.placeholder == other.placeholder
    }
}

impl<T: Clone> Default for SearchableListState<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            filtered_indices: Vec::new(),
            filter_text: String::new(),
            selected: None,
            list_state: ListState::default(),
            scroll: ScrollState::default(),
            internal_focus: Focus::Filter,
            placeholder: "Type to filter...".to_string(),
            matcher: None,
        }
    }
}

impl<T: Clone> SearchableListState<T> {
    /// Creates a new state with the given items and an empty filter.
    ///
    /// All items are initially visible. If the list is non-empty, the
    /// first item is selected.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::new(vec!["Alpha".to_string(), "Beta".to_string()]);
    /// assert_eq!(state.items().len(), 2);
    /// assert_eq!(state.filtered_items().len(), 2);
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn new(items: Vec<T>) -> Self {
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
        let selected = if items.is_empty() { None } else { Some(0) };
        let mut list_state = ListState::default();
        list_state.select(selected);
        let scroll = ScrollState::new(filtered_indices.len());
        Self {
            items,
            filtered_indices,
            filter_text: String::new(),
            selected,
            list_state,
            scroll,
            internal_focus: Focus::Filter,
            placeholder: "Type to filter...".to_string(),
            matcher: None,
        }
    }

    /// Returns all items (unfiltered).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::new(vec!["A".to_string(), "B".to_string()]);
    /// assert_eq!(state.items().len(), 2);
    /// assert_eq!(state.items()[0], "A");
    /// ```
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Returns the items that match the current filter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::new(vec![
    ///     "Apple".to_string(), "Banana".to_string(), "Cherry".to_string(),
    /// ]);
    /// // No filter applied, all items visible
    /// assert_eq!(state.filtered_items().len(), 3);
    /// ```
    pub fn filtered_items(&self) -> Vec<&T> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.items.get(i))
            .collect()
    }

    /// Returns the number of filtered items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::new(vec!["A".to_string(), "B".to_string()]);
    /// assert_eq!(state.filtered_count(), 2);
    /// ```
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Returns the current filter text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::<String>::new(vec![]);
    /// assert_eq!(state.filter_text(), "");
    /// ```
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Returns the currently selected index within the filtered list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::new(vec!["A".to_string()]);
    /// assert_eq!(state.selected_index(), Some(0));
    ///
    /// let empty = SearchableListState::<String>::new(vec![]);
    /// assert_eq!(empty.selected_index(), None);
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Alias for [`selected_index()`](Self::selected_index).
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }

    /// Returns a reference to the currently selected item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::new(vec!["Alpha".to_string(), "Beta".to_string()]);
    /// assert_eq!(state.selected_item(), Some(&"Alpha".to_string()));
    /// ```
    pub fn selected_item(&self) -> Option<&T> {
        self.selected
            .and_then(|si| self.filtered_indices.get(si))
            .and_then(|&i| self.items.get(i))
    }

    /// Sets the selected index within the filtered list.
    ///
    /// The index is clamped to the valid range of filtered items. Has no effect
    /// on empty filtered lists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let mut state = SearchableListState::new(vec![
    ///     "Apple".to_string(),
    ///     "Banana".to_string(),
    ///     "Cherry".to_string(),
    /// ]);
    /// state.set_selected(Some(2));
    /// assert_eq!(state.selected_index(), Some(2));
    /// assert_eq!(state.selected_item(), Some(&"Cherry".to_string()));
    /// ```
    pub fn set_selected(&mut self, index: Option<usize>) {
        match index {
            Some(i) => {
                if self.filtered_indices.is_empty() {
                    return;
                }
                self.selected = Some(i.min(self.filtered_indices.len() - 1));
            }
            None => self.selected = None,
        }
        self.sync_list_state();
    }

    /// Returns true if the filter input has focus (vs the list).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::new(vec!["A".to_string()]);
    /// assert!(state.is_filter_focused()); // filter focused by default
    /// ```
    pub fn is_filter_focused(&self) -> bool {
        self.internal_focus == Focus::Filter
    }

    /// Returns true if the list has focus (vs the filter).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::new(vec!["A".to_string()]);
    /// assert!(!state.is_list_focused()); // filter is focused by default
    /// ```
    pub fn is_list_focused(&self) -> bool {
        self.internal_focus == Focus::List
    }

    /// Returns the placeholder text for the filter input.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::<String>::new(vec![]);
    /// assert_eq!(state.placeholder(), "Type to filter...");
    /// ```
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Sets the placeholder text for the filter input.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let mut state = SearchableListState::<String>::new(vec![]);
    /// state.set_placeholder("Search...");
    /// assert_eq!(state.placeholder(), "Search...");
    /// ```
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Sets the placeholder text (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::<String>::new(vec![])
    ///     .with_placeholder("Search items...");
    /// assert_eq!(state.placeholder(), "Search items...");
    /// ```
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets a custom matcher function (builder pattern).
    ///
    /// The matcher takes `(query, item_text)` and returns `None` for no match
    /// or `Some(score)` for a ranked match. Higher scores appear first in the
    /// filtered results.
    ///
    /// When no custom matcher is set, the default case-insensitive substring
    /// match is used (all matches are treated equally and maintain original order).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// // Prefix-only matcher that scores by match length
    /// let state = SearchableListState::new(vec![
    ///     "Apple".to_string(),
    ///     "Banana".to_string(),
    ///     "Apricot".to_string(),
    /// ]).with_matcher(|query, item| {
    ///     let item_lower = item.to_lowercase();
    ///     let query_lower = query.to_lowercase();
    ///     if item_lower.starts_with(&query_lower) {
    ///         // Shorter items rank higher (more specific match)
    ///         Some(1000 - item.len() as i64)
    ///     } else {
    ///         None
    ///     }
    /// });
    /// ```
    pub fn with_matcher(
        mut self,
        matcher: impl Fn(&str, &str) -> Option<i64> + Send + Sync + 'static,
    ) -> Self {
        self.matcher = Some(Arc::new(matcher));
        self
    }

    /// Updates an item at the given index via a closure.
    ///
    /// No-ops if the index is out of bounds. This is safe because it
    /// Returns true if the component is empty (no items at all).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::<String>::new(vec![]);
    /// assert!(state.is_empty());
    ///
    /// let state2 = SearchableListState::new(vec!["A".to_string()]);
    /// assert!(!state2.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the total number of items (unfiltered).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let state = SearchableListState::new(vec!["A".to_string(), "B".to_string()]);
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Synchronizes the ratatui ListState with our selected index.
    fn sync_list_state(&mut self) {
        self.list_state.select(self.selected);
    }
}

impl<T: Clone + Display + 'static> SearchableListState<T> {
    /// Pushes an item and recomputes the filter.
    ///
    /// The new item is appended to the end of the list and the
    /// current filter is re-applied.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let mut state = SearchableListState::new(vec!["apple".to_string()]);
    /// state.push_item("banana".to_string());
    /// assert_eq!(state.len(), 2);
    /// assert_eq!(state.items()[1], "banana");
    /// ```
    pub fn push_item(&mut self, item: T) {
        self.items.push(item);
        self.refilter();
    }

    /// Updates an item at the given index and re-applies the active filter.
    ///
    /// No-ops if the index is out of bounds. The filter is recomputed
    /// after mutation to maintain consistency.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let mut state = SearchableListState::new(vec!["apple".to_string(), "banana".to_string()]);
    /// state.update_item(0, |item| *item = "APPLE".to_string());
    /// assert_eq!(state.items()[0], "APPLE");
    /// ```
    pub fn update_item(&mut self, index: usize, f: impl FnOnce(&mut T)) {
        if let Some(item) = self.items.get_mut(index) {
            f(item);
            self.refilter();
        }
    }

    /// Removes an item by index and recomputes the filter and selection.
    ///
    /// Returns the removed item, or `None` if the index is out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let mut state = SearchableListState::new(vec![
    ///     "apple".to_string(), "banana".to_string(), "cherry".to_string(),
    /// ]);
    /// let removed = state.remove_item(1);
    /// assert_eq!(removed, Some("banana".to_string()));
    /// assert_eq!(state.len(), 2);
    /// assert_eq!(state.items()[1], "cherry");
    /// ```
    pub fn remove_item(&mut self, index: usize) -> Option<T> {
        if index >= self.items.len() {
            return None;
        }
        let item = self.items.remove(index);
        self.refilter();
        Some(item)
    }

    /// Sets the items, recomputing the filter and resetting selection.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SearchableListState;
    ///
    /// let mut state = SearchableListState::new(vec!["A".to_string()]);
    /// state.set_items(vec!["X".to_string(), "Y".to_string(), "Z".to_string()]);
    /// assert_eq!(state.len(), 3);
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.refilter();
    }

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SearchableListState, SearchableListMessage, SearchableListOutput};
    ///
    /// let mut state = SearchableListState::new(vec![
    ///     "Apple".to_string(), "Banana".to_string(), "Cherry".to_string(),
    /// ]);
    /// let output = state.update(SearchableListMessage::FilterChanged("an".to_string()));
    /// assert_eq!(state.filtered_count(), 1);
    /// assert_eq!(state.selected_item(), Some(&"Banana".to_string()));
    /// ```
    pub fn update(&mut self, msg: SearchableListMessage) -> Option<SearchableListOutput<T>> {
        SearchableList::<T>::update(self, msg)
    }

    /// Recomputes the filtered indices based on the current filter text.
    fn refilter(&mut self) {
        let filter_lower = self.filter_text.to_lowercase();
        if filter_lower.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else if let Some(ref matcher) = self.matcher {
            // Custom matcher: collect (index, score) pairs, sort by score descending
            let mut scored: Vec<(usize, i64)> = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    let text = format!("{}", item);
                    matcher(&self.filter_text, &text).map(|score| (i, score))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_indices = scored.into_iter().map(|(i, _)| i).collect();
        } else {
            // Default: case-insensitive substring match
            self.filtered_indices = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    let text = format!("{}", item).to_lowercase();
                    text.contains(&filter_lower)
                })
                .map(|(i, _)| i)
                .collect();
        };

        self.scroll.set_content_length(self.filtered_indices.len());

        // Reset selection to first filtered item
        if self.filtered_indices.is_empty() {
            self.selected = None;
        } else {
            self.selected = Some(0);
        }
        self.sync_list_state();
    }
}

/// A searchable list component combining a filter input with a selectable list.
///
/// This compound component composes an input field for filtering with a
/// selectable list for browsing and selecting items. The filter is
/// case-insensitive and matches anywhere in the item's display text.
///
/// # Type Parameters
///
/// - `T`: The item type. Must implement `Clone` and `Display`.
///
/// # Sub-component Focus
///
/// The component manages internal focus between the filter input and the list:
/// - **Tab** toggles focus between filter and list
/// - When the filter is focused, typing updates the filter and refilters the list
/// - When the list is focused, Up/Down navigates and Enter selects
///
/// # Navigation
///
/// - `Up` / `Down` — Move selection (works from either sub-component)
/// - `Enter` — Select the current item (when list is focused) or move to list (when filter is focused)
/// - `Tab` — Toggle between filter and list focus
/// - `Esc` — Clear filter text
/// - Typing — Updates the filter (when filter is focused)
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, SearchableList, SearchableListState,
///     SearchableListMessage, SearchableListOutput,
/// };
///
/// let items = vec!["Apple".to_string(), "Banana".to_string(), "Cherry".to_string()];
/// let mut state = SearchableListState::new(items);
/// SearchableList::set_focused(&mut state, true);
///
/// // Filter to items containing "an"
/// SearchableList::update(&mut state, SearchableListMessage::FilterChanged("an".into()));
/// assert_eq!(state.filtered_items().len(), 1); // "Banana"
///
/// // Select the highlighted item
/// let output = SearchableList::update(&mut state, SearchableListMessage::Select);
/// assert_eq!(output, Some(SearchableListOutput::Selected("Banana".to_string())));
/// ```
pub struct SearchableList<T: Clone>(PhantomData<T>);

impl<T: Clone + Display + 'static> Component for SearchableList<T> {
    type State = SearchableListState<T>;
    type Message = SearchableListMessage;
    type Output = SearchableListOutput<T>;

    fn init() -> Self::State {
        SearchableListState::default()
    }

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        if let Some(key) = event.as_key() {
            // Tab always toggles focus between filter and list
            if key.code == KeyCode::Tab || key.code == KeyCode::BackTab {
                return Some(SearchableListMessage::ToggleFocus);
            }

            // Esc clears the filter
            if key.code == KeyCode::Esc {
                return Some(SearchableListMessage::FilterClear);
            }

            match state.internal_focus {
                Focus::Filter => {
                    match key.code {
                        // Navigation keys work from filter too
                        KeyCode::Up | KeyCode::Char('k')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            Some(SearchableListMessage::Up)
                        }
                        KeyCode::Down | KeyCode::Char('j')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            Some(SearchableListMessage::Down)
                        }
                        // Enter in filter moves focus to list
                        KeyCode::Enter => Some(SearchableListMessage::ToggleFocus),
                        // Backspace deletes from filter
                        KeyCode::Backspace => Some(SearchableListMessage::FilterBackspace),
                        // Regular characters go to filter
                        KeyCode::Char(c) => Some(SearchableListMessage::FilterChar(c)),
                        _ => None,
                    }
                }
                Focus::List => {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => Some(SearchableListMessage::Up),
                        KeyCode::Down | KeyCode::Char('j') => Some(SearchableListMessage::Down),
                        KeyCode::Home | KeyCode::Char('g') => Some(SearchableListMessage::First),
                        KeyCode::End | KeyCode::Char('G') => Some(SearchableListMessage::Last),
                        KeyCode::PageUp => Some(SearchableListMessage::PageUp(10)),
                        KeyCode::PageDown => Some(SearchableListMessage::PageDown(10)),
                        KeyCode::Enter => Some(SearchableListMessage::Select),
                        // In list mode, typing switches to filter
                        KeyCode::Char(c) => {
                            // Let user type to start filtering from list view
                            Some(SearchableListMessage::FilterChar(c))
                        }
                        _ => None,
                    }
                }
            }
        } else {
            None
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            SearchableListMessage::FilterChanged(text) => {
                state.filter_text = text.clone();
                state.refilter();
                Some(SearchableListOutput::FilterChanged(text))
            }
            SearchableListMessage::FilterChar(c) => {
                // If typing from list, switch focus to filter
                if state.internal_focus == Focus::List {
                    state.internal_focus = Focus::Filter;
                }
                state.filter_text.push(c);
                let text = state.filter_text.clone();
                state.refilter();
                Some(SearchableListOutput::FilterChanged(text))
            }
            SearchableListMessage::FilterBackspace => {
                if !state.filter_text.is_empty() {
                    state.filter_text.pop();
                    let text = state.filter_text.clone();
                    state.refilter();
                    Some(SearchableListOutput::FilterChanged(text))
                } else {
                    None
                }
            }
            SearchableListMessage::FilterClear => {
                if !state.filter_text.is_empty() {
                    state.filter_text.clear();
                    state.refilter();
                    Some(SearchableListOutput::FilterChanged(String::new()))
                } else {
                    None
                }
            }
            SearchableListMessage::Up => {
                if let Some(current) = state.selected {
                    let new_index = current.saturating_sub(1);
                    if new_index != current {
                        state.selected = Some(new_index);
                        state.sync_list_state();
                        return Some(SearchableListOutput::SelectionChanged(new_index));
                    }
                }
                None
            }
            SearchableListMessage::Down => {
                if let Some(current) = state.selected {
                    let len = state.filtered_indices.len();
                    if len > 0 {
                        let new_index = (current + 1).min(len - 1);
                        if new_index != current {
                            state.selected = Some(new_index);
                            state.sync_list_state();
                            return Some(SearchableListOutput::SelectionChanged(new_index));
                        }
                    }
                }
                None
            }
            SearchableListMessage::First => {
                if !state.filtered_indices.is_empty() && state.selected != Some(0) {
                    state.selected = Some(0);
                    state.sync_list_state();
                    return Some(SearchableListOutput::SelectionChanged(0));
                }
                None
            }
            SearchableListMessage::Last => {
                let len = state.filtered_indices.len();
                if len > 0 && state.selected != Some(len - 1) {
                    state.selected = Some(len - 1);
                    state.sync_list_state();
                    return Some(SearchableListOutput::SelectionChanged(len - 1));
                }
                None
            }
            SearchableListMessage::PageUp(page_size) => {
                if let Some(current) = state.selected {
                    let new_index = current.saturating_sub(page_size);
                    if new_index != current {
                        state.selected = Some(new_index);
                        state.sync_list_state();
                        return Some(SearchableListOutput::SelectionChanged(new_index));
                    }
                }
                None
            }
            SearchableListMessage::PageDown(page_size) => {
                if let Some(current) = state.selected {
                    let len = state.filtered_indices.len();
                    if len > 0 {
                        let new_index = (current + page_size).min(len - 1);
                        if new_index != current {
                            state.selected = Some(new_index);
                            state.sync_list_state();
                            return Some(SearchableListOutput::SelectionChanged(new_index));
                        }
                    }
                }
                None
            }
            SearchableListMessage::Select => state
                .selected
                .and_then(|si| state.filtered_indices.get(si).copied())
                .and_then(|i| state.items.get(i).cloned())
                .map(SearchableListOutput::Selected),
            SearchableListMessage::ToggleFocus => {
                state.internal_focus = match state.internal_focus {
                    Focus::Filter => Focus::List,
                    Focus::List => Focus::Filter,
                };
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        render::render_searchable_list(state, frame, area, theme, ctx.focused, ctx.disabled);
    }
}

#[cfg(test)]
mod event_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
