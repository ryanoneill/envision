//! Composable UI components for TUI applications.
//!
//! This module provides traits for building reusable UI components that
//! follow the TEA (The Elm Architecture) pattern at a granular level.
//!
//! While the [`App`](crate::app::App) trait defines the top-level application,
//! components are smaller, composable pieces that can be combined to build
//! complex interfaces.
//!
//! # Core Traits
//!
//! - [`Component`]: The base trait for all components
//! - [`Focusable`]: Components that can receive keyboard focus
//! - [`Toggleable`]: Components that can be shown or hidden
//!
//! # Built-in Components
//!
//! - [`SelectableList`]: A scrollable list with keyboard navigation
//! - [`InputField`]: A text input field with cursor navigation
//! - [`Button`]: A clickable button with keyboard activation
//! - [`Checkbox`]: A toggleable checkbox with keyboard activation
//! - [`FocusManager`]: Focus coordination between components
//!
//! # Component vs App
//!
//! | Aspect | App | Component |
//! |--------|-----|-----------|
//! | Scope | Entire application | Part of the UI |
//! | State | Owns all state | Owns its own state |
//! | Messages | All app messages | Component-specific messages |
//! | Output | Commands (side effects) | Messages to parent |
//! | Rendering | Full frame | Specific area |
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Focusable};
//! use envision::theme::Theme;
//! use ratatui::prelude::*;
//!
//! struct Counter;
//!
//! #[derive(Clone, Default)]
//! struct CounterState {
//!     value: i32,
//!     focused: bool,
//! }
//!
//! #[derive(Clone)]
//! enum CounterMsg {
//!     Increment,
//!     Decrement,
//! }
//!
//! #[derive(Clone)]
//! enum CounterOutput {
//!     ValueChanged(i32),
//! }
//!
//! impl Component for Counter {
//!     type State = CounterState;
//!     type Message = CounterMsg;
//!     type Output = CounterOutput;
//!
//!     fn init() -> Self::State {
//!         CounterState::default()
//!     }
//!
//!     fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
//!         match msg {
//!             CounterMsg::Increment => state.value += 1,
//!             CounterMsg::Decrement => state.value -= 1,
//!         }
//!         Some(CounterOutput::ValueChanged(state.value))
//!     }
//!
//!     fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
//!         let style = if state.focused {
//!             theme.focused_style()
//!         } else {
//!             theme.normal_style()
//!         };
//!         let text = format!("Count: {}", state.value);
//!         frame.render_widget(
//!             ratatui::widgets::Paragraph::new(text).style(style),
//!             area,
//!         );
//!     }
//! }
//!
//! impl Focusable for Counter {
//!     fn is_focused(state: &Self::State) -> bool {
//!         state.focused
//!     }
//!
//!     fn set_focused(state: &mut Self::State, focused: bool) {
//!         state.focused = focused;
//!     }
//! }
//! ```

use ratatui::prelude::*;

use crate::theme::Theme;

mod accordion;
mod breadcrumb;
mod button;
mod checkbox;
mod dialog;
mod dropdown;
mod focus_manager;
mod input_field;
mod key_hints;
mod loading_list;
mod menu;
mod multi_progress;
mod progress_bar;
mod radio_group;
mod router;
mod select;
mod selectable_list;
mod spinner;
mod status_bar;
mod status_log;
mod table;
mod tabs;
mod text_area;
mod toast;
mod tooltip;
mod tree;

pub use accordion::{Accordion, AccordionMessage, AccordionOutput, AccordionPanel, AccordionState};
pub use breadcrumb::{
    Breadcrumb, BreadcrumbMessage, BreadcrumbOutput, BreadcrumbSegment, BreadcrumbState,
};
pub use button::{Button, ButtonMessage, ButtonOutput, ButtonState};
pub use checkbox::{Checkbox, CheckboxMessage, CheckboxOutput, CheckboxState};
pub use dialog::{Dialog, DialogButton, DialogMessage, DialogOutput, DialogState};
pub use dropdown::{Dropdown, DropdownMessage, DropdownOutput, DropdownState};
pub use focus_manager::FocusManager;
pub use input_field::{InputField, InputFieldState, InputMessage, InputOutput};
pub use key_hints::{
    KeyHint, KeyHints, KeyHintsLayout, KeyHintsMessage, KeyHintsOutput, KeyHintsState,
};
pub use loading_list::{
    ItemState, LoadingList, LoadingListItem, LoadingListMessage, LoadingListOutput,
    LoadingListState,
};
pub use menu::{Menu, MenuItem, MenuMessage, MenuOutput, MenuState};
pub use multi_progress::{
    MultiProgress, MultiProgressMessage, MultiProgressOutput, MultiProgressState, ProgressItem,
    ProgressItemStatus,
};
pub use progress_bar::{ProgressBar, ProgressBarState, ProgressMessage, ProgressOutput};
pub use radio_group::{RadioGroup, RadioGroupState, RadioMessage, RadioOutput};
pub use router::{NavigationMode, Router, RouterMessage, RouterOutput, RouterState};
pub use select::{Select, SelectMessage, SelectOutput, SelectState};
pub use selectable_list::{ListMessage, ListOutput, SelectableList, SelectableListState};
pub use spinner::{Spinner, SpinnerMessage, SpinnerState, SpinnerStyle};
pub use status_bar::{
    Section, StatusBar, StatusBarItem, StatusBarItemContent, StatusBarMessage, StatusBarOutput,
    StatusBarState, StatusBarStyle,
};
pub use status_log::{
    StatusLog, StatusLogEntry, StatusLogLevel, StatusLogMessage, StatusLogOutput, StatusLogState,
};
pub use table::{Column, SortDirection, Table, TableMessage, TableOutput, TableRow, TableState};
pub use tabs::{TabMessage, TabOutput, Tabs, TabsState};
pub use text_area::{TextArea, TextAreaMessage, TextAreaOutput, TextAreaState};
pub use toast::{Toast, ToastItem, ToastLevel, ToastMessage, ToastOutput, ToastState};
pub use tooltip::{Tooltip, TooltipMessage, TooltipOutput, TooltipPosition, TooltipState};
pub use tree::{Tree, TreeMessage, TreeNode, TreeOutput, TreeState};

/// A composable UI component with its own state and message handling.
///
/// Components are the building blocks of complex TUI applications. Each
/// component manages its own state, handles its own messages, and renders
/// to a specific area of the screen.
///
/// # Associated Types
///
/// - `State`: The component's internal state. Derive `Clone` if you need snapshots.
/// - `Message`: Messages the component can receive from its parent or from
///   user interaction.
/// - `Output`: Messages the component emits to communicate with its parent.
///   Use `()` if the component doesn't need to communicate outward.
///
/// # Design Pattern
///
/// Components follow the same TEA pattern as [`App`](crate::app::App), but
/// at a smaller scale:
///
/// 1. Parent sends `Message` to component
/// 2. Component updates its `State`
/// 3. Component optionally emits `Output` to parent
/// 4. Component renders itself to its designated area
pub trait Component: Sized {
    /// The component's internal state type.
    ///
    /// This should contain all data needed to render the component.
    /// Deriving `Clone` is recommended but not required.
    type State;

    /// Messages this component can receive.
    ///
    /// These typically come from user input or parent components.
    type Message: Clone;

    /// Messages this component can emit to its parent.
    ///
    /// Use `()` if the component doesn't need to communicate upward.
    /// This enables child-to-parent communication without tight coupling.
    type Output: Clone;

    /// Initialize the component state.
    ///
    /// Returns the initial state for this component.
    fn init() -> Self::State;

    /// Update component state based on a message.
    ///
    /// Returns an optional output message for the parent to handle.
    /// Return `None` if no parent notification is needed.
    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output>;

    /// Render the component to the given area.
    ///
    /// Unlike [`App::view`](crate::app::App::view) which renders to the full
    /// frame, components render to a specific `Rect` area provided by their
    /// parent.
    ///
    /// The `theme` parameter provides the color scheme to use for rendering.
    /// Use [`Theme::default()`] for the standard color scheme, or
    /// [`Theme::nord()`] for the Nord color palette.
    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme);
}

/// A component that can receive keyboard focus.
///
/// Focus is used to determine which component receives keyboard input
/// when multiple components are on screen. Only one component should
/// typically be focused at a time.
///
/// # Visual Feedback
///
/// Components should provide visual feedback when focused, such as:
/// - Highlighted borders
/// - Changed colors
/// - Cursor visibility
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, Focusable};
/// use envision::theme::Theme;
/// use ratatui::prelude::*;
///
/// struct TextInput;
///
/// #[derive(Clone, Default)]
/// struct TextInputState {
///     value: String,
///     focused: bool,
/// }
///
/// # #[derive(Clone)]
/// # enum TextInputMsg {}
/// # #[derive(Clone)]
/// # enum TextInputOutput {}
/// #
/// # impl Component for TextInput {
/// #     type State = TextInputState;
/// #     type Message = TextInputMsg;
/// #     type Output = TextInputOutput;
/// #     fn init() -> Self::State { TextInputState::default() }
/// #     fn update(_: &mut Self::State, _: Self::Message) -> Option<Self::Output> { None }
/// #     fn view(_: &Self::State, _: &mut Frame, _: Rect, _: &Theme) {}
/// # }
/// #
/// impl Focusable for TextInput {
///     fn is_focused(state: &Self::State) -> bool {
///         state.focused
///     }
///
///     fn set_focused(state: &mut Self::State, focused: bool) {
///         state.focused = focused;
///     }
/// }
/// ```
pub trait Focusable: Component {
    /// Returns true if this component is currently focused.
    fn is_focused(state: &Self::State) -> bool;

    /// Sets the focus state of this component.
    fn set_focused(state: &mut Self::State, focused: bool);

    /// Gives focus to this component.
    ///
    /// Convenience method equivalent to `set_focused(state, true)`.
    fn focus(state: &mut Self::State) {
        Self::set_focused(state, true);
    }

    /// Removes focus from this component.
    ///
    /// Convenience method equivalent to `set_focused(state, false)`.
    fn blur(state: &mut Self::State) {
        Self::set_focused(state, false);
    }
}

/// A component that can be shown or hidden.
///
/// This is useful for panels, dialogs, and other UI elements that
/// can be toggled on and off. When hidden, the component should not
/// be rendered or receive input.
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, Toggleable};
/// use envision::theme::Theme;
/// use ratatui::prelude::*;
///
/// struct HelpPanel;
///
/// #[derive(Clone)]
/// struct HelpPanelState {
///     visible: bool,
///     content: String,
/// }
///
/// # #[derive(Clone)]
/// # enum HelpPanelMsg {}
/// # #[derive(Clone)]
/// # enum HelpPanelOutput {}
/// #
/// # impl Component for HelpPanel {
/// #     type State = HelpPanelState;
/// #     type Message = HelpPanelMsg;
/// #     type Output = HelpPanelOutput;
/// #     fn init() -> Self::State { HelpPanelState { visible: false, content: String::new() } }
/// #     fn update(_: &mut Self::State, _: Self::Message) -> Option<Self::Output> { None }
/// #     fn view(_: &Self::State, _: &mut Frame, _: Rect, _: &Theme) {}
/// # }
/// #
/// impl Toggleable for HelpPanel {
///     fn is_visible(state: &Self::State) -> bool {
///         state.visible
///     }
///
///     fn set_visible(state: &mut Self::State, visible: bool) {
///         state.visible = visible;
///     }
/// }
///
/// // Usage:
/// let mut state = HelpPanel::init();
/// assert!(!HelpPanel::is_visible(&state));
///
/// HelpPanel::toggle(&mut state);
/// assert!(HelpPanel::is_visible(&state));
///
/// HelpPanel::hide(&mut state);
/// assert!(!HelpPanel::is_visible(&state));
/// ```
pub trait Toggleable: Component {
    /// Returns true if this component is currently visible.
    fn is_visible(state: &Self::State) -> bool;

    /// Sets the visibility of this component.
    fn set_visible(state: &mut Self::State, visible: bool);

    /// Toggles the visibility of this component.
    fn toggle(state: &mut Self::State) {
        let visible = Self::is_visible(state);
        Self::set_visible(state, !visible);
    }

    /// Shows this component.
    ///
    /// Convenience method equivalent to `set_visible(state, true)`.
    fn show(state: &mut Self::State) {
        Self::set_visible(state, true);
    }

    /// Hides this component.
    ///
    /// Convenience method equivalent to `set_visible(state, false)`.
    fn hide(state: &mut Self::State) {
        Self::set_visible(state, false);
    }
}

#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;
