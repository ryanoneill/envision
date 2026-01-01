#![warn(missing_docs)]

//! # Envision
//!
//! A ratatui framework for collaborative TUI development with headless testing support.
//!
//! Envision provides a custom `CaptureBackend` that implements ratatui's `Backend` trait,
//! enabling you to:
//!
//! - Capture rendered frames as inspectable text or structured data
//! - Track frame history and compute diffs between renders
//! - Annotate widgets with semantic information
//! - Simulate user input for testing
//! - Run applications in headless mode for CI/testing
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use envision::backend::CaptureBackend;
//! use ratatui::Terminal;
//! use ratatui::widgets::Paragraph;
//!
//! // Create a headless terminal
//! let backend = CaptureBackend::new(80, 24);
//! let mut terminal = Terminal::new(backend).unwrap();
//!
//! // Render something
//! terminal.draw(|frame| {
//!     frame.render_widget(Paragraph::new("Hello, Envision!"), frame.area());
//! }).unwrap();
//!
//! // Capture the output
//! let output = terminal.backend().to_string();
//! println!("{}", output);
//! ```
//!
//! ## Input Simulation
//!
//! ```rust
//! use envision::input::{EventQueue, KeyCode};
//!
//! let mut queue = EventQueue::new();
//! queue.type_str("hello");
//! queue.enter();
//!
//! // Events can be consumed by your app's event loop
//! while let Some(event) = queue.pop() {
//!     // handle event...
//! }
//! ```

pub mod adapter;
pub mod annotation;
pub mod app;
pub mod backend;
pub mod component;
pub mod harness;
pub mod input;

// Re-export commonly used types
pub use adapter::DualBackend;
pub use annotation::{Annotate, Annotation, AnnotationRegistry, WidgetType};
pub use app::{
    App, AsyncCommandHandler, AsyncRuntime, AsyncRuntimeConfig, Command, DebounceSubscription,
    FilterSubscription, IntervalImmediateSubscription, Runtime, RuntimeConfig, Subscription,
    SubscriptionExt, TakeSubscription, TerminalEventSubscription, ThrottleSubscription,
    TickSubscription, TimerSubscription,
};
pub use backend::{CaptureBackend, EnhancedCell, FrameSnapshot};
pub use component::{
    Button, ButtonMessage, ButtonOutput, ButtonState, Checkbox, CheckboxMessage, CheckboxOutput,
    CheckboxState, Component, FocusManager, Focusable, InputField, InputFieldState, InputMessage,
    InputOutput, ListMessage, ListOutput, ProgressBar, ProgressBarState, ProgressMessage,
    ProgressOutput, RadioGroup, RadioGroupState, RadioMessage, RadioOutput, SelectableList,
    SelectableListState, Spinner, SpinnerMessage, SpinnerState, SpinnerStyle, TabMessage,
    TabOutput, Tabs, TabsState, TextArea, TextAreaMessage, TextAreaOutput, TextAreaState,
    Toggleable,
};
pub use harness::{Assertion, AsyncTestHarness, Snapshot, TestHarness};
pub use input::{EventQueue, SimulatedEvent};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::adapter::DualBackend;
    pub use crate::annotation::{Annotate, Annotation, AnnotationRegistry, RegionInfo, WidgetType};
    pub use crate::app::{
        App, AsyncCommandHandler, AsyncRuntime, AsyncRuntimeConfig, Command, Runtime,
        RuntimeConfig, Subscription, SubscriptionExt, TickSubscription, TimerSubscription, Update,
    };
    pub use crate::backend::{CaptureBackend, EnhancedCell, FrameSnapshot, OutputFormat};
    pub use crate::component::{
        Button, ButtonMessage, ButtonOutput, ButtonState, Checkbox, CheckboxMessage,
        CheckboxOutput, CheckboxState, Component, FocusManager, Focusable, InputField,
        InputFieldState, InputMessage, InputOutput, ListMessage, ListOutput, ProgressBar,
        ProgressBarState, ProgressMessage, ProgressOutput, RadioGroup, RadioGroupState,
        RadioMessage, RadioOutput, SelectableList, SelectableListState, Spinner, SpinnerMessage,
        SpinnerState, SpinnerStyle, TabMessage, TabOutput, Tabs, TabsState, TextArea,
        TextAreaMessage, TextAreaOutput, TextAreaState, Toggleable,
    };
    pub use crate::harness::{
        Assertion, AssertionError, AsyncTestHarness, Snapshot, SnapshotFormat, TestHarness,
    };
    pub use crate::input::{EventQueue, KeyCode, KeyModifiers, SimulatedEvent};
    pub use ratatui::prelude::*;
}
