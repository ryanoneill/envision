//! A mutually exclusive option selection component.
//!
//! `RadioGroup` provides a group of radio buttons where exactly one option
//! can be selected at a time. Unlike [`SelectableList`](super::SelectableList),
//! navigation immediately changes the selection (traditional radio button behavior).
//!
//! # Example
//!
//! ```rust
//! use envision::component::{RadioGroup, RadioGroupMessage, RadioGroupOutput, RadioGroupState, Component};
//!
//! // Create a radio group with options
//! let mut state = RadioGroupState::new(vec!["Small", "Medium", "Large"]);
//! assert_eq!(state.selected_index(), Some(0));
//!
//! // Navigate down - immediately selects next option
//! let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
//! assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(1)));
//! assert_eq!(state.selected_index(), Some(1));
//!
//! // Confirm selection (e.g., for form submission)
//! let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Confirm);
//! assert_eq!(output, Some(RadioGroupOutput::Confirmed("Medium")));
//!
//! // Disabled radio groups don't respond
//! state.set_disabled(true);
//! let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
//! assert_eq!(output, None);
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem};

use super::{Component, Focusable};
use crate::theme::Theme;

/// Messages that can be sent to a RadioGroup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RadioGroupMessage {
    /// Move selection up.
    Up,
    /// Move selection down.
    Down,
    /// Confirm the current selection (e.g., for form submission).
    Confirm,
}

/// Output messages from a RadioGroup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RadioGroupOutput<T: Clone> {
    /// The selection changed to a new value.
    Selected(T),
    /// The current selection was confirmed (Enter pressed).
    Confirmed(T),
    /// The selection changed during navigation (contains the new index).
    SelectionChanged(usize),
}

/// State for a RadioGroup component.
#[derive(Clone, Debug)]
pub struct RadioGroupState<T: Clone> {
    /// The available options.
    options: Vec<T>,
    /// The currently selected index, or `None` if empty.
    selected: Option<usize>,
    /// Whether the radio group is focused.
    focused: bool,
    /// Whether the radio group is disabled.
    disabled: bool,
}

impl<T: Clone> Default for RadioGroupState<T> {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            selected: None,
            focused: false,
            disabled: false,
        }
    }
}

impl<T: Clone> RadioGroupState<T> {
    /// Creates a new radio group with the given options.
    ///
    /// The first option is selected by default. If the options are empty,
    /// the selection is `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::RadioGroupState;
    ///
    /// let state = RadioGroupState::new(vec!["Option A", "Option B", "Option C"]);
    /// assert_eq!(state.selected_index(), Some(0));
    /// assert_eq!(state.selected(), Some(&"Option A"));
    /// ```
    pub fn new(options: Vec<T>) -> Self {
        let selected = if options.is_empty() { None } else { Some(0) };
        Self {
            options,
            selected,
            focused: false,
            disabled: false,
        }
    }

    /// Creates a radio group with a specific initial selection.
    ///
    /// If the selected index is out of bounds, it will be clamped to the
    /// last valid index. Returns `None` selection for empty options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::RadioGroupState;
    ///
    /// let state = RadioGroupState::with_selected(vec!["A", "B", "C"], 1);
    /// assert_eq!(state.selected_index(), Some(1));
    /// assert_eq!(state.selected(), Some(&"B"));
    /// ```
    pub fn with_selected(options: Vec<T>, selected: usize) -> Self {
        let selected = if options.is_empty() {
            None
        } else {
            Some(selected.min(options.len() - 1))
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
    ///
    /// Returns `None` if the options are empty.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Returns a reference to the currently selected option.
    ///
    /// Returns `None` if the options are empty or no selection exists.
    pub fn selected(&self) -> Option<&T> {
        self.options.get(self.selected?)
    }

    /// Sets the selected index.
    ///
    /// If the index is out of bounds, it will be ignored.
    pub fn set_selected(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected = Some(index);
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
/// use envision::component::{RadioGroup, RadioGroupMessage, RadioGroupOutput, RadioGroupState, Component, Focusable};
///
/// let mut state = RadioGroupState::new(vec!["Small", "Medium", "Large"]);
///
/// // Navigate to select "Medium"
/// let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Down);
/// assert_eq!(output, Some(RadioGroupOutput::SelectionChanged(1)));
///
/// // Focus the component for visual feedback
/// RadioGroup::<&str>::set_focused(&mut state, true);
///
/// // Confirm the selection
/// let output = RadioGroup::<&str>::update(&mut state, RadioGroupMessage::Confirm);
/// assert_eq!(output, Some(RadioGroupOutput::Confirmed("Medium")));
/// ```
pub struct RadioGroup<T: Clone>(PhantomData<T>);

impl<T: Clone + std::fmt::Display + 'static> Component for RadioGroup<T> {
    type State = RadioGroupState<T>;
    type Message = RadioGroupMessage;
    type Output = RadioGroupOutput<T>;

    fn init() -> Self::State {
        RadioGroupState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled || state.options.is_empty() {
            return None;
        }

        let selected = state.selected?;

        match msg {
            RadioGroupMessage::Up => {
                if selected > 0 {
                    let new_index = selected - 1;
                    state.selected = Some(new_index);
                    Some(RadioGroupOutput::SelectionChanged(new_index))
                } else {
                    None
                }
            }
            RadioGroupMessage::Down => {
                if selected < state.options.len() - 1 {
                    let new_index = selected + 1;
                    state.selected = Some(new_index);
                    Some(RadioGroupOutput::SelectionChanged(new_index))
                } else {
                    None
                }
            }
            RadioGroupMessage::Confirm => state
                .options
                .get(selected)
                .cloned()
                .map(RadioGroupOutput::Confirmed),
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let items: Vec<ListItem> = state
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let is_selected = state.selected == Some(i);
                let indicator = if is_selected { "(•)" } else { "( )" };
                let text = format!("{} {}", indicator, option);

                let style = if state.disabled {
                    theme.disabled_style()
                } else if is_selected && state.focused {
                    theme.focused_style()
                } else {
                    theme.normal_style()
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
mod tests;
