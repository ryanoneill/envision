//! A searchable, fuzzy-filtered action/item picker overlay.
//!
//! [`CommandPalette`] provides a popup-style command picker that users can
//! search with fuzzy matching, navigate with arrow keys, and confirm with
//! Enter. State is stored in [`CommandPaletteState`], updated via
//! [`CommandPaletteMessage`], and produces [`CommandPaletteOutput`].
//!
//! Implements [`Focusable`], [`Disableable`], and [`Toggleable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     CommandPalette, CommandPaletteMessage, CommandPaletteOutput,
//!     CommandPaletteState, Component, Focusable, PaletteItem,
//! };
//!
//! let items = vec![
//!     PaletteItem::new("open", "Open File").with_shortcut("Ctrl+O"),
//!     PaletteItem::new("save", "Save File").with_shortcut("Ctrl+S"),
//!     PaletteItem::new("quit", "Quit Application").with_shortcut("Ctrl+Q"),
//! ];
//!
//! let mut state = CommandPaletteState::new(items);
//! state.set_focused(true);
//! state.set_visible(true);
//!
//! // Type to filter
//! CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('o'));
//! assert_eq!(state.query(), "o");
//!
//! // Confirm selection
//! let output = CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
//! assert!(matches!(output, Some(CommandPaletteOutput::Selected(_))));
//! ```

mod item;
mod render;

pub use item::{fuzzy_score, PaletteItem};

use ratatui::prelude::*;

use super::{Component, Disableable, Focusable, Toggleable, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

/// Messages that can be sent to a CommandPalette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandPaletteMessage {
    /// Update the full query text.
    SetQuery(String),
    /// Append a character to the query.
    TypeChar(char),
    /// Delete the last character from the query.
    Backspace,
    /// Clear the entire query.
    ClearQuery,
    /// Move selection down.
    SelectNext,
    /// Move selection up.
    SelectPrev,
    /// Confirm the currently selected item.
    Confirm,
    /// Dismiss (hide) the palette.
    Dismiss,
    /// Show the palette.
    Show,
    /// Replace all items.
    SetItems(Vec<PaletteItem>),
}

/// Output messages from a CommandPalette.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommandPaletteOutput {
    /// An item was confirmed/selected.
    Selected(PaletteItem),
    /// The palette was dismissed.
    Dismissed,
    /// The query text changed.
    QueryChanged(String),
}

/// State for a CommandPalette component.
///
/// Contains all available items, the current search query, filtered results,
/// and display configuration.
///
/// # Example
///
/// ```rust
/// use envision::component::{CommandPaletteState, PaletteItem};
///
/// let items = vec![
///     PaletteItem::new("open", "Open File"),
///     PaletteItem::new("save", "Save File"),
/// ];
/// let state = CommandPaletteState::new(items)
///     .with_title("Actions")
///     .with_placeholder("Search actions...")
///     .with_max_visible(5);
///
/// assert_eq!(state.items().len(), 2);
/// assert_eq!(state.query(), "");
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct CommandPaletteState {
    /// All available items.
    items: Vec<PaletteItem>,
    /// Current search query.
    query: String,
    /// Indices into `items` that match the current query, sorted by score.
    filtered_indices: Vec<usize>,
    /// Selected index within the filtered list.
    selected: Option<usize>,
    /// Whether the palette is shown.
    visible: bool,
    /// Maximum items visible at once (default: 10).
    max_visible: usize,
    /// Input placeholder text.
    placeholder: String,
    /// Optional title.
    title: Option<String>,
    /// Whether the component has focus.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
    /// Scroll state for scrollbar rendering.
    #[cfg_attr(feature = "serialization", serde(skip))]
    scroll: ScrollState,
}

impl PartialEq for CommandPaletteState {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
            && self.query == other.query
            && self.filtered_indices == other.filtered_indices
            && self.selected == other.selected
            && self.visible == other.visible
            && self.max_visible == other.max_visible
            && self.placeholder == other.placeholder
            && self.title == other.title
            && self.focused == other.focused
            && self.disabled == other.disabled
    }
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            query: String::new(),
            filtered_indices: Vec::new(),
            selected: None,
            visible: false,
            max_visible: 10,
            placeholder: "Type to search...".to_string(),
            title: Some("Command Palette".to_string()),
            focused: false,
            disabled: false,
            scroll: ScrollState::default(),
        }
    }
}

impl CommandPaletteState {
    /// Creates a new command palette state with the given items.
    ///
    /// All items are initially visible. If the list is non-empty, the
    /// first item is selected. The palette starts hidden.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![
    ///     PaletteItem::new("a", "Alpha"),
    ///     PaletteItem::new("b", "Beta"),
    /// ]);
    /// assert_eq!(state.items().len(), 2);
    /// assert_eq!(state.filtered_items().len(), 2);
    /// assert!(!state.is_visible());
    /// ```
    pub fn new(items: Vec<PaletteItem>) -> Self {
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
        let selected = if items.is_empty() { None } else { Some(0) };
        let scroll = ScrollState::new(filtered_indices.len());
        Self {
            items,
            filtered_indices,
            selected,
            scroll,
            ..Default::default()
        }
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]).with_title("Actions");
    /// assert_eq!(state.title(), Some("Actions"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the placeholder text (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]).with_placeholder("Search commands...");
    /// assert_eq!(state.placeholder(), "Search commands...");
    /// ```
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the maximum number of visible items (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]).with_max_visible(5);
    /// assert_eq!(state.max_visible(), 5);
    /// ```
    pub fn with_max_visible(mut self, max_visible: usize) -> Self {
        self.max_visible = max_visible;
        self
    }

    /// Sets the initial visibility (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]).with_visible(true);
    /// assert!(state.is_visible());
    /// ```
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]).with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns all items (unfiltered).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![
    ///     PaletteItem::new("a", "Alpha"),
    /// ]);
    /// assert_eq!(state.items().len(), 1);
    /// assert_eq!(state.items()[0].label, "Alpha");
    /// ```
    pub fn items(&self) -> &[PaletteItem] {
        &self.items
    }

    /// Returns the current search query.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]);
    /// assert_eq!(state.query(), "");
    /// ```
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the items matching the current query.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![
    ///     PaletteItem::new("a", "Alpha"),
    ///     PaletteItem::new("b", "Beta"),
    /// ]);
    /// assert_eq!(state.filtered_items().len(), 2);
    /// ```
    pub fn filtered_items(&self) -> Vec<&PaletteItem> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.items.get(i))
            .collect()
    }

    /// Returns the currently highlighted/selected item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![
    ///     PaletteItem::new("a", "Alpha"),
    ///     PaletteItem::new("b", "Beta"),
    /// ]);
    /// assert_eq!(state.selected_item().map(|i| &i.label), Some(&"Alpha".to_string()));
    /// ```
    pub fn selected_item(&self) -> Option<&PaletteItem> {
        self.selected
            .and_then(|si| self.filtered_indices.get(si))
            .and_then(|&i| self.items.get(i))
    }

    /// Replaces all items and refilters.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let mut state = CommandPaletteState::new(vec![PaletteItem::new("a", "Alpha")]);
    /// state.set_items(vec![
    ///     PaletteItem::new("x", "X-ray"),
    ///     PaletteItem::new("y", "Yankee"),
    /// ]);
    /// assert_eq!(state.items().len(), 2);
    /// ```
    pub fn set_items(&mut self, items: Vec<PaletteItem>) {
        self.items = items;
        self.refilter();
    }

    /// Shows the palette and gives it focus.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let mut state = CommandPaletteState::new(vec![]);
    /// state.show();
    /// assert!(state.is_visible());
    /// assert!(state.is_focused());
    /// ```
    pub fn show(&mut self) {
        self.visible = true;
        self.focused = true;
    }

    /// Dismisses the palette: hides it and clears the query.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{
    ///     CommandPalette, CommandPaletteState, CommandPaletteMessage, PaletteItem, Component,
    /// };
    ///
    /// let mut state = CommandPaletteState::new(vec![PaletteItem::new("a", "Alpha")]);
    /// state.show();
    /// CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('a'));
    /// state.dismiss();
    /// assert!(!state.is_visible());
    /// assert_eq!(state.query(), "");
    /// ```
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.query.clear();
        self.refilter();
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]);
    /// assert_eq!(state.title(), Some("Command Palette"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let mut state = CommandPaletteState::new(vec![]);
    /// state.set_title("Actions");
    /// assert_eq!(state.title(), Some("Actions"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns the placeholder text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]);
    /// assert_eq!(state.placeholder(), "Type to search...");
    /// ```
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Sets the placeholder text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let mut state = CommandPaletteState::new(vec![]);
    /// state.set_placeholder("Search commands...");
    /// assert_eq!(state.placeholder(), "Search commands...");
    /// ```
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Returns the maximum number of visible items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]);
    /// assert_eq!(state.max_visible(), 10);
    /// ```
    pub fn max_visible(&self) -> usize {
        self.max_visible
    }

    /// Sets the maximum number of visible items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let mut state = CommandPaletteState::new(vec![]);
    /// state.set_max_visible(20);
    /// assert_eq!(state.max_visible(), 20);
    /// ```
    pub fn set_max_visible(&mut self, max_visible: usize) {
        self.max_visible = max_visible;
    }

    /// Returns the number of items matching the current query.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![
    ///     PaletteItem::new("a", "Alpha"),
    ///     PaletteItem::new("b", "Beta"),
    /// ]);
    /// assert_eq!(state.filtered_count(), 2);
    /// ```
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Returns the selected index within the filtered list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![PaletteItem::new("a", "Alpha")]);
    /// assert_eq!(state.selected_index(), Some(0));
    ///
    /// let empty = CommandPaletteState::new(vec![]);
    /// assert_eq!(empty.selected_index(), None);
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Returns true if the palette is visible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]);
    /// assert!(!state.is_visible());
    /// ```
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets the visibility.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let mut state = CommandPaletteState::new(vec![]);
    /// state.set_visible(true);
    /// assert!(state.is_visible());
    /// ```
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Returns true if the component is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]);
    /// assert!(!state.is_focused());
    /// ```
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let mut state = CommandPaletteState::new(vec![]);
    /// state.set_focused(true);
    /// assert!(state.is_focused());
    /// ```
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let state = CommandPaletteState::new(vec![]);
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, PaletteItem};
    ///
    /// let mut state = CommandPaletteState::new(vec![]);
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Maps an input event to a command palette message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, CommandPaletteMessage, PaletteItem};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = CommandPaletteState::new(vec![PaletteItem::new("a", "Alpha")]);
    /// state.set_focused(true);
    /// state.set_visible(true);
    /// let msg = state.handle_event(&Event::char('x'));
    /// assert_eq!(msg, Some(CommandPaletteMessage::TypeChar('x')));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<CommandPaletteMessage> {
        CommandPalette::handle_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, CommandPaletteMessage, CommandPaletteOutput, PaletteItem};
    ///
    /// let mut state = CommandPaletteState::new(vec![PaletteItem::new("a", "Alpha")]);
    /// state.set_focused(true);
    /// state.set_visible(true);
    /// let output = state.update(CommandPaletteMessage::TypeChar('a'));
    /// assert!(matches!(output, Some(CommandPaletteOutput::QueryChanged(_))));
    /// ```
    pub fn update(&mut self, msg: CommandPaletteMessage) -> Option<CommandPaletteOutput> {
        CommandPalette::update(self, msg)
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CommandPaletteState, CommandPaletteOutput, PaletteItem};
    /// use envision::input::Event;
    ///
    /// let mut state = CommandPaletteState::new(vec![PaletteItem::new("a", "Alpha")]);
    /// state.set_focused(true);
    /// state.set_visible(true);
    /// let output = state.dispatch_event(&Event::char('a'));
    /// assert!(matches!(output, Some(CommandPaletteOutput::QueryChanged(_))));
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<CommandPaletteOutput> {
        CommandPalette::dispatch_event(self, event)
    }

    /// Recomputes filtered indices based on the current query.
    fn refilter(&mut self) {
        if self.query.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            let mut scored: Vec<(usize, usize)> = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    fuzzy_score(&self.query, &item.label).map(|score| (i, score))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_indices = scored.into_iter().map(|(i, _)| i).collect();
        }

        self.scroll.set_content_length(self.filtered_indices.len());

        if self.filtered_indices.is_empty() {
            self.selected = None;
        } else {
            self.selected = Some(0);
        }
    }
}

/// A searchable, fuzzy-filtered command palette overlay component.
///
/// Displays a popup-style list of actions/items that can be searched,
/// navigated, and selected. Designed to be used as an overlay.
///
/// # Keyboard Navigation
///
/// - Character keys: type to filter
/// - Backspace: delete last character
/// - Up / Ctrl+P: move selection up
/// - Down / Ctrl+N: move selection down
/// - Enter: confirm selection
/// - Escape: dismiss palette
/// - Ctrl+U: clear query
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     CommandPalette, CommandPaletteMessage, CommandPaletteOutput,
///     CommandPaletteState, Component, Focusable, PaletteItem,
/// };
///
/// let items = vec![
///     PaletteItem::new("open", "Open File"),
///     PaletteItem::new("save", "Save File"),
/// ];
/// let mut state = CommandPaletteState::new(items);
/// state.set_focused(true);
/// state.set_visible(true);
///
/// // Filter to "save"
/// CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('s'));
/// CommandPalette::update(&mut state, CommandPaletteMessage::TypeChar('a'));
///
/// // Confirm selection
/// let output = CommandPalette::update(&mut state, CommandPaletteMessage::Confirm);
/// assert!(matches!(output, Some(CommandPaletteOutput::Selected(_))));
/// ```
pub struct CommandPalette;

impl Component for CommandPalette {
    type State = CommandPaletteState;
    type Message = CommandPaletteMessage;
    type Output = CommandPaletteOutput;

    fn init() -> Self::State {
        CommandPaletteState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled || !state.visible {
            return None;
        }

        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Esc => Some(CommandPaletteMessage::Dismiss),
                KeyCode::Enter => Some(CommandPaletteMessage::Confirm),
                KeyCode::Backspace => Some(CommandPaletteMessage::Backspace),
                KeyCode::Up => Some(CommandPaletteMessage::SelectPrev),
                KeyCode::Down => Some(CommandPaletteMessage::SelectNext),
                KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(CommandPaletteMessage::SelectPrev)
                }
                KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(CommandPaletteMessage::SelectNext)
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Some(CommandPaletteMessage::ClearQuery)
                }
                KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                    Some(CommandPaletteMessage::TypeChar(c))
                }
                KeyCode::Char(c) if key.modifiers == KeyModifiers::SHIFT => {
                    Some(CommandPaletteMessage::TypeChar(c))
                }
                _ => None,
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
            CommandPaletteMessage::SetQuery(text) => {
                state.query = text.clone();
                state.refilter();
                Some(CommandPaletteOutput::QueryChanged(text))
            }
            CommandPaletteMessage::TypeChar(c) => {
                state.query.push(c);
                let text = state.query.clone();
                state.refilter();
                Some(CommandPaletteOutput::QueryChanged(text))
            }
            CommandPaletteMessage::Backspace => {
                if !state.query.is_empty() {
                    state.query.pop();
                    let text = state.query.clone();
                    state.refilter();
                    Some(CommandPaletteOutput::QueryChanged(text))
                } else {
                    None
                }
            }
            CommandPaletteMessage::ClearQuery => {
                if !state.query.is_empty() {
                    state.query.clear();
                    state.refilter();
                    Some(CommandPaletteOutput::QueryChanged(String::new()))
                } else {
                    None
                }
            }
            CommandPaletteMessage::SelectNext => {
                if let Some(current) = state.selected {
                    let len = state.filtered_indices.len();
                    if len > 0 {
                        let new_index = (current + 1) % len;
                        state.selected = Some(new_index);
                        state.scroll.ensure_visible(new_index);
                    }
                }
                None
            }
            CommandPaletteMessage::SelectPrev => {
                if let Some(current) = state.selected {
                    let len = state.filtered_indices.len();
                    if len > 0 {
                        let new_index = if current == 0 { len - 1 } else { current - 1 };
                        state.selected = Some(new_index);
                        state.scroll.ensure_visible(new_index);
                    }
                }
                None
            }
            CommandPaletteMessage::Confirm => {
                let item = state
                    .selected
                    .and_then(|si| state.filtered_indices.get(si).copied())
                    .and_then(|i| state.items.get(i).cloned());
                if let Some(item) = item {
                    state.visible = false;
                    state.query.clear();
                    state.refilter();
                    Some(CommandPaletteOutput::Selected(item))
                } else {
                    None
                }
            }
            CommandPaletteMessage::Dismiss => {
                state.visible = false;
                state.query.clear();
                state.refilter();
                Some(CommandPaletteOutput::Dismissed)
            }
            CommandPaletteMessage::Show => {
                state.visible = true;
                state.focused = true;
                None
            }
            CommandPaletteMessage::SetItems(items) => {
                state.items = items;
                state.refilter();
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        render::render_command_palette(state, frame, area, theme, ctx.focused, ctx.disabled);
    }
}

impl Focusable for CommandPalette {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for CommandPalette {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

impl Toggleable for CommandPalette {
    fn is_visible(state: &Self::State) -> bool {
        state.visible
    }

    fn set_visible(state: &mut Self::State, visible: bool) {
        state.visible = visible;
    }
}

#[cfg(test)]
mod event_tests;
#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
