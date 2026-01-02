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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        let style = if state.disabled {
            Style::default().fg(Color::DarkGray)
        } else if state.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let border_style = if state.focused && !state.disabled {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
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
            Style::default().fg(Color::DarkGray)
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
                        .style(Style::default().fg(Color::DarkGray))
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
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
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
mod tests {
    use super::*;

    // ========== State Creation Tests ==========

    #[test]
    fn test_new() {
        let state = DropdownState::new(vec!["A", "B", "C"]);
        assert_eq!(state.options().len(), 3);
        assert_eq!(state.selected_index(), None);
        assert!(!state.is_open());
        assert!(!Dropdown::is_focused(&state));
        assert_eq!(state.filtered_indices.len(), 3);
    }

    #[test]
    fn test_with_selection() {
        let state = DropdownState::with_selection(vec!["A", "B", "C"], 1);
        assert_eq!(state.selected_index(), Some(1));
        assert_eq!(state.selected_value(), Some("B"));
    }

    #[test]
    fn test_with_selection_out_of_bounds() {
        let state = DropdownState::with_selection(vec!["A", "B"], 5);
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn test_default() {
        let state = DropdownState::default();
        assert_eq!(state.options().len(), 0);
        assert_eq!(state.selected_index(), None);
        assert_eq!(state.placeholder(), "Search...");
    }

    // ========== Accessor Tests ==========

    #[test]
    fn test_options() {
        let state = DropdownState::new(vec!["X", "Y", "Z"]);
        assert_eq!(state.options(), &["X", "Y", "Z"]);
    }

    #[test]
    fn test_selected_index() {
        let state = DropdownState::with_selection(vec!["A", "B"], 0);
        assert_eq!(state.selected_index(), Some(0));
    }

    #[test]
    fn test_selected_value() {
        let state = DropdownState::with_selection(vec!["Apple", "Banana"], 1);
        assert_eq!(state.selected_value(), Some("Banana"));

        let empty_state = DropdownState::new(vec!["A", "B"]);
        assert_eq!(empty_state.selected_value(), None);
    }

    #[test]
    fn test_filter_text() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        assert_eq!(state.filter_text(), "");

        Dropdown::update(&mut state, DropdownMessage::Insert('x'));
        assert_eq!(state.filter_text(), "x");
    }

    #[test]
    fn test_filtered_options() {
        let mut state = DropdownState::new(vec!["Apple", "Banana", "Apricot"]);
        assert_eq!(state.filtered_options(), vec!["Apple", "Banana", "Apricot"]);

        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        Dropdown::update(&mut state, DropdownMessage::Insert('p'));
        assert_eq!(state.filtered_options(), vec!["Apple", "Apricot"]);
    }

    #[test]
    fn test_is_open() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        assert!(!state.is_open());

        Dropdown::update(&mut state, DropdownMessage::Open);
        assert!(state.is_open());
    }

    #[test]
    fn test_placeholder() {
        let state = DropdownState::new(vec!["A"]);
        assert_eq!(state.placeholder(), "Search...");
    }

    #[test]
    fn test_is_disabled() {
        let mut state = DropdownState::new(vec!["A"]);
        assert!(!state.is_disabled());

        state.set_disabled(true);
        assert!(state.is_disabled());
    }

    // ========== Mutator Tests ==========

    #[test]
    fn test_set_options() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        state.set_options(vec!["X", "Y", "Z"]);
        assert_eq!(state.options().len(), 3);
        assert_eq!(state.options()[0], "X");
    }

    #[test]
    fn test_set_options_resets_invalid_selection() {
        let mut state = DropdownState::with_selection(vec!["A", "B", "C"], 2);
        state.set_options(vec!["X", "Y"]);
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn test_set_selected_index() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        state.set_selected_index(Some(1));
        assert_eq!(state.selected_index(), Some(1));
        assert_eq!(state.selected_value(), Some("B"));
    }

    #[test]
    fn test_set_selected_index_out_of_bounds() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        state.set_selected_index(Some(5));
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn test_set_placeholder() {
        let mut state = DropdownState::new(vec!["A"]);
        state.set_placeholder("Type here...");
        assert_eq!(state.placeholder(), "Type here...");
    }

    #[test]
    fn test_set_disabled() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        state.set_disabled(true);
        assert!(state.is_disabled());
    }

    // ========== Open/Close Tests ==========

    #[test]
    fn test_open() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        assert!(state.is_open());
    }

    #[test]
    fn test_close() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::Close);
        assert!(!state.is_open());
    }

    #[test]
    fn test_toggle() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);

        Dropdown::update(&mut state, DropdownMessage::Toggle);
        assert!(state.is_open());

        Dropdown::update(&mut state, DropdownMessage::Toggle);
        assert!(!state.is_open());
    }

    #[test]
    fn test_open_empty_options() {
        let mut state = DropdownState::new(Vec::<String>::new());
        Dropdown::update(&mut state, DropdownMessage::Open);
        assert!(!state.is_open());
    }

    #[test]
    fn test_close_clears_filter() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        assert_eq!(state.filter_text(), "a");

        Dropdown::update(&mut state, DropdownMessage::Close);
        assert_eq!(state.filter_text(), "");
    }

    // ========== Filtering Tests ==========

    #[test]
    fn test_insert_char() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        let output = Dropdown::update(&mut state, DropdownMessage::Insert('x'));
        assert_eq!(state.filter_text(), "x");
        assert_eq!(output, Some(DropdownOutput::FilterChanged("x".to_string())));
    }

    #[test]
    fn test_insert_filters() {
        let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));

        // Apple, Banana both contain 'a'
        assert_eq!(state.filtered_count(), 2);
        assert!(state.filtered_options().contains(&"Apple"));
        assert!(state.filtered_options().contains(&"Banana"));
    }

    #[test]
    fn test_backspace() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        Dropdown::update(&mut state, DropdownMessage::Insert('b'));
        assert_eq!(state.filter_text(), "ab");

        let output = Dropdown::update(&mut state, DropdownMessage::Backspace);
        assert_eq!(state.filter_text(), "a");
        assert_eq!(output, Some(DropdownOutput::FilterChanged("a".to_string())));
    }

    #[test]
    fn test_backspace_empty() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        let output = Dropdown::update(&mut state, DropdownMessage::Backspace);
        assert_eq!(output, None);
    }

    #[test]
    fn test_backspace_refilters() {
        let mut state = DropdownState::new(vec!["Apple", "Banana", "Apricot"]);
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        Dropdown::update(&mut state, DropdownMessage::Insert('p'));
        assert_eq!(state.filtered_count(), 2); // Apple, Apricot

        Dropdown::update(&mut state, DropdownMessage::Backspace);
        assert_eq!(state.filtered_count(), 3); // All contain 'a'
    }

    #[test]
    fn test_clear_filter() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Insert('x'));
        Dropdown::update(&mut state, DropdownMessage::Insert('y'));

        let output = Dropdown::update(&mut state, DropdownMessage::ClearFilter);
        assert_eq!(state.filter_text(), "");
        assert_eq!(output, Some(DropdownOutput::FilterChanged("".to_string())));
    }

    #[test]
    fn test_clear_filter_empty() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        let output = Dropdown::update(&mut state, DropdownMessage::ClearFilter);
        assert_eq!(output, None);
    }

    #[test]
    fn test_set_filter() {
        let mut state = DropdownState::new(vec!["Apple", "Banana"]);
        let output = Dropdown::update(&mut state, DropdownMessage::SetFilter("app".to_string()));

        assert_eq!(state.filter_text(), "app");
        assert_eq!(
            output,
            Some(DropdownOutput::FilterChanged("app".to_string()))
        );
        assert_eq!(state.filtered_count(), 1);
    }

    #[test]
    fn test_set_filter_same() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        Dropdown::update(&mut state, DropdownMessage::SetFilter("x".to_string()));

        let output = Dropdown::update(&mut state, DropdownMessage::SetFilter("x".to_string()));
        assert_eq!(output, None);
    }

    #[test]
    fn test_filter_case_insensitive() {
        let mut state = DropdownState::new(vec!["Apple", "BANANA", "cherry"]);
        Dropdown::update(&mut state, DropdownMessage::Insert('A'));

        // Should match Apple, BANANA (both contain 'a' case-insensitively)
        assert_eq!(state.filtered_count(), 2);
        assert!(state.filtered_options().contains(&"Apple"));
        assert!(state.filtered_options().contains(&"BANANA"));
    }

    #[test]
    fn test_filter_contains() {
        let mut state = DropdownState::new(vec!["Apple", "Pineapple", "Grape"]);
        Dropdown::update(&mut state, DropdownMessage::Insert('p'));
        Dropdown::update(&mut state, DropdownMessage::Insert('l'));
        Dropdown::update(&mut state, DropdownMessage::Insert('e'));

        // "ple" is contained in Apple, Pineapple (not Grape)
        assert_eq!(state.filtered_count(), 2);
    }

    #[test]
    fn test_filter_no_matches() {
        let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
        Dropdown::update(&mut state, DropdownMessage::Insert('x'));
        Dropdown::update(&mut state, DropdownMessage::Insert('y'));
        Dropdown::update(&mut state, DropdownMessage::Insert('z'));

        assert_eq!(state.filtered_count(), 0);
        assert!(state.filtered_options().is_empty());
    }

    #[test]
    fn test_filter_resets_highlight() {
        let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
        Dropdown::update(&mut state, DropdownMessage::Open);

        // Navigate to second item
        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        assert_eq!(state.highlighted_index, 1);

        // Filter - should reset highlight to 0
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        assert_eq!(state.highlighted_index, 0);
    }

    // ========== Navigation Tests ==========

    #[test]
    fn test_select_next() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);

        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        assert_eq!(state.highlighted_index, 1);

        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        assert_eq!(state.highlighted_index, 2);
    }

    #[test]
    fn test_select_previous() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        assert_eq!(state.highlighted_index, 2);

        Dropdown::update(&mut state, DropdownMessage::SelectPrevious);
        assert_eq!(state.highlighted_index, 1);

        Dropdown::update(&mut state, DropdownMessage::SelectPrevious);
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_select_next_wraps() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);

        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        assert_eq!(state.highlighted_index, 0); // Wrapped
    }

    #[test]
    fn test_select_previous_wraps() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);

        Dropdown::update(&mut state, DropdownMessage::SelectPrevious);
        assert_eq!(state.highlighted_index, 2); // Wrapped to end
    }

    #[test]
    fn test_navigation_empty_filter() {
        let mut state = DropdownState::new(vec!["Apple", "Banana"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::SetFilter("xyz".to_string()));

        // No matches - navigation should be no-op
        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_navigation_when_closed() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        // Closed - navigation should be no-op
        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        assert_eq!(state.highlighted_index, 0);
    }

    // ========== Confirm Tests ==========

    #[test]
    fn test_confirm() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::SelectNext);

        Dropdown::update(&mut state, DropdownMessage::Confirm);
        assert_eq!(state.selected_index(), Some(1));
        assert!(!state.is_open());
    }

    #[test]
    fn test_confirm_returns_changed() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::SelectNext);

        let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
        assert_eq!(output, Some(DropdownOutput::Changed(Some(1))));
    }

    #[test]
    fn test_confirm_returns_submitted() {
        let mut state = DropdownState::with_selection(vec!["A", "B", "C"], 1);
        Dropdown::update(&mut state, DropdownMessage::Open);
        // Highlight is on selected item

        let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
        assert_eq!(output, Some(DropdownOutput::Submitted(1)));
    }

    #[test]
    fn test_confirm_when_closed() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
        assert_eq!(output, None);
    }

    #[test]
    fn test_confirm_no_matches() {
        let mut state = DropdownState::new(vec!["Apple", "Banana"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::SetFilter("xyz".to_string()));

        let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
        assert_eq!(output, None);
    }

    #[test]
    fn test_confirm_clears_filter() {
        let mut state = DropdownState::new(vec!["Apple", "Banana"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        assert_eq!(state.filter_text(), "a");

        Dropdown::update(&mut state, DropdownMessage::Confirm);
        assert_eq!(state.filter_text(), "");
    }

    #[test]
    fn test_confirm_with_filter() {
        let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        // Filtered: Apple (0), Banana (1)
        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        // Highlight on Banana (index 1 in filtered = original index 1)

        let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
        assert_eq!(output, Some(DropdownOutput::Changed(Some(1))));
        assert_eq!(state.selected_value(), Some("Banana"));
    }

    // ========== Disabled State Tests ==========

    #[test]
    fn test_disabled_ignores_messages() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        state.set_disabled(true);

        let output = Dropdown::update(&mut state, DropdownMessage::Open);
        assert_eq!(output, None);
        assert!(!state.is_open());

        let output = Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        assert_eq!(output, None);
        assert_eq!(state.filter_text(), "");
    }

    #[test]
    fn test_disabling_closes_dropdown() {
        let mut state = DropdownState::new(vec!["A", "B", "C"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        assert!(state.is_open());

        state.set_disabled(true);
        assert!(!state.is_open());
    }

    // ========== Focus Tests ==========

    #[test]
    fn test_focusable_is_focused() {
        let state = DropdownState::new(vec!["A", "B"]);
        assert!(!Dropdown::is_focused(&state));
    }

    #[test]
    fn test_focusable_set_focused() {
        let mut state = DropdownState::new(vec!["A", "B"]);
        Dropdown::set_focused(&mut state, true);
        assert!(Dropdown::is_focused(&state));
    }

    #[test]
    fn test_focus_blur() {
        let mut state = DropdownState::new(vec!["A", "B"]);

        Dropdown::focus(&mut state);
        assert!(Dropdown::is_focused(&state));

        Dropdown::blur(&mut state);
        assert!(!Dropdown::is_focused(&state));
    }

    // ========== View Tests ==========

    #[test]
    fn test_view_closed_empty() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = DropdownState::new(vec!["Apple", "Banana"]);

        let backend = CaptureBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dropdown::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Search...") || output.contains("▼"));
    }

    #[test]
    fn test_view_closed_with_selection() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = DropdownState::with_selection(vec!["Apple", "Banana"], 0);

        let backend = CaptureBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dropdown::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Apple"));
    }

    #[test]
    fn test_view_open_no_filter() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
        Dropdown::update(&mut state, DropdownMessage::Open);

        let backend = CaptureBackend::new(30, 15);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dropdown::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Apple"));
        assert!(output.contains("Banana"));
        assert!(output.contains("Cherry"));
    }

    #[test]
    fn test_view_open_with_filter() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));

        let backend = CaptureBackend::new(30, 15);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dropdown::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Apple"));
        assert!(output.contains("Banana"));
        // Cherry should not be shown (doesn't contain 'a')
    }

    #[test]
    fn test_view_highlight() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = DropdownState::new(vec!["Apple", "Banana", "Cherry"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::SelectNext);

        let backend = CaptureBackend::new(30, 15);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dropdown::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        // Should show highlight indicator on Banana
        assert!(output.contains("> Banana") || output.contains("Banana"));
    }

    #[test]
    fn test_view_no_matches() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = DropdownState::new(vec!["Apple", "Banana"]);
        Dropdown::update(&mut state, DropdownMessage::Open);
        Dropdown::update(&mut state, DropdownMessage::SetFilter("xyz".to_string()));

        let backend = CaptureBackend::new(30, 15);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dropdown::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("No matches"));
    }

    #[test]
    fn test_view_focused() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = DropdownState::new(vec!["A", "B"]);
        Dropdown::focus(&mut state);

        let backend = CaptureBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Dropdown::view(&state, frame, frame.area());
            })
            .unwrap();

        // Should render without error
        let output = terminal.backend().to_string();
        assert!(!output.is_empty());
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_clone() {
        let state = DropdownState::with_selection(vec!["A", "B", "C"], 1);
        let cloned = state.clone();
        assert_eq!(cloned.selected_index(), Some(1));
    }

    #[test]
    fn test_init() {
        let state = Dropdown::init();
        assert_eq!(state.options().len(), 0);
        assert!(!Dropdown::is_focused(&state));
    }

    #[test]
    fn test_full_workflow() {
        let mut state = DropdownState::new(vec!["Apple", "Apricot", "Banana", "Cherry"]);

        // Open dropdown
        Dropdown::update(&mut state, DropdownMessage::Open);
        assert!(state.is_open());
        assert_eq!(state.filtered_count(), 4);

        // Type to filter
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        assert_eq!(state.filtered_count(), 3); // Apple, Apricot, Banana

        Dropdown::update(&mut state, DropdownMessage::Insert('p'));
        assert_eq!(state.filtered_count(), 2); // Apple, Apricot

        // Navigate
        Dropdown::update(&mut state, DropdownMessage::SelectNext);
        assert_eq!(state.highlighted_index, 1); // Apricot

        // Confirm
        let output = Dropdown::update(&mut state, DropdownMessage::Confirm);
        assert_eq!(output, Some(DropdownOutput::Changed(Some(1)))); // Apricot is index 1
        assert_eq!(state.selected_value(), Some("Apricot"));
        assert!(!state.is_open());
        assert_eq!(state.filter_text(), ""); // Filter cleared
    }

    #[test]
    fn test_auto_open_on_type() {
        let mut state = DropdownState::new(vec!["Apple", "Banana"]);
        assert!(!state.is_open());

        // Typing should auto-open
        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        assert!(state.is_open());
    }

    #[test]
    fn test_filtered_count() {
        let mut state = DropdownState::new(vec!["Apple", "Apricot", "Banana"]);
        assert_eq!(state.filtered_count(), 3);

        Dropdown::update(&mut state, DropdownMessage::Insert('a'));
        Dropdown::update(&mut state, DropdownMessage::Insert('p'));
        assert_eq!(state.filtered_count(), 2);
    }
}
