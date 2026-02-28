//! A searchable dropdown selection component.
//!
//! `Dropdown` provides a filterable dropdown menu for selecting a single option
//! from a list. Users can type to filter options, then navigate and select
//! using keyboard controls.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Dropdown, DropdownMessage, DropdownOutput, DropdownState, Component, Focusable};
//!
//! // Create a dropdown with options
//! let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry", "Date"]);
//! state.set_placeholder("Search fruits...");
//!
//! // Focus and open it
//! Dropdown::focus(&mut state);
//! let _ = Dropdown::update(&mut state, DropdownMessage::Open);
//!
//! // Type to filter (shows Apple, Banana, Date - all contain 'a')
//! let _ = Dropdown::update(&mut state, DropdownMessage::Insert('a'));
//!
//! // Navigate to second filtered option and confirm
//! let _ = Dropdown::update(&mut state, DropdownMessage::SelectNext);
//! let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
//! assert_eq!(output, Some(DropdownOutput::Changed(Some(1)))); // Banana selected
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::{Component, Focusable};
use crate::theme::Theme;

/// Messages that can be sent to a Dropdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropdownMessage {
    /// Open the dropdown.
    Open,
    /// Close the dropdown.
    Close,
    /// Toggle the dropdown open/closed.
    Toggle,
    /// Insert a character into the filter.
    Insert(char),
    /// Delete character before cursor (backspace).
    Backspace,
    /// Clear the filter text.
    ClearFilter,
    /// Move highlight to next filtered option.
    SelectNext,
    /// Move highlight to previous filtered option.
    SelectPrevious,
    /// Confirm current highlighted selection.
    Confirm,
    /// Set the filter text directly.
    SetFilter(String),
}

/// Output messages from a Dropdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropdownOutput {
    /// Selection changed (index in original options list).
    Changed(Option<usize>),
    /// User confirmed selection (index in original options list).
    Submitted(usize),
    /// Filter text changed.
    FilterChanged(String),
}

/// State for a Dropdown component.
#[derive(Clone, Debug)]
pub struct DropdownState {
    /// All available options.
    options: Vec<String>,
    /// Currently selected option index (into options).
    selected_index: Option<usize>,
    /// Current filter/search text.
    filter_text: String,
    /// Indices of options matching the filter.
    filtered_indices: Vec<usize>,
    /// Currently highlighted index (into filtered_indices).
    highlighted_index: usize,
    /// Whether the dropdown is open.
    is_open: bool,
    /// Whether the component is focused.
    focused: bool,
    /// Placeholder text when nothing selected and filter empty.
    placeholder: String,
    /// Whether the dropdown is disabled.
    disabled: bool,
}

impl Default for DropdownState {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            selected_index: None,
            filter_text: String::new(),
            filtered_indices: Vec::new(),
            highlighted_index: 0,
            is_open: false,
            focused: false,
            placeholder: String::from("Search..."),
            disabled: false,
        }
    }
}

impl DropdownState {
    /// Creates a new dropdown with the given options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DropdownState;
    ///
    /// let state = DropdownState::new(vec!["Option 1", "Option 2", "Option 3"]);
    /// assert_eq!(state.options().len(), 3);
    /// assert!(state.selected_index().is_none());
    /// ```
    pub fn new<S: Into<String>>(options: Vec<S>) -> Self {
        let options: Vec<String> = options.into_iter().map(|s| s.into()).collect();
        let filtered_indices: Vec<usize> = (0..options.len()).collect();

        Self {
            options,
            filtered_indices,
            ..Default::default()
        }
    }

    /// Creates a new dropdown with the given options and a pre-selected index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DropdownState;
    ///
    /// let state = DropdownState::with_selection(vec!["A", "B", "C"], 1);
    /// assert_eq!(state.selected_index(), Some(1));
    /// assert_eq!(state.selected_value(), Some("B"));
    /// ```
    pub fn with_selection<S: Into<String>>(options: Vec<S>, selected: usize) -> Self {
        let options: Vec<String> = options.into_iter().map(|s| s.into()).collect();
        let selected_index = if selected < options.len() {
            Some(selected)
        } else {
            None
        };
        let filtered_indices: Vec<usize> = (0..options.len()).collect();

        Self {
            options,
            selected_index,
            filtered_indices,
            ..Default::default()
        }
    }

    /// Returns the options list.
    pub fn options(&self) -> &[String] {
        &self.options
    }

    /// Sets the options list.
    ///
    /// Resets selection if the current selected index is out of bounds.
    /// Also updates the filtered indices.
    pub fn set_options<S: Into<String>>(&mut self, options: Vec<S>) {
        self.options = options.into_iter().map(|s| s.into()).collect();

        // Reset selection if out of bounds
        if let Some(idx) = self.selected_index {
            if idx >= self.options.len() {
                self.selected_index = None;
            }
        }

        // Re-filter with current filter text
        self.update_filter();
    }

    /// Returns the selected option index.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Returns the selected option value.
    pub fn selected_value(&self) -> Option<&str> {
        self.selected_index
            .and_then(|idx| self.options.get(idx).map(|s| s.as_str()))
    }

    /// Sets the selected option index.
    pub fn set_selected_index(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.options.len() {
                self.selected_index = Some(idx);
            }
        } else {
            self.selected_index = None;
        }
    }

    /// Returns the current filter text.
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Returns the filtered options (values, not indices).
    pub fn filtered_options(&self) -> Vec<&str> {
        self.filtered_indices
            .iter()
            .filter_map(|&idx| self.options.get(idx).map(|s| s.as_str()))
            .collect()
    }

    /// Returns the number of filtered options.
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Returns true if the dropdown is open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns the placeholder text.
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Sets the placeholder text.
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Returns true if the dropdown is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
        if disabled {
            self.is_open = false;
        }
    }

    /// Updates the filtered indices based on current filter text.
    fn update_filter(&mut self) {
        let filter_lower = self.filter_text.to_lowercase();

        self.filtered_indices = self
            .options
            .iter()
            .enumerate()
            .filter(|(_, opt)| {
                if filter_lower.is_empty() {
                    true
                } else {
                    opt.to_lowercase().contains(&filter_lower)
                }
            })
            .map(|(i, _)| i)
            .collect();

        // Reset highlight to first match (or 0 if no matches)
        self.highlighted_index = 0;
    }
}

/// A searchable dropdown selection component.
///
/// This component provides a filterable dropdown menu for selecting a single
/// option from a list. Users can type to filter options, then navigate and
/// select using keyboard controls.
///
/// # Features
///
/// - Case-insensitive "contains" matching
/// - Keyboard navigation through filtered results
/// - Selection from existing options only
/// - Filter clears on close/confirm
///
/// # Keyboard Navigation
///
/// The dropdown itself doesn't handle keyboard events directly. Your application
/// should map:
/// - Characters to [`DropdownMessage::Insert`]
/// - Backspace to [`DropdownMessage::Backspace`]
/// - Up arrow to [`DropdownMessage::SelectPrevious`]
/// - Down arrow to [`DropdownMessage::SelectNext`]
/// - Enter to [`DropdownMessage::Confirm`]
/// - Escape to [`DropdownMessage::Close`]
///
/// # Visual States
///
/// **Closed (no selection):**
/// ```text
/// ┌──────────────────────┐
/// │ Search...          ▼ │
/// └──────────────────────┘
/// ```
///
/// **Closed (with selection):**
/// ```text
/// ┌──────────────────────┐
/// │ Apple              ▼ │
/// └──────────────────────┘
/// ```
///
/// **Open (with filter):**
/// ```text
/// ┌──────────────────────┐
/// │ app█               ▲ │
/// ├──────────────────────┤
/// │ > Apple              │  ← highlighted
/// │   Pineapple          │
/// └──────────────────────┘
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{Dropdown, DropdownMessage, DropdownOutput, DropdownState, Component};
///
/// let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
///
/// // Open and filter
/// Dropdown::update(&mut state, DropdownMessage::Open);
/// Dropdown::update(&mut state, DropdownMessage::Insert('a'));
///
/// // Navigate and select
/// Dropdown::update(&mut state, DropdownMessage::SelectNext);
/// let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
/// assert_eq!(output, Some(DropdownOutput::Changed(Some(1)))); // Banana
/// ```
pub struct Dropdown;

impl Component for Dropdown {
    type State = DropdownState;
    type Message = DropdownMessage;
    type Output = DropdownOutput;

    fn init() -> Self::State {
        DropdownState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            DropdownMessage::Open => {
                if !state.options.is_empty() {
                    state.is_open = true;
                    // Reset filter and show all options
                    state.filter_text.clear();
                    state.update_filter();
                    // Set highlight to current selection if exists
                    if let Some(selected) = state.selected_index {
                        state.highlighted_index = state
                            .filtered_indices
                            .iter()
                            .position(|&idx| idx == selected)
                            .unwrap_or(0);
                    }
                }
                None
            }
            DropdownMessage::Close => {
                state.is_open = false;
                state.filter_text.clear();
                state.update_filter();
                None
            }
            DropdownMessage::Toggle => {
                if state.is_open {
                    state.is_open = false;
                    state.filter_text.clear();
                    state.update_filter();
                } else if !state.options.is_empty() {
                    state.is_open = true;
                    state.filter_text.clear();
                    state.update_filter();
                    if let Some(selected) = state.selected_index {
                        state.highlighted_index = state
                            .filtered_indices
                            .iter()
                            .position(|&idx| idx == selected)
                            .unwrap_or(0);
                    }
                }
                None
            }
            DropdownMessage::Insert(c) => {
                state.filter_text.push(c);
                state.update_filter();
                // Auto-open when typing
                if !state.is_open && !state.options.is_empty() {
                    state.is_open = true;
                }
                Some(DropdownOutput::FilterChanged(state.filter_text.clone()))
            }
            DropdownMessage::Backspace => {
                if state.filter_text.pop().is_some() {
                    state.update_filter();
                    Some(DropdownOutput::FilterChanged(state.filter_text.clone()))
                } else {
                    None
                }
            }
            DropdownMessage::ClearFilter => {
                if !state.filter_text.is_empty() {
                    state.filter_text.clear();
                    state.update_filter();
                    Some(DropdownOutput::FilterChanged(state.filter_text.clone()))
                } else {
                    None
                }
            }
            DropdownMessage::SetFilter(text) => {
                if state.filter_text != text {
                    state.filter_text = text;
                    state.update_filter();
                    // Auto-open when setting filter
                    if !state.is_open && !state.options.is_empty() {
                        state.is_open = true;
                    }
                    Some(DropdownOutput::FilterChanged(state.filter_text.clone()))
                } else {
                    None
                }
            }
            DropdownMessage::SelectNext => {
                if state.is_open && !state.filtered_indices.is_empty() {
                    state.highlighted_index =
                        (state.highlighted_index + 1) % state.filtered_indices.len();
                }
                None
            }
            DropdownMessage::SelectPrevious => {
                if state.is_open && !state.filtered_indices.is_empty() {
                    if state.highlighted_index == 0 {
                        state.highlighted_index = state.filtered_indices.len() - 1;
                    } else {
                        state.highlighted_index -= 1;
                    }
                }
                None
            }
            DropdownMessage::Confirm => {
                if state.is_open && !state.filtered_indices.is_empty() {
                    let original_index = state.filtered_indices[state.highlighted_index];
                    let old_selection = state.selected_index;
                    state.selected_index = Some(original_index);
                    state.is_open = false;
                    state.filter_text.clear();
                    state.update_filter();

                    if old_selection != state.selected_index {
                        Some(DropdownOutput::Changed(state.selected_index))
                    } else {
                        Some(DropdownOutput::Submitted(original_index))
                    }
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_style()
        } else {
            theme.normal_style()
        };

        let border_style = if state.focused && !state.disabled {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        // Determine what to show in the input area
        let display_text = if state.is_open {
            // When open, show filter text with cursor indicator
            let arrow = "▲";
            if state.filter_text.is_empty() {
                format!("█ {}", arrow)
            } else {
                format!("{}█ {}", state.filter_text, arrow)
            }
        } else if let Some(value) = state.selected_value() {
            format!("{} ▼", value)
        } else {
            format!("{} ▼", state.placeholder)
        };

        let text_style = if !state.is_open
            && state.selected_value().is_none()
            && !state.disabled
            && !state.focused
        {
            theme.placeholder_style()
        } else {
            style
        };

        let paragraph = Paragraph::new(display_text).style(text_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        if !state.is_open {
            frame.render_widget(paragraph, area);
        } else {
            // Render input area at top
            let closed_height = 3; // 1 line + 2 borders
            let closed_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: closed_height.min(area.height),
            };
            frame.render_widget(paragraph, closed_area);

            // Render dropdown list below
            if area.height > closed_height {
                let list_area = Rect {
                    x: area.x,
                    y: area.y + closed_height,
                    width: area.width,
                    height: area.height.saturating_sub(closed_height),
                };

                if state.filtered_indices.is_empty() {
                    // Show "no matches" message
                    let no_match = Paragraph::new("  No matches")
                        .style(theme.placeholder_style())
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(border_style),
                        );
                    frame.render_widget(no_match, list_area);
                } else {
                    let items: Vec<ListItem> = state
                        .filtered_indices
                        .iter()
                        .enumerate()
                        .map(|(i, &orig_idx)| {
                            let opt = &state.options[orig_idx];
                            let prefix = if i == state.highlighted_index {
                                "> "
                            } else {
                                "  "
                            };
                            let text = format!("{}{}", prefix, opt);
                            let item_style = if i == state.highlighted_index {
                                theme.selected_style(state.focused)
                            } else {
                                theme.normal_style()
                            };
                            ListItem::new(text).style(item_style)
                        })
                        .collect();

                    let list = List::new(items).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(border_style),
                    );

                    frame.render_widget(list, list_area);
                }
            }
        }
    }
}

impl Focusable for Dropdown {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
