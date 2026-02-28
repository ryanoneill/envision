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
//! let output = Menu::update(&mut state, MenuMessage::Activate);
//! assert_eq!(output, Some(MenuOutput::ItemActivated(0)));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, Focusable};
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
    /// Move to the next menu item.
    SelectNext,
    /// Move to the previous menu item.
    SelectPrevious,
    /// Activate the currently selected item.
    Activate,
    /// Select a specific item by index.
    SelectItem(usize),
}

/// Output messages from a Menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuOutput {
    /// A menu item was activated (contains the item index).
    ItemActivated(usize),
}

/// State for a Menu component.
#[derive(Clone, Debug, Default)]
pub struct MenuState {
    /// Menu items.
    items: Vec<MenuItem>,
    /// Currently selected item index.
    selected_index: usize,
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
        Self {
            items,
            selected_index: 0,
            focused: false,
        }
    }

    /// Returns the menu items.
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// Sets the menu items.
    ///
    /// Resets selection to 0 if the current selection is out of bounds.
    pub fn set_items(&mut self, items: Vec<MenuItem>) {
        self.items = items;
        if self.selected_index >= self.items.len() && !self.items.is_empty() {
            self.selected_index = 0;
        }
    }

    /// Adds a menu item.
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// Returns the currently selected item index.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Sets the selected item index.
    ///
    /// If the index is out of bounds, it will be clamped to the valid range.
    pub fn set_selected_index(&mut self, index: usize) {
        if !self.items.is_empty() {
            self.selected_index = index.min(self.items.len() - 1);
        }
    }

    /// Returns the currently selected item.
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_index)
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
/// - Left arrow to [`MenuMessage::SelectPrevious`]
/// - Right arrow to [`MenuMessage::SelectNext`]
/// - Enter to [`MenuMessage::Activate`]
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
/// Menu::update(&mut state, MenuMessage::SelectNext);
///
/// // Activate
/// let output = Menu::update(&mut state, MenuMessage::Activate);
/// assert_eq!(output, Some(MenuOutput::ItemActivated(1)));
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

        match msg {
            MenuMessage::SelectNext => {
                // Move to next item, wrapping around
                state.selected_index = (state.selected_index + 1) % state.items.len();
                None
            }
            MenuMessage::SelectPrevious => {
                // Move to previous item, wrapping around
                if state.selected_index == 0 {
                    state.selected_index = state.items.len() - 1;
                } else {
                    state.selected_index -= 1;
                }
                None
            }
            MenuMessage::Activate => {
                // Activate only if item is enabled
                if let Some(item) = state.items.get(state.selected_index) {
                    if item.is_enabled() {
                        Some(MenuOutput::ItemActivated(state.selected_index))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            MenuMessage::SelectItem(index) => {
                if index < state.items.len() {
                    state.selected_index = index;
                }
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let mut menu_text = String::new();

        for (idx, item) in state.items.iter().enumerate() {
            if idx > 0 {
                menu_text.push_str("  ");
            }

            let item_text = if idx == state.selected_index && state.focused {
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
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_new() {
        let item = MenuItem::new("File");
        assert_eq!(item.label(), "File");
        assert!(item.is_enabled());
    }

    #[test]
    fn test_menu_item_disabled() {
        let item = MenuItem::disabled("Save");
        assert_eq!(item.label(), "Save");
        assert!(!item.is_enabled());
    }

    #[test]
    fn test_menu_item_set_enabled() {
        let mut item = MenuItem::new("Edit");
        item.set_enabled(false);
        assert!(!item.is_enabled());

        item.set_enabled(true);
        assert!(item.is_enabled());
    }

    #[test]
    fn test_new() {
        let state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
        assert_eq!(state.items().len(), 2);
        assert_eq!(state.selected_index(), 0);
        assert!(!Menu::is_focused(&state));
    }

    #[test]
    fn test_default() {
        let state = MenuState::default();
        assert_eq!(state.items().len(), 0);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_set_items() {
        let mut state = MenuState::new(vec![MenuItem::new("A")]);
        state.set_items(vec![MenuItem::new("X"), MenuItem::new("Y")]);
        assert_eq!(state.items().len(), 2);
        assert_eq!(state.items()[0].label(), "X");
    }

    #[test]
    fn test_set_items_resets_invalid_selection() {
        let mut state = MenuState::new(vec![
            MenuItem::new("A"),
            MenuItem::new("B"),
            MenuItem::new("C"),
        ]);
        state.set_selected_index(2);

        state.set_items(vec![MenuItem::new("X")]);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_add_item() {
        let mut state = MenuState::new(vec![MenuItem::new("File")]);
        state.add_item(MenuItem::new("Edit"));
        assert_eq!(state.items().len(), 2);
    }

    #[test]
    fn test_selected_index() {
        let mut state = MenuState::new(vec![
            MenuItem::new("A"),
            MenuItem::new("B"),
            MenuItem::new("C"),
        ]);

        state.set_selected_index(1);
        assert_eq!(state.selected_index(), 1);

        state.set_selected_index(2);
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_selected_index_clamps() {
        let mut state = MenuState::new(vec![MenuItem::new("A"), MenuItem::new("B")]);

        state.set_selected_index(10);
        assert_eq!(state.selected_index(), 1);
    }

    #[test]
    fn test_selected_item() {
        let state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);

        let item = state.selected_item().unwrap();
        assert_eq!(item.label(), "File");
    }

    #[test]
    fn test_select_next() {
        let mut state = MenuState::new(vec![
            MenuItem::new("A"),
            MenuItem::new("B"),
            MenuItem::new("C"),
        ]);

        Menu::update(&mut state, MenuMessage::SelectNext);
        assert_eq!(state.selected_index(), 1);

        Menu::update(&mut state, MenuMessage::SelectNext);
        assert_eq!(state.selected_index(), 2);

        // Wrap around
        Menu::update(&mut state, MenuMessage::SelectNext);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_select_previous() {
        let mut state = MenuState::new(vec![
            MenuItem::new("A"),
            MenuItem::new("B"),
            MenuItem::new("C"),
        ]);

        // Wrap around from start
        Menu::update(&mut state, MenuMessage::SelectPrevious);
        assert_eq!(state.selected_index(), 2);

        Menu::update(&mut state, MenuMessage::SelectPrevious);
        assert_eq!(state.selected_index(), 1);

        Menu::update(&mut state, MenuMessage::SelectPrevious);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_select_item() {
        let mut state = MenuState::new(vec![
            MenuItem::new("A"),
            MenuItem::new("B"),
            MenuItem::new("C"),
        ]);

        Menu::update(&mut state, MenuMessage::SelectItem(2));
        assert_eq!(state.selected_index(), 2);

        Menu::update(&mut state, MenuMessage::SelectItem(0));
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_select_item_out_of_bounds() {
        let mut state = MenuState::new(vec![MenuItem::new("A"), MenuItem::new("B")]);

        Menu::update(&mut state, MenuMessage::SelectItem(10));
        // Should remain at 0
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_activate_enabled() {
        let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);

        let output = Menu::update(&mut state, MenuMessage::Activate);
        assert_eq!(output, Some(MenuOutput::ItemActivated(0)));
    }

    #[test]
    fn test_activate_disabled() {
        let mut state = MenuState::new(vec![MenuItem::disabled("File"), MenuItem::new("Edit")]);

        let output = Menu::update(&mut state, MenuMessage::Activate);
        assert_eq!(output, None);
    }

    #[test]
    fn test_activate_empty() {
        let mut state = MenuState::new(vec![]);

        let output = Menu::update(&mut state, MenuMessage::Activate);
        assert_eq!(output, None);
    }

    #[test]
    fn test_empty_menu_ignores_navigation() {
        let mut state = MenuState::new(vec![]);

        let output = Menu::update(&mut state, MenuMessage::SelectNext);
        assert_eq!(output, None);

        let output = Menu::update(&mut state, MenuMessage::SelectPrevious);
        assert_eq!(output, None);
    }

    #[test]
    fn test_focusable() {
        let mut state = MenuState::new(vec![MenuItem::new("Test")]);

        assert!(!Menu::is_focused(&state));

        Menu::focus(&mut state);
        assert!(Menu::is_focused(&state));

        Menu::blur(&mut state);
        assert!(!Menu::is_focused(&state));
    }

    #[test]
    fn test_init() {
        let state = Menu::init();
        assert_eq!(state.items().len(), 0);
        assert!(!Menu::is_focused(&state));
    }

    #[test]
    fn test_clone() {
        let state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
        let cloned = state.clone();
        assert_eq!(cloned.items().len(), 2);
    }

    #[test]
    fn test_view() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let state = MenuState::new(vec![
            MenuItem::new("File"),
            MenuItem::new("Edit"),
            MenuItem::new("View"),
        ]);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Menu::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("File"));
        assert!(output.contains("Edit"));
        assert!(output.contains("View"));
    }

    #[test]
    fn test_view_focused() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
        Menu::focus(&mut state);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Menu::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        // Should have brackets around selected item when focused
        assert!(output.contains("[File]"));
    }

    #[test]
    fn test_view_selected() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let mut state = MenuState::new(vec![
            MenuItem::new("File"),
            MenuItem::new("Edit"),
            MenuItem::new("View"),
        ]);
        Menu::focus(&mut state);
        state.set_selected_index(1);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Menu::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("[Edit]"));
    }

    #[test]
    fn test_view_empty() {
        use crate::backend::CaptureBackend;
        use crate::theme::Theme;
        use ratatui::Terminal;

        let state = MenuState::new(vec![]);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Menu::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        // Should not panic with empty menu
        let output = terminal.backend().to_string();
        assert!(!output.is_empty());
    }
}
