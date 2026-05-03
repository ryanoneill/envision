//! Core App trait defining the TEA application structure.
//!
//! # Two Construction Patterns
//!
//! Applications can be started in two ways, depending on whether
//! [`App::init()`] needs injected dependencies:
//!
//! ## Standard pattern — `init()` creates the state
//!
//! Apps with `type Args = ();` can call `.build()` directly. The builder
//! invokes [`App::init()`] internally to create the initial state and any
//! startup commands.
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
//! #     type Args = ();
//! #     fn init(_: ()) -> (MyState, Command<MyMsg>) { (MyState, Command::none()) }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24).build()?;
//! # Ok::<(), envision::EnvisionError>(())
//! ```
//!
//! ## Args pattern — passing dependencies into `init`
//!
//! Apps that need injected config declare a non-`()` `Args` type and pass
//! values via [`RuntimeBuilder::with_args`]. Common uses include CLI-parsed
//! paths, env-derived URLs, opened DB handles, or preloaded fixture data.
//!
//! ```rust
//! # use envision::prelude::*;
//! # use std::path::PathBuf;
//! # struct MyApp;
//! # struct MyArgs { dir: PathBuf }
//! # #[derive(Default, Clone)]
//! # struct MyState { dir: PathBuf }
//! # #[derive(Clone)]
//! # enum MyMsg {}
//! # impl App for MyApp {
//! #     type State = MyState;
//! #     type Message = MyMsg;
//! #     type Args = MyArgs;
//! #     fn init(args: MyArgs) -> (MyState, Command<MyMsg>) {
//! #         (MyState { dir: args.dir }, Command::none())
//! #     }
//! #     fn update(state: &mut MyState, msg: MyMsg) -> Command<MyMsg> { Command::none() }
//! #     fn view(state: &MyState, frame: &mut Frame) {}
//! # }
//! let args = MyArgs { dir: PathBuf::from("/tmp/example") };
//! let mut vt = Runtime::<MyApp, _>::virtual_builder(80, 24)
//!     .with_args(args)
//!     .build()?;
//! # Ok::<(), envision::EnvisionError>(())
//! ```
//!
//! [`Runtime::terminal_builder()`]: crate::app::Runtime::terminal_builder
//! [`Runtime::virtual_builder()`]: crate::app::Runtime::virtual_builder
//! [`RuntimeBuilder::with_args`]: crate::app::RuntimeBuilder::with_args

use ratatui::Frame;

use super::command::Command;
use crate::input::Event;

pub(crate) mod optional_args;
pub use optional_args::OptionalArgs;

/// The core trait for TEA-style applications.
///
/// This trait defines the structure of an application following
/// The Elm Architecture pattern:
///
/// - `State`: The complete application state
/// - `Message`: Events that can modify state
/// - `Args`: Configuration / dependencies handed in at construction time
/// - `init`: Initialize state from args, plus any startup commands
/// - `update`: Handle messages and produce new state
/// - `view`: Render the current state
///
/// # Type Parameters
///
/// - `State`: Your application's state type. Derive `Clone` if you need snapshots.
/// - `Message`: The type representing all possible events/actions.
/// - `Args`: Construction-time dependencies. Use `()` if none.
///
/// # Construction
///
/// All apps must implement [`init`](App::init). For apps that need
/// construction-time dependencies (CLI args, opened DB handles, fixture
/// data, etc.), declare a custom [`Args`](App::Args) type and pass it via
/// [`RuntimeBuilder::with_args`](crate::app::RuntimeBuilder::with_args).
/// For apps with no dependencies, declare `type Args = ();` and call
/// `.build()` directly — the unit shortcut is permitted only because `()`
/// implements the sealed [`OptionalArgs`] marker.
///
/// # Examples
///
/// ## Standard pattern — `Args = ()`
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
///     type Args = ();
///
///     fn init(_args: ()) -> (Self::State, Command<Self::Message>) {
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
/// ## Args pattern — injecting dependencies
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
/// struct ExternalArgs {
///     initial_value: String,
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
///     type Args = ExternalArgs;
///
///     fn init(args: ExternalArgs) -> (Self::State, Command<Self::Message>) {
///         (ExternalState { config_value: args.initial_value }, Command::none())
///     }
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

    /// Configuration / dependencies handed in at construction time.
    ///
    /// Apps that need no injected config declare `type Args = ();`. When
    /// `Args = ()`, [`RuntimeBuilder::build`] is callable directly without
    /// [`RuntimeBuilder::with_args`]. For any other `Args` type, callers
    /// must invoke `.with_args(...)` first.
    ///
    /// `Args` is consumed (move semantics) by [`init`](App::init). Apps
    /// that need to keep args around store the relevant fields in `State`
    /// during `init`.
    ///
    /// [`RuntimeBuilder::build`]: crate::app::RuntimeBuilder::build
    /// [`RuntimeBuilder::with_args`]: crate::app::RuntimeBuilder::with_args
    type Args;

    /// Initializes the application state from the provided args.
    ///
    /// Called exactly once per [`Runtime`](crate::app::Runtime) construction
    /// by the builder. `args` is consumed (move semantics).
    fn init(args: Self::Args) -> (Self::State, Command<Self::Message>);

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
