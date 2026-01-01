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
//! let _ = Select::update(&mut state, SelectMessage::SelectNext);
//! let output = Select::update(&mut state, SelectMessage::Confirm);
//! assert_eq!(output, Some(SelectOutput::Changed(Some(1))));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use super::{Component, Focusable};

/// Messages that can be sent to a Select.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectMessage {
    /// Open the dropdown.
    Open,
    /// Close the dropdown.
    Close,
    /// Toggle the dropdown open/closed.
    Toggle,
    /// Move selection to next option.
    SelectNext,
    /// Move selection to previous option.
    SelectPrevious,
    /// Confirm current selection.
    Confirm,
}

/// Output messages from a Select.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectOutput {
    /// Selection changed (index in options list).
    Changed(Option<usize>),
    /// User confirmed selection (index in options list).
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
/// - Up arrow to [`SelectMessage::SelectPrevious`]
/// - Down arrow to [`SelectMessage::SelectNext`]
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
/// Select::update(&mut state, SelectMessage::SelectNext);
/// let output = Select::update(&mut state, SelectMessage::Confirm);
/// assert_eq!(output, Some(SelectOutput::Changed(Some(1))));
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
            SelectMessage::SelectNext => {
                if state.is_open && !state.options.is_empty() {
                    state.highlighted_index = (state.highlighted_index + 1) % state.options.len();
                }
                None
            }
            SelectMessage::SelectPrevious => {
                if state.is_open && !state.options.is_empty() {
                    if state.highlighted_index == 0 {
                        state.highlighted_index = state.options.len() - 1;
                    } else {
                        state.highlighted_index -= 1;
                    }
                }
                None
            }
            SelectMessage::Confirm => {
                if state.is_open && !state.options.is_empty() {
                    let old_selection = state.selected_index;
                    state.selected_index = Some(state.highlighted_index);
                    state.is_open = false;

                    // Emit output only if selection changed or confirmed
                    if old_selection != state.selected_index {
                        Some(SelectOutput::Changed(state.selected_index))
                    } else {
                        Some(SelectOutput::Submitted(state.highlighted_index))
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

        // Display selected value or placeholder
        let display_text = if let Some(value) = state.selected_value() {
            let arrow = if state.is_open { "▲" } else { "▼" };
            format!("{} {}", value, arrow)
        } else {
            let arrow = if state.is_open { "▲" } else { "▼" };
            format!("{} {}", state.placeholder, arrow)
        };

        let paragraph = Paragraph::new(display_text)
            .style(style)
            .block(
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
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
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

impl Focusable for Select {
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
    fn test_new() {
        let state = SelectState::new(vec!["A", "B", "C"]);
        assert_eq!(state.options().len(), 3);
        assert_eq!(state.selected_index(), None);
        assert!(!state.is_open());
        assert!(!Select::is_focused(&state));
    }

    #[test]
    fn test_with_selection() {
        let state = SelectState::with_selection(vec!["A", "B", "C"], 1);
        assert_eq!(state.selected_index(), Some(1));
        assert_eq!(state.selected_value(), Some("B"));
    }

    #[test]
    fn test_with_selection_out_of_bounds() {
        let state = SelectState::with_selection(vec!["A", "B"], 5);
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn test_default() {
        let state = SelectState::default();
        assert_eq!(state.options().len(), 0);
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn test_set_options() {
        let mut state = SelectState::new(vec!["A", "B"]);
        state.set_options(vec!["X", "Y", "Z"]);
        assert_eq!(state.options().len(), 3);
        assert_eq!(state.options()[0], "X");
    }

    #[test]
    fn test_set_options_resets_invalid_selection() {
        let mut state = SelectState::with_selection(vec!["A", "B", "C"], 2);
        state.set_options(vec!["X", "Y"]);
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn test_set_selected_index() {
        let mut state = SelectState::new(vec!["A", "B", "C"]);
        state.set_selected_index(Some(1));
        assert_eq!(state.selected_index(), Some(1));
        assert_eq!(state.selected_value(), Some("B"));
    }

    #[test]
    fn test_set_selected_index_out_of_bounds() {
        let mut state = SelectState::new(vec!["A", "B"]);
        state.set_selected_index(Some(5));
        assert_eq!(state.selected_index(), None);
    }

    #[test]
    fn test_placeholder() {
        let mut state = SelectState::new(vec!["A", "B"]);
        assert_eq!(state.placeholder(), "Select...");

        state.set_placeholder("Choose one");
        assert_eq!(state.placeholder(), "Choose one");
    }

    #[test]
    fn test_disabled() {
        let mut state = SelectState::new(vec!["A", "B"]);
        assert!(!state.is_disabled());

        state.set_disabled(true);
        assert!(state.is_disabled());
    }

    #[test]
    fn test_open_close() {
        let mut state = SelectState::new(vec!["A", "B", "C"]);

        Select::update(&mut state, SelectMessage::Open);
        assert!(state.is_open());

        Select::update(&mut state, SelectMessage::Close);
        assert!(!state.is_open());
    }

    #[test]
    fn test_toggle() {
        let mut state = SelectState::new(vec!["A", "B", "C"]);

        Select::update(&mut state, SelectMessage::Toggle);
        assert!(state.is_open());

        Select::update(&mut state, SelectMessage::Toggle);
        assert!(!state.is_open());
    }

    #[test]
    fn test_open_empty_options() {
        let mut state = SelectState::new(Vec::<String>::new());

        Select::update(&mut state, SelectMessage::Open);
        assert!(!state.is_open());
    }

    #[test]
    fn test_select_next() {
        let mut state = SelectState::new(vec!["A", "B", "C"]);
        Select::update(&mut state, SelectMessage::Open);

        Select::update(&mut state, SelectMessage::SelectNext);
        assert_eq!(state.highlighted_index, 1);

        Select::update(&mut state, SelectMessage::SelectNext);
        assert_eq!(state.highlighted_index, 2);

        // Wrap around
        Select::update(&mut state, SelectMessage::SelectNext);
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_select_previous() {
        let mut state = SelectState::new(vec!["A", "B", "C"]);
        Select::update(&mut state, SelectMessage::Open);

        // Wrap around from start
        Select::update(&mut state, SelectMessage::SelectPrevious);
        assert_eq!(state.highlighted_index, 2);

        Select::update(&mut state, SelectMessage::SelectPrevious);
        assert_eq!(state.highlighted_index, 1);

        Select::update(&mut state, SelectMessage::SelectPrevious);
        assert_eq!(state.highlighted_index, 0);
    }

    #[test]
    fn test_confirm_selection() {
        let mut state = SelectState::new(vec!["A", "B", "C"]);
        Select::update(&mut state, SelectMessage::Open);
        Select::update(&mut state, SelectMessage::SelectNext);

        let output = Select::update(&mut state, SelectMessage::Confirm);
        assert_eq!(output, Some(SelectOutput::Changed(Some(1))));
        assert_eq!(state.selected_index(), Some(1));
        assert!(!state.is_open());
    }

    #[test]
    fn test_confirm_same_selection() {
        let mut state = SelectState::with_selection(vec!["A", "B", "C"], 1);
        Select::update(&mut state, SelectMessage::Open);

        let output = Select::update(&mut state, SelectMessage::Confirm);
        assert_eq!(output, Some(SelectOutput::Submitted(1)));
        assert!(!state.is_open());
    }

    #[test]
    fn test_confirm_when_closed() {
        let mut state = SelectState::new(vec!["A", "B", "C"]);

        let output = Select::update(&mut state, SelectMessage::Confirm);
        assert_eq!(output, None);
    }

    #[test]
    fn test_disabled_ignores_messages() {
        let mut state = SelectState::new(vec!["A", "B", "C"]);
        state.set_disabled(true);

        let output = Select::update(&mut state, SelectMessage::Open);
        assert_eq!(output, None);
        assert!(!state.is_open());

        let output = Select::update(&mut state, SelectMessage::SelectNext);
        assert_eq!(output, None);
    }

    #[test]
    fn test_disabling_closes_dropdown() {
        let mut state = SelectState::new(vec!["A", "B", "C"]);
        Select::update(&mut state, SelectMessage::Open);
        assert!(state.is_open());

        state.set_disabled(true);
        assert!(!state.is_open());
    }

    #[test]
    fn test_focusable() {
        let mut state = SelectState::new(vec!["A", "B"]);

        assert!(!Select::is_focused(&state));

        Select::focus(&mut state);
        assert!(Select::is_focused(&state));

        Select::blur(&mut state);
        assert!(!Select::is_focused(&state));
    }

    #[test]
    fn test_init() {
        let state = Select::init();
        assert_eq!(state.options().len(), 0);
        assert!(!Select::is_focused(&state));
    }

    #[test]
    fn test_clone() {
        let state = SelectState::with_selection(vec!["A", "B", "C"], 1);
        let cloned = state.clone();
        assert_eq!(cloned.selected_index(), Some(1));
    }

    #[test]
    fn test_view_closed() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = SelectState::new(vec!["Red", "Green", "Blue"]);

        let backend = CaptureBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Select::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Select...") || output.contains("▼"));
    }

    #[test]
    fn test_view_open() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = SelectState::new(vec!["Red", "Green", "Blue"]);
        Select::update(&mut state, SelectMessage::Open);

        let backend = CaptureBackend::new(30, 15);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Select::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Red") || output.contains("Green") || output.contains("Blue"));
    }

    #[test]
    fn test_view_with_selection() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = SelectState::with_selection(vec!["Small", "Medium", "Large"], 1);

        let backend = CaptureBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Select::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Medium"));
    }

    #[test]
    fn test_view_focused() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = SelectState::new(vec!["A", "B"]);
        Select::focus(&mut state);

        let backend = CaptureBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Select::view(&state, frame, frame.area());
            })
            .unwrap();

        // Should render without error
        let output = terminal.backend().to_string();
        assert!(!output.is_empty());
    }
}
