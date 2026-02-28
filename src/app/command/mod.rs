//! Commands for handling side effects in TEA applications.
//!
//! Commands represent side effects that should be executed after
//! an update. They're the bridge between pure state updates and
//! the outside world (IO, network, etc.).

use std::future::Future;
use std::pin::Pin;

use crate::overlay::Overlay;

/// A command that can produce messages or perform side effects.
///
/// Commands are returned from `update` functions to trigger
/// asynchronous operations or batch multiple messages.
#[derive(Default)]
pub struct Command<M> {
    actions: Vec<CommandAction<M>>,
}

/// A boxed error type for fallible async commands.
pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Result type for fallible async commands.
pub type AsyncFallibleResult<M> = Result<Option<M>, BoxedError>;

pub(crate) enum CommandAction<M> {
    /// A message to dispatch immediately
    Message(M),

    /// A batch of messages to dispatch
    Batch(Vec<M>),

    /// Quit the application
    Quit,

    /// A callback to execute (for sync side effects)
    Callback(Box<dyn FnOnce() -> Option<M> + Send + 'static>),

    /// An async future to execute
    Async(Pin<Box<dyn Future<Output = Option<M>> + Send + 'static>>),

    /// An async future that can fail - errors are sent to the error channel
    AsyncFallible(Pin<Box<dyn Future<Output = AsyncFallibleResult<M>> + Send + 'static>>),

    /// Push an overlay onto the stack
    PushOverlay(Box<dyn Overlay<M> + Send>),

    /// Pop the topmost overlay
    PopOverlay,
}

impl<M> Command<M> {
    /// Creates an empty command (no-op).
    pub fn none() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Returns true if this command has no actions.
    pub fn is_none(&self) -> bool {
        self.actions.is_empty()
    }

    /// Creates a command that dispatches a single message.
    pub fn message(msg: M) -> Self {
        Self {
            actions: vec![CommandAction::Message(msg)],
        }
    }

    /// Creates a command that dispatches multiple messages.
    pub fn batch(messages: impl IntoIterator<Item = M>) -> Self {
        let msgs: Vec<M> = messages.into_iter().collect();
        if msgs.is_empty() {
            Self::none()
        } else {
            Self {
                actions: vec![CommandAction::Batch(msgs)],
            }
        }
    }

    /// Creates a command that quits the application.
    pub fn quit() -> Self {
        Self {
            actions: vec![CommandAction::Quit],
        }
    }

    /// Creates a command from a synchronous callback.
    ///
    /// The callback will be executed and may optionally return a message.
    pub fn perform<F>(f: F) -> Self
    where
        F: FnOnce() -> Option<M> + Send + 'static,
    {
        Self {
            actions: vec![CommandAction::Callback(Box::new(f))],
        }
    }

    /// Creates a command from an async operation.
    ///
    /// The future will be spawned and may optionally return a message.
    ///
    /// # Example
    ///
    /// ```ignore
    /// Command::perform_async(async {
    ///     let result = fetch_data().await;
    ///     Some(Msg::DataLoaded(result))
    /// })
    /// ```
    pub fn perform_async<Fut>(future: Fut) -> Self
    where
        Fut: Future<Output = Option<M>> + Send + 'static,
    {
        Self {
            actions: vec![CommandAction::Async(Box::pin(future))],
        }
    }

    /// Creates a command from an async future.
    ///
    /// Alias for [`perform_async`](Command::perform_async). Requires
    /// [`AsyncRuntime`](crate::AsyncRuntime) — the sync `Runtime` will
    /// drop async commands with a debug warning.
    pub fn future<Fut>(future: Fut) -> Self
    where
        Fut: Future<Output = Option<M>> + Send + 'static,
    {
        Self::perform_async(future)
    }

    /// Creates a command from an async operation that can fail.
    ///
    /// On success, the future returns `Ok(Some(message))` or `Ok(None)`.
    /// On failure, the error is converted to a message using the provided function.
    ///
    /// # Example
    ///
    /// ```ignore
    /// Command::perform_async_fallible(
    ///     async { fetch_data().await },
    ///     |result| match result {
    ///         Ok(data) => Msg::DataLoaded(data),
    ///         Err(e) => Msg::Error(e.to_string()),
    ///     }
    /// )
    /// ```
    pub fn perform_async_fallible<Fut, T, E, F>(future: Fut, on_result: F) -> Self
    where
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        F: FnOnce(Result<T, E>) -> M + Send + 'static,
        M: Send + 'static,
    {
        Self {
            actions: vec![CommandAction::Async(Box::pin(async move {
                let result = future.await;
                Some(on_result(result))
            }))],
        }
    }

    /// Creates a command from an async operation that reports errors to the error channel.
    ///
    /// On success, the callback is called to optionally produce a message.
    /// On failure, the error is sent to the runtime's error channel via
    /// [`AsyncRuntime::take_errors`](crate::AsyncRuntime::take_errors).
    ///
    /// This is useful when you want errors to be collected rather than
    /// converted to messages.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Errors go to runtime.take_errors()
    /// Command::try_perform_async(
    ///     async { fetch_data().await },
    ///     |data| Some(Msg::DataLoaded(data))
    /// )
    /// ```
    pub fn try_perform_async<Fut, T, E, F>(future: Fut, on_success: F) -> Self
    where
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        E: std::error::Error + Send + Sync + 'static,
        F: FnOnce(T) -> Option<M> + Send + 'static,
        M: Send + 'static,
    {
        Self {
            actions: vec![CommandAction::AsyncFallible(Box::pin(async move {
                match future.await {
                    Ok(value) => Ok(on_success(value)),
                    Err(e) => Err(Box::new(e) as BoxedError),
                }
            }))],
        }
    }

    /// Creates a command that pushes an overlay onto the runtime's overlay stack.
    pub fn push_overlay(overlay: impl Overlay<M> + 'static) -> Self {
        Self {
            actions: vec![CommandAction::PushOverlay(Box::new(overlay))],
        }
    }

    /// Creates a command that pops the topmost overlay from the runtime's overlay stack.
    pub fn pop_overlay() -> Self {
        Self {
            actions: vec![CommandAction::PopOverlay],
        }
    }

    /// Combines multiple commands into one.
    pub fn combine(commands: impl IntoIterator<Item = Command<M>>) -> Self {
        let mut actions = Vec::new();
        for cmd in commands {
            actions.extend(cmd.actions);
        }
        Self { actions }
    }

    /// Appends another command to this one.
    pub fn and(mut self, other: Command<M>) -> Self {
        self.actions.extend(other.actions);
        self
    }

    /// Consumes the command and returns its actions.
    ///
    /// This is used internally by the async command handler.
    pub(crate) fn into_actions(self) -> Vec<CommandAction<M>> {
        self.actions
    }

    /// Maps the message type to a different type.
    pub fn map<N, F>(self, f: F) -> Command<N>
    where
        F: Fn(M) -> N + Clone + Send + 'static,
        M: Send + 'static,
        N: Send + 'static,
    {
        let actions = self
            .actions
            .into_iter()
            .filter_map(|action| match action {
                CommandAction::Message(m) => Some(CommandAction::Message(f(m))),
                CommandAction::Batch(msgs) => Some(CommandAction::Batch(
                    msgs.into_iter().map(|m| f.clone()(m)).collect(),
                )),
                CommandAction::Quit => Some(CommandAction::Quit),
                CommandAction::Callback(cb) => {
                    let f = f.clone();
                    Some(CommandAction::Callback(Box::new(move || cb().map(&f))))
                }
                CommandAction::Async(fut) => {
                    let f = f.clone();
                    Some(CommandAction::Async(Box::pin(
                        async move { fut.await.map(&f) },
                    )))
                }
                CommandAction::AsyncFallible(fut) => {
                    let f = f.clone();
                    Some(CommandAction::AsyncFallible(Box::pin(async move {
                        fut.await.map(|opt| opt.map(&f))
                    })))
                }
                CommandAction::PushOverlay(_) => None,
                CommandAction::PopOverlay => Some(CommandAction::PopOverlay),
            })
            .collect();

        Command { actions }
    }
}

impl<M: Clone> Clone for Command<M> {
    fn clone(&self) -> Self {
        // Note: Callbacks, Async, and PushOverlay can't be cloned, so we only clone Message/Batch/Quit/PopOverlay
        let actions = self
            .actions
            .iter()
            .filter_map(|action| match action {
                CommandAction::Message(m) => Some(CommandAction::Message(m.clone())),
                CommandAction::Batch(msgs) => Some(CommandAction::Batch(msgs.clone())),
                CommandAction::Quit => Some(CommandAction::Quit),
                CommandAction::PopOverlay => Some(CommandAction::PopOverlay),
                _ => None, // Skip non-clonable actions
            })
            .collect();

        Self { actions }
    }
}

impl<M> std::fmt::Debug for Command<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("action_count", &self.actions.len())
            .finish()
    }
}

/// Handles execution of commands.
///
/// This is used by the runtime to process commands returned from updates.
pub struct CommandHandler<M> {
    core: super::command_core::CommandHandlerCore<M>,
}

impl<M> CommandHandler<M> {
    /// Creates a new command handler.
    pub fn new() -> Self {
        Self {
            core: super::command_core::CommandHandlerCore::new(),
        }
    }

    /// Executes a command and collects any resulting messages.
    ///
    /// Note: Async actions are skipped by this sync handler. Use `AsyncCommandHandler`
    /// for full async support.
    pub fn execute(&mut self, command: Command<M>) {
        for action in command.actions {
            if self.core.execute_sync_action(action).is_some() {
                // Async actions are handled by the async runtime
                #[cfg(debug_assertions)]
                eprintln!(
                    "[envision] Warning: Async command ignored by sync Runtime. \
                     Use AsyncRuntime for async commands."
                );
            }
        }
    }

    /// Takes all pending messages.
    pub fn take_messages(&mut self) -> Vec<M> {
        self.core.take_messages()
    }

    /// Takes all pending overlay pushes.
    pub fn take_overlay_pushes(&mut self) -> Vec<Box<dyn Overlay<M> + Send>> {
        self.core.take_overlay_pushes()
    }

    /// Takes the count of pending overlay pops and resets the counter.
    pub fn take_overlay_pops(&mut self) -> usize {
        self.core.take_overlay_pops()
    }

    /// Returns true if a quit command was executed.
    pub fn should_quit(&self) -> bool {
        self.core.should_quit()
    }

    /// Resets the quit flag.
    pub fn reset_quit(&mut self) {
        self.core.reset_quit()
    }
}

impl<M> Default for CommandHandler<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
