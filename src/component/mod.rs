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
mod number_input;
#[cfg(feature = "input-components")]
mod radio_group;
#[cfg(feature = "input-components")]
mod select;
#[cfg(feature = "input-components")]
mod slider;
#[cfg(feature = "input-components")]
mod switch;
#[cfg(feature = "input-components")]
mod text_area;

// Compound components
#[cfg(feature = "compound-components")]
mod alert_panel;
#[cfg(feature = "compound-components")]
mod box_plot;
#[cfg(feature = "compound-components")]
mod chart;
#[cfg(feature = "compound-components")]
mod chat_view;
#[cfg(feature = "compound-components")]
mod conversation_view;
#[cfg(feature = "compound-components")]
mod data_grid;
#[cfg(feature = "compound-components")]
pub mod dependency_graph;

#[cfg(feature = "compound-components")]
pub mod diff_viewer;
#[cfg(feature = "compound-components")]
mod event_stream;
#[cfg(feature = "compound-components")]
pub mod file_browser;
#[cfg(feature = "compound-components")]
mod flame_graph;
#[cfg(feature = "compound-components")]
mod form;
#[cfg(feature = "compound-components")]
mod heatmap;
#[cfg(feature = "compound-components")]
mod histogram;
#[cfg(feature = "compound-components")]
mod log_correlation;
#[cfg(feature = "compound-components")]
mod log_viewer;
#[cfg(feature = "compound-components")]
mod metrics_dashboard;
#[cfg(feature = "compound-components")]
pub mod pane_layout;
#[cfg(feature = "compound-components")]
mod searchable_list;
#[cfg(feature = "compound-components")]
mod span_tree;
#[cfg(feature = "compound-components")]
mod split_panel;
#[cfg(feature = "compound-components")]
mod timeline;
#[cfg(feature = "compound-components")]
pub mod treemap;

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
mod big_text;
#[cfg(feature = "display-components")]
mod calendar;
#[cfg(feature = "display-components")]
mod canvas;
#[cfg(feature = "display-components")]
pub mod code_block;
#[cfg(feature = "display-components")]
mod collapsible;
#[cfg(feature = "display-components")]
mod divider;
#[cfg(feature = "display-components")]
mod gauge;
#[cfg(feature = "display-components")]
mod help_panel;
#[cfg(feature = "display-components")]
mod key_hints;
#[cfg(feature = "display-components")]
mod multi_progress;
#[cfg(feature = "display-components")]
mod paginator;
#[cfg(feature = "display-components")]
pub mod progress_bar;
#[cfg(feature = "display-components")]
mod scroll_view;
#[cfg(feature = "display-components")]
mod scrollable_text;
#[cfg(feature = "display-components")]
mod sparkline;
#[cfg(feature = "display-components")]
mod spinner;
#[cfg(feature = "display-components")]
mod status_bar;
#[cfg(feature = "display-components")]
mod status_log;
#[cfg(feature = "display-components")]
pub mod styled_text;
#[cfg(feature = "display-components")]
pub mod terminal_output;
#[cfg(feature = "display-components")]
mod title_card;
#[cfg(feature = "display-components")]
mod toast;
#[cfg(feature = "display-components")]
mod usage_display;

// Navigation components
#[cfg(feature = "navigation-components")]
mod accordion;
#[cfg(feature = "navigation-components")]
mod breadcrumb;
#[cfg(feature = "navigation-components")]
pub mod command_palette;
#[cfg(feature = "navigation-components")]
mod menu;
#[cfg(feature = "navigation-components")]
mod router;
#[cfg(feature = "navigation-components")]
pub mod step_indicator;
#[cfg(feature = "navigation-components")]
mod tab_bar;
#[cfg(feature = "navigation-components")]
mod tabs;

// Overlay components
#[cfg(feature = "overlay-components")]
pub mod confirm_dialog;
#[cfg(feature = "overlay-components")]
mod dialog;
#[cfg(feature = "overlay-components")]
mod tooltip;

// Markdown components
#[cfg(feature = "markdown")]
pub mod markdown_renderer;

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
pub use number_input::{NumberInput, NumberInputMessage, NumberInputOutput, NumberInputState};
#[cfg(feature = "input-components")]
pub use radio_group::{RadioGroup, RadioGroupMessage, RadioGroupOutput, RadioGroupState};
#[cfg(feature = "input-components")]
pub use select::{Select, SelectMessage, SelectOutput, SelectState};
#[cfg(feature = "input-components")]
pub use slider::{Slider, SliderMessage, SliderOrientation, SliderOutput, SliderState};
#[cfg(feature = "input-components")]
pub use switch::{Switch, SwitchMessage, SwitchOutput, SwitchState};
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
pub use table::{
    date_comparator, numeric_comparator, Column, SortComparator, SortDirection, Table,
    TableMessage, TableOutput, TableRow, TableState,
};
#[cfg(feature = "data-components")]
pub use tree::{Tree, TreeMessage, TreeNode, TreeOutput, TreeState};

// Display components
#[cfg(feature = "display-components")]
pub use big_text::{big_char, big_char_width, BigText, BigTextMessage, BigTextState};
#[cfg(feature = "display-components")]
pub use calendar::{Calendar, CalendarMessage, CalendarOutput, CalendarState};
#[cfg(feature = "display-components")]
pub use canvas::{Canvas, CanvasMarker, CanvasMessage, CanvasShape, CanvasState};
#[cfg(feature = "display-components")]
pub use code_block::{CodeBlock, CodeBlockMessage, CodeBlockState};
#[cfg(feature = "display-components")]
pub use collapsible::{Collapsible, CollapsibleMessage, CollapsibleOutput, CollapsibleState};
#[cfg(feature = "display-components")]
pub use divider::{Divider, DividerMessage, DividerOrientation, DividerState};
#[cfg(feature = "display-components")]
pub use gauge::{Gauge, GaugeMessage, GaugeOutput, GaugeState, GaugeVariant, ThresholdZone};
#[cfg(feature = "display-components")]
pub use help_panel::{HelpPanel, HelpPanelMessage, HelpPanelState, KeyBinding, KeyBindingGroup};
#[cfg(feature = "display-components")]
pub use key_hints::{KeyHint, KeyHints, KeyHintsLayout, KeyHintsMessage, KeyHintsState};
#[cfg(feature = "display-components")]
pub use multi_progress::{
    MultiProgress, MultiProgressMessage, MultiProgressOutput, MultiProgressState, ProgressItem,
    ProgressItemStatus,
};
#[cfg(feature = "display-components")]
pub use paginator::{Paginator, PaginatorMessage, PaginatorOutput, PaginatorState, PaginatorStyle};
#[cfg(feature = "display-components")]
pub use progress_bar::{
    format_eta, ProgressBar, ProgressBarMessage, ProgressBarOutput, ProgressBarState,
};
#[cfg(feature = "display-components")]
pub use sparkline::{
    Sparkline, SparklineDirection, SparklineMessage, SparklineOutput, SparklineState,
};
#[cfg(feature = "display-components")]
pub use spinner::{Spinner, SpinnerMessage, SpinnerState, SpinnerStyle};

// Compound components
#[cfg(feature = "compound-components")]
pub use alert_panel::{
    AlertMetric, AlertPanel, AlertPanelMessage, AlertPanelOutput, AlertPanelState, AlertState,
    AlertThreshold,
};
#[cfg(feature = "compound-components")]
pub use box_plot::{BoxPlot, BoxPlotData, BoxPlotMessage, BoxPlotOrientation, BoxPlotState};
#[cfg(feature = "compound-components")]
pub use chart::{
    Chart, ChartKind, ChartMessage, ChartOutput, ChartState, DataSeries, ThresholdLine,
};
#[cfg(feature = "compound-components")]
pub use chat_view::{
    ChatMessage, ChatRole, ChatView, ChatViewMessage, ChatViewOutput, ChatViewState,
};
#[cfg(feature = "compound-components")]
pub use conversation_view::{
    ConversationMessage, ConversationRole, ConversationView, ConversationViewMessage,
    ConversationViewOutput, ConversationViewState, MessageBlock,
};
#[cfg(feature = "compound-components")]
pub use data_grid::{DataGrid, DataGridMessage, DataGridOutput, DataGridState};
#[cfg(feature = "compound-components")]
pub use dependency_graph::{
    layout::LayoutEdge as DependencyGraphLayoutEdge,
    layout::LayoutNode as DependencyGraphLayoutNode, DependencyGraph, DependencyGraphMessage,
    DependencyGraphOutput, DependencyGraphState, GraphEdge, GraphNode, GraphOrientation,
    NodeStatus,
};
#[cfg(feature = "compound-components")]
pub use diff_viewer::{
    DiffHunk, DiffLine, DiffLineType, DiffMode, DiffViewer, DiffViewerMessage, DiffViewerOutput,
    DiffViewerState,
};
#[cfg(feature = "compound-components")]
pub use event_stream::{
    EventLevel, EventStream, EventStreamMessage, EventStreamOutput, EventStreamState, StreamEvent,
};
#[cfg(feature = "compound-components")]
pub use file_browser::{FileBrowser, FileBrowserMessage, FileBrowserOutput, FileBrowserState};
#[cfg(feature = "compound-components")]
pub use flame_graph::{
    FlameGraph, FlameGraphMessage, FlameGraphOutput, FlameGraphState, FlameNode,
};
#[cfg(feature = "compound-components")]
pub use form::{Form, FormField, FormFieldKind, FormMessage, FormOutput, FormState, FormValue};
#[cfg(feature = "compound-components")]
pub use heatmap::{
    value_to_color, Heatmap, HeatmapColorScale, HeatmapMessage, HeatmapOutput, HeatmapState,
};
#[cfg(feature = "compound-components")]
pub use histogram::{Histogram, HistogramMessage, HistogramState};
#[cfg(feature = "compound-components")]
pub use log_correlation::{
    CorrelationEntry, CorrelationLevel, LogCorrelation, LogCorrelationMessage,
    LogCorrelationOutput, LogCorrelationState, LogStream,
};
#[cfg(feature = "compound-components")]
pub use log_viewer::{LogViewer, LogViewerMessage, LogViewerOutput, LogViewerState};
#[cfg(feature = "compound-components")]
pub use metrics_dashboard::{
    MetricKind, MetricWidget, MetricsDashboard, MetricsDashboardMessage, MetricsDashboardOutput,
    MetricsDashboardState,
};
#[cfg(feature = "compound-components")]
pub use pane_layout::{PaneLayout, PaneLayoutMessage, PaneLayoutOutput, PaneLayoutState};
#[cfg(feature = "compound-components")]
pub use searchable_list::{
    SearchableList, SearchableListMessage, SearchableListOutput, SearchableListState,
};
#[cfg(feature = "compound-components")]
pub use span_tree::{FlatSpan, SpanNode, SpanTree, SpanTreeMessage, SpanTreeOutput, SpanTreeState};
#[cfg(feature = "compound-components")]
pub use split_panel::{
    SplitOrientation, SplitPanel, SplitPanelMessage, SplitPanelOutput, SplitPanelState,
};
#[cfg(feature = "compound-components")]
pub use timeline::{
    SelectedType, Timeline, TimelineEvent, TimelineMessage, TimelineOutput, TimelineSpan,
    TimelineState,
};
#[cfg(feature = "compound-components")]
pub use treemap::{Treemap, TreemapMessage, TreemapNode, TreemapOutput, TreemapState};

#[cfg(feature = "display-components")]
pub use scroll_view::{ScrollView, ScrollViewMessage, ScrollViewState};
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
pub use styled_text::{StyledText, StyledTextMessage, StyledTextOutput, StyledTextState};
#[cfg(feature = "display-components")]
pub use terminal_output::{
    parse_ansi, AnsiSegment, TerminalOutput, TerminalOutputMessage, TerminalOutputOutput,
    TerminalOutputState,
};
#[cfg(feature = "display-components")]
pub use title_card::{TitleCard, TitleCardMessage, TitleCardState};
#[cfg(feature = "display-components")]
pub use toast::{Toast, ToastItem, ToastLevel, ToastMessage, ToastOutput, ToastState};
#[cfg(feature = "display-components")]
pub use usage_display::{
    UsageDisplay, UsageDisplayMessage, UsageDisplayState, UsageLayout, UsageMetric,
};

// Navigation components
#[cfg(feature = "navigation-components")]
pub use accordion::{Accordion, AccordionMessage, AccordionOutput, AccordionPanel, AccordionState};
#[cfg(feature = "navigation-components")]
pub use breadcrumb::{
    Breadcrumb, BreadcrumbMessage, BreadcrumbOutput, BreadcrumbSegment, BreadcrumbState,
};
#[cfg(feature = "navigation-components")]
pub use command_palette::{
    CommandPalette, CommandPaletteMessage, CommandPaletteOutput, CommandPaletteState, PaletteItem,
};
#[cfg(feature = "navigation-components")]
pub use menu::{Menu, MenuItem, MenuMessage, MenuOutput, MenuState};
#[cfg(feature = "navigation-components")]
pub use router::{NavigationMode, Router, RouterMessage, RouterOutput, RouterState};
#[cfg(feature = "navigation-components")]
pub use step_indicator::{
    StepIndicator, StepIndicatorMessage, StepIndicatorOutput, StepIndicatorState,
};
#[cfg(feature = "navigation-components")]
pub use tab_bar::{Tab, TabBar, TabBarMessage, TabBarOutput, TabBarState};
#[cfg(feature = "navigation-components")]
pub use tabs::{Tabs, TabsMessage, TabsOutput, TabsState};

// Overlay components
#[cfg(feature = "overlay-components")]
pub use confirm_dialog::{
    ConfirmDialog, ConfirmDialogMessage, ConfirmDialogOutput, ConfirmDialogResult,
    ConfirmDialogState,
};
#[cfg(feature = "overlay-components")]
pub use dialog::{Dialog, DialogButton, DialogMessage, DialogOutput, DialogState};
#[cfg(feature = "overlay-components")]
pub use tooltip::{Tooltip, TooltipMessage, TooltipOutput, TooltipPosition, TooltipState};

// Markdown components
#[cfg(feature = "markdown")]
pub use markdown_renderer::{MarkdownRenderer, MarkdownRendererMessage, MarkdownRendererState};

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
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!(
            "component_dispatch",
            component = std::any::type_name::<Self>(),
            event_kind = event.kind_name(),
        )
        .entered();

        let msg = Self::handle_event(state, event);

        #[cfg(feature = "tracing")]
        tracing::trace!(produced_message = msg.is_some(), "handle_event complete");

        if let Some(msg) = msg {
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

    /// Renders the component with focus temporarily overridden.
    ///
    /// This avoids the need to clone state just to change the focus flag
    /// before rendering. The focus state is set before rendering and
    /// restored after, using `&mut State`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Render as focused without permanently changing state:
    /// Button::view_with_focus(&mut state, frame, area, &theme, true);
    /// // state.is_focused() is restored to its original value
    /// ```
    fn view_with_focus(
        state: &mut Self::State,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        focused: bool,
    ) {
        let was_focused = Self::is_focused(state);
        Self::set_focused(state, focused);
        Self::view(state, frame, area, theme);
        Self::set_focused(state, was_focused);
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

/// A component that can be disabled.
///
/// When disabled, a component should render in a visually distinct style
/// (e.g., grayed out) and ignore interactive input events.
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "input-components")]
/// # {
/// use envision::component::{Button, Component, Disableable, Focusable};
///
/// let mut state = Button::init();
/// assert!(!Button::is_disabled(&state));
///
/// Button::disable(&mut state);
/// assert!(Button::is_disabled(&state));
///
/// Button::enable(&mut state);
/// assert!(!Button::is_disabled(&state));
/// # }
/// ```
pub trait Disableable: Component {
    /// Returns true if this component is currently disabled.
    fn is_disabled(state: &Self::State) -> bool;

    /// Sets the disabled state of this component.
    fn set_disabled(state: &mut Self::State, disabled: bool);

    /// Disables this component.
    ///
    /// Convenience method equivalent to `set_disabled(state, true)`.
    fn disable(state: &mut Self::State) {
        Self::set_disabled(state, true);
    }

    /// Enables this component.
    ///
    /// Convenience method equivalent to `set_disabled(state, false)`.
    fn enable(state: &mut Self::State) {
        Self::set_disabled(state, false);
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
#[cfg(test)]
mod tests;
