//! Core App trait defining the TEA application structure.

use ratatui::Frame;

use super::command::Command;
use crate::input::Event;

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
/// # Example
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
pub trait App: Sized {
    /// The application state type.
    ///
    /// This should contain all data needed to render the UI.
    /// Deriving `Clone` is recommended but not required.
    type State;

    /// The message type representing all possible events.
    ///
    /// This should be an enum covering all ways the state can change.
    type Message: Clone;

    /// Initialize the application.
    ///
    /// Returns the initial state and any commands to run on startup.
    fn init() -> (Self::State, Command<Self::Message>);

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
