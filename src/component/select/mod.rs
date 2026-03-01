//! A dropdown selection component.
//!
//! `Select` provides a compact dropdown menu for selecting a single option
//! from a list. It displays the selected value when closed and shows all
//! options when opened.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Select, SelectMessage, SelectOutput, SelectState, Component, Focusable};
//!
//! // Create a select with options
//! let mut state = SelectState::new(vec!["Red", "Green", "Blue"]);
//! state.set_placeholder("Choose a color");
//!
//! // Focus and open it
//! Select::focus(&mut state);
//! let _ = Select::update(&mut state, SelectMessage::Open);
//!
//! // Select an option (navigating to index 1, then confirming)
//! let _ = Select::update(&mut state, SelectMessage::Down);
//! let output = Select::update(&mut state, SelectMessage::Confirm);
//! assert_eq!(output, Some(SelectOutput::Selected("Green".to_string())));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::{Component, Focusable};
use crate::theme::Theme;

/// Messages that can be sent to a Select.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectMessage {
    /// Open the dropdown.
    Open,
    /// Close the dropdown.
    Close,
    /// Toggle the dropdown open/closed.
    Toggle,
    /// Move selection down to the next option.
    Down,
    /// Move selection up to the previous option.
    Up,
    /// Confirm current selection.
    Confirm,
}

/// Output messages from a Select.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectOutput {
    /// A new item was selected (contains the selected value).
    Selected(String),
    /// The highlight changed during navigation (contains the highlighted option index).
    SelectionChanged(usize),
    /// User re-confirmed an already-selected item (contains the index).
    Submitted(usize),
}

/// State for a Select component.
#[derive(Clone, Debug, Default)]
pub struct SelectState {
    /// Available options.
    options: Vec<String>,
    /// Currently selected option index.
    selected_index: Option<usize>,
    /// Currently highlighted option (when dropdown is open).
    highlighted_index: usize,
    /// Whether the dropdown is open.
    is_open: bool,
    /// Whether the component is focused.
    focused: bool,
    /// Placeholder text when nothing is selected.
    placeholder: String,
    /// Whether the select is disabled.
    disabled: bool,
}

impl SelectState {
    /// Creates a new select with the given options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SelectState;
    ///
    /// let state = SelectState::new(vec!["Option 1", "Option 2", "Option 3"]);
    /// assert_eq!(state.options().len(), 3);
    /// assert!(state.selected_index().is_none());
    /// ```
    pub fn new<S: Into<String>>(options: Vec<S>) -> Self {
        Self {
            options: options.into_iter().map(|s| s.into()).collect(),
            selected_index: None,
            highlighted_index: 0,
            is_open: false,
            focused: false,
            placeholder: String::from("Select..."),
            disabled: false,
        }
    }

    /// Creates a new select with the given options and a pre-selected index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SelectState;
    ///
    /// let state = SelectState::with_selection(vec!["A", "B", "C"], 1);
    /// assert_eq!(state.selected_index(), Some(1));
    /// ```
    pub fn with_selection<S: Into<String>>(options: Vec<S>, selected: usize) -> Self {
        let options_vec: Vec<String> = options.into_iter().map(|s| s.into()).collect();
        let selected_index = if selected < options_vec.len() {
            Some(selected)
        } else {
            None
        };

        Self {
            options: options_vec,
            selected_index,
            highlighted_index: selected_index.unwrap_or(0),
            is_open: false,
            focused: false,
            placeholder: String::from("Select..."),
            disabled: false,
        }
    }

    /// Returns the options list.
    pub fn options(&self) -> &[String] {
        &self.options
    }

    /// Sets the options list.
    ///
    /// Resets selection if the current selected index is out of bounds.
    pub fn set_options<S: Into<String>>(&mut self, options: Vec<S>) {
        self.options = options.into_iter().map(|s| s.into()).collect();

        // Reset selection if out of bounds
        if let Some(idx) = self.selected_index {
            if idx >= self.options.len() {
                self.selected_index = None;
            }
        }

        // Reset highlight if out of bounds
        if self.highlighted_index >= self.options.len() && !self.options.is_empty() {
            self.highlighted_index = 0;
        }
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
                self.highlighted_index = idx;
            }
        } else {
            self.selected_index = None;
        }
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

    /// Returns true if the select is disabled.
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
}

/// A dropdown selection component.
///
/// This component provides a compact dropdown menu for selecting a single
/// option from a list. When closed, it displays the currently selected value
/// or a placeholder. When opened, it shows all available options with keyboard
/// navigation.
///
/// # Keyboard Navigation
///
/// The select itself doesn't handle keyboard events directly. Your application
/// should map:
/// - Enter/Space to [`SelectMessage::Toggle`] (open/close)
/// - Down arrow to [`SelectMessage::Down`]
/// - Up arrow to [`SelectMessage::Up`]
/// - Enter to [`SelectMessage::Confirm`] (when open)
/// - Escape to [`SelectMessage::Close`]
///
/// # Visual States
///
/// **Closed:**
/// ```text
/// ┌────────────────┐
/// │ Selected Value ▼│
/// └────────────────┘
/// ```
///
/// **Open:**
/// ```text
/// ┌────────────────┐
/// │ Selected Value ▲│
/// ├────────────────┤
/// │ Option 1       │
/// │ > Option 2     │  ← highlighted
/// │ Option 3       │
/// └────────────────┘
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{Select, SelectMessage, SelectOutput, SelectState, Component};
///
/// let mut state = SelectState::new(vec!["Small", "Medium", "Large"]);
///
/// // Open dropdown
/// Select::update(&mut state, SelectMessage::Open);
///
/// // Navigate to index 1 and confirm (selection changes from None to Some(1))
/// Select::update(&mut state, SelectMessage::Down);
/// let output = Select::update(&mut state, SelectMessage::Confirm);
/// assert_eq!(output, Some(SelectOutput::Selected("Medium".to_string())));
/// ```
pub struct Select;

impl Component for Select {
    type State = SelectState;
    type Message = SelectMessage;
    type Output = SelectOutput;

    fn init() -> Self::State {
        SelectState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            SelectMessage::Open => {
                if !state.options.is_empty() {
                    state.is_open = true;
                    // Set highlight to current selection or first item
                    state.highlighted_index = state.selected_index.unwrap_or(0);
                }
                None
            }
            SelectMessage::Close => {
                state.is_open = false;
                None
            }
            SelectMessage::Toggle => {
                if state.is_open {
                    state.is_open = false;
                } else if !state.options.is_empty() {
                    state.is_open = true;
                    state.highlighted_index = state.selected_index.unwrap_or(0);
                }
                None
            }
            SelectMessage::Down => {
                if state.is_open && !state.options.is_empty() {
                    state.highlighted_index = (state.highlighted_index + 1) % state.options.len();
                    Some(SelectOutput::SelectionChanged(state.highlighted_index))
                } else {
                    None
                }
            }
            SelectMessage::Up => {
                if state.is_open && !state.options.is_empty() {
                    if state.highlighted_index == 0 {
                        state.highlighted_index = state.options.len() - 1;
                    } else {
                        state.highlighted_index -= 1;
                    }
                    Some(SelectOutput::SelectionChanged(state.highlighted_index))
                } else {
                    None
                }
            }
            SelectMessage::Confirm => {
                if state.is_open && !state.options.is_empty() {
                    let old_selection = state.selected_index;
                    let highlighted = state.highlighted_index;
                    state.selected_index = Some(highlighted);
                    state.is_open = false;

                    if old_selection != state.selected_index {
                        Some(SelectOutput::Selected(state.options[highlighted].clone()))
                    } else {
                        Some(SelectOutput::Submitted(highlighted))
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

        // Display selected value or placeholder
        let display_text = if let Some(value) = state.selected_value() {
            let arrow = if state.is_open { "▲" } else { "▼" };
            format!("{} {}", value, arrow)
        } else {
            let arrow = if state.is_open { "▲" } else { "▼" };
            format!("{} {}", state.placeholder, arrow)
        };

        let paragraph = Paragraph::new(display_text).style(style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        if !state.is_open {
            frame.render_widget(paragraph, area);
        } else {
            // Render closed state in first line
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

                let items: Vec<ListItem> = state
                    .options
                    .iter()
                    .enumerate()
                    .map(|(idx, opt)| {
                        let prefix = if idx == state.highlighted_index {
                            "> "
                        } else {
                            "  "
                        };
                        let text = format!("{}{}", prefix, opt);
                        let item_style = if idx == state.highlighted_index {
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

impl Focusable for Select {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests;
