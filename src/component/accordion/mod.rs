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
mod tests;
