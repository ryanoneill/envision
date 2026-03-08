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
pub use runtime::{Runtime, RuntimeConfig, TerminalHook, TerminalRuntime, VirtualRuntime};
pub use subscription::{
    batch, interval_immediate, terminal_events, tick, BatchSubscription, BoxedSubscription,
    ChannelSubscription, DebounceSubscription, FilterSubscription, IntervalImmediateBuilder,
    IntervalImmediateSubscription, MappedSubscription, StreamSubscription, Subscription,
    SubscriptionExt, TakeSubscription, TerminalEventSubscription, ThrottleSubscription,
    TickSubscription, TickSubscriptionBuilder, TimerSubscription, UnboundedChannelSubscription,
};
pub use update::{FnUpdate, StateExt, Update, UpdateResult};
