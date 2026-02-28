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
    core: super::command_core::CommandHandlerCore<M>,
    pending_futures: Vec<BoxedFuture<M>>,
    pending_fallible_futures: Vec<BoxedFallibleFuture<M>>,
}

impl<M: Send + 'static> AsyncCommandHandler<M> {
    /// Creates a new async command handler.
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
    /// Async actions are collected for later spawning.
    pub fn execute(&mut self, command: Command<M>) {
        for action in command.into_actions() {
            if let Some(async_action) = self.core.execute_sync_action(action) {
                match async_action {
                    CommandAction::Async(fut) => {
                        self.pending_futures.push(fut);
                    }
                    CommandAction::AsyncFallible(fut) => {
                        self.pending_fallible_futures.push(fut);
                    }
                    _ => unreachable!("execute_sync_action only returns async actions"),
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

impl<M: Send + 'static> Default for AsyncCommandHandler<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
