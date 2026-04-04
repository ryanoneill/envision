//! A mutually exclusive option selection component.
//!
//! [`RadioGroup<T>`] provides a group of radio buttons where exactly one option
//! can be selected at a time. Unlike [`SelectableList`](super::SelectableList),
//! navigation immediately changes the selection (traditional radio button behavior).
//! State is stored in [`RadioGroupState<T>`], updated via [`RadioGroupMessage`],
//! and produces [`RadioGroupOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
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

use super::{Component, Disableable, Focusable, ViewContext};
use crate::input::{Event, KeyCode};
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
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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

impl<T: Clone + PartialEq> PartialEq for RadioGroupState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.options == other.options
            && self.selected == other.selected
            && self.focused == other.focused
            && self.disabled == other.disabled
    }
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
    /// assert_eq!(state.selected_item(), Some(&"Option A"));
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
    /// assert_eq!(state.selected_item(), Some(&"B"));
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

    /// Sets the available options.
    ///
    /// If the current selected index would be out of bounds for the new options,
    /// it is clamped to the last valid index. If the new options are empty,
    /// the selection is set to `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::RadioGroupState;
    ///
    /// let mut state = RadioGroupState::new(vec!["A", "B", "C"]);
    /// assert_eq!(state.selected_index(), Some(0));
    ///
    /// state.set_options(vec!["X", "Y"]);
    /// assert_eq!(state.options(), &["X", "Y"]);
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn set_options(&mut self, options: Vec<T>) {
        self.options = options;

        if self.options.is_empty() {
            self.selected = None;
        } else if let Some(idx) = self.selected {
            if idx >= self.options.len() {
                self.selected = Some(self.options.len() - 1);
            }
        }
    }

    /// Returns the currently selected index.
    ///
    /// Returns `None` if the options are empty.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Alias for [`selected_index()`](Self::selected_index).
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }

    /// Returns a reference to the currently selected item.
    ///
    /// Returns `None` if the options are empty or no selection exists.
    pub fn selected_item(&self) -> Option<&T> {
        self.options.get(self.selected?)
    }

    /// Sets the selected index.
    ///
    /// Pass `Some(index)` to select an option (out-of-bounds indices are
    /// ignored), or `None` to clear the selection.
    pub fn set_selected(&mut self, index: Option<usize>) {
        match index {
            Some(i) if i < self.options.len() => self.selected = Some(i),
            Some(_) => {} // Out of bounds, ignore
            None => self.selected = None,
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

    /// Sets the disabled state (builder method).
    ///
    /// Disabled radio groups do not respond to messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::RadioGroupState;
    ///
    /// let state = RadioGroupState::new(vec!["A", "B", "C"]).with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
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

impl<T: Clone + std::fmt::Display + 'static> RadioGroupState<T> {
    /// Returns true if the radio group is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Maps an input event to a radio group message.
    pub fn handle_event(&self, event: &Event) -> Option<RadioGroupMessage> {
        RadioGroup::<T>::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<RadioGroupOutput<T>> {
        RadioGroup::<T>::dispatch_event(self, event)
    }

    /// Updates the radio group state with a message, returning any output.
    pub fn update(&mut self, msg: RadioGroupMessage) -> Option<RadioGroupOutput<T>> {
        RadioGroup::<T>::update(self, msg)
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

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(RadioGroupMessage::Up),
                KeyCode::Down | KeyCode::Char('j') => Some(RadioGroupMessage::Down),
                KeyCode::Enter => Some(RadioGroupMessage::Confirm),
                _ => None,
            }
        } else {
            None
        }
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        let items: Vec<ListItem> = state
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let is_selected = state.selected == Some(i);
                let indicator = if is_selected { "(•)" } else { "( )" };
                let text = format!("{} {}", indicator, option);

                let style = if ctx.disabled {
                    theme.disabled_style()
                } else if is_selected && ctx.focused {
                    theme.focused_style()
                } else {
                    theme.normal_style()
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items);

        let mut ann = crate::annotation::Annotation::new(crate::annotation::WidgetType::RadioGroup)
            .with_id("radio_group")
            .with_focus(ctx.focused)
            .with_disabled(ctx.disabled);
        if let Some(idx) = state.selected {
            ann = ann.with_selected(true).with_value(idx.to_string());
        }
        let annotated = crate::annotation::Annotate::new(list, ann);
        frame.render_widget(annotated, area);
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

impl<T: Clone + std::fmt::Display + 'static> Disableable for RadioGroup<T> {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod tests;
