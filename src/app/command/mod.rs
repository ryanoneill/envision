//! Commands for handling side effects in TEA applications.
//!
//! Commands represent side effects that should be executed after
//! an update. They're the bridge between pure state updates and
//! the outside world (IO, network, etc.).

use std::future::Future;
use std::pin::Pin;

use crate::app::subscription::BoxedSubscription;
use crate::overlay::Overlay;
use tokio_util::sync::CancellationToken;

/// A command that can produce messages or perform side effects.
///
/// Commands are returned from `update` functions to trigger
/// asynchronous operations or batch multiple messages.
#[derive(Default)]
pub struct Command<M> {
    actions: Vec<CommandAction<M>>,
}

/// A boxed error type for fallible async commands.
///
/// Re-exported from [`crate::error::BoxedError`].
pub use crate::error::BoxedError;

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

    /// Request the runtime's cancellation token
    RequestCancelToken(Box<dyn FnOnce(CancellationToken) -> M + Send + 'static>),

    /// Register a subscription dynamically from within update()
    Subscribe(BoxedSubscription<M>),
}

impl<M> CommandAction<M> {
    /// Returns a human-readable name for this action kind.
    ///
    /// Used for tracing instrumentation to identify which type of command
    /// action is being executed.
    #[cfg(feature = "tracing")]
    pub(crate) fn kind_name(&self) -> &'static str {
        match self {
            CommandAction::Message(_) => "message",
            CommandAction::Batch(_) => "batch",
            CommandAction::Quit => "quit",
            CommandAction::Callback(_) => "callback",
            CommandAction::Async(_) => "async",
            CommandAction::AsyncFallible(_) => "async_fallible",
            CommandAction::PushOverlay(_) => "push_overlay",
            CommandAction::PopOverlay => "pop_overlay",
            CommandAction::RequestCancelToken(_) => "request_cancel_token",
            CommandAction::Subscribe(_) => "subscribe",
        }
    }
}

impl<M> Command<M> {
    /// Creates an empty command (no-op).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<String> = Command::none();
    /// assert!(cmd.is_none());
    /// ```
    pub fn none() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Returns true if this command has no actions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// assert!(Command::<String>::none().is_none());
    /// assert!(!Command::message("hello".to_string()).is_none());
    /// ```
    pub fn is_none(&self) -> bool {
        self.actions.is_empty()
    }

    /// Returns true if this command contains a quit action.
    ///
    /// This is useful in tests to verify that an `update()` function
    /// returns a quit command in response to specific messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<String> = Command::quit();
    /// assert!(cmd.is_quit());
    /// assert!(!cmd.is_none());
    ///
    /// let cmd: Command<String> = Command::none();
    /// assert!(!cmd.is_quit());
    /// ```
    pub fn is_quit(&self) -> bool {
        self.actions
            .iter()
            .any(|a| matches!(a, CommandAction::Quit))
    }

    /// Returns true if this command contains a message action.
    ///
    /// This is useful in tests to verify that an `update()` function
    /// produces a follow-up message command.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd = Command::message("hello".to_string());
    /// assert!(cmd.is_message());
    ///
    /// let cmd: Command<String> = Command::quit();
    /// assert!(!cmd.is_message());
    /// ```
    pub fn is_message(&self) -> bool {
        self.actions
            .iter()
            .any(|a| matches!(a, CommandAction::Message(_)))
    }

    /// Returns true if this command contains a batch action.
    ///
    /// This is useful in tests to verify that an `update()` function
    /// dispatches multiple messages at once.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd = Command::batch(vec!["a".to_string(), "b".to_string()]);
    /// assert!(cmd.is_batch());
    ///
    /// let cmd = Command::message("hello".to_string());
    /// assert!(!cmd.is_batch());
    /// ```
    pub fn is_batch(&self) -> bool {
        self.actions
            .iter()
            .any(|a| matches!(a, CommandAction::Batch(_)))
    }

    /// Returns true if this command contains an async action.
    ///
    /// This matches both regular async commands (from [`perform_async`](Command::perform_async))
    /// and fallible async commands (from [`try_perform_async`](Command::try_perform_async)).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<String> = Command::perform_async(async {
    ///     Some("loaded".to_string())
    /// });
    /// assert!(cmd.is_async());
    ///
    /// let cmd: Command<String> = Command::message("hello".to_string());
    /// assert!(!cmd.is_async());
    /// ```
    pub fn is_async(&self) -> bool {
        self.actions
            .iter()
            .any(|a| matches!(a, CommandAction::Async(_) | CommandAction::AsyncFallible(_)))
    }

    /// Returns true if this command contains a push overlay action.
    ///
    /// This is useful in tests to verify that an `update()` function
    /// opens an overlay (e.g., a dialog or modal).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<String> = Command::pop_overlay();
    /// assert!(!cmd.is_overlay_push());
    ///
    /// let cmd: Command<String> = Command::none();
    /// assert!(!cmd.is_overlay_push());
    /// ```
    pub fn is_overlay_push(&self) -> bool {
        self.actions
            .iter()
            .any(|a| matches!(a, CommandAction::PushOverlay(_)))
    }

    /// Returns true if this command contains a pop overlay action.
    ///
    /// This is useful in tests to verify that an `update()` function
    /// closes the topmost overlay.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<String> = Command::pop_overlay();
    /// assert!(cmd.is_overlay_pop());
    /// assert!(!cmd.is_none());
    ///
    /// let cmd: Command<String> = Command::quit();
    /// assert!(!cmd.is_overlay_pop());
    /// ```
    pub fn is_overlay_pop(&self) -> bool {
        self.actions
            .iter()
            .any(|a| matches!(a, CommandAction::PopOverlay))
    }

    /// Returns the number of actions in this command.
    ///
    /// A command can contain multiple actions when created with
    /// [`combine`](Command::combine) or [`and`](Command::and).
    /// An empty command ([`none`](Command::none)) has zero actions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// assert_eq!(Command::<String>::none().action_count(), 0);
    /// assert_eq!(Command::message("hello".to_string()).action_count(), 1);
    ///
    /// let combined: Command<String> = Command::combine(vec![
    ///     Command::message("a".to_string()),
    ///     Command::quit(),
    /// ]);
    /// assert_eq!(combined.action_count(), 2);
    /// ```
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// Creates a command that dispatches a single message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd = Command::message("data_loaded".to_string());
    /// assert!(!cmd.is_none());
    /// ```
    pub fn message(msg: M) -> Self {
        Self {
            actions: vec![CommandAction::Message(msg)],
        }
    }

    /// Creates a command that dispatches multiple messages.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd = Command::batch(vec!["first".to_string(), "second".to_string()]);
    /// assert!(!cmd.is_none());
    ///
    /// // An empty batch produces a no-op command
    /// let empty: Command<String> = Command::batch(vec![]);
    /// assert!(empty.is_none());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<String> = Command::quit();
    /// assert!(!cmd.is_none());
    /// ```
    pub fn quit() -> Self {
        Self {
            actions: vec![CommandAction::Quit],
        }
    }

    /// Creates a command from a synchronous callback.
    ///
    /// The callback will be executed and may optionally return a message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<String> = Command::perform(|| {
    ///     Some("done".to_string())
    /// });
    /// ```
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
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<String> = Command::perform_async(async {
    ///     Some("loaded".to_string())
    /// });
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

    /// Spawns an async task that does not produce a message.
    ///
    /// Use this for fire-and-forget operations like writing to a file,
    /// sending a network request, or logging, where no response message
    /// is needed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::Command;
    ///
    /// let cmd: Command<String> = Command::spawn(async {
    ///     // Fire-and-forget: no message returned
    ///     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    /// });
    /// assert!(!cmd.is_none());
    /// ```
    pub fn spawn<Fut>(future: Fut) -> Self
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self::perform_async(async move {
            future.await;
            None
        })
    }

    /// Creates a command from an async operation that can fail.
    ///
    /// On success, the future returns `Ok(Some(message))` or `Ok(None)`.
    /// On failure, the error is converted to a message using the provided function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<String> = Command::perform_async_fallible(
    ///     async { Ok::<_, std::io::Error>("data".to_string()) },
    ///     |result| match result {
    ///         Ok(data) => format!("loaded: {}", data),
    ///         Err(e) => format!("error: {}", e),
    ///     }
    /// );
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
    /// ```rust
    /// use envision::app::Command;
    ///
    /// // Errors go to runtime.take_errors()
    /// let cmd: Command<String> = Command::try_perform_async(
    ///     async { Ok::<_, std::io::Error>("data".to_string()) },
    ///     |data| Some(format!("loaded: {}", data))
    /// );
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

    /// Creates a command that requests the runtime's cancellation token.
    ///
    /// When processed, the runtime calls the provided function with its
    /// [`CancellationToken`] and dispatches the resulting message. This
    /// lets you store the token in your application state and pass it to
    /// background workers for cooperative shutdown.
    ///
    /// The token is cancelled when the runtime begins shutting down
    /// (via [`Command::quit()`], [`App::should_quit()`](crate::App::should_quit),
    /// or [`Runtime::quit()`](crate::Runtime::quit)).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    /// use tokio_util::sync::CancellationToken;
    ///
    /// #[derive(Clone)]
    /// enum Msg {
    ///     GotCancelToken(CancellationToken),
    /// }
    ///
    /// // In App::init():
    /// let cmd: Command<Msg> = Command::request_cancel_token(Msg::GotCancelToken);
    /// ```
    pub fn request_cancel_token<F>(f: F) -> Self
    where
        F: FnOnce(CancellationToken) -> M + Send + 'static,
    {
        Self {
            actions: vec![CommandAction::RequestCancelToken(Box::new(f))],
        }
    }

    /// Creates a command that dynamically registers a subscription.
    ///
    /// Use this to add subscriptions from within `update()` — for example,
    /// when a user action triggers a background worker that reports progress
    /// via a channel subscription.
    ///
    /// The subscription is registered by the runtime on the next command
    /// processing cycle, and begins producing messages immediately.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::{ChannelSubscription, MappedSubscription, Command};
    /// use tokio::sync::mpsc;
    ///
    /// #[derive(Clone)]
    /// enum Msg { WorkerProgress(String) }
    ///
    /// let (tx, rx) = mpsc::channel::<String>(32);
    /// let subscription = ChannelSubscription::new(rx);
    ///
    /// // Map the subscription's output to your message type
    /// let mapped = MappedSubscription::new(subscription, Msg::WorkerProgress);
    ///
    /// let cmd: Command<Msg> = Command::subscribe(Box::new(mapped));
    /// // Return `cmd` from update() — runtime registers the subscription
    /// ```
    pub fn subscribe(subscription: BoxedSubscription<M>) -> Self
    where
        M: Send + Clone + 'static,
    {
        Self {
            actions: vec![CommandAction::Subscribe(subscription)],
        }
    }

    /// Creates a command that saves application state to a JSON file.
    ///
    /// Serializes the state to JSON synchronously, then writes the file
    /// asynchronously via `tokio::fs::write`.
    ///
    /// # Errors
    ///
    /// If JSON serialization fails, returns [`Command::none`] silently (no
    /// error is reported). If the file write fails, the error is reported to
    /// the runtime's error channel as `EnvisionError::Io`; retrieve it
    /// with [`Runtime::take_errors`](crate::Runtime::take_errors).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct MyState { count: i32 }
    ///
    /// let state = MyState { count: 42 };
    /// let cmd: Command<String> = Command::save_state(&state, "/tmp/state.json");
    /// ```
    #[cfg(feature = "serialization")]
    pub fn save_state<S: serde::Serialize>(
        state: &S,
        path: impl Into<std::path::PathBuf>,
    ) -> Command<M>
    where
        M: Send + 'static,
    {
        let json = match serde_json::to_string(state) {
            Ok(json) => json,
            Err(_) => return Command::none(),
        };
        let path = path.into();
        Command::try_perform_async(async move { tokio::fs::write(path, json).await }, |_| None)
    }

    /// Combines multiple commands into one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let combined: Command<String> = Command::combine(vec![
    ///     Command::message("first".to_string()),
    ///     Command::message("second".to_string()),
    /// ]);
    /// assert!(!combined.is_none());
    /// ```
    pub fn combine(commands: impl IntoIterator<Item = Command<M>>) -> Self {
        let mut actions = Vec::new();
        for cmd in commands {
            actions.extend(cmd.actions);
        }
        Self { actions }
    }

    /// Appends another command to this one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd = Command::message("first".to_string())
    ///     .and(Command::message("second".to_string()));
    /// assert!(!cmd.is_none());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::Command;
    ///
    /// let cmd: Command<i32> = Command::message(42);
    /// let mapped: Command<String> = cmd.map(|n| n.to_string());
    /// ```
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
                CommandAction::RequestCancelToken(cb) => {
                    let f = f.clone();
                    Some(CommandAction::RequestCancelToken(Box::new(move |token| {
                        f(cb(token))
                    })))
                }
                // Subscriptions can't be remapped after boxing — map them
                // before creating the Command::subscribe.
                CommandAction::Subscribe(_) => None,
            })
            .collect();

        Command { actions }
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

/// A boxed callback that accepts a cancellation token and produces a message.
pub(crate) type CancelTokenCallback<M> = Box<dyn FnOnce(CancellationToken) -> M + Send + 'static>;

/// Handles execution of commands.
///
/// This handler processes sync actions immediately and collects async futures
/// for later spawning as tokio tasks.
pub struct CommandHandler<M> {
    core: super::command_core::CommandHandlerCore<M>,
    pending_futures: Vec<BoxedFuture<M>>,
    pending_fallible_futures: Vec<BoxedFallibleFuture<M>>,
    pending_cancel_token_requests: Vec<CancelTokenCallback<M>>,
}

impl<M: Send + 'static> CommandHandler<M> {
    /// Creates a new command handler.
    pub fn new() -> Self {
        Self {
            core: super::command_core::CommandHandlerCore::new(),
            pending_futures: Vec::new(),
            pending_fallible_futures: Vec::new(),
            pending_cancel_token_requests: Vec::new(),
        }
    }

    /// Executes a command, collecting sync messages and async futures.
    ///
    /// Sync actions (Message, Batch, Quit, Callback) are processed immediately.
    /// Async actions are collected for later spawning via [`spawn_pending`](CommandHandler::spawn_pending).
    pub fn execute(&mut self, command: Command<M>) {
        for action in command.into_actions() {
            #[cfg(feature = "tracing")]
            tracing::debug!(action = action.kind_name(), "executing command action");

            if let Some(async_action) = self.core.execute_action(action) {
                match async_action {
                    CommandAction::Async(fut) => {
                        self.pending_futures.push(fut);
                    }
                    CommandAction::AsyncFallible(fut) => {
                        self.pending_fallible_futures.push(fut);
                    }
                    CommandAction::RequestCancelToken(cb) => {
                        self.pending_cancel_token_requests.push(cb);
                    }
                    _ => unreachable!("execute_action only returns async or cancel-token actions"),
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
        #[cfg(feature = "tracing")]
        {
            let regular = self.pending_futures.len();
            let fallible = self.pending_fallible_futures.len();
            if regular > 0 || fallible > 0 {
                tracing::debug!(regular, fallible, "spawning async command tasks");
            }
        }

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

    /// Takes all pending dynamic subscription registrations.
    pub(crate) fn take_subscriptions(&mut self) -> Vec<BoxedSubscription<M>> {
        self.core.take_subscriptions()
    }

    /// Takes all pending cancel token request callbacks.
    pub(crate) fn take_cancel_token_requests(&mut self) -> Vec<CancelTokenCallback<M>> {
        std::mem::take(&mut self.pending_cancel_token_requests)
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
