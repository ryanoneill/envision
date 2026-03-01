//! A horizontal menu bar component.
//!
//! `Menu` provides a horizontal menu bar for application commands and navigation.
//! It supports keyboard navigation, item activation, and disabled states.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Menu, MenuMessage, MenuOutput, MenuState, MenuItem, Component, Focusable};
//!
//! // Create a menu
//! let mut state = MenuState::new(vec![
//!     MenuItem::new("File"),
//!     MenuItem::new("Edit"),
//!     MenuItem::new("View"),
//! ]);
//!
//! // Focus and activate
//! Menu::focus(&mut state);
//! let output = Menu::update(&mut state, MenuMessage::Select);
//! assert_eq!(output, Some(MenuOutput::Selected(0)));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, Focusable};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// A menu item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    label: String,
    enabled: bool,
}

impl MenuItem {
    /// Returns the item label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns whether the item is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets the item label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Creates a new enabled menu item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MenuItem;
    ///
    /// let item = MenuItem::new("File");
    /// assert_eq!(item.label(), "File");
    /// assert!(item.is_enabled());
    /// ```
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
        }
    }

    /// Creates a new disabled menu item.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::MenuItem;
    ///
    /// let item = MenuItem::disabled("Save");
    /// assert!(!item.is_enabled());
    /// ```
    pub fn disabled(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: false,
        }
    }

    /// Sets whether this item is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Messages that can be sent to a Menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuMessage {
    /// Move to the next (right) menu item.
    Right,
    /// Move to the previous (left) menu item.
    Left,
    /// Select the currently highlighted item.
    Select,
    /// Select a specific item by index.
    SelectIndex(usize),
}

/// Output messages from a Menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuOutput {
    /// A menu item was selected (contains the item index).
    Selected(usize),
    /// The selection changed during navigation (contains the new item index).
    SelectionChanged(usize),
}

/// State for a Menu component.
#[derive(Clone, Debug, Default)]
pub struct MenuState {
    /// Menu items.
    items: Vec<MenuItem>,
    /// Currently selected item index, or `None` if no items.
    selected_index: Option<usize>,
    /// Whether the menu is focused.
    focused: bool,
}

impl MenuState {
    /// Creates a new menu with the given items.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{MenuState, MenuItem};
    ///
    /// let state = MenuState::new(vec![
    ///     MenuItem::new("File"),
    ///     MenuItem::new("Edit"),
    /// ]);
    /// assert_eq!(state.items().len(), 2);
    /// ```
    pub fn new(items: Vec<MenuItem>) -> Self {
        let selected_index = if items.is_empty() { None } else { Some(0) };
        Self {
            items,
            selected_index,
            focused: false,
        }
    }

    /// Returns the menu items.
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// Sets the menu items.
    ///
    /// Resets selection to the first item if the current selection is out of bounds.
    /// Sets selection to `None` if the new items list is empty.
    pub fn set_items(&mut self, items: Vec<MenuItem>) {
        self.items = items;
        if self.items.is_empty() {
            self.selected_index = None;
        } else if self.selected_index.map_or(true, |i| i >= self.items.len()) {
            self.selected_index = Some(0);
        }
    }

    /// Adds a menu item.
    ///
    /// If this is the first item, it becomes selected.
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
        if self.selected_index.is_none() {
            self.selected_index = Some(0);
        }
    }

    /// Removes a menu item by index.
    ///
    /// If the index is out of bounds, this is a no-op.
    /// Adjusts the selection after removal:
    /// - If the removed item was the selected item, selects the previous item
    ///   (or the first if at the beginning).
    /// - If the menu becomes empty, selection becomes `None`.
    pub fn remove_item(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }
        self.items.remove(index);
        if self.items.is_empty() {
            self.selected_index = None;
        } else if let Some(selected) = self.selected_index {
            if selected >= self.items.len() {
                self.selected_index = Some(self.items.len() - 1);
            }
        }
    }

    /// Returns the currently selected item index.
    ///
    /// Returns `None` if the menu is empty.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Sets the selected item index.
    ///
    /// If the index is out of bounds, it will be clamped to the valid range.
    /// Has no effect on an empty menu.
    pub fn set_selected_index(&mut self, index: usize) {
        if !self.items.is_empty() {
            self.selected_index = Some(index.min(self.items.len() - 1));
        }
    }

    /// Returns the currently selected item.
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_index?)
    }

    /// Returns true if the menu is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Maps an input event to a menu message.
    pub fn handle_event(&self, event: &Event) -> Option<MenuMessage> {
        Menu::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<MenuOutput> {
        Menu::dispatch_event(self, event)
    }

    /// Updates the menu state with a message, returning any output.
    pub fn update(&mut self, msg: MenuMessage) -> Option<MenuOutput> {
        Menu::update(self, msg)
    }
}

/// A horizontal menu bar component.
///
/// This component provides a horizontal menu bar for application commands
/// and navigation. Items can be navigated with Left/Right arrows and
/// activated with Enter.
///
/// # Keyboard Navigation
///
/// The menu itself doesn't handle keyboard events directly. Your application
/// should map:
/// - Right arrow to [`MenuMessage::Right`]
/// - Left arrow to [`MenuMessage::Left`]
/// - Enter to [`MenuMessage::Select`]
///
/// # Visual Layout
///
/// ```text
/// File  Edit  View  Help
/// ^^^^
/// └── Selected item (highlighted)
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{Menu, MenuMessage, MenuOutput, MenuState, MenuItem, Component};
///
/// let mut state = MenuState::new(vec![
///     MenuItem::new("New"),
///     MenuItem::new("Open"),
///     MenuItem::disabled("Save"),
/// ]);
///
/// // Navigate
/// Menu::update(&mut state, MenuMessage::Right);
///
/// // Select
/// let output = Menu::update(&mut state, MenuMessage::Select);
/// assert_eq!(output, Some(MenuOutput::Selected(1)));
/// ```
pub struct Menu;

impl Component for Menu {
    type State = MenuState;
    type Message = MenuMessage;
    type Output = MenuOutput;

    fn init() -> Self::State {
        MenuState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.items.is_empty() {
            return None;
        }

        let selected = state.selected_index?;

        match msg {
            MenuMessage::Right => {
                // Move to next item, wrapping around
                let new_index = (selected + 1) % state.items.len();
                state.selected_index = Some(new_index);
                Some(MenuOutput::SelectionChanged(new_index))
            }
            MenuMessage::Left => {
                // Move to previous item, wrapping around
                let new_index = if selected == 0 {
                    state.items.len() - 1
                } else {
                    selected - 1
                };
                state.selected_index = Some(new_index);
                Some(MenuOutput::SelectionChanged(new_index))
            }
            MenuMessage::Select => {
                // Activate only if item is enabled
                if let Some(item) = state.items.get(selected) {
                    if item.is_enabled() {
                        Some(MenuOutput::Selected(selected))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            MenuMessage::SelectIndex(index) => {
                if index < state.items.len() && state.selected_index != Some(index) {
                    state.selected_index = Some(index);
                    Some(MenuOutput::SelectionChanged(index))
                } else {
                    None
                }
            }
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Left => Some(MenuMessage::Left),
                KeyCode::Right => Some(MenuMessage::Right),
                KeyCode::Enter => Some(MenuMessage::Select),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let mut menu_text = String::new();

        for (idx, item) in state.items.iter().enumerate() {
            if idx > 0 {
                menu_text.push_str("  ");
            }

            let item_text = if Some(idx) == state.selected_index && state.focused {
                format!("[{}]", item.label())
            } else {
                item.label().to_string()
            };

            menu_text.push_str(&item_text);
        }

        // Determine style based on state
        let style = if state.focused {
            theme.focused_style()
        } else {
            theme.normal_style()
        };

        let paragraph = Paragraph::new(menu_text).style(style);

        frame.render_widget(paragraph, area);
    }
}

impl Focusable for Menu {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
