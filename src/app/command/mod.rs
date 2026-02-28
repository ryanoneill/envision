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
    /// Alias for [`perform_async`](Command::perform_async).
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
    /// [`Runtime::take_errors`](crate::Runtime::take_errors).
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

/// A boxed future that produces an optional message.
pub type BoxedFuture<M> = Pin<Box<dyn Future<Output = Option<M>> + Send + 'static>>;

/// A boxed future that can fail - errors are sent to the error channel.
pub type BoxedFallibleFuture<M> =
    Pin<Box<dyn Future<Output = AsyncFallibleResult<M>> + Send + 'static>>;

/// Handles execution of commands.
///
/// This handler processes sync actions immediately and collects async futures
/// for later spawning as tokio tasks.
pub struct CommandHandler<M> {
    core: super::command_core::CommandHandlerCore<M>,
    pending_futures: Vec<BoxedFuture<M>>,
    pending_fallible_futures: Vec<BoxedFallibleFuture<M>>,
}

impl<M: Send + 'static> CommandHandler<M> {
    /// Creates a new command handler.
    pub fn new() -> Self {
        Self {
            core: super::command_core::CommandHandlerCore::new(),
            pending_futures: Vec::new(),
            pending_fallible_futures: Vec::new(),
        }
    }

    /// Executes a command, collecting sync messages and async futures.
    ///
    /// Sync actions (Message, Batch, Quit, Callback) are processed immediately.
    /// Async actions are collected for later spawning via [`spawn_pending`](CommandHandler::spawn_pending).
    pub fn execute(&mut self, command: Command<M>) {
        for action in command.into_actions() {
            if let Some(async_action) = self.core.execute_action(action) {
                match async_action {
                    CommandAction::Async(fut) => {
                        self.pending_futures.push(fut);
                    }
                    CommandAction::AsyncFallible(fut) => {
                        self.pending_fallible_futures.push(fut);
                    }
                    _ => unreachable!("execute_action only returns async actions"),
                }
            }
        }
    }

    /// Spawns all pending async futures as tokio tasks.
    ///
    /// Each future is spawned with access to the message sender and cancellation token.
    /// When a future completes with `Some(message)`, the message is sent to the runtime.
    ///
    /// For fallible futures, errors are sent to the error channel instead.
    pub fn spawn_pending(
        &mut self,
        msg_tx: tokio::sync::mpsc::Sender<M>,
        err_tx: tokio::sync::mpsc::Sender<BoxedError>,
        cancel: tokio_util::sync::CancellationToken,
    ) {
        // Spawn regular async futures
        for fut in self.pending_futures.drain(..) {
            let tx = msg_tx.clone();
            let cancel = cancel.clone();

            tokio::spawn(async move {
                tokio::select! {
                    result = fut => {
                        if let Some(msg) = result {
                            // Ignore send errors - the runtime may have shut down
                            let _ = tx.send(msg).await;
                        }
                    }
                    _ = cancel.cancelled() => {
                        // Task was cancelled, exit gracefully
                    }
                }
            });
        }

        // Spawn fallible async futures
        for fut in self.pending_fallible_futures.drain(..) {
            let msg_tx = msg_tx.clone();
            let err_tx = err_tx.clone();
            let cancel = cancel.clone();

            tokio::spawn(async move {
                tokio::select! {
                    result = fut => {
                        match result {
                            Ok(Some(msg)) => {
                                // Send message on success
                                let _ = msg_tx.send(msg).await;
                            }
                            Ok(None) => {
                                // No message to send
                            }
                            Err(e) => {
                                // Send error to error channel
                                let _ = err_tx.send(e).await;
                            }
                        }
                    }
                    _ = cancel.cancelled() => {
                        // Task was cancelled, exit gracefully
                    }
                }
            });
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

    /// Returns true if any async futures are pending.
    pub fn has_pending_futures(&self) -> bool {
        !self.pending_futures.is_empty()
    }

    /// Returns the number of pending async futures.
    pub fn pending_future_count(&self) -> usize {
        self.pending_futures.len()
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

impl<M: Send + 'static> Default for CommandHandler<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
