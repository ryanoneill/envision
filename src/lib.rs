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
//! ```rust,ignore
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     Runtime::<MyApp>::new_terminal()?.run_terminal().await
//! }
//! ```
//!
//! ### Virtual Terminal Mode - For Programmatic Control
//!
//! ```rust,ignore
//! // Create a virtual terminal
//! let mut vt = Runtime::<MyApp>::virtual_terminal(80, 24)?;
//!
//! // Inject events programmatically
//! vt.send(Event::key(KeyCode::Char('j')));
//! vt.tick()?;
//!
//! // Inspect the display
//! println!("{}", vt.display());
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
//! ```rust,ignore
//! struct MyApp;
//!
//! impl App for MyApp {
//!     type State = MyState;
//!     type Message = MyMsg;
//!
//!     fn init() -> (Self::State, Command<Self::Message>) { /* ... */ }
//!     fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> { /* ... */ }
//!     fn view(state: &Self::State, frame: &mut Frame) { /* ... */ }
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
pub mod component;
pub mod harness;
pub mod input;
pub mod overlay;
pub mod theme;

// Re-export commonly used types
pub use adapter::{DualBackend, DualBackendBuilder};
pub use annotation::{Annotate, Annotation, AnnotationRegistry, WidgetType};
pub use app::{
    App, Command, DebounceSubscription, FilterSubscription, IntervalImmediateSubscription, Runtime,
    RuntimeConfig, Subscription, SubscriptionExt, TakeSubscription, TerminalEventSubscription,
    ThrottleSubscription, TickSubscription, TimerSubscription,
};
pub use backend::{CaptureBackend, EnhancedCell, FrameSnapshot};
pub use component::{
    Accordion, AccordionMessage, AccordionOutput, AccordionPanel, AccordionState, Breadcrumb,
    BreadcrumbMessage, BreadcrumbOutput, BreadcrumbSegment, BreadcrumbState, Button, ButtonMessage,
    ButtonOutput, ButtonState, Checkbox, CheckboxMessage, CheckboxOutput, CheckboxState, Column,
    Component, Dialog, DialogButton, DialogMessage, DialogOutput, DialogState, Dropdown,
    DropdownMessage, DropdownOutput, DropdownState, FocusManager, Focusable, InputField,
    InputFieldMessage, InputFieldOutput, InputFieldState, Menu, MenuItem, MenuMessage, MenuOutput,
    MenuState, ProgressBar, ProgressBarMessage, ProgressBarOutput, ProgressBarState, RadioGroup,
    RadioGroupMessage, RadioGroupOutput, RadioGroupState, Select, SelectMessage, SelectOutput,
    SelectState, SelectableList, SelectableListMessage, SelectableListOutput, SelectableListState,
    SortDirection, Spinner, SpinnerMessage, SpinnerState, SpinnerStyle, StatusBar, StatusBarItem,
    StatusBarMessage, StatusBarOutput, StatusBarState, StatusBarStyle, Table, TableMessage,
    TableOutput, TableRow, TableState, Tabs, TabsMessage, TabsOutput, TabsState, TextArea,
    TextAreaMessage, TextAreaOutput,
    TextAreaState, Toast, ToastItem, ToastLevel, ToastMessage, ToastOutput, ToastState, Toggleable,
    Tooltip, TooltipMessage, TooltipOutput, TooltipPosition, TooltipState, Tree, TreeMessage,
    TreeNode, TreeOutput, TreeState,
};
pub use harness::{AppHarness, Assertion, Snapshot, TestHarness};
pub use input::{Event, EventQueue};
pub use overlay::{Overlay, OverlayAction, OverlayStack};
pub use theme::Theme;

/// Prelude module for convenient imports.
///
/// Provides all framework types and component types needed by most applications.
///
/// ```rust,ignore
/// use envision::prelude::*;
///
/// // All component types are now available:
/// let button = ButtonState::new("Submit");
/// let checkbox = CheckboxState::new("Accept");
/// ```
pub mod prelude {
    // Core framework
    pub use crate::app::{App, Command, Runtime, RuntimeConfig};

    // Subscriptions
    pub use crate::app::{Subscription, SubscriptionExt, Update};

    // Input
    pub use crate::input::{Event, EventQueue, KeyCode, KeyModifiers};

    // Overlay
    pub use crate::overlay::{Overlay, OverlayAction, OverlayStack};

    // Theme
    pub use crate::theme::Theme;

    // All component types (the primary user-facing API)
    pub use crate::component::*;

    // Testing essentials
    pub use crate::backend::{CaptureBackend, EnhancedCell};
    pub use crate::harness::{AppHarness, TestHarness};

    // Ratatui re-export
    pub use ratatui::prelude::*;
}
