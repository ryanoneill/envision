//! A searchable list component combining a text filter with a selectable list.
//!
//! `SearchableList` composes an [`InputField`](super::InputField) and a
//! [`SelectableList`](super::SelectableList) into a single component. Typing
//! in the filter field narrows the visible items, and keyboard navigation
//! lets the user select from the filtered results.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, SearchableList, SearchableListState,
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

use std::fmt::Display;
use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::{Component, Focusable};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

/// A matcher function that takes `(query, item_text)` and returns
/// `None` for no match or `Some(score)` for a ranked match (higher = better).
type MatcherFn = dyn Fn(&str, &str) -> Option<i64>;

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
enum Focus {
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
    items: Vec<T>,
    /// Indices into `items` that match the current filter.
    filtered_indices: Vec<usize>,
    /// Current filter text.
    filter_text: String,
    /// Index into `filtered_indices` for the currently selected item.
    selected: Option<usize>,
    /// Ratatui list state for scroll tracking.
    #[cfg_attr(feature = "serialization", serde(skip))]
    list_state: ListState,
    /// Which sub-component has internal focus.
    internal_focus: Focus,
    /// Whether the overall component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
    /// Placeholder text for the filter input.
    placeholder: String,
    /// Custom matcher function: `(query, item_text) -> Option<score>`.
    /// `None` means no match, `Some(score)` for ranked match (higher = better).
    #[cfg_attr(feature = "serialization", serde(skip))]
    matcher: Option<Box<MatcherFn>>,
}

impl<T: Clone> Clone for SearchableListState<T> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            filtered_indices: self.filtered_indices.clone(),
            filter_text: self.filter_text.clone(),
            selected: self.selected,
            list_state: self.list_state.clone(),
            internal_focus: self.internal_focus.clone(),
            focused: self.focused,
            disabled: self.disabled,
            placeholder: self.placeholder.clone(),
            // The matcher closure cannot be cloned; clones start with no custom matcher.
            matcher: None,
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
            .field("focused", &self.focused)
            .field("disabled", &self.disabled)
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
            && self.focused == other.focused
            && self.disabled == other.disabled
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
            internal_focus: Focus::Filter,
            focused: false,
            disabled: false,
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
        Self {
            items,
            filtered_indices,
            filter_text: String::new(),
            selected,
            list_state,
            internal_focus: Focus::Filter,
            focused: false,
            disabled: false,
            placeholder: "Type to filter...".to_string(),
            matcher: None,
        }
    }

    /// Returns all items (unfiltered).
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Returns the items that match the current filter.
    pub fn filtered_items(&self) -> Vec<&T> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.items.get(i))
            .collect()
    }

    /// Returns the number of filtered items.
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Returns the current filter text.
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Returns the currently selected index within the filtered list.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Returns a reference to the currently selected item.
    pub fn selected_item(&self) -> Option<&T> {
        self.selected
            .and_then(|si| self.filtered_indices.get(si))
            .and_then(|&i| self.items.get(i))
    }

    /// Returns true if the filter input has focus (vs the list).
    pub fn is_filter_focused(&self) -> bool {
        self.internal_focus == Focus::Filter
    }

    /// Returns true if the list has focus (vs the filter).
    pub fn is_list_focused(&self) -> bool {
        self.internal_focus == Focus::List
    }

    /// Returns the placeholder text for the filter input.
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Sets the placeholder text for the filter input.
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
    pub fn with_matcher(mut self, matcher: impl Fn(&str, &str) -> Option<i64> + 'static) -> Self {
        self.matcher = Some(Box::new(matcher));
        self
    }

    /// Returns true if the component is empty (no items at all).
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the total number of items (unfiltered).
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Synchronizes the ratatui ListState with our selected index.
    fn sync_list_state(&mut self) {
        self.list_state.select(self.selected);
    }
}

impl<T: Clone + Display + 'static> SearchableListState<T> {
    /// Sets the items, recomputing the filter and resetting selection.
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.refilter();
    }

    /// Returns true if the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the disabled state (builder pattern).
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Maps an input event to a searchable list message.
    pub fn handle_event(&self, event: &Event) -> Option<SearchableListMessage> {
        SearchableList::<T>::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<SearchableListOutput<T>> {
        SearchableList::<T>::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
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
///     Component, Focusable, SearchableList, SearchableListState,
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

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
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
        if state.disabled {
            return None;
        }

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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Split area: filter input on top (3 lines), list below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        // Render filter input
        let filter_focused = state.focused && state.internal_focus == Focus::Filter;
        let filter_border_style = if state.disabled {
            theme.disabled_style()
        } else if filter_focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let filter_display = if state.filter_text.is_empty() {
            Span::styled(&state.placeholder, theme.disabled_style())
        } else {
            Span::styled(&state.filter_text, theme.normal_style())
        };

        let match_count = format!(" {}/{} ", state.filtered_indices.len(), state.items.len());
        let filter_block = Block::default()
            .borders(Borders::ALL)
            .border_style(filter_border_style)
            .title(Span::styled(" Filter ", theme.normal_style()))
            .title_bottom(Line::from(match_count).alignment(Alignment::Right));

        let filter_widget = Paragraph::new(Line::from(filter_display)).block(filter_block);
        frame.render_widget(filter_widget, chunks[0]);

        // Show cursor in filter when focused
        if filter_focused && !state.disabled {
            let cursor_x = chunks[0].x + 1 + state.filter_text.len() as u16;
            let cursor_y = chunks[0].y + 1;
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }

        // Render filtered list
        let list_focused = state.focused && state.internal_focus == Focus::List;
        let list_border_style = if state.disabled {
            theme.disabled_style()
        } else if list_focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let items: Vec<ListItem> = state
            .filtered_indices
            .iter()
            .filter_map(|&i| state.items.get(i))
            .map(|item| ListItem::new(format!("{}", item)))
            .collect();

        let highlight_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.selected_highlight_style(list_focused)
        };

        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(list_border_style);

        let list_widget = List::new(items)
            .block(list_block)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        let mut list_state = state.list_state.clone();
        frame.render_stateful_widget(list_widget, chunks[1], &mut list_state);
    }
}

impl<T: Clone + Display + 'static> Focusable for SearchableList<T> {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
