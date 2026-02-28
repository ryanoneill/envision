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
        M: Send + 'static,
        N: Send + 'static,
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
    fn update(&self, state: &mut Self::State, msg: Self::Message) -> Command<Self::Message>;
}

/// A function-based update implementation.
///
/// Wraps a function to implement the `Update` trait.
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
mod tests;
