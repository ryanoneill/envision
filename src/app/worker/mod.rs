//! Background task abstraction for TEA applications.
//!
//! The `Worker` module bridges async tasks to the TEA message loop with
//! progress reporting and cancellation support. It provides a high-level
//! API for spawning background work that integrates naturally with the
//! [`Command`](crate::app::Command) and [`Subscription`](crate::app::Subscription)
//! system.
//!
//! # Example
//!
//! ```rust
//! use envision::app::worker::{WorkerBuilder, WorkerProgress};
//! use envision::app::Command;
//! use std::time::Duration;
//!
//! #[derive(Clone)]
//! enum Msg {
//!     Progress(WorkerProgress),
//!     Done(String),
//!     Failed(String),
//! }
//!
//! // Spawn a simple worker (no progress reporting)
//! let (cmd, handle) = WorkerBuilder::new("download")
//!     .spawn_simple(
//!         |_cancel| async move {
//!             Ok::<_, String>("data".to_string())
//!         },
//!         |result: Result<String, String>| match result {
//!             Ok(data) => Msg::Done(data),
//!             Err(e) => Msg::Failed(e),
//!         },
//!     );
//!
//! // Cancel if needed
//! handle.cancel();
//! assert!(handle.is_cancelled());
//! ```

use std::future::Future;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::command::Command;
use super::subscription::{BoxedSubscription, ChannelSubscription, MappedSubscription};

/// Progress information from a background worker.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkerProgress {
    /// Progress percentage (0.0 to 1.0, clamped).
    percentage: f32,
    /// Optional status message describing current work.
    status: Option<String>,
}

impl WorkerProgress {
    /// Creates a new progress update.
    ///
    /// The percentage is clamped to the range 0.0..=1.0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::worker::WorkerProgress;
    ///
    /// let progress = WorkerProgress::new(0.5, Some("Downloading...".to_string()));
    /// assert_eq!(progress.percentage(), 0.5);
    /// assert_eq!(progress.status(), Some("Downloading..."));
    /// ```
    pub fn new(percentage: f32, status: Option<String>) -> Self {
        Self {
            percentage: percentage.clamp(0.0, 1.0),
            status,
        }
    }

    /// Returns the progress percentage (0.0 to 1.0).
    pub fn percentage(&self) -> f32 {
        self.percentage
    }

    /// Returns the status message, if any.
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
}

/// A sender for reporting progress from within a worker task.
///
/// This is passed to the worker's task function to allow it to
/// send progress updates back to the application's message loop.
#[derive(Clone)]
pub struct ProgressSender {
    tx: mpsc::Sender<WorkerProgress>,
}

impl ProgressSender {
    /// Sends a progress update.
    ///
    /// Returns `Ok(())` if the message was sent, or `Err` if the
    /// channel is closed (e.g., the worker was cancelled).
    pub async fn send(&self, progress: WorkerProgress) -> Result<(), mpsc::error::SendError<WorkerProgress>> {
        self.tx.send(progress).await
    }

    /// Sends a progress update with just a percentage.
    pub async fn send_percentage(&self, percentage: f32) -> Result<(), mpsc::error::SendError<WorkerProgress>> {
        self.send(WorkerProgress::new(percentage, None)).await
    }

    /// Sends a progress update with a percentage and status message.
    pub async fn send_status(&self, percentage: f32, status: impl Into<String>) -> Result<(), mpsc::error::SendError<WorkerProgress>> {
        self.send(WorkerProgress::new(percentage, Some(status.into()))).await
    }
}

/// A handle to a running worker that supports cancellation.
///
/// When dropped, the worker is automatically cancelled.
///
/// # Example
///
/// ```rust
/// use envision::app::worker::WorkerHandle;
///
/// // Handles are returned from Worker::spawn and Worker::spawn_simple
/// // Cancellation is automatic on drop, or explicit via cancel()
/// ```
pub struct WorkerHandle {
    cancel: CancellationToken,
    id: String,
}

impl WorkerHandle {
    /// Cancels the worker.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// Returns true if the worker has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// Returns the worker's identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

/// Default channel capacity for progress updates.
const DEFAULT_CHANNEL_CAPACITY: usize = 32;

/// Builder for creating and spawning workers.
///
/// Created via [`WorkerBuilder::new`].
pub struct WorkerBuilder {
    id: String,
    channel_capacity: usize,
}

impl WorkerBuilder {
    /// Creates a new worker builder with the given identifier.
    ///
    /// The identifier is used for tracking and debugging purposes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::worker::{WorkerBuilder, WorkerProgress};
    /// use envision::app::Command;
    ///
    /// #[derive(Clone)]
    /// enum Msg {
    ///     Done(Vec<u8>),
    ///     Failed(String),
    /// }
    ///
    /// let (cmd, handle) = WorkerBuilder::new("fetch")
    ///     .spawn_simple(
    ///         |cancel| async move {
    ///             Ok::<_, String>(vec![1, 2, 3])
    ///         },
    ///         |result: Result<Vec<u8>, String>| match result {
    ///             Ok(data) => Msg::Done(data),
    ///             Err(e) => Msg::Failed(e),
    ///         },
    ///     );
    /// ```
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            channel_capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }
    /// Sets the channel capacity for progress updates.
    ///
    /// Default is 32. Higher values prevent the worker from blocking
    /// when the application is slow to process progress messages.
    pub fn with_channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    /// Spawns a worker with progress reporting.
    ///
    /// Returns:
    /// - A [`Command`] that executes the async task
    /// - A [`BoxedSubscription`] that delivers progress updates as mapped messages
    /// - A [`WorkerHandle`] for cancellation
    ///
    /// # Type Parameters
    ///
    /// - `T`: The success type of the task
    /// - `E`: The error type of the task
    /// - `Fut`: The future returned by `task_fn`
    /// - `F`: The task function, receiving a `ProgressSender` and `CancellationToken`
    /// - `P`: Progress message mapper
    /// - `C`: Completion message mapper
    pub fn spawn<M, T, E, Fut, F, P, C>(
        self,
        task_fn: F,
        on_progress: P,
        on_complete: C,
    ) -> (Command<M>, BoxedSubscription<M>, WorkerHandle)
    where
        M: Send + Clone + 'static,
        T: Send + 'static,
        E: Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        F: FnOnce(ProgressSender, CancellationToken) -> Fut + Send + 'static,
        P: Fn(WorkerProgress) -> M + Send + 'static,
        C: FnOnce(Result<T, E>) -> M + Send + 'static,
    {
        let cancel = CancellationToken::new();
        let cancel_task = cancel.clone();

        let (progress_tx, progress_rx) = mpsc::channel(self.channel_capacity);
        let sender = ProgressSender { tx: progress_tx };

        let cmd = Command::perform_async(async move {
            let result = tokio::select! {
                result = task_fn(sender, cancel_task.clone()) => result,
                _ = cancel_task.cancelled() => return None,
            };
            Some(on_complete(result))
        });

        let subscription: BoxedSubscription<M> = Box::new(
            MappedSubscription::new(ChannelSubscription::new(progress_rx), on_progress),
        );

        let handle = WorkerHandle {
            cancel,
            id: self.id,
        };

        (cmd, subscription, handle)
    }

    /// Spawns a simple worker without progress reporting.
    ///
    /// Returns:
    /// - A [`Command`] that executes the async task
    /// - A [`WorkerHandle`] for cancellation
    ///
    /// # Type Parameters
    ///
    /// - `T`: The success type of the task
    /// - `E`: The error type of the task
    /// - `Fut`: The future returned by `task_fn`
    /// - `F`: The task function, receiving a `CancellationToken`
    /// - `C`: Completion message mapper
    pub fn spawn_simple<M, T, E, Fut, F, C>(
        self,
        task_fn: F,
        on_complete: C,
    ) -> (Command<M>, WorkerHandle)
    where
        M: Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        C: FnOnce(Result<T, E>) -> M + Send + 'static,
    {
        let cancel = CancellationToken::new();
        let cancel_task = cancel.clone();

        let cmd = Command::perform_async(async move {
            let result = tokio::select! {
                result = task_fn(cancel_task.clone()) => result,
                _ = cancel_task.cancelled() => return None,
            };
            Some(on_complete(result))
        });

        let handle = WorkerHandle {
            cancel,
            id: self.id,
        };

        (cmd, handle)
    }
}

#[cfg(test)]
mod tests;
