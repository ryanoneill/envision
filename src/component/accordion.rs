//! An accordion component with collapsible panels.
//!
//! `Accordion` provides a vertically stacked list of panels that can be
//! expanded or collapsed. Multiple panels can be open simultaneously,
//! and keyboard navigation is supported.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Accordion, AccordionMessage, AccordionOutput, AccordionPanel, AccordionState, Component, Focusable};
//!
//! // Create panels
//! let panels = vec![
//!     AccordionPanel::new("Getting Started", "Welcome to the app..."),
//!     AccordionPanel::new("Configuration", "Set up your preferences..."),
//!     AccordionPanel::new("FAQ", "Frequently asked questions..."),
//! ];
//!
//! let mut state = AccordionState::new(panels);
//! Accordion::focus(&mut state);
//!
//! // Toggle first panel (expands it)
//! let output = Accordion::update(&mut state, AccordionMessage::Toggle);
//! assert_eq!(output, Some(AccordionOutput::Expanded(0)));
//! assert!(state.panels()[0].is_expanded());
//!
//! // Navigate to next panel and toggle
//! Accordion::update(&mut state, AccordionMessage::Next);
//! Accordion::update(&mut state, AccordionMessage::Toggle);
//! // Now panels 0 and 1 are both expanded
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, Focusable};
use crate::theme::Theme;

/// A single accordion panel with a title and content.
///
/// Panels can be created collapsed (default) or expanded using the builder method.
///
/// # Example
///
/// ```rust
/// use envision::component::AccordionPanel;
///
/// // Create a collapsed panel
/// let panel = AccordionPanel::new("Title", "Content here");
/// assert!(!panel.is_expanded());
///
/// // Create an expanded panel
/// let panel = AccordionPanel::new("Title", "Content").expanded();
/// assert!(panel.is_expanded());
/// ```
#[derive(Clone, Debug)]
pub struct AccordionPanel {
    /// The panel header/title.
    title: String,
    /// The panel content.
    content: String,
    /// Whether this panel is expanded.
    expanded: bool,
}

impl AccordionPanel {
    /// Creates a new collapsed panel with the given title and content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AccordionPanel;
    ///
    /// let panel = AccordionPanel::new("Section 1", "Content for section 1");
    /// assert_eq!(panel.title(), "Section 1");
    /// assert_eq!(panel.content(), "Content for section 1");
    /// assert!(!panel.is_expanded());
    /// ```
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            expanded: false,
        }
    }

    /// Sets the panel to be expanded (builder method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AccordionPanel;
    ///
    /// let panel = AccordionPanel::new("Title", "Content").expanded();
    /// assert!(panel.is_expanded());
    /// ```
    pub fn expanded(mut self) -> Self {
        self.expanded = true;
        self
    }

    /// Returns the panel title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the panel content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns whether the panel is expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }
}

/// Messages that can be sent to an Accordion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccordionMessage {
    /// Move focus to the next panel.
    Next,
    /// Move focus to the previous panel.
    Previous,
    /// Jump to the first panel.
    First,
    /// Jump to the last panel.
    Last,
    /// Toggle the currently focused panel.
    Toggle,
    /// Expand the currently focused panel.
    Expand,
    /// Collapse the currently focused panel.
    Collapse,
    /// Toggle a specific panel by index.
    ToggleIndex(usize),
    /// Expand all panels.
    ExpandAll,
    /// Collapse all panels.
    CollapseAll,
}

/// Output messages from an Accordion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccordionOutput {
    /// A panel was expanded (index).
    Expanded(usize),
    /// A panel was collapsed (index).
    Collapsed(usize),
    /// Focus moved to a panel (index).
    FocusChanged(usize),
}

/// State for an Accordion component.
#[derive(Clone, Debug, Default)]
pub struct AccordionState {
    /// The accordion panels.
    panels: Vec<AccordionPanel>,
    /// Currently focused panel index.
    focused_index: usize,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl AccordionState {
    /// Creates a new accordion with the given panels.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{AccordionPanel, AccordionState};
    ///
    /// let panels = vec![
    ///     AccordionPanel::new("Section 1", "Content 1"),
    ///     AccordionPanel::new("Section 2", "Content 2"),
    /// ];
    /// let state = AccordionState::new(panels);
    /// assert_eq!(state.len(), 2);
    /// assert_eq!(state.focused_index(), 0);
    /// ```
    pub fn new(panels: Vec<AccordionPanel>) -> Self {
        Self {
            panels,
            focused_index: 0,
            focused: false,
            disabled: false,
        }
    }

    /// Creates an accordion from title/content pairs.
    ///
    /// All panels start collapsed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::AccordionState;
    ///
    /// let state = AccordionState::from_pairs(vec![
    ///     ("Section 1", "Content 1"),
    ///     ("Section 2", "Content 2"),
    /// ]);
    /// assert_eq!(state.len(), 2);
    /// ```
    pub fn from_pairs<S: Into<String>, T: Into<String>>(pairs: Vec<(S, T)>) -> Self {
        let panels = pairs
            .into_iter()
            .map(|(title, content)| AccordionPanel::new(title, content))
            .collect();
        Self::new(panels)
    }

    /// Returns the panels slice.
    pub fn panels(&self) -> &[AccordionPanel] {
        &self.panels
    }

    /// Returns the number of panels.
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    /// Returns true if there are no panels.
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }

    /// Returns the currently focused panel index.
    pub fn focused_index(&self) -> usize {
        self.focused_index
    }

    /// Returns the currently focused panel.
    pub fn focused_panel(&self) -> Option<&AccordionPanel> {
        self.panels.get(self.focused_index)
    }

    /// Returns whether the accordion is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets new panels, resetting the focused index if needed.
    pub fn set_panels(&mut self, panels: Vec<AccordionPanel>) {
        self.panels = panels;
        if self.focused_index >= self.panels.len() && !self.panels.is_empty() {
            self.focused_index = 0;
        }
    }

    /// Adds a panel to the accordion.
    pub fn add_panel(&mut self, panel: AccordionPanel) {
        self.panels.push(panel);
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Returns the count of expanded panels.
    pub fn expanded_count(&self) -> usize {
        self.panels.iter().filter(|p| p.expanded).count()
    }

    /// Returns true if any panel is expanded.
    pub fn is_any_expanded(&self) -> bool {
        self.panels.iter().any(|p| p.expanded)
    }

    /// Returns true if all panels are expanded.
    pub fn is_all_expanded(&self) -> bool {
        !self.panels.is_empty() && self.panels.iter().all(|p| p.expanded)
    }
}

/// An accordion component with collapsible panels.
///
/// The accordion displays a vertical list of panels. Each panel has a header
/// that can be clicked (or toggled via keyboard) to expand or collapse its
/// content. Multiple panels can be expanded simultaneously.
///
/// # Keyboard Navigation
///
/// The accordion itself doesn't handle keyboard events directly. Your application
/// should map:
/// - Up arrow to [`AccordionMessage::Previous`]
/// - Down arrow to [`AccordionMessage::Next`]
/// - Enter/Space to [`AccordionMessage::Toggle`]
/// - Home to [`AccordionMessage::First`]
/// - End to [`AccordionMessage::Last`]
///
/// # Visual Layout
///
/// ```text
/// ▼ Section 1            ← Focused, expanded
///   Content for section 1...
///   More content here.
/// ▶ Section 2            ← Collapsed
/// ▼ Section 3            ← Expanded
///   Content for section 3...
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{Accordion, AccordionMessage, AccordionPanel, AccordionState, Component};
///
/// let panels = vec![
///     AccordionPanel::new("FAQ", "Frequently asked questions..."),
///     AccordionPanel::new("Help", "How to get help..."),
/// ];
///
/// let mut state = AccordionState::new(panels);
///
/// // Toggle first panel
/// Accordion::update(&mut state, AccordionMessage::Toggle);
/// assert!(state.panels()[0].is_expanded());
///
/// // Navigate and toggle second
/// Accordion::update(&mut state, AccordionMessage::Next);
/// Accordion::update(&mut state, AccordionMessage::Toggle);
/// // Both panels are now expanded
/// assert_eq!(state.expanded_count(), 2);
/// ```
pub struct Accordion;

impl Component for Accordion {
    type State = AccordionState;
    type Message = AccordionMessage;
    type Output = AccordionOutput;

    fn init() -> Self::State {
        AccordionState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            AccordionMessage::Next => {
                if !state.panels.is_empty() {
                    state.focused_index = (state.focused_index + 1) % state.panels.len();
                    Some(AccordionOutput::FocusChanged(state.focused_index))
                } else {
                    None
                }
            }
            AccordionMessage::Previous => {
                if !state.panels.is_empty() {
                    if state.focused_index == 0 {
                        state.focused_index = state.panels.len() - 1;
                    } else {
                        state.focused_index -= 1;
                    }
                    Some(AccordionOutput::FocusChanged(state.focused_index))
                } else {
                    None
                }
            }
            AccordionMessage::First => {
                if !state.panels.is_empty() && state.focused_index != 0 {
                    state.focused_index = 0;
                    Some(AccordionOutput::FocusChanged(0))
                } else {
                    None
                }
            }
            AccordionMessage::Last => {
                if !state.panels.is_empty() {
                    let last = state.panels.len() - 1;
                    if state.focused_index != last {
                        state.focused_index = last;
                        Some(AccordionOutput::FocusChanged(last))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            AccordionMessage::Toggle => {
                if let Some(panel) = state.panels.get_mut(state.focused_index) {
                    panel.expanded = !panel.expanded;
                    if panel.expanded {
                        Some(AccordionOutput::Expanded(state.focused_index))
                    } else {
                        Some(AccordionOutput::Collapsed(state.focused_index))
                    }
                } else {
                    None
                }
            }
            AccordionMessage::Expand => {
                if let Some(panel) = state.panels.get_mut(state.focused_index) {
                    if !panel.expanded {
                        panel.expanded = true;
                        Some(AccordionOutput::Expanded(state.focused_index))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            AccordionMessage::Collapse => {
                if let Some(panel) = state.panels.get_mut(state.focused_index) {
                    if panel.expanded {
                        panel.expanded = false;
                        Some(AccordionOutput::Collapsed(state.focused_index))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            AccordionMessage::ToggleIndex(index) => {
                if let Some(panel) = state.panels.get_mut(index) {
                    panel.expanded = !panel.expanded;
                    if panel.expanded {
                        Some(AccordionOutput::Expanded(index))
                    } else {
                        Some(AccordionOutput::Collapsed(index))
                    }
                } else {
                    None
                }
            }
            AccordionMessage::ExpandAll => {
                let mut any_changed = false;
                for (i, panel) in state.panels.iter_mut().enumerate() {
                    if !panel.expanded {
                        panel.expanded = true;
                        any_changed = true;
                        // Return the first one that was expanded
                        if !any_changed {
                            return Some(AccordionOutput::Expanded(i));
                        }
                    }
                }
                if any_changed {
                    // Return a general expanded signal for the first panel
                    Some(AccordionOutput::Expanded(0))
                } else {
                    None
                }
            }
            AccordionMessage::CollapseAll => {
                let mut any_changed = false;
                for (i, panel) in state.panels.iter_mut().enumerate() {
                    if panel.expanded {
                        panel.expanded = false;
                        any_changed = true;
                        if !any_changed {
                            return Some(AccordionOutput::Collapsed(i));
                        }
                    }
                }
                if any_changed {
                    Some(AccordionOutput::Collapsed(0))
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        if state.panels.is_empty() {
            return;
        }

        let mut y = area.y;

        for (i, panel) in state.panels.iter().enumerate() {
            if y >= area.bottom() {
                break;
            }

            // Header line
            let is_focused_panel = state.focused && i == state.focused_index;
            let icon = if panel.expanded { "▼" } else { "▶" };
            let header = format!("{} {}", icon, panel.title);

            let header_style = if state.disabled {
                theme.disabled_style()
            } else if is_focused_panel {
                theme.focused_bold_style()
            } else {
                theme.normal_style()
            };

            let header_area = Rect::new(area.x, y, area.width, 1);
            frame.render_widget(Paragraph::new(header).style(header_style), header_area);
            y += 1;

            // Content (if expanded)
            if panel.expanded && y < area.bottom() {
                let content_lines = panel.content.lines().count().max(1) as u16;
                let available_height = area.bottom().saturating_sub(y);
                let content_height = content_lines.min(available_height);

                if content_height > 0 {
                    let content_area =
                        Rect::new(area.x + 2, y, area.width.saturating_sub(2), content_height);
                    let content_style = if state.disabled {
                        theme.disabled_style()
                    } else {
                        theme.placeholder_style()
                    };
                    frame.render_widget(
                        Paragraph::new(panel.content.as_str()).style(content_style),
                        content_area,
                    );
                    y += content_height;
                }
            }
        }
    }
}

impl Focusable for Accordion {
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

    // ========== AccordionPanel Tests ==========

    #[test]
    fn test_panel_new() {
        let panel = AccordionPanel::new("Title", "Content");
        assert_eq!(panel.title(), "Title");
        assert_eq!(panel.content(), "Content");
        assert!(!panel.is_expanded());
    }

    #[test]
    fn test_panel_expanded_builder() {
        let panel = AccordionPanel::new("Title", "Content").expanded();
        assert!(panel.is_expanded());
    }

    #[test]
    fn test_panel_accessors() {
        let panel = AccordionPanel::new("My Title", "My Content");
        assert_eq!(panel.title(), "My Title");
        assert_eq!(panel.content(), "My Content");
        assert!(!panel.is_expanded());
    }

    #[test]
    fn test_panel_clone() {
        let panel = AccordionPanel::new("Title", "Content").expanded();
        let cloned = panel.clone();
        assert_eq!(cloned.title(), "Title");
        assert!(cloned.is_expanded());
    }

    // ========== State Creation Tests ==========

    #[test]
    fn test_new() {
        let panels = vec![
            AccordionPanel::new("A", "Content A"),
            AccordionPanel::new("B", "Content B"),
        ];
        let state = AccordionState::new(panels);
        assert_eq!(state.len(), 2);
        assert_eq!(state.focused_index(), 0);
        assert!(!state.is_disabled());
    }

    #[test]
    fn test_from_pairs() {
        let state = AccordionState::from_pairs(vec![("A", "Content A"), ("B", "Content B")]);
        assert_eq!(state.len(), 2);
        assert_eq!(state.panels()[0].title(), "A");
        assert_eq!(state.panels()[1].content(), "Content B");
    }

    #[test]
    fn test_default() {
        let state = AccordionState::default();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_new_empty() {
        let state = AccordionState::new(Vec::new());
        assert!(state.is_empty());
        assert_eq!(state.focused_index(), 0);
    }

    // ========== Accessor Tests ==========

    #[test]
    fn test_panels() {
        let state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
        assert_eq!(state.panels().len(), 2);
        assert_eq!(state.panels()[0].title(), "A");
    }

    #[test]
    fn test_len() {
        let state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
        assert_eq!(state.len(), 3);
    }

    #[test]
    fn test_is_empty() {
        let empty = AccordionState::default();
        assert!(empty.is_empty());

        let not_empty = AccordionState::from_pairs(vec![("A", "1")]);
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_focused_index() {
        let state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_focused_panel() {
        let state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
        assert_eq!(state.focused_panel().unwrap().title(), "A");

        let empty = AccordionState::default();
        assert!(empty.focused_panel().is_none());
    }

    #[test]
    fn test_is_disabled() {
        let mut state = AccordionState::default();
        assert!(!state.is_disabled());
        state.set_disabled(true);
        assert!(state.is_disabled());
    }

    // ========== Mutator Tests ==========

    #[test]
    fn test_set_panels() {
        let mut state = AccordionState::from_pairs(vec![("A", "1")]);
        state.set_panels(vec![
            AccordionPanel::new("X", "10"),
            AccordionPanel::new("Y", "20"),
        ]);
        assert_eq!(state.len(), 2);
        assert_eq!(state.panels()[0].title(), "X");
    }

    #[test]
    fn test_add_panel() {
        let mut state = AccordionState::from_pairs(vec![("A", "1")]);
        state.add_panel(AccordionPanel::new("B", "2"));
        assert_eq!(state.len(), 2);
        assert_eq!(state.panels()[1].title(), "B");
    }

    #[test]
    fn test_set_disabled() {
        let mut state = AccordionState::default();
        state.set_disabled(true);
        assert!(state.is_disabled());
        state.set_disabled(false);
        assert!(!state.is_disabled());
    }

    // ========== Query Method Tests ==========

    #[test]
    fn test_expanded_count() {
        let panels = vec![
            AccordionPanel::new("A", "1").expanded(),
            AccordionPanel::new("B", "2"),
            AccordionPanel::new("C", "3").expanded(),
        ];
        let state = AccordionState::new(panels);
        assert_eq!(state.expanded_count(), 2);
    }

    #[test]
    fn test_is_any_expanded() {
        let none_expanded = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
        assert!(!none_expanded.is_any_expanded());

        let some_expanded = AccordionState::new(vec![
            AccordionPanel::new("A", "1"),
            AccordionPanel::new("B", "2").expanded(),
        ]);
        assert!(some_expanded.is_any_expanded());
    }

    #[test]
    fn test_is_all_expanded() {
        let all_expanded = AccordionState::new(vec![
            AccordionPanel::new("A", "1").expanded(),
            AccordionPanel::new("B", "2").expanded(),
        ]);
        assert!(all_expanded.is_all_expanded());

        let partial = AccordionState::new(vec![
            AccordionPanel::new("A", "1").expanded(),
            AccordionPanel::new("B", "2"),
        ]);
        assert!(!partial.is_all_expanded());

        let empty = AccordionState::default();
        assert!(!empty.is_all_expanded());
    }

    // ========== Navigation Tests ==========

    #[test]
    fn test_next() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
        assert_eq!(state.focused_index(), 0);

        Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(state.focused_index(), 1);

        Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(state.focused_index(), 2);
    }

    #[test]
    fn test_previous() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
        Accordion::update(&mut state, AccordionMessage::Next);
        Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(state.focused_index(), 2);

        Accordion::update(&mut state, AccordionMessage::Previous);
        assert_eq!(state.focused_index(), 1);

        Accordion::update(&mut state, AccordionMessage::Previous);
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_next_wraps() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
        Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(state.focused_index(), 1);

        Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(state.focused_index(), 0); // Wrapped
    }

    #[test]
    fn test_previous_wraps() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
        assert_eq!(state.focused_index(), 0);

        Accordion::update(&mut state, AccordionMessage::Previous);
        assert_eq!(state.focused_index(), 1); // Wrapped to end
    }

    #[test]
    fn test_first() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
        Accordion::update(&mut state, AccordionMessage::Next);
        Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(state.focused_index(), 2);

        let output = Accordion::update(&mut state, AccordionMessage::First);
        assert_eq!(state.focused_index(), 0);
        assert_eq!(output, Some(AccordionOutput::FocusChanged(0)));
    }

    #[test]
    fn test_last() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
        assert_eq!(state.focused_index(), 0);

        let output = Accordion::update(&mut state, AccordionMessage::Last);
        assert_eq!(state.focused_index(), 2);
        assert_eq!(output, Some(AccordionOutput::FocusChanged(2)));
    }

    #[test]
    fn test_navigation_empty() {
        let mut state = AccordionState::default();

        let output = Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(output, None);

        let output = Accordion::update(&mut state, AccordionMessage::Previous);
        assert_eq!(output, None);

        let output = Accordion::update(&mut state, AccordionMessage::First);
        assert_eq!(output, None);

        let output = Accordion::update(&mut state, AccordionMessage::Last);
        assert_eq!(output, None);
    }

    #[test]
    fn test_navigation_returns_focus_changed() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);

        let output = Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(output, Some(AccordionOutput::FocusChanged(1)));
    }

    // ========== Toggle/Expand/Collapse Tests ==========

    #[test]
    fn test_toggle() {
        let mut state = AccordionState::from_pairs(vec![("A", "1")]);
        assert!(!state.panels()[0].is_expanded());

        Accordion::update(&mut state, AccordionMessage::Toggle);
        assert!(state.panels()[0].is_expanded());

        Accordion::update(&mut state, AccordionMessage::Toggle);
        assert!(!state.panels()[0].is_expanded());
    }

    #[test]
    fn test_toggle_returns_expanded() {
        let mut state = AccordionState::from_pairs(vec![("A", "1")]);
        let output = Accordion::update(&mut state, AccordionMessage::Toggle);
        assert_eq!(output, Some(AccordionOutput::Expanded(0)));
    }

    #[test]
    fn test_toggle_returns_collapsed() {
        let mut state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
        let output = Accordion::update(&mut state, AccordionMessage::Toggle);
        assert_eq!(output, Some(AccordionOutput::Collapsed(0)));
    }

    #[test]
    fn test_expand() {
        let mut state = AccordionState::from_pairs(vec![("A", "1")]);
        let output = Accordion::update(&mut state, AccordionMessage::Expand);
        assert_eq!(output, Some(AccordionOutput::Expanded(0)));
        assert!(state.panels()[0].is_expanded());
    }

    #[test]
    fn test_expand_already_expanded() {
        let mut state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
        let output = Accordion::update(&mut state, AccordionMessage::Expand);
        assert_eq!(output, None);
    }

    #[test]
    fn test_collapse() {
        let mut state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
        let output = Accordion::update(&mut state, AccordionMessage::Collapse);
        assert_eq!(output, Some(AccordionOutput::Collapsed(0)));
        assert!(!state.panels()[0].is_expanded());
    }

    #[test]
    fn test_collapse_already_collapsed() {
        let mut state = AccordionState::from_pairs(vec![("A", "1")]);
        let output = Accordion::update(&mut state, AccordionMessage::Collapse);
        assert_eq!(output, None);
    }

    #[test]
    fn test_toggle_index() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);

        let output = Accordion::update(&mut state, AccordionMessage::ToggleIndex(1));
        assert_eq!(output, Some(AccordionOutput::Expanded(1)));
        assert!(state.panels()[1].is_expanded());

        let output = Accordion::update(&mut state, AccordionMessage::ToggleIndex(1));
        assert_eq!(output, Some(AccordionOutput::Collapsed(1)));
        assert!(!state.panels()[1].is_expanded());
    }

    #[test]
    fn test_toggle_index_out_of_bounds() {
        let mut state = AccordionState::from_pairs(vec![("A", "1")]);
        let output = Accordion::update(&mut state, AccordionMessage::ToggleIndex(5));
        assert_eq!(output, None);
    }

    // ========== Bulk Operations Tests ==========

    #[test]
    fn test_expand_all() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2"), ("C", "3")]);
        assert_eq!(state.expanded_count(), 0);

        let output = Accordion::update(&mut state, AccordionMessage::ExpandAll);
        assert!(output.is_some());
        assert_eq!(state.expanded_count(), 3);
        assert!(state.is_all_expanded());
    }

    #[test]
    fn test_collapse_all() {
        let mut state = AccordionState::new(vec![
            AccordionPanel::new("A", "1").expanded(),
            AccordionPanel::new("B", "2").expanded(),
        ]);
        assert_eq!(state.expanded_count(), 2);

        let output = Accordion::update(&mut state, AccordionMessage::CollapseAll);
        assert!(output.is_some());
        assert_eq!(state.expanded_count(), 0);
        assert!(!state.is_any_expanded());
    }

    #[test]
    fn test_expand_all_already_expanded() {
        let mut state = AccordionState::new(vec![
            AccordionPanel::new("A", "1").expanded(),
            AccordionPanel::new("B", "2").expanded(),
        ]);
        let output = Accordion::update(&mut state, AccordionMessage::ExpandAll);
        assert_eq!(output, None);
    }

    #[test]
    fn test_collapse_all_already_collapsed() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
        let output = Accordion::update(&mut state, AccordionMessage::CollapseAll);
        assert_eq!(output, None);
    }

    // ========== Disabled State Tests ==========

    #[test]
    fn test_disabled_ignores_messages() {
        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
        state.set_disabled(true);

        let output = Accordion::update(&mut state, AccordionMessage::Toggle);
        assert_eq!(output, None);
        assert!(!state.panels()[0].is_expanded());

        let output = Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(output, None);
        assert_eq!(state.focused_index(), 0);
    }

    #[test]
    fn test_disabling_preserves_state() {
        let mut state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
        assert!(state.panels()[0].is_expanded());

        state.set_disabled(true);
        assert!(state.panels()[0].is_expanded()); // Still expanded
    }

    // ========== Focus Tests ==========

    #[test]
    fn test_focusable_is_focused() {
        let state = AccordionState::default();
        assert!(!Accordion::is_focused(&state));
    }

    #[test]
    fn test_focusable_set_focused() {
        let mut state = AccordionState::default();
        Accordion::set_focused(&mut state, true);
        assert!(Accordion::is_focused(&state));
    }

    #[test]
    fn test_focus_blur() {
        let mut state = AccordionState::default();

        Accordion::focus(&mut state);
        assert!(Accordion::is_focused(&state));

        Accordion::blur(&mut state);
        assert!(!Accordion::is_focused(&state));
    }

    // ========== View Tests ==========

    #[test]
    fn test_view_empty() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = AccordionState::default();
        let theme = Theme::default();

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Accordion::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        // Should render without error
        let _ = terminal.backend().to_string();
    }

    #[test]
    fn test_view_collapsed() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = AccordionState::from_pairs(vec![("Section 1", "Content 1")]);
        let theme = Theme::default();

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Accordion::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("▶")); // Collapsed indicator
        assert!(output.contains("Section 1"));
    }

    #[test]
    fn test_view_expanded() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = AccordionState::new(vec![
            AccordionPanel::new("Section 1", "Content 1").expanded()
        ]);
        let theme = Theme::default();

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Accordion::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("▼")); // Expanded indicator
        assert!(output.contains("Section 1"));
        assert!(output.contains("Content 1"));
    }

    #[test]
    fn test_view_mixed() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = AccordionState::new(vec![
            AccordionPanel::new("Expanded", "Expanded content").expanded(),
            AccordionPanel::new("Collapsed", "Collapsed content"),
        ]);
        let theme = Theme::default();

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Accordion::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Expanded"));
        assert!(output.contains("Collapsed"));
        assert!(output.contains("Expanded content"));
    }

    #[test]
    fn test_view_focused_highlight() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = AccordionState::from_pairs(vec![("A", "1"), ("B", "2")]);
        Accordion::focus(&mut state);
        let theme = Theme::default();

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Accordion::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        // Should render without error (we can't easily check color in text)
        let output = terminal.backend().to_string();
        assert!(output.contains("A"));
    }

    #[test]
    fn test_view_long_content() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = AccordionState::new(vec![AccordionPanel::new(
            "Multi-line",
            "Line 1\nLine 2\nLine 3",
        )
        .expanded()]);
        let theme = Theme::default();

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Accordion::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Multi-line"));
        assert!(output.contains("Line 1"));
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_clone() {
        let state = AccordionState::new(vec![AccordionPanel::new("A", "1").expanded()]);
        let cloned = state.clone();
        assert_eq!(cloned.len(), 1);
        assert!(cloned.panels()[0].is_expanded());
    }

    #[test]
    fn test_init() {
        let state = Accordion::init();
        assert!(state.is_empty());
        assert!(!Accordion::is_focused(&state));
    }

    #[test]
    fn test_full_workflow() {
        let mut state = AccordionState::from_pairs(vec![
            ("Section 1", "Content 1"),
            ("Section 2", "Content 2"),
            ("Section 3", "Content 3"),
        ]);
        Accordion::focus(&mut state);

        // Initially no panels expanded
        assert_eq!(state.expanded_count(), 0);

        // Toggle first panel
        let output = Accordion::update(&mut state, AccordionMessage::Toggle);
        assert_eq!(output, Some(AccordionOutput::Expanded(0)));
        assert_eq!(state.expanded_count(), 1);

        // Navigate to next and toggle
        Accordion::update(&mut state, AccordionMessage::Next);
        assert_eq!(state.focused_index(), 1);
        Accordion::update(&mut state, AccordionMessage::Toggle);
        assert_eq!(state.expanded_count(), 2);

        // Both panels 0 and 1 are expanded (multi-expand)
        assert!(state.panels()[0].is_expanded());
        assert!(state.panels()[1].is_expanded());
        assert!(!state.panels()[2].is_expanded());

        // Collapse all
        Accordion::update(&mut state, AccordionMessage::CollapseAll);
        assert_eq!(state.expanded_count(), 0);

        // Expand all
        Accordion::update(&mut state, AccordionMessage::ExpandAll);
        assert!(state.is_all_expanded());
    }
}
