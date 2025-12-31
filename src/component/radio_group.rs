//! A mutually exclusive option selection component.
//!
//! `RadioGroup` provides a group of radio buttons where exactly one option
//! can be selected at a time. Unlike [`SelectableList`](super::SelectableList),
//! navigation immediately changes the selection (traditional radio button behavior).
//!
//! # Example
//!
//! ```rust
//! use envision::component::{RadioGroup, RadioMessage, RadioOutput, RadioGroupState, Component};
//!
//! // Create a radio group with options
//! let mut state = RadioGroupState::new(vec!["Small", "Medium", "Large"]);
//! assert_eq!(state.selected_index(), 0);
//!
//! // Navigate down - immediately selects next option
//! let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
//! assert_eq!(output, Some(RadioOutput::Selected("Medium")));
//! assert_eq!(state.selected_index(), 1);
//!
//! // Confirm selection (e.g., for form submission)
//! let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Confirm);
//! assert_eq!(output, Some(RadioOutput::Confirmed("Medium")));
//!
//! // Disabled radio groups don't respond
//! state.set_disabled(true);
//! let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
//! assert_eq!(output, None);
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem};

use super::{Component, Focusable};

/// Messages that can be sent to a RadioGroup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RadioMessage {
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Confirm the current selection (e.g., for form submission).
    Confirm,
}

/// Output messages from a RadioGroup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RadioOutput<T: Clone> {
    /// The selection changed to a new value.
    Selected(T),
    /// The current selection was confirmed (Enter pressed).
    Confirmed(T),
}

/// State for a RadioGroup component.
#[derive(Clone, Debug)]
pub struct RadioGroupState<T: Clone> {
    /// The available options.
    options: Vec<T>,
    /// The currently selected index.
    selected: usize,
    /// Whether the radio group is focused.
    focused: bool,
    /// Whether the radio group is disabled.
    disabled: bool,
}

impl<T: Clone> Default for RadioGroupState<T> {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            selected: 0,
            focused: false,
            disabled: false,
        }
    }
}

impl<T: Clone> RadioGroupState<T> {
    /// Creates a new radio group with the given options.
    ///
    /// The first option is selected by default. If the options are empty,
    /// no selection is possible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::RadioGroupState;
    ///
    /// let state = RadioGroupState::new(vec!["Option A", "Option B", "Option C"]);
    /// assert_eq!(state.selected_index(), 0);
    /// assert_eq!(state.selected(), Some(&"Option A"));
    /// ```
    pub fn new(options: Vec<T>) -> Self {
        Self {
            options,
            selected: 0,
            focused: false,
            disabled: false,
        }
    }

    /// Creates a radio group with a specific initial selection.
    ///
    /// If the selected index is out of bounds, it will be clamped to the
    /// last valid index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::RadioGroupState;
    ///
    /// let state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);
    /// assert_eq!(state.selected_index(), 1);
    /// assert_eq!(state.selected(), Some(&"B"));
    /// ```
    pub fn with_selected(options: Vec<T>, selected: usize) -> Self {
        let selected = if options.is_empty() {
            0
        } else {
            selected.min(options.len() - 1)
        };
        Self {
            options,
            selected,
            focused: false,
            disabled: false,
        }
    }

    /// Returns the available options.
    pub fn options(&self) -> &[T] {
        &self.options
    }

    /// Returns the currently selected index.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Returns a reference to the currently selected option.
    ///
    /// Returns `None` if the options are empty.
    pub fn selected(&self) -> Option<&T> {
        self.options.get(self.selected)
    }

    /// Sets the selected index.
    ///
    /// If the index is out of bounds, it will be ignored.
    pub fn set_selected(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected = index;
        }
    }

    /// Returns true if the radio group is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// Disabled radio groups do not respond to messages.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Returns true if the options are empty.
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    /// Returns the number of options.
    pub fn len(&self) -> usize {
        self.options.len()
    }
}

/// A mutually exclusive option selection component.
///
/// `RadioGroup` provides a group of radio buttons where exactly one option
/// can be selected at a time. Navigation (Up/Down) immediately changes
/// the selection, following traditional radio button behavior.
///
/// # Type Parameter
///
/// - `T`: The type of options. Must implement `Clone` and `Display` for rendering.
///
/// # Navigation
///
/// - `Up` - Select the previous option
/// - `Down` - Select the next option
/// - `Confirm` - Emit the current selection for form submission
///
/// # Visual States
///
/// - **Selected**: `(•) Label`
/// - **Unselected**: `( ) Label`
/// - **Focused**: Yellow highlight on selected option
/// - **Disabled**: DarkGray text
///
/// # Example
///
/// ```rust
/// use envision::component::{RadioGroup, RadioMessage, RadioOutput, RadioGroupState, Component, Focusable};
///
/// let mut state = RadioGroupState::new(vec!["Small", "Medium", "Large"]);
///
/// // Navigate to select "Medium"
/// let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
/// assert_eq!(output, Some(RadioOutput::Selected("Medium")));
///
/// // Focus the component for visual feedback
/// RadioGroup::<&str>::set_focused(&mut state, true);
///
/// // Confirm the selection
/// let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Confirm);
/// assert_eq!(output, Some(RadioOutput::Confirmed("Medium")));
/// ```
pub struct RadioGroup<T: Clone>(PhantomData<T>);

impl<T: Clone + std::fmt::Display + 'static> Component for RadioGroup<T> {
    type State = RadioGroupState<T>;
    type Message = RadioMessage;
    type Output = RadioOutput<T>;

    fn init() -> Self::State {
        RadioGroupState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled || state.options.is_empty() {
            return None;
        }

        match msg {
            RadioMessage::Up => {
                if state.selected > 0 {
                    state.selected -= 1;
                    state
                        .options
                        .get(state.selected)
                        .cloned()
                        .map(RadioOutput::Selected)
                } else {
                    None
                }
            }
            RadioMessage::Down => {
                if state.selected < state.options.len() - 1 {
                    state.selected += 1;
                    state
                        .options
                        .get(state.selected)
                        .cloned()
                        .map(RadioOutput::Selected)
                } else {
                    None
                }
            }
            RadioMessage::Confirm => state
                .options
                .get(state.selected)
                .cloned()
                .map(RadioOutput::Confirmed),
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = state
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let indicator = if i == state.selected { "(•)" } else { "( )" };
                let text = format!("{} {}", indicator, option);

                let style = if state.disabled {
                    Style::default().fg(Color::DarkGray)
                } else if i == state.selected && state.focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, area);
    }
}

impl<T: Clone + std::fmt::Display + 'static> Focusable for RadioGroup<T> {
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
        let state = RadioGroupState::new(vec!["A", "B", "C"]);
        assert_eq!(state.len(), 3);
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected(), Some(&"A"));
        assert!(!state.is_disabled());
        assert!(!RadioGroup::<&str>::is_focused(&state));
    }

    #[test]
    fn test_with_selected() {
        let state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);
        assert_eq!(state.selected_index(), 1);
        assert_eq!(state.selected(), Some(&"B"));
    }

    #[test]
    fn test_with_selected_clamps() {
        let state = RadioGroupState::with_selected(vec!["A", "B", "C"], 10);
        assert_eq!(state.selected_index(), 2); // Clamped to last
    }

    #[test]
    fn test_empty() {
        let state = RadioGroupState::<String>::new(vec![]);
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_default() {
        let state = RadioGroupState::<String>::default();
        assert!(state.is_empty());
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected(), None);
    }

    #[test]
    fn test_options_accessor() {
        let state = RadioGroupState::new(vec!["X", "Y", "Z"]);
        assert_eq!(state.options(), &["X", "Y", "Z"]);
    }

    #[test]
    fn test_selected_accessors() {
        let mut state = RadioGroupState::new(vec!["A", "B", "C"]);

        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.selected(), Some(&"A"));

        state.set_selected(2);
        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.selected(), Some(&"C"));

        // Out of bounds is ignored
        state.set_selected(100);
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_disabled_accessors() {
        let mut state = RadioGroupState::new(vec!["A", "B"]);
        assert!(!state.is_disabled());

        state.set_disabled(true);
        assert!(state.is_disabled());

        state.set_disabled(false);
        assert!(!state.is_disabled());
    }

    #[test]
    fn test_navigate_down() {
        let mut state = RadioGroupState::new(vec!["A", "B", "C"]);

        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
        assert_eq!(output, Some(RadioOutput::Selected("B")));
        assert_eq!(state.selected_index(), 1);

        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
        assert_eq!(output, Some(RadioOutput::Selected("C")));
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_navigate_up() {
        let mut state = RadioGroupState::with_selected(vec!["A", "B", "C"], 2);

        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
        assert_eq!(output, Some(RadioOutput::Selected("B")));
        assert_eq!(state.selected_index(), 1);

        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
        assert_eq!(output, Some(RadioOutput::Selected("A")));
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_navigate_at_bounds() {
        let mut state = RadioGroupState::new(vec!["A", "B", "C"]);

        // At first, Up returns None
        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
        assert_eq!(output, None);
        assert_eq!(state.selected_index(), 0);

        // Go to last
        state.set_selected(2);

        // At last, Down returns None
        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
        assert_eq!(output, None);
        assert_eq!(state.selected_index(), 2);
    }

    #[test]
    fn test_confirm() {
        let mut state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);

        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Confirm);
        assert_eq!(output, Some(RadioOutput::Confirmed("B")));
        // Selection unchanged
        assert_eq!(state.selected_index(), 1);
    }

    #[test]
    fn test_confirm_empty() {
        let mut state = RadioGroupState::<String>::new(vec![]);

        let output = RadioGroup::<String>::update(&mut state, RadioMessage::Confirm);
        assert_eq!(output, None);
    }

    #[test]
    fn test_disabled() {
        let mut state = RadioGroupState::new(vec!["A", "B", "C"]);
        state.set_disabled(true);

        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
        assert_eq!(output, None);
        assert_eq!(state.selected_index(), 0);

        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
        assert_eq!(output, None);

        let output = RadioGroup::<&str>::update(&mut state, RadioMessage::Confirm);
        assert_eq!(output, None);
    }

    #[test]
    fn test_empty_navigation() {
        let mut state = RadioGroupState::<String>::new(vec![]);

        assert_eq!(
            RadioGroup::<String>::update(&mut state, RadioMessage::Down),
            None
        );
        assert_eq!(
            RadioGroup::<String>::update(&mut state, RadioMessage::Up),
            None
        );
        assert_eq!(
            RadioGroup::<String>::update(&mut state, RadioMessage::Confirm),
            None
        );
    }

    #[test]
    fn test_focusable() {
        let mut state = RadioGroupState::new(vec!["A", "B"]);

        assert!(!RadioGroup::<&str>::is_focused(&state));

        RadioGroup::<&str>::set_focused(&mut state, true);
        assert!(RadioGroup::<&str>::is_focused(&state));

        RadioGroup::<&str>::blur(&mut state);
        assert!(!RadioGroup::<&str>::is_focused(&state));

        RadioGroup::<&str>::focus(&mut state);
        assert!(RadioGroup::<&str>::is_focused(&state));
    }

    #[test]
    fn test_init() {
        let state = RadioGroup::<String>::init();
        assert!(state.is_empty());
        assert_eq!(state.selected_index(), 0);
        assert!(!state.is_disabled());
        assert!(!RadioGroup::<String>::is_focused(&state));
    }

    #[test]
    fn test_clone() {
        let state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);
        let cloned = state.clone();

        assert_eq!(cloned.options(), &["A", "B", "C"]);
        assert_eq!(cloned.selected_index(), 1);
    }

    #[test]
    fn test_view_renders_indicators() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = RadioGroupState::with_selected(vec!["Option A", "Option B", "Option C"], 1);
        state.focused = true;

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                RadioGroup::<&str>::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("( ) Option A"));
        assert!(output.contains("(•) Option B")); // Selected
        assert!(output.contains("( ) Option C"));
    }

    #[test]
    fn test_view_disabled() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut state = RadioGroupState::new(vec!["Test"]);
        state.set_disabled(true);

        let backend = CaptureBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                RadioGroup::<&str>::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("(•) Test"));
    }

    #[test]
    fn test_multiple_navigations() {
        let mut state = RadioGroupState::new(vec!["1", "2", "3", "4", "5"]);

        // Navigate down multiple times
        RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
        RadioGroup::<&str>::update(&mut state, RadioMessage::Down);
        assert_eq!(state.selected_index(), 2);
        assert_eq!(state.selected(), Some(&"3"));

        // Navigate up
        RadioGroup::<&str>::update(&mut state, RadioMessage::Up);
        assert_eq!(state.selected_index(), 1);
        assert_eq!(state.selected(), Some(&"2"));
    }
}
