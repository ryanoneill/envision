//! Background task abstraction for TEA applications.
//!
//! The `Worker` module bridges async tasks to the TEA message loop with
//! typed progress reporting and cancellation support. It provides a high-level
//! API for spawning background work that integrates naturally with the
//! [`Command`] and [`Subscription`](crate::app::subscription::Subscription)
//! system.
//!
//! # Progress Reporting
//!
//! Workers report progress via a [`ProgressSender<P>`] where `P` is your
//! own progress type — an enum, struct, or any `Send + 'static` type.
//! Use [`send`](ProgressSender::send) for important lifecycle events and
//! [`try_send`](ProgressSender::try_send) for high-frequency updates where
//! dropping one is acceptable.
//!
//! # Example
//!
//! ```rust
//! use envision::app::worker::WorkerBuilder;
//! use envision::app::Command;
//!
//! #[derive(Clone)]
//! enum Progress {
//!     ChapterCount(usize),
//!     Encoding { percent: f32 },
//!     FileSize(u64),
//! }
//!
//! #[derive(Clone)]
//! enum Msg {
//!     Update(Progress),
//!     Done(String),
//!     Failed(String),
//! }
//!
//! let (cmd, sub, handle) = WorkerBuilder::new("transcode")
//!     .with_channel_capacity(128)
//!     .spawn(
//!         |sender, _cancel| async move {
//!             sender.send(Progress::ChapterCount(12)).await.ok();
//!             // High-frequency updates: try_send drops if channel full
//!             sender.try_send(Progress::Encoding { percent: 0.5 }).ok();
//!             Ok::<_, String>("output.m4b".to_string())
//!         },
//!         Msg::Update,
//!         |result: Result<String, String>| match result {
//!             Ok(path) => Msg::Done(path),
//!             Err(e) => Msg::Failed(e),
//!         },
//!     );
//!
//! // Cancel if needed
//! handle.cancel();
//! ```

use std::future::Future;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::command::Command;
use super::subscription::{BoxedSubscription, ChannelSubscription, MappedSubscription};

/// A convenience progress type providing percentage and status string.
///
/// This is one possible type for `ProgressSender<P>`. Use it when your
/// worker only needs to report a completion percentage and an optional
/// status message. For richer progress reporting, define your own type.
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
    /// let progress = WorkerProgress::new(0.75, Some("Processing...".to_string()));
    /// assert_eq!(progress.percentage(), 0.75);
    /// assert_eq!(progress.status(), Some("Processing..."));
    ///
    /// // Clamping
    /// let clamped = WorkerProgress::new(1.5, None);
    /// assert_eq!(clamped.percentage(), 1.0);
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

/// A sender for reporting typed progress from within a worker task.
///
/// Generic over `P` — the progress message type. Use any `Send + 'static`
/// type: an enum with domain-specific variants, a struct, or the built-in
/// [`WorkerProgress`] for simple percentage+string reporting.
///
/// # Backpressure vs fire-and-forget
///
/// - [`send`](Self::send): async, applies backpressure. Use for important
///   lifecycle events (started, completed, failed) that must not be dropped.
/// - [`try_send`](Self::try_send): non-blocking, drops the message if the
///   channel is full. Use for high-frequency informational updates (per-frame
///   progress ticks, per-segment completions) where dropping one is better
///   than blocking the worker pipeline.
///
/// # Example
///
/// ```rust
/// use envision::app::worker::{WorkerBuilder, ProgressSender};
///
/// #[derive(Clone)]
/// enum Update {
///     Started,
///     Tick(f32),
///     Finished,
/// }
///
/// #[derive(Clone)]
/// enum Msg { Update(Update), Done(()) }
///
/// let (cmd, sub, handle) = WorkerBuilder::new("work")
///     .spawn(
///         |sender: ProgressSender<Update>, _cancel| async move {
///             sender.send(Update::Started).await.ok();
///             for i in 0..100 {
///                 // Non-blocking: ok to drop if channel is full
///                 sender.try_send(Update::Tick(i as f32 / 100.0)).ok();
///             }
///             sender.send(Update::Finished).await.ok();
///             Ok::<_, ()>(())
///         },
///         Msg::Update,
///         |_| Msg::Done(()),
///     );
/// ```
#[derive(Clone)]
pub struct ProgressSender<P> {
    tx: mpsc::Sender<P>,
}

impl<P: Send + 'static> ProgressSender<P> {
    /// Creates a new `ProgressSender` wrapping a tokio mpsc sender.
    ///
    /// Use this to construct a `ProgressSender` outside of
    /// [`WorkerBuilder`] — for example, in tests or when bridging
    /// to an existing channel.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::worker::ProgressSender;
    /// use tokio::sync::mpsc;
    ///
    /// let (tx, mut rx) = mpsc::channel::<String>(32);
    /// let sender = ProgressSender::new(tx);
    /// ```
    pub fn new(tx: mpsc::Sender<P>) -> Self {
        Self { tx }
    }

    /// Sends a progress update, waiting if the channel is full.
    ///
    /// Use this for important messages that must not be dropped (lifecycle
    /// transitions, final results, error reports).
    ///
    /// # Errors
    ///
    /// Returns `Err` if the progress channel is closed, which occurs when the
    /// worker has been cancelled or the runtime has shut down.
    pub async fn send(&self, progress: P) -> Result<(), mpsc::error::SendError<P>> {
        self.tx.send(progress).await
    }

    /// Attempts to send a progress update without blocking.
    ///
    /// Use this for high-frequency informational updates where dropping one
    /// is acceptable (progress ticks, per-segment completions, metrics).
    /// If the channel is full, the message is returned in the error.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the channel is full or closed.
    pub fn try_send(&self, progress: P) -> Result<(), mpsc::error::TrySendError<P>> {
        self.tx.try_send(progress)
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
/// // Handles are returned from WorkerBuilder::spawn and spawn_simple
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
///
/// # Example
///
/// ```rust
/// use envision::app::worker::WorkerBuilder;
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
///         |_cancel| async move {
///             Ok::<_, String>(vec![1, 2, 3])
///         },
///         |result: Result<Vec<u8>, String>| match result {
///             Ok(data) => Msg::Done(data),
///             Err(e) => Msg::Failed(e),
///         },
///     );
/// ```
pub struct WorkerBuilder {
    id: String,
    channel_capacity: usize,
}

impl WorkerBuilder {
    /// Creates a new worker builder with the given identifier.
    ///
    /// The identifier is used for tracking and debugging purposes.
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
    /// For high-frequency producers using [`try_send`](ProgressSender::try_send),
    /// 128 is a good starting point.
    pub fn with_channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    /// Spawns a worker with typed progress reporting.
    ///
    /// The worker receives a [`ProgressSender<P>`] for sending progress
    /// updates of any user-defined type back to the application's message
    /// loop.
    ///
    /// Returns:
    /// - A [`Command`] that executes the async task
    /// - A [`BoxedSubscription`] that delivers progress updates as mapped messages
    /// - A [`WorkerHandle`] for cancellation
    ///
    /// # Type Parameters
    ///
    /// - `P`: The progress type — any `Send + 'static` type (your enum, struct, etc.)
    /// - `T`: The success type of the task
    /// - `E`: The error type of the task
    /// - `Fut`: The future returned by `task_fn`
    /// - `F`: The task function, receiving a `ProgressSender<P>` and `CancellationToken`
    /// - `Pm`: Progress message mapper (`P -> M`)
    /// - `C`: Completion message mapper (`Result<T, E> -> M`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::app::worker::{WorkerBuilder, ProgressSender};
    /// use envision::app::Command;
    ///
    /// #[derive(Clone)]
    /// enum Progress { Started, Percent(f32), Done }
    ///
    /// #[derive(Clone)]
    /// enum Msg { Progress(Progress), Complete(String) }
    ///
    /// let (cmd, sub, handle) = WorkerBuilder::new("process")
    ///     .spawn(
    ///         |sender: ProgressSender<Progress>, _cancel| async move {
    ///             sender.send(Progress::Started).await.ok();
    ///             sender.try_send(Progress::Percent(0.5)).ok();
    ///             sender.send(Progress::Done).await.ok();
    ///             Ok::<_, String>("result".to_string())
    ///         },
    ///         Msg::Progress,
    ///         |result: Result<String, String>| match result {
    ///             Ok(s) => Msg::Complete(s),
    ///             Err(_) => Msg::Complete("failed".into()),
    ///         },
    ///     );
    /// ```
    pub fn spawn<M, P, T, E, Fut, F, Pm, C>(
        self,
        task_fn: F,
        on_progress: Pm,
        on_complete: C,
    ) -> (Command<M>, BoxedSubscription<M>, WorkerHandle)
    where
        M: Send + Clone + 'static,
        P: Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        F: FnOnce(ProgressSender<P>, CancellationToken) -> Fut + Send + 'static,
        Pm: Fn(P) -> M + Send + 'static,
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

        let subscription: BoxedSubscription<M> = Box::new(MappedSubscription::new(
            ChannelSubscription::new(progress_rx),
            on_progress,
        ));

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
    /// # Example
    ///
    /// ```rust
    /// use envision::app::worker::WorkerBuilder;
    /// use envision::app::Command;
    ///
    /// #[derive(Clone)]
    /// enum Msg { Done(String), Failed(String) }
    ///
    /// let (cmd, handle) = WorkerBuilder::new("fetch")
    ///     .spawn_simple(
    ///         |_cancel| async move {
    ///             Ok::<_, String>("data".to_string())
    ///         },
    ///         |result: Result<String, String>| match result {
    ///             Ok(data) => Msg::Done(data),
    ///             Err(e) => Msg::Failed(e),
    ///         },
    ///     );
    /// ```
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
