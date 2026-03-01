//! A horizontal tab navigation component.
//!
//! `Tabs` provides a horizontal tab bar for switching between views or panels.
//! It supports keyboard navigation with Left/Right keys and generic tab types.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Focusable, Tabs, TabsState, TabsMessage, TabsOutput};
//!
//! // Create tabs with string labels
//! let mut state = TabsState::new(vec!["Home", "Settings", "Help"]);
//! Tabs::set_focused(&mut state, true);
//!
//! assert_eq!(state.selected_index(), Some(0));
//! assert_eq!(state.selected(), Some(&"Home"));
//!
//! // Navigate right
//! let output = Tabs::<&str>::update(&mut state, TabsMessage::Right);
//! assert_eq!(output, Some(TabsOutput::SelectionChanged(1)));
//! assert_eq!(state.selected_index(), Some(1));
//! ```

use std::fmt::Display;
use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Messages that can be sent to a Tabs component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabsMessage {
    /// Navigate to the previous (left) tab.
    Left,
    /// Navigate to the next (right) tab.
    Right,
    /// Jump to a specific tab by index.
    Select(usize),
    /// Go to the first tab.
    First,
    /// Go to the last tab.
    Last,
    /// Confirm the current selection.
    Confirm,
}

/// Output messages from a Tabs component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabsOutput<T: Clone> {
    /// The selected tab changed.
    Selected(T),
    /// The current selection was confirmed.
    Confirmed(T),
    /// The selection changed during navigation (contains the new index).
    SelectionChanged(usize),
}

/// State for a Tabs component.
///
/// The state tracks the available tabs, the currently selected tab,
/// and focus/disabled states.
#[derive(Clone, Debug)]
pub struct TabsState<T: Clone> {
    /// The available tabs.
    tabs: Vec<T>,
    /// Currently selected tab index, or `None` if empty.
    selected: Option<usize>,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl<T: Clone> Default for TabsState<T> {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            selected: None,
            focused: false,
            disabled: false,
        }
    }
}

impl<T: Clone> TabsState<T> {
    /// Creates a new tabs state with the first tab selected.
    ///
    /// If tabs are empty, the selection is `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TabsState;
    ///
    /// let state = TabsState::new(vec!["Tab1", "Tab2", "Tab3"]);
    /// assert_eq!(state.selected_index(), Some(0));
    /// assert_eq!(state.len(), 3);
    /// ```
    pub fn new(tabs: Vec<T>) -> Self {
        let selected = if tabs.is_empty() { None } else { Some(0) };
        Self {
            tabs,
            selected,
            focused: false,
            disabled: false,
        }
    }

    /// Creates a tabs state with a specific tab selected.
    ///
    /// The index is clamped to the valid range. Returns `None` selection
    /// for empty tabs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TabsState;
    ///
    /// let state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    /// assert_eq!(state.selected_index(), Some(1));
    /// assert_eq!(state.selected(), Some(&"B"));
    /// ```
    pub fn with_selected(tabs: Vec<T>, selected: usize) -> Self {
        let selected = if tabs.is_empty() {
            None
        } else {
            Some(selected.min(tabs.len() - 1))
        };
        Self {
            tabs,
            selected,
            focused: false,
            disabled: false,
        }
    }

    /// Returns the currently selected index.
    ///
    /// Returns `None` if there are no tabs.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Returns the currently selected tab.
    ///
    /// Returns `None` if there are no tabs or no selection.
    pub fn selected(&self) -> Option<&T> {
        self.tabs.get(self.selected?)
    }

    /// Sets the selected tab by index.
    ///
    /// The index is clamped to the valid range. Has no effect on empty tabs.
    pub fn set_selected(&mut self, index: usize) {
        if self.tabs.is_empty() {
            self.selected = None;
        } else {
            self.selected = Some(index.min(self.tabs.len() - 1));
        }
    }

    /// Returns all tabs.
    pub fn tabs(&self) -> &[T] {
        &self.tabs
    }

    /// Returns the number of tabs.
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Returns true if there are no tabs.
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Move selection to the left (previous tab).
    ///
    /// Returns true if the selection changed.
    fn move_left(&mut self) -> bool {
        match self.selected {
            Some(idx) if idx > 0 => {
                self.selected = Some(idx - 1);
                true
            }
            _ => false,
        }
    }

    /// Move selection to the right (next tab).
    ///
    /// Returns true if the selection changed.
    fn move_right(&mut self) -> bool {
        match self.selected {
            Some(idx) if idx < self.tabs.len().saturating_sub(1) => {
                self.selected = Some(idx + 1);
                true
            }
            _ => false,
        }
    }
}

impl<T: Clone + std::fmt::Display + 'static> TabsState<T> {
    /// Returns true if the tabs component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Maps an input event to a tabs message.
    pub fn handle_event(&self, event: &Event) -> Option<TabsMessage> {
        Tabs::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<TabsOutput<T>> {
        Tabs::dispatch_event(self, event)
    }

    /// Updates the tabs state with a message, returning any output.
    pub fn update(&mut self, msg: TabsMessage) -> Option<TabsOutput<T>> {
        Tabs::update(self, msg)
    }
}

/// A horizontal tab navigation component.
///
/// `Tabs` provides a horizontal tab bar for switching between different views
/// or panels. Navigation is done with Left/Right keys, and the component
/// supports generic tab types that implement `Clone` and `Display`.
///
/// # Type Parameters
///
/// - `T`: The type of tab labels. Must implement `Clone` and `Display`.
///
/// # Navigation
///
/// - `Left` - Move to the previous tab
/// - `Right` - Move to the next tab
/// - `First` - Jump to the first tab
/// - `Last` - Jump to the last tab
/// - `Select(index)` - Jump to a specific tab
/// - `Confirm` - Confirm the current selection
///
/// # Output
///
/// - `SelectionChanged(usize)` - Emitted when navigation changes the selection (contains the new index)
/// - `Confirmed(T)` - Emitted when the user confirms their selection
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, Focusable, Tabs, TabsState, TabsMessage, TabsOutput};
///
/// // Using with string slices
/// let mut state = TabsState::new(vec!["Home", "Settings", "Help"]);
///
/// // Using with an enum
/// #[derive(Clone, Debug, PartialEq)]
/// enum Page { Home, Settings, Help }
///
/// impl std::fmt::Display for Page {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             Page::Home => write!(f, "Home"),
///             Page::Settings => write!(f, "Settings"),
///             Page::Help => write!(f, "Help"),
///         }
///     }
/// }
///
/// let mut state = TabsState::new(vec![Page::Home, Page::Settings, Page::Help]);
/// let output = Tabs::<Page>::update(&mut state, TabsMessage::Right);
/// assert_eq!(output, Some(TabsOutput::SelectionChanged(1)));
/// ```
pub struct Tabs<T: Clone>(PhantomData<T>);

impl<T: Clone + Display + 'static> Component for Tabs<T> {
    type State = TabsState<T>;
    type Message = TabsMessage;
    type Output = TabsOutput<T>;

    fn init() -> Self::State {
        TabsState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled || state.tabs.is_empty() {
            return None;
        }

        let selected = state.selected?;

        match msg {
            TabsMessage::Left => {
                if state.move_left() {
                    state.selected.map(TabsOutput::SelectionChanged)
                } else {
                    None
                }
            }
            TabsMessage::Right => {
                if state.move_right() {
                    state.selected.map(TabsOutput::SelectionChanged)
                } else {
                    None
                }
            }
            TabsMessage::Select(index) => {
                let clamped = index.min(state.tabs.len().saturating_sub(1));
                if clamped != selected {
                    state.selected = Some(clamped);
                    Some(TabsOutput::SelectionChanged(clamped))
                } else {
                    None
                }
            }
            TabsMessage::First => {
                if selected != 0 {
                    state.selected = Some(0);
                    Some(TabsOutput::SelectionChanged(0))
                } else {
                    None
                }
            }
            TabsMessage::Last => {
                let last = state.tabs.len().saturating_sub(1);
                if selected != last {
                    state.selected = Some(last);
                    Some(TabsOutput::SelectionChanged(last))
                } else {
                    None
                }
            }
            TabsMessage::Confirm => state.selected().cloned().map(TabsOutput::Confirmed),
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => Some(TabsMessage::Left),
                KeyCode::Right | KeyCode::Char('l') => Some(TabsMessage::Right),
                KeyCode::Home => Some(TabsMessage::First),
                KeyCode::End => Some(TabsMessage::Last),
                KeyCode::Enter => Some(TabsMessage::Confirm),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let selected_idx = state.selected.unwrap_or(0);

        let titles: Vec<Line> = state
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let style = if state.disabled {
                    theme.disabled_style()
                } else if i == selected_idx {
                    theme.selected_style(state.focused)
                } else {
                    theme.normal_style()
                };
                Line::from(Span::styled(format!(" {} ", tab), style))
            })
            .collect();

        let border_style = if state.focused && !state.disabled {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let highlight_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.selected_style(state.focused)
        };

        let tabs_widget = ratatui::widgets::Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .select(selected_idx)
            .highlight_style(highlight_style);

        frame.render_widget(tabs_widget, area);
    }
}

impl<T: Clone + Display + 'static> Focusable for Tabs<T> {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
