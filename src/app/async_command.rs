//! Async command handling for TEA applications.
//!
//! This module provides the infrastructure for executing async commands
//! returned from update functions. It spawns futures as tokio tasks and
//! sends their results back via a message channel.

use std::future::Future;
use std::pin::Pin;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::command::{AsyncFallibleResult, BoxedError, Command, CommandAction};
use crate::overlay::Overlay;

/// A boxed future that produces an optional message.
pub type BoxedFuture<M> = Pin<Box<dyn Future<Output = Option<M>> + Send + 'static>>;

/// A boxed future that can fail - errors are sent to the error channel.
pub type BoxedFallibleFuture<M> =
    Pin<Box<dyn Future<Output = AsyncFallibleResult<M>> + Send + 'static>>;

/// Handles execution of async commands.
///
/// This handler spawns async futures as tokio tasks and sends their results
/// back to the runtime via a message channel. It also handles synchronous
/// command actions directly.
pub struct AsyncCommandHandler<M> {
    pending_messages: Vec<M>,
    pending_futures: Vec<BoxedFuture<M>>,
    pending_fallible_futures: Vec<BoxedFallibleFuture<M>>,
    pending_overlay_pushes: Vec<Box<dyn Overlay<M> + Send>>,
    pending_overlay_pops: usize,
    should_quit: bool,
}

impl<M: Send + 'static> AsyncCommandHandler<M> {
    /// Creates a new async command handler.
    pub fn new() -> Self {
        Self {
            pending_messages: Vec::new(),
            pending_futures: Vec::new(),
            pending_fallible_futures: Vec::new(),
            pending_overlay_pushes: Vec::new(),
            pending_overlay_pops: 0,
            should_quit: false,
        }
    }

    /// Executes a command, collecting sync messages and async futures.
    ///
    /// Sync actions (Message, Batch, Quit, Callback) are processed immediately.
    /// Async actions are collected for later spawning.
    pub fn execute(&mut self, command: Command<M>) {
        for action in command.into_actions() {
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
                CommandAction::Async(fut) => {
                    self.pending_futures.push(fut);
                }
                CommandAction::AsyncFallible(fut) => {
                    self.pending_fallible_futures.push(fut);
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

    /// Spawns all pending async futures as tokio tasks.
    ///
    /// Each future is spawned with access to the message sender and cancellation token.
    /// When a future completes with `Some(message)`, the message is sent to the runtime.
    ///
    /// For fallible futures, errors are sent to the error channel instead.
    pub fn spawn_pending(
        &mut self,
        msg_tx: mpsc::Sender<M>,
        err_tx: mpsc::Sender<BoxedError>,
        cancel: CancellationToken,
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

    /// Takes all pending sync messages.
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
        self.should_quit
    }

    /// Resets the quit flag.
    pub fn reset_quit(&mut self) {
        self.should_quit = false;
    }
}

impl<M: Send + 'static> Default for AsyncCommandHandler<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        A,
        B,
        AsyncResult(i32),
    }

    #[test]
    fn test_async_handler_sync_message() {
        let mut handler = AsyncCommandHandler::new();
        handler.execute(Command::message(TestMsg::A));

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A]);
        assert!(!handler.has_pending_futures());
    }

    #[test]
    fn test_async_handler_sync_batch() {
        let mut handler = AsyncCommandHandler::new();
        handler.execute(Command::batch([TestMsg::A, TestMsg::B]));

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A, TestMsg::B]);
    }

    #[test]
    fn test_async_handler_quit() {
        let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
        assert!(!handler.should_quit());

        handler.execute(Command::quit());
        assert!(handler.should_quit());
    }

    #[test]
    fn test_async_handler_callback() {
        let mut handler = AsyncCommandHandler::new();
        handler.execute(Command::perform(|| Some(TestMsg::A)));

        let messages = handler.take_messages();
        assert_eq!(messages, vec![TestMsg::A]);
    }

    #[test]
    fn test_async_handler_async_command_pending() {
        let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();

        handler.execute(Command::perform_async(async {
            Some(TestMsg::AsyncResult(42))
        }));

        assert!(handler.has_pending_futures());
        assert_eq!(handler.pending_future_count(), 1);
        assert!(handler.take_messages().is_empty());
    }

    #[tokio::test]
    async fn test_async_handler_spawn_and_receive() {
        let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
        let (msg_tx, mut msg_rx) = mpsc::channel(10);
        let (err_tx, _err_rx) = mpsc::channel(10);
        let cancel = CancellationToken::new();

        handler.execute(Command::perform_async(async {
            Some(TestMsg::AsyncResult(42))
        }));

        handler.spawn_pending(msg_tx, err_tx, cancel);
        assert!(!handler.has_pending_futures());

        // Receive the message from the spawned task
        let msg = msg_rx.recv().await.expect("Should receive message");
        assert_eq!(msg, TestMsg::AsyncResult(42));
    }

    #[tokio::test]
    async fn test_async_handler_spawn_none_result() {
        let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
        let (msg_tx, mut msg_rx) = mpsc::channel(10);
        let (err_tx, _err_rx) = mpsc::channel(10);
        let cancel = CancellationToken::new();

        handler.execute(Command::perform_async(async { None }));

        handler.spawn_pending(msg_tx, err_tx, cancel);

        // Give the task time to complete
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should not receive any message
        assert!(msg_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_async_handler_cancellation() {
        let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
        let (msg_tx, mut msg_rx) = mpsc::channel(10);
        let (err_tx, _err_rx) = mpsc::channel(10);
        let cancel = CancellationToken::new();

        // Create a slow async command
        handler.execute(Command::perform_async(async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            Some(TestMsg::AsyncResult(42))
        }));

        handler.spawn_pending(msg_tx, err_tx, cancel.clone());

        // Cancel immediately
        cancel.cancel();

        // Give the task time to notice cancellation
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should not receive any message
        assert!(msg_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_async_handler_multiple_async() {
        let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
        let (msg_tx, mut msg_rx) = mpsc::channel(10);
        let (err_tx, _err_rx) = mpsc::channel(10);
        let cancel = CancellationToken::new();

        handler.execute(Command::perform_async(async {
            Some(TestMsg::AsyncResult(1))
        }));
        handler.execute(Command::perform_async(async {
            Some(TestMsg::AsyncResult(2))
        }));
        handler.execute(Command::perform_async(async {
            Some(TestMsg::AsyncResult(3))
        }));

        assert_eq!(handler.pending_future_count(), 3);

        handler.spawn_pending(msg_tx, err_tx, cancel);

        // Collect all messages
        let mut messages = Vec::new();
        for _ in 0..3 {
            let msg = msg_rx.recv().await.expect("Should receive message");
            messages.push(msg);
        }

        // Order is not guaranteed, so just check we got all values
        assert!(messages.contains(&TestMsg::AsyncResult(1)));
        assert!(messages.contains(&TestMsg::AsyncResult(2)));
        assert!(messages.contains(&TestMsg::AsyncResult(3)));
    }

    #[test]
    fn test_async_handler_reset_quit() {
        let mut handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::new();
        handler.execute(Command::quit());
        assert!(handler.should_quit());

        handler.reset_quit();
        assert!(!handler.should_quit());
    }

    #[test]
    fn test_async_handler_default() {
        let handler: AsyncCommandHandler<TestMsg> = AsyncCommandHandler::default();
        assert!(!handler.should_quit());
        assert!(!handler.has_pending_futures());
    }
}
