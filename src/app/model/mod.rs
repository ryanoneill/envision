//! Core App trait defining the TEA application structure.
//!
//! # Two Construction Patterns
//!
//! Applications can be started in two ways, which affects whether
//! [`App::init()`] is called:
//!
//! ## Standard pattern — `init()` creates the state
//!
//! Use [`Runtime::terminal_builder()`] or [`Runtime::virtual_builder()`].
//! These call [`App::init()`] internally to create the initial state and
//! any startup commands.
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
//! let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
//! # Ok::<(), envision::EnvisionError>(())
//! ```
//!
//! ## External state pattern — `init()` is bypassed
//!
//! Use the builder's `.state()` method to provide a pre-built state directly.
//! [`App::init()`] is **never called**. This is useful when initial state
//! comes from external sources such as CLI arguments, config files, or
//! databases.
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
//! let state = MyState::default();
//! let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24)
//!     .state(state, Command::none())
//!     .build()?;
//! # Ok::<(), envision::EnvisionError>(())
//! ```
//!
//! When using `.state()`, `App::init()` does **not** need to be
//! implemented — the default implementation will panic if called, which
//! is safe because `.state()` prevents it from being called.
//!
//! [`Runtime::terminal_builder()`]: crate::app::Runtime::terminal_builder
//! [`Runtime::virtual_builder()`]: crate::app::Runtime::virtual_builder

use ratatui::Frame;

use super::command::Command;
use crate::input::Event;

mod optional_args;
pub use optional_args::OptionalArgs;

/// The core trait for TEA-style applications.
///
/// This trait defines the structure of an application following
/// The Elm Architecture pattern:
///
/// - `State`: The complete application state
/// - `Message`: Events that can modify state
/// - `init`: Initialize state and any startup commands
/// - `update`: Handle messages and produce new state
/// - `view`: Render the current state
///
/// # Type Parameters
///
/// - `State`: Your application's state type. Derive `Clone` if you need snapshots.
/// - `Message`: The type representing all possible events/actions.
///
/// # Construction
///
/// There are two ways to start an application, which determine whether
/// [`init()`](App::init) is called:
///
/// - **Standard**: [`Runtime::terminal_builder()`](crate::app::Runtime::terminal_builder) and
///   [`Runtime::virtual_builder()`](crate::app::Runtime::virtual_builder) call `init()`
///   to create the initial state (when `.state()` is not called on the builder).
/// - **External state**: Use `.state(s, cmd)` on the builder to provide a
///   pre-built state and **skip** `init()` entirely. In this case, `init()`
///   does not need to be implemented.
///
/// # Examples
///
/// ## Standard pattern — implementing `init()`
///
/// ```rust
/// use envision::app::{App, Command};
/// use ratatui::Frame;
///
/// struct MyApp;
///
/// #[derive(Clone, Default)]
/// struct MyState {
///     value: String,
/// }
///
/// #[derive(Clone)]
/// enum MyMessage {
///     SetValue(String),
///     Clear,
/// }
///
/// impl App for MyApp {
///     type State = MyState;
///     type Message = MyMessage;
///
///     fn init() -> (Self::State, Command<Self::Message>) {
///         (MyState::default(), Command::none())
///     }
///
///     fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
///         match msg {
///             MyMessage::SetValue(v) => state.value = v,
///             MyMessage::Clear => state.value.clear(),
///         }
///         Command::none()
///     }
///
///     fn view(state: &Self::State, frame: &mut Frame) {
///         // Render UI
///     }
/// }
/// ```
///
/// ## External state pattern — omitting `init()`
///
/// When using `with_state` constructors, `init()` can be omitted:
///
/// ```rust
/// use envision::app::{App, Command};
/// use ratatui::Frame;
///
/// struct ExternalApp;
///
/// struct ExternalState {
///     config_value: String,
/// }
///
/// #[derive(Clone)]
/// enum ExternalMsg {
///     Update(String),
/// }
///
/// impl App for ExternalApp {
///     type State = ExternalState;
///     type Message = ExternalMsg;
///
///     // init() is not implemented — this app uses with_state constructors
///
///     fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
///         match msg {
///             ExternalMsg::Update(v) => state.config_value = v,
///         }
///         Command::none()
///     }
///
///     fn view(state: &Self::State, frame: &mut Frame) {
///         // Render UI
///     }
/// }
/// ```
pub trait App: Sized {
    /// The application state type.
    ///
    /// This should contain all data needed to render the UI.
    /// Deriving `Clone` is recommended but not required.
    type State;

    /// The message type representing all possible events.
    ///
    /// This should be an enum covering all ways the state can change.
    type Message: Send + 'static;

    /// Initializes the application state and optional startup commands.
    ///
    /// This is called automatically when using the builder without `.state()`.
    /// When using `.state(s, cmd)` on the builder, this method is **not
    /// called** — the provided state is used directly instead.
    ///
    /// # Panics
    ///
    /// The default implementation panics if called without being overridden.
    /// This allows applications that exclusively use `.state()` on the builder
    /// to omit `init()` entirely, since it will never be called. If you
    /// build a runtime without calling `.state()`, you **must** override this
    /// method to provide valid initial state.
    fn init() -> (Self::State, Command<Self::Message>) {
        panic!(
            "App::init() is not implemented. \
             Override this method when building a Runtime without .state(). \
             When using .state() on the builder, this method is never called \
             and can be left unimplemented."
        );
    }

    /// Handle a message and update the state.
    ///
    /// This should be a pure function - given the same state and message,
    /// it should always produce the same result.
    ///
    /// Returns any commands to execute after the update.
    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message>;

    /// Render the current state to the frame.
    ///
    /// This should be a pure function - it only reads from state
    /// and writes to the frame. No side effects.
    fn view(state: &Self::State, frame: &mut Frame);

    /// Convert an input event to a message.
    ///
    /// Override this for simple stateless event mapping (most apps).
    /// Return `None` to ignore the event.
    fn handle_event(_event: &Event) -> Option<Self::Message> {
        None
    }

    /// Convert an input event to a message, with access to the current state.
    ///
    /// Override this instead of [`handle_event`](App::handle_event) when you
    /// need state for overlay-precedence checks or mode-dependent key bindings.
    ///
    /// The default implementation delegates to `handle_event`, ignoring state.
    fn handle_event_with_state(state: &Self::State, event: &Event) -> Option<Self::Message> {
        let _ = state;
        Self::handle_event(event)
    }

    /// Called when the application is about to exit.
    ///
    /// Override to perform cleanup or save state.
    fn on_exit(_state: &Self::State) {}

    /// Returns true if the application should quit.
    ///
    /// Override to implement custom quit logic.
    /// By default, returns false (never quits automatically).
    fn should_quit(_state: &Self::State) -> bool {
        false
    }

    /// Handle a tick event (for animations or periodic updates).
    ///
    /// Override to handle periodic updates.
    /// Return a message to process, or None to skip.
    fn on_tick(_state: &Self::State) -> Option<Self::Message> {
        None
    }
}

#[cfg(test)]
mod tests;
