//! A horizontal tab navigation component.
//!
//! `Tabs` provides a horizontal tab bar for switching between views or panels.
//! It supports keyboard navigation with Left/Right keys and generic tab types.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Focusable, Tabs, TabsState, TabMessage, TabOutput};
//!
//! // Create tabs with string labels
//! let mut state = TabsState::new(vec!["Home", "Settings", "Help"]);
//! Tabs::set_focused(&mut state, true);
//!
//! assert_eq!(state.selected_index(), 0);
//! assert_eq!(state.selected(), Some(&"Home"));
//!
//! // Navigate right
//! let output = Tabs::<&str>::update(&mut state, TabMessage::Right);
//! assert_eq!(output, Some(TabOutput::Selected("Settings")));
//! assert_eq!(state.selected_index(), 1);
//! ```

use std::fmt::Display;
use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, Focusable};
use crate::theme::Theme;

/// Messages that can be sent to a Tabs component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabMessage {
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
pub enum TabOutput<T: Clone> {
    /// The selected tab changed.
    Selected(T),
    /// The current selection was confirmed.
    Confirmed(T),
}

/// State for a Tabs component.
///
/// The state tracks the available tabs, the currently selected tab,
/// and focus/disabled states.
#[derive(Clone, Debug)]
pub struct TabsState<T: Clone> {
    /// The available tabs.
    tabs: Vec<T>,
    /// Currently selected tab index.
    selected: usize,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl<T: Clone> Default for TabsState<T> {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            selected: 0,
            focused: false,
            disabled: false,
        }
    }
}

impl<T: Clone> TabsState<T> {
    /// Creates a new tabs state with the first tab selected.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TabsState;
    ///
    /// let state = TabsState::new(vec!["Tab1", "Tab2", "Tab3"]);
    /// assert_eq!(state.selected_index(), 0);
    /// assert_eq!(state.len(), 3);
    /// ```
    pub fn new(tabs: Vec<T>) -> Self {
        Self {
            tabs,
            selected: 0,
            focused: false,
            disabled: false,
        }
    }

    /// Creates a tabs state with a specific tab selected.
    ///
    /// The index is clamped to the valid range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TabsState;
    ///
    /// let state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    /// assert_eq!(state.selected_index(), 1);
    /// assert_eq!(state.selected(), Some(&"B"));
    /// ```
    pub fn with_selected(tabs: Vec<T>, selected: usize) -> Self {
        let clamped = if tabs.is_empty() {
            0
        } else {
            selected.min(tabs.len() - 1)
        };
        Self {
            tabs,
            selected: clamped,
            focused: false,
            disabled: false,
        }
    }

    /// Returns the currently selected index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Returns the currently selected tab.
    ///
    /// Returns `None` if there are no tabs.
    pub fn selected(&self) -> Option<&T> {
        self.tabs.get(self.selected)
    }

    /// Sets the selected tab by index.
    ///
    /// The index is clamped to the valid range.
    pub fn set_selected(&mut self, index: usize) {
        if self.tabs.is_empty() {
            self.selected = 0;
        } else {
            self.selected = index.min(self.tabs.len() - 1);
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
        if self.selected > 0 {
            self.selected -= 1;
            true
        } else {
            false
        }
    }

    /// Move selection to the right (next tab).
    ///
    /// Returns true if the selection changed.
    fn move_right(&mut self) -> bool {
        if self.selected < self.tabs.len().saturating_sub(1) {
            self.selected += 1;
            true
        } else {
            false
        }
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
/// - `Selected(T)` - Emitted when the selection changes
/// - `Confirmed(T)` - Emitted when the user confirms their selection
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, Focusable, Tabs, TabsState, TabMessage, TabOutput};
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
/// let output = Tabs::<Page>::update(&mut state, TabMessage::Right);
/// assert_eq!(output, Some(TabOutput::Selected(Page::Settings)));
/// ```
pub struct Tabs<T: Clone>(PhantomData<T>);

impl<T: Clone + Display + 'static> Component for Tabs<T> {
    type State = TabsState<T>;
    type Message = TabMessage;
    type Output = TabOutput<T>;

    fn init() -> Self::State {
        TabsState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled || state.tabs.is_empty() {
            return None;
        }

        match msg {
            TabMessage::Left => {
                if state.move_left() {
                    state.selected().cloned().map(TabOutput::Selected)
                } else {
                    None
                }
            }
            TabMessage::Right => {
                if state.move_right() {
                    state.selected().cloned().map(TabOutput::Selected)
                } else {
                    None
                }
            }
            TabMessage::Select(index) => {
                let clamped = index.min(state.tabs.len().saturating_sub(1));
                if clamped != state.selected {
                    state.selected = clamped;
                    state.selected().cloned().map(TabOutput::Selected)
                } else {
                    None
                }
            }
            TabMessage::First => {
                if state.selected != 0 {
                    state.selected = 0;
                    state.selected().cloned().map(TabOutput::Selected)
                } else {
                    None
                }
            }
            TabMessage::Last => {
                let last = state.tabs.len().saturating_sub(1);
                if state.selected != last {
                    state.selected = last;
                    state.selected().cloned().map(TabOutput::Selected)
                } else {
                    None
                }
            }
            TabMessage::Confirm => state.selected().cloned().map(TabOutput::Confirmed),
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let titles: Vec<Line> = state
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let style = if state.disabled {
                    theme.disabled_style()
                } else if i == state.selected {
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
            .select(state.selected)
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
mod tests {
    use super::*;

    // State Tests

    #[test]
    fn test_new() {
        let state = TabsState::new(vec!["Tab1", "Tab2", "Tab3"]);
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.len(), 3);
        assert!(!state.is_empty());
    }

    #[test]
    fn test_new_empty() {
        let state: TabsState<&str> = TabsState::new(vec![]);
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.len(), 0);
        assert!(state.is_empty());
    }

    #[test]
    fn test_default() {
        let state: TabsState<String> = TabsState::default();
        assert!(state.is_empty());
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_with_selected() {
        let state = TabsState::with_selected(vec!["A", "B", "C"], 1);
        assert_eq!(state.selected_index(), 1);
        assert_eq!(state.selected(), Some(&"B"));
    }

    #[test]
    fn test_with_selected_clamps() {
        let state = TabsState::with_selected(vec!["A", "B", "C"], 10);
        assert_eq!(state.selected_index(), 2); // Clamped to last valid index
    }

    #[test]
    fn test_with_selected_empty() {
        let state: TabsState<&str> = TabsState::with_selected(vec![], 5);
        assert_eq!(state.selected_index(), 0);
    }

    // Accessors

    #[test]
    fn test_selected_index() {
        let state = TabsState::with_selected(vec!["A", "B", "C"], 2);
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_selected() {
        let state = TabsState::with_selected(vec!["A", "B", "C"], 1);
        assert_eq!(state.selected(), Some(&"B"));
    }

    #[test]
    fn test_selected_empty() {
        let state: TabsState<&str> = TabsState::new(vec![]);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_set_selected() {
        let mut state = TabsState::new(vec!["A", "B", "C"]);
        state.set_selected(2);
        assert_eq!(state.selected_index(), 2);

        // Test clamping
        state.set_selected(100);
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_tabs() {
        let state = TabsState::new(vec!["A", "B", "C"]);
        assert_eq!(state.tabs(), &["A", "B", "C"]);
    }

    #[test]
    fn test_len() {
        let state = TabsState::new(vec!["A", "B", "C"]);
        assert_eq!(state.len(), 3);
    }

    #[test]
    fn test_is_empty() {
        let empty: TabsState<&str> = TabsState::new(vec![]);
        assert!(empty.is_empty());

        let not_empty = TabsState::new(vec!["A"]);
        assert!(!not_empty.is_empty());
    }

    // Navigation

    #[test]
    fn test_left() {
        let mut state = TabsState::with_selected(vec!["A", "B", "C"], 1);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Left);
        assert_eq!(output, Some(TabOutput::Selected("A")));
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_left_at_first() {
        let mut state = TabsState::new(vec!["A", "B", "C"]);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Left);
        assert_eq!(output, None);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_right() {
        let mut state = TabsState::new(vec!["A", "B", "C"]);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Right);
        assert_eq!(output, Some(TabOutput::Selected("B")));
        assert_eq!(state.selected_index(), 1);
    }

    #[test]
    fn test_right_at_last() {
        let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Right);
        assert_eq!(output, None);
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_select() {
        let mut state = TabsState::new(vec!["A", "B", "C"]);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Select(2));
        assert_eq!(output, Some(TabOutput::Selected("C")));
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_select_same() {
        let mut state = TabsState::new(vec!["A", "B", "C"]);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Select(0));
        assert_eq!(output, None); // Already selected
    }

    #[test]
    fn test_select_clamps() {
        let mut state = TabsState::new(vec!["A", "B", "C"]);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Select(100));
        assert_eq!(output, Some(TabOutput::Selected("C"))); // Clamped to last
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_first() {
        let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
        let output = Tabs::<&str>::update(&mut state, TabMessage::First);
        assert_eq!(output, Some(TabOutput::Selected("A")));
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_first_already_first() {
        let mut state = TabsState::new(vec!["A", "B", "C"]);
        let output = Tabs::<&str>::update(&mut state, TabMessage::First);
        assert_eq!(output, None);
    }

    #[test]
    fn test_last() {
        let mut state = TabsState::new(vec!["A", "B", "C"]);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Last);
        assert_eq!(output, Some(TabOutput::Selected("C")));
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_last_already_last() {
        let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Last);
        assert_eq!(output, None);
    }

    // Confirm

    #[test]
    fn test_confirm() {
        let mut state = TabsState::with_selected(vec!["A", "B", "C"], 1);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
        assert_eq!(output, Some(TabOutput::Confirmed("B")));
    }

    #[test]
    fn test_confirm_empty() {
        let mut state: TabsState<&str> = TabsState::new(vec![]);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
        assert_eq!(output, None);
    }

    // Disabled State

    #[test]
    fn test_disabled() {
        let mut state = TabsState::new(vec!["A", "B", "C"]);
        state.set_disabled(true);

        // All messages should be ignored
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Right), None);
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Left), None);
        assert_eq!(
            Tabs::<&str>::update(&mut state, TabMessage::Select(2)),
            None
        );
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::First), None);
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Last), None);
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Confirm), None);

        // State should not have changed
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_disabled_accessors() {
        let mut state = TabsState::new(vec!["A", "B"]);
        assert!(!state.is_disabled());

        state.set_disabled(true);
        assert!(state.is_disabled());

        state.set_disabled(false);
        assert!(!state.is_disabled());
    }

    // Empty State

    #[test]
    fn test_empty_navigation() {
        let mut state: TabsState<&str> = TabsState::new(vec![]);

        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Right), None);
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Left), None);
        assert_eq!(
            Tabs::<&str>::update(&mut state, TabMessage::Select(0)),
            None
        );
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::First), None);
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Last), None);
    }

    #[test]
    fn test_empty_confirm() {
        let mut state: TabsState<&str> = TabsState::new(vec![]);
        let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
        assert_eq!(output, None);
    }

    // Focus

    #[test]
    fn test_focusable() {
        let mut state = TabsState::new(vec!["A", "B"]);
        assert!(!Tabs::<&str>::is_focused(&state));

        Tabs::<&str>::set_focused(&mut state, true);
        assert!(Tabs::<&str>::is_focused(&state));

        Tabs::<&str>::blur(&mut state);
        assert!(!Tabs::<&str>::is_focused(&state));

        Tabs::<&str>::focus(&mut state);
        assert!(Tabs::<&str>::is_focused(&state));
    }

    // View Tests

    #[test]
    fn test_view_renders() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = TabsState::new(vec!["Home", "Settings", "Help"]);

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Tabs::<&str>::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Home"));
        assert!(output.contains("Settings"));
        assert!(output.contains("Help"));
    }

    #[test]
    fn test_view_focused() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = TabsState::new(vec!["Tab1", "Tab2"]);
        state.focused = true;

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Tabs::<&str>::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Tab1"));
        assert!(output.contains("Tab2"));
    }

    #[test]
    fn test_view_disabled() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = TabsState::new(vec!["Tab1", "Tab2"]);
        state.disabled = true;

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Tabs::<&str>::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Tab1"));
    }

    #[test]
    fn test_view_empty() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state: TabsState<&str> = TabsState::new(vec![]);

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Tabs::<&str>::view(&state, frame, frame.area(), &Theme::default());
            })
            .unwrap();

        // Should render without panicking
        let _output = terminal.backend().to_string();
    }

    // Integration

    #[test]
    fn test_clone() {
        let mut state = TabsState::with_selected(vec!["A", "B", "C"], 1);
        state.focused = true;
        state.disabled = true;

        let cloned = state.clone();
        assert_eq!(cloned.selected_index(), 1);
        assert!(cloned.focused);
        assert!(cloned.disabled);
        assert_eq!(cloned.tabs(), &["A", "B", "C"]);
    }

    #[test]
    fn test_init() {
        let state: TabsState<String> = Tabs::<String>::init();
        assert!(state.is_empty());
        assert!(!state.focused);
        assert!(!state.disabled);
    }

    #[test]
    fn test_full_workflow() {
        let mut state = TabsState::new(vec!["Home", "Settings", "Profile", "Help"]);
        Tabs::<&str>::set_focused(&mut state, true);

        // Start at first tab
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected(), Some(&"Home"));

        // Navigate right twice
        Tabs::<&str>::update(&mut state, TabMessage::Right);
        Tabs::<&str>::update(&mut state, TabMessage::Right);
        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.selected(), Some(&"Profile"));

        // Navigate left once
        Tabs::<&str>::update(&mut state, TabMessage::Left);
        assert_eq!(state.selected_index(), 1);
        assert_eq!(state.selected(), Some(&"Settings"));

        // Jump to last
        Tabs::<&str>::update(&mut state, TabMessage::Last);
        assert_eq!(state.selected_index(), 3);
        assert_eq!(state.selected(), Some(&"Help"));

        // Jump to first
        Tabs::<&str>::update(&mut state, TabMessage::First);
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected(), Some(&"Home"));

        // Confirm selection
        let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
        assert_eq!(output, Some(TabOutput::Confirmed("Home")));
    }

    #[test]
    fn test_with_enum_tabs() {
        #[derive(Clone, Debug, PartialEq)]
        enum Page {
            Home,
            Settings,
            Help,
        }

        impl std::fmt::Display for Page {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Page::Home => write!(f, "Home"),
                    Page::Settings => write!(f, "Settings"),
                    Page::Help => write!(f, "Help"),
                }
            }
        }

        let mut state = TabsState::new(vec![Page::Home, Page::Settings, Page::Help]);

        let output = Tabs::<Page>::update(&mut state, TabMessage::Right);
        assert_eq!(output, Some(TabOutput::Selected(Page::Settings)));

        let output = Tabs::<Page>::update(&mut state, TabMessage::Confirm);
        assert_eq!(output, Some(TabOutput::Confirmed(Page::Settings)));
    }

    #[test]
    fn test_with_string_tabs() {
        let mut state = TabsState::new(vec![
            "Dashboard".to_string(),
            "Analytics".to_string(),
            "Reports".to_string(),
        ]);

        let output = Tabs::<String>::update(&mut state, TabMessage::Select(1));
        assert_eq!(output, Some(TabOutput::Selected("Analytics".to_string())));
    }

    #[test]
    fn test_single_tab() {
        let mut state = TabsState::new(vec!["Only"]);

        // Can't navigate anywhere
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Left), None);
        assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Right), None);

        // But can confirm
        let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
        assert_eq!(output, Some(TabOutput::Confirmed("Only")));
    }
}
