//! Core App trait defining the TEA application structure.

use ratatui::Frame;

use super::command::Command;
use crate::input::SimulatedEvent;

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
/// - `State`: Your application's state type. Should be `Clone` for snapshots.
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
    /// It's recommended to derive `Clone` for testing and snapshots.
    type State: Clone;

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
    /// Override this to handle keyboard/mouse input.
    /// Return `None` to ignore the event.
    fn handle_event(_state: &Self::State, _event: &SimulatedEvent) -> Option<Self::Message> {
        None
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

/// A boxed dynamic app for runtime flexibility.
///
/// This allows storing apps with different State/Message types
/// behind a common interface.
#[allow(dead_code)]
pub trait DynApp {
    /// Returns the app name for debugging.
    fn name(&self) -> &'static str;

    /// Initializes and runs the app with the given runtime.
    fn run(&self) -> std::io::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Paragraph;

    struct TestApp;

    #[derive(Clone, Default)]
    struct TestState {
        counter: i32,
    }

    #[derive(Clone)]
    enum TestMsg {
        Increment,
        Decrement,
    }

    impl App for TestApp {
        type State = TestState;
        type Message = TestMsg;

        fn init() -> (Self::State, Command<Self::Message>) {
            (TestState::default(), Command::none())
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
            match msg {
                TestMsg::Increment => state.counter += 1,
                TestMsg::Decrement => state.counter -= 1,
            }
            Command::none()
        }

        fn view(state: &Self::State, frame: &mut Frame) {
            let text = format!("Counter: {}", state.counter);
            frame.render_widget(Paragraph::new(text), frame.area());
        }
    }

    #[test]
    fn test_app_init() {
        let (state, cmd) = TestApp::init();
        assert_eq!(state.counter, 0);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_app_update() {
        let (mut state, _) = TestApp::init();

        TestApp::update(&mut state, TestMsg::Increment);
        assert_eq!(state.counter, 1);

        TestApp::update(&mut state, TestMsg::Increment);
        assert_eq!(state.counter, 2);

        TestApp::update(&mut state, TestMsg::Decrement);
        assert_eq!(state.counter, 1);
    }

    #[test]
    fn test_state_clone() {
        let (mut state, _) = TestApp::init();
        TestApp::update(&mut state, TestMsg::Increment);

        let snapshot = state.clone();
        TestApp::update(&mut state, TestMsg::Increment);

        // Original snapshot unchanged
        assert_eq!(snapshot.counter, 1);
        assert_eq!(state.counter, 2);
    }
}
