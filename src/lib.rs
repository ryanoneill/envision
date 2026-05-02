#![warn(missing_docs)]

//! # Envision
//!
//! A ratatui framework for building TUI applications with first-class support for
//! both interactive terminal use and programmatic control (AI agents, automation, testing).
//!
//! ## Two Runtime Modes
//!
//! Envision provides two distinct ways to run your application:
//!
//! ### Terminal Mode - For Interactive Use
//!
//! ```rust,no_run
//! # use envision::prelude::*;
//! # struct MyApp;
//! # #[derive(Default, Clone)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! #[tokio::main]
//! async fn main() -> envision::Result<()> {
//!     let _final_state = Runtime::<MyApp, _>::terminal_builder()?
//!         .build()?
//!         .run_terminal()
//!         .await?;
//!     Ok(())
//! }
//! ```
//!
//! Or without your own tokio runtime:
//!
//! ```rust,no_run
//! # use envision::prelude::*;
//! # struct MyApp;
//! # #[derive(Default, Clone)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! fn main() -> envision::Result<()> {
//!     let _final_state = Runtime::<MyApp, _>::terminal_builder()?
//!         .build()?
//!         .run_terminal_blocking()?;
//!     Ok(())
//! }
//! ```
//!
//! ### Virtual Terminal Mode - For Programmatic Control
//!
//! ```rust
//! # use envision::prelude::*;
//! # struct MyApp;
//! # #[derive(Default, Clone)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     fn init() -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! // Create a virtual terminal
//! let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
//!
//! // Inject events programmatically
//! vt.send(Event::char('j'));
//! vt.tick()?;
//!
//! // Inspect the display
//! println!("{}", vt.display());
//! # Ok::<(), envision::EnvisionError>(())
//! ```
//!
//! The same application code works in both modes - your `App` implementation
//! doesn't need to know which mode it's running in.
//!
//! ## The Elm Architecture (TEA)
//!
//! Envision uses The Elm Architecture pattern:
//!
//! - **State**: Your application's data model
//! - **Message**: Events that can update state
//! - **Update**: Pure function that produces new state from old state + message
//! - **View**: Pure function that renders state to the UI
//!
//! ```rust
//! use envision::prelude::*;
//!
//! struct MyApp;
//!
//! #[derive(Default, Clone)]
//! struct MyState;
//!
//! #[derive(Clone)]
//! enum MyMsg {}
//!
//! impl App for MyApp {
//!     type State = MyState;
//!     type Message = MyMsg;
//!
//!     fn init() -> (Self::State, Command<Self::Message>) {
//!         (MyState, Command::none())
//!     }
//!     fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
//!         Command::none()
//!     }
//!     fn view(state: &Self::State, frame: &mut Frame) {
//!         // Render UI
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - **Capture rendered frames** as inspectable text or structured data
//! - **Track frame history** and compute diffs between renders
//! - **Annotate widgets** with semantic information for accessibility and testing
//! - **Inject events programmatically** for automation and AI agents
//! - **Async support** with tokio integration for subscriptions and commands
//! - **Component library** with common UI elements (buttons, inputs, lists, etc.)
//! - **Theming** support for consistent styling

pub mod adapter;
pub mod annotation;
pub mod app;
pub mod backend;
#[cfg(feature = "clipboard")]
pub(crate) mod clipboard;
pub mod component;
pub mod error;
pub mod harness;
pub mod input;
pub mod layout;
pub mod overlay;
pub mod scroll;
pub mod style;
pub mod theme;
#[cfg(feature = "input-components")]
pub(crate) mod undo;
pub mod util;

/// Terminal lifecycle utilities.
///
/// Provides `restore()` for cleaning up terminal state in panic
/// handlers or other non-runtime cleanup paths.
pub mod terminal {
    pub use crate::app::restore_terminal as restore;
}

// Re-export commonly used types
pub use adapter::{DualBackend, DualBackendBuilder};
pub use annotation::{
    Annotate, AnnotatedOutput, Annotation, AnnotationArea, AnnotationRegistry, WidgetAnnotation,
    WidgetType,
};
#[cfg(feature = "serialization")]
pub use app::load_state;
pub use app::{
    App, BatchSubscription, BoxedSubscription, ChannelSubscription, Command, CommandHandler,
    DebounceSubscription, FilterSubscription, FnUpdate, IntervalImmediateBuilder,
    IntervalImmediateSubscription, MappedSubscription, Runtime, RuntimeBuilder, RuntimeConfig,
    StateExt, StreamSubscription, Subscription, SubscriptionExt, TakeSubscription,
    TerminalEventSubscription, TerminalHook, TerminalRuntime, ThrottleSubscription,
    TickSubscription, TickSubscriptionBuilder, TimerSubscription, UnboundedChannelSubscription,
    Update, UpdateResult, VirtualRuntime, batch, interval_immediate, terminal_events, tick,
};
pub use backend::{CaptureBackend, EnhancedCell, FrameSnapshot};
// Core component traits and utilities (always available)
pub use component::{Component, EventContext, FocusManager, RenderContext, Toggleable};

// Input components
#[cfg(feature = "input-components")]
pub use component::{
    Button, ButtonMessage, ButtonOutput, ButtonState, Checkbox, CheckboxMessage, CheckboxOutput,
    CheckboxState, Dropdown, DropdownMessage, DropdownOutput, DropdownState, InputField,
    InputFieldMessage, InputFieldOutput, InputFieldState, LineInput, LineInputMessage,
    LineInputOutput, LineInputState, NumberInput, NumberInputMessage, NumberInputOutput,
    NumberInputState, RadioGroup, RadioGroupMessage, RadioGroupOutput, RadioGroupState, Select,
    SelectMessage, SelectOutput, SelectState, Slider, SliderMessage, SliderOrientation,
    SliderOutput, SliderState, Switch, SwitchMessage, SwitchOutput, SwitchState, TextArea,
    TextAreaMessage, TextAreaOutput, TextAreaState,
};

// Data components
#[cfg(feature = "data-components")]
pub use component::{
    Cell, CellStyle, Column, InitialSort, ItemState, LoadingList, LoadingListItem,
    LoadingListMessage, LoadingListOutput, LoadingListState, RowStatus, SelectableList,
    SelectableListMessage, SelectableListOutput, SelectableListState, SortDirection, SortKey,
    Table, TableMessage, TableOutput, TableRow, TableState, Tree, TreeMessage, TreeNode,
    TreeOutput, TreeState,
};

// Display components
#[cfg(feature = "display-components")]
pub use component::{
    BigText, BigTextMessage, BigTextState, Calendar, CalendarMessage, CalendarOutput,
    CalendarState, Canvas, CanvasMarker, CanvasMessage, CanvasShape, CanvasState, CodeBlock,
    CodeBlockMessage, CodeBlockState, Collapsible, CollapsibleMessage, CollapsibleOutput,
    CollapsibleState, Divider, DividerMessage, DividerOrientation, DividerState, Gauge,
    GaugeMessage, GaugeOrientation, GaugeOutput, GaugeState, GaugeVariant, HelpPanel,
    HelpPanelMessage, HelpPanelState, KeyBinding, KeyBindingGroup, KeyHint, KeyHints,
    KeyHintsLayout, KeyHintsMessage, KeyHintsState, MultiProgress, MultiProgressMessage,
    MultiProgressOutput, MultiProgressState, Paginator, PaginatorMessage, PaginatorOutput,
    PaginatorState, PaginatorStyle, ProgressBar, ProgressBarMessage, ProgressBarOutput,
    ProgressBarState, ProgressItem, ProgressItemStatus, ResourceGauge, ResourceGaugeMessage,
    ResourceGaugeOutput, ResourceGaugeState, ScrollView, ScrollViewMessage, ScrollViewState,
    ScrollableText, ScrollableTextMessage, ScrollableTextOutput, ScrollableTextState, Section,
    Sparkline, SparklineDirection, SparklineMessage, SparklineOutput, SparklineState, Spinner,
    SpinnerMessage, SpinnerState, SpinnerStyle, StatusBar, StatusBarItem, StatusBarItemContent,
    StatusBarMessage, StatusBarState, StatusBarStyle, StatusLog, StatusLogEntry, StatusLogLevel,
    StatusLogMessage, StatusLogOutput, StatusLogState, StyledText, StyledTextMessage,
    StyledTextOutput, StyledTextState, TerminalOutput, TerminalOutputMessage, TerminalOutputOutput,
    TerminalOutputState, ThresholdZone, TitleCard, TitleCardMessage, TitleCardState, Toast,
    ToastItem, ToastLevel, ToastMessage, ToastOutput, ToastState, UsageDisplay,
    UsageDisplayMessage, UsageDisplayState, UsageLayout, UsageMetric, big_char, big_char_width,
    format_eta,
};

// Navigation components
#[cfg(feature = "navigation-components")]
pub use component::{
    Accordion, AccordionMessage, AccordionOutput, AccordionPanel, AccordionState, Breadcrumb,
    BreadcrumbMessage, BreadcrumbOutput, BreadcrumbSegment, BreadcrumbState, CommandPalette,
    CommandPaletteMessage, CommandPaletteOutput, CommandPaletteState, Menu, MenuItem, MenuMessage,
    MenuOutput, MenuState, NavigationMode, PaletteItem, Router, RouterMessage, RouterOutput,
    RouterState, StepIndicator, StepIndicatorMessage, StepIndicatorOutput, StepIndicatorState, Tab,
    TabBar, TabBarMessage, TabBarOutput, TabBarState, Tabs, TabsMessage, TabsOutput, TabsState,
};

// Compound components
#[cfg(feature = "compound-components")]
pub use component::{
    AlertMetric,
    AlertPanel,
    AlertPanelMessage,
    AlertPanelOutput,
    AlertPanelState,
    AlertState,
    AlertThreshold,
    BarMode,
    BinMethod,
    // Diagram
    BoundingBox,
    BoxPlot,
    BoxPlotData,
    BoxPlotMessage,
    BoxPlotOrientation,
    BoxPlotState,
    Chart,
    ChartAnnotation,
    ChartKind,
    ChartMessage,
    ChartOutput,
    ChartState,
    ConversationMessage,
    ConversationRole,
    ConversationView,
    ConversationViewMessage,
    ConversationViewOutput,
    ConversationViewState,
    DataGrid,
    DataGridMessage,
    DataGridOutput,
    DataGridState,
    DataSeries,
    Diagram,
    DiagramCluster,
    DiagramEdge,
    DiagramMessage,
    DiagramNode,
    DiagramOrientation,
    DiagramOutput,
    DiagramState,
    DiffHunk,
    DiffLine,
    DiffLineType,
    DiffMode,
    DiffViewer,
    DiffViewerMessage,
    DiffViewerOutput,
    DiffViewerState,
    EdgePath,
    EdgeStyle,
    EventLevel,
    EventStream,
    EventStreamMessage,
    EventStreamOutput,
    EventStreamState,
    FileBrowser,
    FileBrowserMessage,
    FileBrowserOutput,
    FileBrowserState,
    FlameGraph,
    FlameGraphMessage,
    FlameGraphOutput,
    FlameGraphState,
    FlameNode,
    Form,
    FormField,
    FormFieldKind,
    FormMessage,
    FormOutput,
    FormState,
    FormValue,
    Heatmap,
    HeatmapColorScale,
    HeatmapMessage,
    HeatmapOutput,
    HeatmapState,
    Histogram,
    HistogramMessage,
    HistogramState,
    LayoutMode,
    LayoutResult,
    LogCorrelation,
    LogCorrelationMessage,
    LogCorrelationOutput,
    LogCorrelationState,
    LogStream,
    LogViewer,
    LogViewerMessage,
    LogViewerOutput,
    LogViewerState,
    MessageBlock,
    MessageHandle,
    MetricKind,
    MetricWidget,
    MetricsDashboard,
    MetricsDashboardMessage,
    MetricsDashboardOutput,
    MetricsDashboardState,
    NodePosition,
    NodeShape,
    NodeStatus,
    PaneLayout,
    PaneLayoutMessage,
    PaneLayoutOutput,
    PaneLayoutState,
    PathSegment,
    RenderMode,
    SearchableList,
    SearchableListMessage,
    SearchableListOutput,
    SearchableListState,
    SelectedType,
    SpanNode,
    SpanTree,
    SpanTreeMessage,
    SpanTreeOutput,
    SpanTreeState,
    SplitOrientation,
    SplitPanel,
    SplitPanelMessage,
    SplitPanelOutput,
    SplitPanelState,
    StreamEvent,
    ThresholdLine,
    Timeline,
    TimelineEvent,
    TimelineMessage,
    TimelineOutput,
    TimelineSpan,
    TimelineState,
    Treemap,
    TreemapMessage,
    TreemapNode,
    TreemapOutput,
    TreemapState,
    Viewport2D,
};

// Overlay components
#[cfg(feature = "overlay-components")]
pub use component::{
    ConfirmDialog, ConfirmDialogMessage, ConfirmDialogOutput, ConfirmDialogResult,
    ConfirmDialogState, Dialog, DialogButton, DialogMessage, DialogOutput, DialogState, Tooltip,
    TooltipMessage, TooltipOutput, TooltipPosition, TooltipState,
};
// Markdown components
#[cfg(feature = "markdown")]
pub use component::{MarkdownRenderer, MarkdownRendererMessage, MarkdownRendererState};

pub use error::{BoxedError, EnvisionError, Result};
pub use harness::{AppHarness, Assertion, Snapshot, TestHarness};
pub use input::{
    Event, EventQueue, Key, KeyEvent, KeyEventKind, Modifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
pub use overlay::{Overlay, OverlayAction, OverlayStack};
pub use scroll::{ScrollState, render_scrollbar, render_scrollbar_inside_border};
pub use theme::Theme;

/// Prelude module for convenient imports.
///
/// Provides all framework types and component types needed by most applications.
///
/// ```rust
/// use envision::prelude::*;
///
/// // Core framework types are available:
/// let focus: FocusManager<&str> = FocusManager::new(vec!["a", "b"]);
/// assert_eq!(focus.len(), 2);
/// ```
pub mod prelude {
    // Core framework
    pub use crate::app::{
        App, Command, Runtime, RuntimeBuilder, RuntimeConfig, TerminalRuntime, VirtualRuntime,
    };

    // Subscriptions
    pub use crate::app::{
        BoxedSubscription, ChannelSubscription, Subscription, SubscriptionExt, Update, batch,
        interval_immediate, tick,
    };

    // Input
    pub use crate::input::{
        Event, EventQueue, Key, KeyEvent, KeyEventKind, Modifiers, MouseButton, MouseEvent,
        MouseEventKind,
    };

    // Overlay
    pub use crate::overlay::{Overlay, OverlayAction, OverlayStack};

    // Theme
    pub use crate::theme::Theme;

    // Scroll infrastructure
    pub use crate::scroll::ScrollState;

    // All component types (the primary user-facing API)
    pub use crate::component::*;

    // Testing essentials
    pub use crate::backend::{CaptureBackend, EnhancedCell};
    pub use crate::harness::{AppHarness, TestHarness};

    // Layout and style re-exports (replacing `pub use ratatui::prelude::*`)
    pub use crate::layout::{
        Alignment, Constraint, Direction, Frame, Layout, Margin, Position, Rect, Size, Terminal,
    };
    pub use crate::style::{Color, Modifier, Style, Stylize};

    // Text types
    pub use ratatui::text::{Line, Span, Text};

    // Widget traits
    pub use ratatui::widgets::{StatefulWidget, Widget};
}
