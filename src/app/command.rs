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
                CommandAction::Batch(msgs) => {
                    Some(CommandAction::Batch(msgs.into_iter().map(|m| f.clone()(m)).collect()))
                }
                CommandAction::Quit => Some(CommandAction::Quit),
                CommandAction::Callback(cb) => {
                    let f = f.clone();
                    Some(CommandAction::Callback(Box::new(move || cb().map(&f))))
                }
                CommandAction::Async(fut) => {
                    let f = f.clone();
                    Some(CommandAction::Async(Box::pin(async move { fut.await.map(&f) })))
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
    pending_messages: Vec<M>,
    pending_overlay_pushes: Vec<Box<dyn Overlay<M> + Send>>,
    pending_overlay_pops: usize,
    should_quit: bool,
}

impl<M> CommandHandler<M> {
    /// Creates a new command handler.
    pub fn new() -> Self {
        Self {
            pending_messages: Vec::new(),
            pending_overlay_pushes: Vec::new(),
            pending_overlay_pops: 0,
            should_quit: false,
        }
    }

    /// Executes a command and collects any resulting messages.
    ///
    /// Note: Async actions are skipped by this sync handler. Use `AsyncCommandHandler`
    /// for full async support.
    pub fn execute(&mut self, command: Command<M>) {
        for action in command.actions {
            match action {
                CommandAction::Message(m) => {
                    self.pending_messages.push(m);
                }
                CommandAction::Batch(msgs) => {
                    self.pending_messages.extend(msgs);
                }
                CommandAction::Quit => {
                    self.should_quit = true;
                }
                CommandAction::Callback(cb) => {
                    if let Some(m) = cb() {
                        self.pending_messages.push(m);
                    }
                }
                CommandAction::Async(_) | CommandAction::AsyncFallible(_) => {
                    // Async actions are handled by the async runtime
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "[envision] Warning: Async command ignored by sync Runtime. \
                         Use AsyncRuntime for async commands."
                    );
                }
                CommandAction::PushOverlay(overlay) => {
                    self.pending_overlay_pushes.push(overlay);
                }
                CommandAction::PopOverlay => {
                    self.pending_overlay_pops += 1;
                }
            }
        }
    }

    /// Takes all pending messages.
    pub fn take_messages(&mut self) -> Vec<M> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Takes all pending overlay pushes.
    pub fn take_overlay_pushes(&mut self) -> Vec<Box<dyn Overlay<M> + Send>> {
        std::mem::take(&mut self.pending_overlay_pushes)
    }

    /// Takes the count of pending overlay pops and resets the counter.
    pub fn take_overlay_pops(&mut self) -> usize {
        std::mem::replace(&mut self.pending_overlay_pops, 0)
    }

    /// Returns true if a quit command was executed.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Resets the quit flag.
    pub fn reset_quit(&mut self) {
        self.should_quit = false;
    }
}

impl<M> Default for CommandHandler<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        A,
        B,
        C,
        Value(i32),
    }

    #[test]
    fn test_command_none() {
        let cmd: Command<TestMsg> = Command::none();
        assert!(cmd.is_none());
    }

    #[test]
    fn test_command_message() {
        let cmd = Command::message(TestMsg::A);
        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_batch() {
        let cmd = Command::batch([TestMsg::A, TestMsg::B, TestMsg::C]);
        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_handler_message() {
        let mut handler = CommandHandler::new();
        handler.execute(Command::message(TestMsg::A));

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A]);
    }

    #[test]
    fn test_command_handler_batch() {
        let mut handler = CommandHandler::new();
        handler.execute(Command::batch([TestMsg::A, TestMsg::B]));

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
    }

    #[test]
    fn test_command_handler_quit() {
        let mut handler: CommandHandler<TestMsg> = CommandHandler::new();
        assert!(!handler.should_quit());

        handler.execute(Command::quit());
        assert!(handler.should_quit());
    }

    #[test]
    fn test_command_combine() {
        let cmd = Command::combine([Command::message(TestMsg::A), Command::message(TestMsg::B)]);

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
    }

    #[test]
    fn test_command_and() {
        let cmd = Command::message(TestMsg::A).and(Command::message(TestMsg::B));

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
    }

    #[test]
    fn test_command_perform() {
        let cmd = Command::perform(|| Some(TestMsg::A));

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A]);
    }

    #[test]
    fn test_command_perform_none() {
        let cmd: Command<TestMsg> = Command::perform(|| None);

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        let messages = handler.take_messages();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_command_map() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd = Command::message(TestMsg::A);
        let mapped = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![OuterMsg::Inner(TestMsg::A)]);
    }

    #[test]
    fn test_command_batch_empty() {
        let cmd: Command<TestMsg> = Command::batch(Vec::new());
        assert!(cmd.is_none());
    }

    #[test]
    fn test_command_clone_message() {
        let cmd = Command::message(TestMsg::A);
        let cloned = cmd.clone();

        let mut handler = CommandHandler::new();
        handler.execute(cloned);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A]);
    }

    #[test]
    fn test_command_clone_batch() {
        let cmd = Command::batch([TestMsg::A, TestMsg::B]);
        let cloned = cmd.clone();

        let mut handler = CommandHandler::new();
        handler.execute(cloned);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
    }

    #[test]
    fn test_command_clone_quit() {
        let cmd: Command<TestMsg> = Command::quit();
        let cloned = cmd.clone();

        let mut handler = CommandHandler::new();
        handler.execute(cloned);

        assert!(handler.should_quit());
    }

    #[test]
    fn test_command_clone_callback_skipped() {
        let cmd = Command::perform(|| Some(TestMsg::A));
        let cloned = cmd.clone();

        // Callbacks can't be cloned, so cloned should have no actions
        assert!(cloned.is_none());
    }

    #[test]
    fn test_command_debug() {
        let cmd = Command::message(TestMsg::A);
        let debug_str = format!("{:?}", cmd);

        assert!(debug_str.contains("Command"));
        assert!(debug_str.contains("action_count"));
        assert!(debug_str.contains("1"));
    }

    #[test]
    fn test_command_map_batch() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd = Command::batch([TestMsg::A, TestMsg::B]);
        let mapped = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        let messages = handler.take_messages();
        assert_eq!(
            messages,
            vec![OuterMsg::Inner(TestMsg::A), OuterMsg::Inner(TestMsg::B)]
        );
    }

    #[test]
    fn test_command_map_quit() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd: Command<TestMsg> = Command::quit();
        let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        assert!(handler.should_quit());
    }

    #[test]
    fn test_command_map_callback() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd = Command::perform(|| Some(TestMsg::A));
        let mapped = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![OuterMsg::Inner(TestMsg::A)]);
    }

    #[test]
    fn test_command_map_callback_none() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd: Command<TestMsg> = Command::perform(|| None);
        let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

        let mut handler = CommandHandler::new();
        handler.execute(mapped);

        let messages = handler.take_messages();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_command_handler_reset_quit() {
        let mut handler: CommandHandler<TestMsg> = CommandHandler::new();
        handler.execute(Command::quit());
        assert!(handler.should_quit());

        handler.reset_quit();
        assert!(!handler.should_quit());
    }

    #[test]
    fn test_command_handler_default() {
        let handler: CommandHandler<TestMsg> = CommandHandler::default();
        assert!(!handler.should_quit());
        assert!(handler.pending_messages.is_empty());
    }

    #[test]
    fn test_command_perform_async() {
        let cmd: Command<TestMsg> = Command::perform_async(async { Some(TestMsg::A) });

        // Async commands are not empty
        assert!(!cmd.is_none());

        // Sync handler skips async actions
        let mut handler = CommandHandler::new();
        handler.execute(cmd);
        assert!(handler.take_messages().is_empty());
    }

    #[test]
    fn test_command_future_alias() {
        let cmd: Command<TestMsg> = Command::future(async { Some(TestMsg::A) });

        // Should behave identically to perform_async
        assert!(!cmd.is_none());

        // Sync handler skips async actions
        let mut handler = CommandHandler::new();
        handler.execute(cmd);
        assert!(handler.take_messages().is_empty());
    }

    #[test]
    fn test_command_perform_async_none() {
        let cmd: Command<TestMsg> = Command::perform_async(async { None });

        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_perform_async_fallible_ok() {
        let cmd: Command<TestMsg> =
            Command::perform_async_fallible(async { Ok::<_, std::io::Error>(42) }, |result| {
                match result {
                    Ok(n) => TestMsg::Value(n),
                    Err(_) => TestMsg::A,
                }
            });

        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_perform_async_fallible_err() {
        let cmd: Command<TestMsg> = Command::perform_async_fallible(
            async { Err::<i32, _>(std::io::Error::other("test")) },
            |result| match result {
                Ok(n) => TestMsg::Value(n),
                Err(_) => TestMsg::B,
            },
        );

        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_clone_async_skipped() {
        let cmd: Command<TestMsg> = Command::perform_async(async { Some(TestMsg::A) });
        let cloned = cmd.clone();

        // Async actions can't be cloned, so cloned should be empty
        assert!(cloned.is_none());
    }

    #[test]
    fn test_command_map_async() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd: Command<TestMsg> = Command::perform_async(async { Some(TestMsg::A) });
        let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

        // Mapped async command should still exist
        assert!(!mapped.is_none());
    }

    #[test]
    fn test_command_handler_skips_async() {
        let cmd: Command<TestMsg> = Command::perform_async(async { Some(TestMsg::A) });

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        // Sync handler ignores async actions
        assert!(handler.take_messages().is_empty());
        assert!(!handler.should_quit());
    }

    #[test]
    fn test_command_combine_with_async() {
        let cmd = Command::combine([
            Command::message(TestMsg::A),
            Command::perform_async(async { Some(TestMsg::B) }),
            Command::message(TestMsg::C),
        ]);

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        // Only sync messages are processed
        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::C]);
    }

    #[test]
    fn test_command_and_with_async() {
        let cmd = Command::message(TestMsg::A)
            .and(Command::perform_async(async { Some(TestMsg::B) }))
            .and(Command::quit());

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A]);
        assert!(handler.should_quit());
    }

    #[test]
    fn test_command_try_perform_async_ok() {
        let cmd: Command<TestMsg> =
            Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
                Some(TestMsg::Value(n))
            });

        // Command should not be empty
        assert!(!cmd.is_none());

        // Sync handler skips async actions
        let mut handler = CommandHandler::new();
        handler.execute(cmd);
        assert!(handler.take_messages().is_empty());
    }

    #[test]
    fn test_command_try_perform_async_err() {
        let cmd: Command<TestMsg> = Command::try_perform_async(
            async { Err::<i32, _>(std::io::Error::other("test error")) },
            |n| Some(TestMsg::Value(n)),
        );

        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_try_perform_async_returns_none() {
        let cmd: Command<TestMsg> =
            Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |_n| None);

        assert!(!cmd.is_none());
    }

    #[test]
    fn test_command_map_async_fallible() {
        #[derive(Clone, Debug, PartialEq)]
        enum OuterMsg {
            Inner(TestMsg),
        }

        let cmd: Command<TestMsg> =
            Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
                Some(TestMsg::Value(n))
            });

        let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

        // Mapped command should still exist
        assert!(!mapped.is_none());
    }

    #[test]
    fn test_command_clone_async_fallible_skipped() {
        let cmd: Command<TestMsg> =
            Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
                Some(TestMsg::Value(n))
            });

        let cloned = cmd.clone();

        // AsyncFallible can't be cloned, so cloned should be empty
        assert!(cloned.is_none());
    }

    #[test]
    fn test_command_combine_with_async_fallible() {
        let cmd = Command::combine([
            Command::message(TestMsg::A),
            Command::try_perform_async(async { Ok::<_, std::io::Error>(42) }, |n| {
                Some(TestMsg::Value(n))
            }),
            Command::message(TestMsg::C),
        ]);

        let mut handler = CommandHandler::new();
        handler.execute(cmd);

        // Only sync messages are processed
        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::C]);
    }

    mod overlay_tests {
        use super::*;
        use crate::input::Event;
        use crate::overlay::{Overlay, OverlayAction};
        use crate::theme::Theme;
        use ratatui::layout::Rect;
        use ratatui::Frame;

        struct TestOverlay;

        impl Overlay<TestMsg> for TestOverlay {
            fn handle_event(&mut self, _event: &Event) -> OverlayAction<TestMsg> {
                OverlayAction::Consumed
            }

            fn view(&self, _frame: &mut Frame, _area: Rect, _theme: &Theme) {}
        }

        #[test]
        fn test_command_push_overlay() {
            let cmd: Command<TestMsg> = Command::push_overlay(TestOverlay);
            assert!(!cmd.is_none());
        }

        #[test]
        fn test_command_pop_overlay() {
            let cmd: Command<TestMsg> = Command::pop_overlay();
            assert!(!cmd.is_none());
        }

        #[test]
        fn test_command_handler_push_overlay() {
            let mut handler = CommandHandler::new();
            handler.execute(Command::push_overlay(TestOverlay));

            // No messages produced
            assert!(handler.take_messages().is_empty());

            // But there should be a pending overlay push
            let pushes = handler.take_overlay_pushes();
            assert_eq!(pushes.len(), 1);
        }

        #[test]
        fn test_command_handler_pop_overlay() {
            let mut handler: CommandHandler<TestMsg> = CommandHandler::new();
            handler.execute(Command::pop_overlay());

            let pops = handler.take_overlay_pops();
            assert_eq!(pops, 1);
        }

        #[test]
        fn test_command_handler_multiple_overlay_ops() {
            let mut handler = CommandHandler::new();
            let cmd = Command::combine([
                Command::push_overlay(TestOverlay),
                Command::push_overlay(TestOverlay),
                Command::pop_overlay(),
                Command::message(TestMsg::A),
            ]);
            handler.execute(cmd);

            assert_eq!(handler.take_messages(), vec![TestMsg::A]);
            assert_eq!(handler.take_overlay_pushes().len(), 2);
            assert_eq!(handler.take_overlay_pops(), 1);
        }

        #[test]
        fn test_command_clone_push_overlay_skipped() {
            let cmd: Command<TestMsg> = Command::push_overlay(TestOverlay);
            let cloned = cmd.clone();

            // PushOverlay can't be cloned, so cloned should be empty
            assert!(cloned.is_none());
        }

        #[test]
        fn test_command_clone_pop_overlay_preserved() {
            let cmd: Command<TestMsg> = Command::pop_overlay();
            let cloned = cmd.clone();

            // PopOverlay can be cloned
            assert!(!cloned.is_none());
        }

        #[test]
        fn test_command_map_push_overlay_skipped() {
            #[derive(Clone, Debug, PartialEq)]
            enum OuterMsg {
                Inner(TestMsg),
            }

            let cmd: Command<TestMsg> = Command::push_overlay(TestOverlay);
            let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

            // PushOverlay can't be mapped, so mapped should be empty
            assert!(mapped.is_none());
        }

        #[test]
        fn test_command_map_pop_overlay_preserved() {
            #[derive(Clone, Debug, PartialEq)]
            enum OuterMsg {
                Inner(TestMsg),
            }

            let cmd: Command<TestMsg> = Command::pop_overlay();
            let mapped: Command<OuterMsg> = cmd.map(OuterMsg::Inner);

            // PopOverlay passes through map
            assert!(!mapped.is_none());
        }
    }
}
