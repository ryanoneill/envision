//! Update helpers and result types for TEA applications.

use super::command::Command;

/// Result of an update operation.
///
/// This provides a convenient way to return both state changes
/// and commands from an update function.
#[derive(Debug)]
pub struct UpdateResult<S, M> {
    /// The updated state (if changed)
    pub state: Option<S>,

    /// Commands to execute
    pub command: Command<M>,
}

impl<S, M> UpdateResult<S, M> {
    /// Creates a result with no state change and no command.
    pub fn none() -> Self {
        Self {
            state: None,
            command: Command::none(),
        }
    }

    /// Creates a result with updated state.
    pub fn state(state: S) -> Self {
        Self {
            state: Some(state),
            command: Command::none(),
        }
    }

    /// Creates a result with a command.
    pub fn command(command: Command<M>) -> Self {
        Self {
            state: None,
            command,
        }
    }

    /// Creates a result with both state and command.
    pub fn with(state: S, command: Command<M>) -> Self {
        Self {
            state: Some(state),
            command,
        }
    }

    /// Adds a command to this result.
    pub fn and_command(mut self, command: Command<M>) -> Self {
        self.command = self.command.and(command);
        self
    }

    /// Maps the state type.
    pub fn map_state<T, F>(self, f: F) -> UpdateResult<T, M>
    where
        F: FnOnce(S) -> T,
    {
        UpdateResult {
            state: self.state.map(f),
            command: self.command,
        }
    }

    /// Maps the message type.
    pub fn map_message<N, F>(self, f: F) -> UpdateResult<S, N>
    where
        F: Fn(M) -> N + Clone + Send + 'static,
        M: 'static,
        N: 'static,
    {
        UpdateResult {
            state: self.state,
            command: self.command.map(f),
        }
    }
}

impl<S, M> Default for UpdateResult<S, M> {
    fn default() -> Self {
        Self::none()
    }
}

/// A trait for types that can perform updates.
///
/// This is an alternative to implementing `App::update` directly,
/// useful for component-based architectures.
pub trait Update {
    /// The state type being updated.
    type State;

    /// The message type that triggers updates.
    type Message;

    /// Perform the update.
    fn update(
        &self,
        state: &mut Self::State,
        msg: Self::Message,
    ) -> Command<Self::Message>;
}

/// A function-based update implementation.
///
/// Wraps a function to implement the `Update` trait.
#[allow(dead_code)]
pub struct FnUpdate<S, M, F>
where
    F: Fn(&mut S, M) -> Command<M>,
{
    f: F,
    _phantom: std::marker::PhantomData<(S, M)>,
}

impl<S, M, F> FnUpdate<S, M, F>
where
    F: Fn(&mut S, M) -> Command<M>,
{
    /// Creates a new function-based updater.
    #[allow(dead_code)]
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, M, F> Update for FnUpdate<S, M, F>
where
    F: Fn(&mut S, M) -> Command<M>,
{
    type State = S;
    type Message = M;

    fn update(&self, state: &mut S, msg: M) -> Command<M> {
        (self.f)(state, msg)
    }
}

/// Extension trait for ergonomic state updates.
#[allow(dead_code)]
pub trait StateExt: Sized {
    /// Updates self and returns a command.
    fn updated(self, cmd: Command<impl Clone>) -> UpdateResult<Self, impl Clone> {
        UpdateResult {
            state: Some(self),
            command: cmd,
        }
    }

    /// Returns self with no command.
    fn unchanged(self) -> UpdateResult<Self, ()> {
        UpdateResult {
            state: Some(self),
            command: Command::none(),
        }
    }
}

impl<T> StateExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestState {
        value: i32,
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        Add(i32),
        Finished,
    }

    #[test]
    fn test_update_result_none() {
        let result: UpdateResult<TestState, TestMsg> = UpdateResult::none();
        assert!(result.state.is_none());
        assert!(result.command.is_none());
    }

    #[test]
    fn test_update_result_state() {
        let result: UpdateResult<TestState, TestMsg> =
            UpdateResult::state(TestState { value: 42 });
        assert_eq!(result.state.unwrap().value, 42);
        assert!(result.command.is_none());
    }

    #[test]
    fn test_update_result_command() {
        let result: UpdateResult<TestState, TestMsg> =
            UpdateResult::command(Command::message(TestMsg::Finished));
        assert!(result.state.is_none());
        assert!(!result.command.is_none());
    }

    #[test]
    fn test_update_result_with() {
        let result = UpdateResult::with(
            TestState { value: 10 },
            Command::message(TestMsg::Finished),
        );
        assert_eq!(result.state.unwrap().value, 10);
        assert!(!result.command.is_none());
    }

    #[test]
    fn test_fn_update() {
        let updater = FnUpdate::new(|state: &mut TestState, msg: TestMsg| {
            match msg {
                TestMsg::Add(n) => state.value += n,
                TestMsg::Finished => {}
            }
            Command::none()
        });

        let mut state = TestState { value: 0 };
        updater.update(&mut state, TestMsg::Add(5));
        assert_eq!(state.value, 5);

        updater.update(&mut state, TestMsg::Add(3));
        assert_eq!(state.value, 8);
    }

    #[test]
    fn test_map_state() {
        let result: UpdateResult<TestState, TestMsg> =
            UpdateResult::state(TestState { value: 42 });

        let mapped = result.map_state(|s| s.value);
        assert_eq!(mapped.state, Some(42));
    }
}
