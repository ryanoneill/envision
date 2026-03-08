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

use crate::input::Event;
use crate::theme::Theme;

// Input components
#[cfg(feature = "input-components")]
mod button;
#[cfg(feature = "input-components")]
mod checkbox;
#[cfg(feature = "input-components")]
mod dropdown;
#[cfg(feature = "input-components")]
mod input_field;
#[cfg(feature = "input-components")]
mod line_input;
#[cfg(feature = "input-components")]
mod radio_group;
#[cfg(feature = "input-components")]
mod select;
#[cfg(feature = "input-components")]
mod text_area;

// Compound components
#[cfg(feature = "compound-components")]
mod chart;
#[cfg(feature = "compound-components")]
mod chat_view;
#[cfg(feature = "compound-components")]
mod data_grid;
#[cfg(feature = "compound-components")]
pub mod file_browser;
#[cfg(feature = "compound-components")]
mod form;
#[cfg(feature = "compound-components")]
mod log_viewer;
#[cfg(feature = "compound-components")]
mod metrics_dashboard;
#[cfg(feature = "compound-components")]
mod searchable_list;
#[cfg(feature = "compound-components")]
mod split_panel;

// Data components
#[cfg(feature = "data-components")]
mod loading_list;
#[cfg(feature = "data-components")]
mod selectable_list;
#[cfg(feature = "data-components")]
mod table;
#[cfg(feature = "data-components")]
mod tree;

// Display components
#[cfg(feature = "display-components")]
mod key_hints;
#[cfg(feature = "display-components")]
mod multi_progress;
#[cfg(feature = "display-components")]
mod progress_bar;
#[cfg(feature = "display-components")]
mod scrollable_text;
#[cfg(feature = "display-components")]
mod spinner;
#[cfg(feature = "display-components")]
mod status_bar;
#[cfg(feature = "display-components")]
mod status_log;
#[cfg(feature = "display-components")]
mod title_card;
#[cfg(feature = "display-components")]
mod toast;

// Navigation components
#[cfg(feature = "navigation-components")]
mod accordion;
#[cfg(feature = "navigation-components")]
mod breadcrumb;
#[cfg(feature = "navigation-components")]
mod menu;
#[cfg(feature = "navigation-components")]
mod router;
#[cfg(feature = "navigation-components")]
mod tabs;

// Overlay components
#[cfg(feature = "overlay-components")]
mod dialog;
#[cfg(feature = "overlay-components")]
mod tooltip;

// Always available
mod focus_manager;

// Input components
#[cfg(feature = "input-components")]
pub use button::{Button, ButtonMessage, ButtonOutput, ButtonState};
#[cfg(feature = "input-components")]
pub use checkbox::{Checkbox, CheckboxMessage, CheckboxOutput, CheckboxState};
#[cfg(feature = "input-components")]
pub use dropdown::{Dropdown, DropdownMessage, DropdownOutput, DropdownState};
#[cfg(feature = "input-components")]
pub use input_field::{InputField, InputFieldMessage, InputFieldOutput, InputFieldState};
#[cfg(feature = "input-components")]
pub use line_input::{LineInput, LineInputMessage, LineInputOutput, LineInputState};
#[cfg(feature = "input-components")]
pub use radio_group::{RadioGroup, RadioGroupMessage, RadioGroupOutput, RadioGroupState};
#[cfg(feature = "input-components")]
pub use select::{Select, SelectMessage, SelectOutput, SelectState};
#[cfg(feature = "input-components")]
pub use text_area::{TextArea, TextAreaMessage, TextAreaOutput, TextAreaState};

// Data components
#[cfg(feature = "data-components")]
pub use loading_list::{
    ItemState, LoadingList, LoadingListItem, LoadingListMessage, LoadingListOutput,
    LoadingListState,
};
#[cfg(feature = "data-components")]
pub use selectable_list::{
    SelectableList, SelectableListMessage, SelectableListOutput, SelectableListState,
};
#[cfg(feature = "data-components")]
pub use table::{Column, SortDirection, Table, TableMessage, TableOutput, TableRow, TableState};
#[cfg(feature = "data-components")]
pub use tree::{Tree, TreeMessage, TreeNode, TreeOutput, TreeState};

// Display components
#[cfg(feature = "display-components")]
pub use key_hints::{KeyHint, KeyHints, KeyHintsLayout, KeyHintsMessage, KeyHintsState};
#[cfg(feature = "display-components")]
pub use multi_progress::{
    MultiProgress, MultiProgressMessage, MultiProgressOutput, MultiProgressState, ProgressItem,
    ProgressItemStatus,
};
#[cfg(feature = "display-components")]
pub use progress_bar::{ProgressBar, ProgressBarMessage, ProgressBarOutput, ProgressBarState};
#[cfg(feature = "display-components")]
pub use spinner::{Spinner, SpinnerMessage, SpinnerState, SpinnerStyle};

// Compound components
#[cfg(feature = "compound-components")]
pub use chart::{Chart, ChartKind, ChartMessage, ChartOutput, ChartState, DataSeries};
#[cfg(feature = "compound-components")]
pub use chat_view::{
    ChatMessage, ChatRole, ChatView, ChatViewMessage, ChatViewOutput, ChatViewState,
};
#[cfg(feature = "compound-components")]
pub use data_grid::{DataGrid, DataGridMessage, DataGridOutput, DataGridState};
#[cfg(feature = "compound-components")]
pub use file_browser::{FileBrowser, FileBrowserMessage, FileBrowserOutput, FileBrowserState};
#[cfg(feature = "compound-components")]
pub use form::{Form, FormField, FormFieldKind, FormMessage, FormOutput, FormState, FormValue};
#[cfg(feature = "compound-components")]
pub use log_viewer::{LogViewer, LogViewerMessage, LogViewerOutput, LogViewerState};
#[cfg(feature = "compound-components")]
pub use metrics_dashboard::{
    MetricKind, MetricWidget, MetricsDashboard, MetricsDashboardMessage, MetricsDashboardOutput,
    MetricsDashboardState,
};
#[cfg(feature = "compound-components")]
pub use searchable_list::{
    SearchableList, SearchableListMessage, SearchableListOutput, SearchableListState,
};
#[cfg(feature = "compound-components")]
pub use split_panel::{
    SplitOrientation, SplitPanel, SplitPanelMessage, SplitPanelOutput, SplitPanelState,
};

#[cfg(feature = "display-components")]
pub use scrollable_text::{
    ScrollableText, ScrollableTextMessage, ScrollableTextOutput, ScrollableTextState,
};
#[cfg(feature = "display-components")]
pub use status_bar::{
    Section, StatusBar, StatusBarItem, StatusBarItemContent, StatusBarMessage, StatusBarState,
    StatusBarStyle,
};
#[cfg(feature = "display-components")]
pub use status_log::{
    StatusLog, StatusLogEntry, StatusLogLevel, StatusLogMessage, StatusLogOutput, StatusLogState,
};
#[cfg(feature = "display-components")]
pub use title_card::{TitleCard, TitleCardMessage, TitleCardState};
#[cfg(feature = "display-components")]
pub use toast::{Toast, ToastItem, ToastLevel, ToastMessage, ToastOutput, ToastState};

// Navigation components
#[cfg(feature = "navigation-components")]
pub use accordion::{Accordion, AccordionMessage, AccordionOutput, AccordionPanel, AccordionState};
#[cfg(feature = "navigation-components")]
pub use breadcrumb::{
    Breadcrumb, BreadcrumbMessage, BreadcrumbOutput, BreadcrumbSegment, BreadcrumbState,
};
#[cfg(feature = "navigation-components")]
pub use menu::{Menu, MenuItem, MenuMessage, MenuOutput, MenuState};
#[cfg(feature = "navigation-components")]
pub use router::{NavigationMode, Router, RouterMessage, RouterOutput, RouterState};
#[cfg(feature = "navigation-components")]
pub use tabs::{Tabs, TabsMessage, TabsOutput, TabsState};

// Overlay components
#[cfg(feature = "overlay-components")]
pub use dialog::{Dialog, DialogButton, DialogMessage, DialogOutput, DialogState};
#[cfg(feature = "overlay-components")]
pub use tooltip::{Tooltip, TooltipMessage, TooltipOutput, TooltipPosition, TooltipState};

// Always available
pub use focus_manager::FocusManager;

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

    /// Renders the component with optional tracing instrumentation.
    ///
    /// When the `tracing` feature is enabled, this emits a trace-level span
    /// around the [`view`](Component::view) call with the component type name
    /// and render area dimensions. When the feature is disabled, this is
    /// identical to calling `view` directly.
    fn traced_view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        #[cfg(feature = "tracing")]
        let _span = tracing::trace_span!(
            "component_view",
            component = std::any::type_name::<Self>(),
            area.x = area.x,
            area.y = area.y,
            area.width = area.width,
            area.height = area.height,
        )
        .entered();
        Self::view(state, frame, area, theme);
    }

    /// Maps an input event to a component message.
    ///
    /// This is the read-only half of event handling. It inspects the
    /// component's state and the incoming event, and returns an appropriate
    /// message if the event is relevant to this component.
    ///
    /// The default implementation returns `None` (ignores all events).
    /// Components should override this to handle keyboard input when focused.
    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        let _ = (state, event);
        None
    }

    /// Dispatches an event by mapping it to a message and updating state.
    ///
    /// This combines [`handle_event`](Component::handle_event) and
    /// [`update`](Component::update) into a single call. If the event
    /// produces a message, the message is passed to `update` and the
    /// output is returned.
    ///
    /// This is the primary method users should call for event routing.
    fn dispatch_event(state: &mut Self::State, event: &Event) -> Option<Self::Output> {
        if let Some(msg) = Self::handle_event(state, event) {
            #[cfg(feature = "tracing")]
            let _span = tracing::debug_span!(
                "component_dispatch",
                component = std::any::type_name::<Self>(),
            )
            .entered();
            let output = Self::update(state, msg);
            #[cfg(feature = "tracing")]
            tracing::trace!(has_output = output.is_some(), "update complete");
            output
        } else {
            None
        }
    }
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
