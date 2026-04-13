//! TEA (The Elm Architecture) application framework.
//!
//! This module provides a structured way to build TUI applications using
//! the Elm-inspired unidirectional data flow pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    Application                           │
//! │                                                          │
//! │   ┌─────────┐     ┌────────┐     ┌──────────────────┐   │
//! │   │  State  │────▶│  View  │────▶│  Terminal/Frame  │   │
//! │   └─────────┘     └────────┘     └──────────────────┘   │
//! │        ▲                                                 │
//! │        │                                                 │
//! │   ┌─────────┐     ┌────────────────────┐                │
//! │   │ Update  │◀────│  Message/Events    │                │
//! │   └─────────┘     └────────────────────┘                │
//! │        │                    ▲                            │
//! │        ▼                    │                            │
//! │   ┌─────────┐     ┌────────────────────┐                │
//! │   │ Effects │────▶│  Effect Handler    │                │
//! │   └─────────┘     └────────────────────┘                │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Core Concepts
//!
//! - **State**: The complete application state (should be serializable)
//! - **Message**: Discrete events that can change state
//! - **Update**: Pure function: `(state, message) → (state, effects)`
//! - **View**: Pure function: `state → UI`
//! - **Effect**: Side effects to execute (IO, commands, etc.)
//!
//! # State Initialization
//!
//! There are two ways to provide the initial state:
//!
//! - **Standard pattern**: Use [`Runtime::new_terminal()`] or
//!   [`Runtime::virtual_terminal()`]. These call [`App::init()`] to create
//!   the initial state and any startup commands.
//!
//! - **External state pattern**: Use the `with_state` constructors
//!   ([`Runtime::new_terminal_with_state()`],
//!   [`Runtime::virtual_terminal_with_state()`], etc.). These accept a
//!   pre-built state directly and **do not call** [`App::init()`]. This is
//!   useful when initial state comes from CLI arguments, config files,
//!   databases, or test fixtures.
//!
//! Even when using `with_state` constructors, [`App::init()`] must still
//! be implemented because it is a required trait method. A simple stub
//! returning default values is sufficient.
//!
//! [`Runtime::new_terminal()`]: Runtime::new_terminal
//! [`Runtime::virtual_terminal()`]: Runtime::virtual_terminal
//! [`Runtime::new_terminal_with_state()`]: Runtime::new_terminal_with_state
//! [`Runtime::virtual_terminal_with_state()`]: Runtime::virtual_terminal_with_state
//!
//! # Example
//!
//! ```rust
//! use envision::app::{App, Command, Update};
//! use envision::input::Event;
//! use ratatui::Frame;
//!
//! // Define your state
//! #[derive(Default, Clone)]
//! struct CounterState {
//!     count: i32,
//! }
//!
//! // Define your messages
//! #[derive(Clone)]
//! enum CounterMsg {
//!     Increment,
//!     Decrement,
//!     Reset,
//! }
//!
//! // Implement the App trait
//! struct CounterApp;
//!
//! impl App for CounterApp {
//!     type State = CounterState;
//!     type Message = CounterMsg;
//!
//!     fn init() -> (Self::State, Command<Self::Message>) {
//!         (CounterState::default(), Command::none())
//!     }
//!
//!     fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
//!         match msg {
//!             CounterMsg::Increment => state.count += 1,
//!             CounterMsg::Decrement => state.count -= 1,
//!             CounterMsg::Reset => state.count = 0,
//!         }
//!         Command::none()
//!     }
//!
//!     fn view(state: &Self::State, frame: &mut Frame) {
//!         // Render the UI based on state
//!     }
//! }
//! ```

mod command;
mod command_core;
mod model;
#[cfg(feature = "serialization")]
pub mod persistence;
mod runtime;
mod runtime_core;
mod subscription;
mod update;
pub mod worker;

pub use command::{BoxedError, Command, CommandHandler};
pub use model::App;
#[cfg(feature = "serialization")]
pub use persistence::load_state;
pub use runtime::terminal::restore_terminal;
pub use runtime::{
    Runtime, RuntimeBuilder, RuntimeConfig, TerminalHook, TerminalRuntime, VirtualRuntime,
};
pub use subscription::{
    BatchSubscription, BoxedSubscription, ChannelSubscription, DebounceSubscription,
    FilterSubscription, IntervalImmediateBuilder, IntervalImmediateSubscription,
    MappedSubscription, StreamSubscription, Subscription, SubscriptionExt, TakeSubscription,
    TerminalEventSubscription, ThrottleSubscription, TickSubscription, TickSubscriptionBuilder,
    TimerSubscription, UnboundedChannelSubscription, batch, interval_immediate, terminal_events,
    tick,
};
pub use update::{FnUpdate, StateExt, Update, UpdateResult};
