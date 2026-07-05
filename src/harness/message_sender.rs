//! `MessageSender<M>` — first-party wrapper around the async message channel
//! that carries messages into an [`AppHarness`](crate::harness::AppHarness).
//!
//! Hides the underlying `tokio::sync::mpsc::Sender<M>` so envision consumers
//! don't need `tokio` as a direct dependency to use the message-injection
//! surface. Full tokio Sender semantics are preserved through passthrough
//! methods (`send`, `try_send`, `is_closed`, `capacity`, `max_capacity`)
//! plus an explicit [`into_inner`](MessageSender::into_inner) escape hatch
//! for the small number of consumers who need tokio-specific functionality
//! (`reserve`, `send_timeout`, `same_channel`, `downgrade`, `closed()` future).

use tokio::sync::mpsc;

/// Hands the caller a way to inject messages into the AppHarness's Runtime
/// asynchronously — from subscription callbacks, spawned tasks, or any other
/// non-App-loop code path.
///
/// Wraps `tokio::sync::mpsc::Sender<M>` so envision consumers don't need
/// `tokio` as a direct dependency to use `AppHarness::message_sender()`.
/// The Sender's semantics are preserved (bounded, cloneable, `send` returns
/// `Result` on receiver-dropped) and its non-mutating query surface
/// (`is_closed`, `capacity`, `max_capacity`) is passed through. Consumers
/// needing tokio-specific functionality beyond what's exposed can call
/// [`into_inner`](Self::into_inner) as an explicit escape hatch.
///
/// # Type parameter
///
/// `MessageSender<M>` is parameterized on the message type `M`, not on an
/// App-typed generic. This means portable helper functions like
/// `fn spawn_watcher<M: Send + 'static>(sender: MessageSender<M>) { ... }`
/// work without depending on envision's `App` trait.
///
/// # Example
///
/// ```rust,no_run
/// use envision::prelude::*;
///
/// async fn ingest<M: Send + std::fmt::Debug + 'static>(sender: MessageSender<M>, msg: M) {
///     sender.send(msg).await.expect("harness still alive");
/// }
/// ```
pub struct MessageSender<M> {
    inner: mpsc::Sender<M>,
}

impl<M> MessageSender<M> {
    /// Wraps the given tokio Sender. Internal constructor — external
    /// consumers acquire a `MessageSender<M>` via
    /// [`AppHarness::message_sender()`](crate::harness::AppHarness::message_sender).
    pub(crate) fn new(inner: mpsc::Sender<M>) -> Self {
        Self { inner }
    }

    /// Sends a message into the AppHarness. Returns an error only when the
    /// AppHarness (and hence the receiver) has been dropped.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use envision::prelude::*;
    ///
    /// async fn example<M: Send + std::fmt::Debug + 'static>(sender: MessageSender<M>, msg: M) {
    ///     sender.send(msg).await.expect("harness still alive");
    /// }
    /// ```
    pub async fn send(&self, msg: M) -> Result<(), MessageSendError<M>> {
        self.inner
            .send(msg)
            .await
            .map_err(|e| MessageSendError(e.0))
    }

    /// Attempts to send a message without waiting. Returns an error if the
    /// channel is full or the AppHarness has been dropped. The message is
    /// returned inside the error variant when send fails, so the caller can
    /// retry or handle it.
    pub fn try_send(&self, msg: M) -> Result<(), TrySendError<M>> {
        self.inner.try_send(msg).map_err(TrySendError::from_tokio)
    }

    /// Returns `true` if the receiver end of the channel has been dropped.
    ///
    /// Useful for `spawn_watcher`-style loops that want to exit before
    /// wasting work on messages that would fail to send.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Returns the current available capacity of the channel — the number
    /// of messages that can be enqueued without blocking or hitting a
    /// `TrySendError::Full`.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Returns the maximum capacity of the channel (the bound configured
    /// at AppHarness construction).
    pub fn max_capacity(&self) -> usize {
        self.inner.max_capacity()
    }

    /// Explicit escape hatch: consumes the wrapper and returns the underlying
    /// `tokio::sync::mpsc::Sender<M>` for consumers who need tokio-specific
    /// functionality (`reserve`, `send_timeout`, `same_channel`, `downgrade`,
    /// or a `closed()` Future) that this wrapper deliberately doesn't expose
    /// to keep the default surface minimal.
    ///
    /// Using this method re-couples your code to the tokio dep; it's an
    /// escape hatch by design, not a routine call.
    pub fn into_inner(self) -> mpsc::Sender<M> {
        self.inner
    }
}

impl<M> Clone for MessageSender<M> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<M> std::fmt::Debug for MessageSender<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageSender").finish_non_exhaustive()
    }
}

/// Error returned by [`MessageSender::send`] when the AppHarness receiver
/// has been dropped. Carries the message back so the caller can inspect it.
#[derive(Debug)]
pub struct MessageSendError<T>(pub T);

impl<T> std::fmt::Display for MessageSendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "message sender: receiver dropped")
    }
}

impl<T: std::fmt::Debug> std::error::Error for MessageSendError<T> {}

/// Error returned by [`MessageSender::try_send`].
///
/// Preserves tokio's `Full` / `Closed` distinction so consumers can
/// retry-on-full or exit-on-closed with a match arm.
#[derive(Debug)]
pub enum TrySendError<T> {
    /// Channel is full; the message was NOT sent. Caller may retry later
    /// once the AppHarness has drained.
    Full(T),
    /// AppHarness receiver has been dropped; the message was NOT sent and
    /// will never succeed on retry.
    Closed(T),
}

impl<T> TrySendError<T> {
    /// Extracts the message from either variant.
    pub fn into_inner(self) -> T {
        match self {
            Self::Full(t) | Self::Closed(t) => t,
        }
    }

    /// Internal converter from tokio's `TrySendError<T>` — decouples the
    /// call sites from tokio's error path.
    fn from_tokio(err: mpsc::error::TrySendError<T>) -> Self {
        match err {
            mpsc::error::TrySendError::Full(t) => TrySendError::Full(t),
            mpsc::error::TrySendError::Closed(t) => TrySendError::Closed(t),
        }
    }
}

impl<T> std::fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(_) => write!(f, "message sender: channel full"),
            Self::Closed(_) => write!(f, "message sender: receiver dropped"),
        }
    }
}

impl<T: std::fmt::Debug> std::error::Error for TrySendError<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time gate that MessageSender<M> is Send + Sync + Clone when
    // M is Send. Uses an inline shim rather than the `static_assertions`
    // crate (not in dev-dependencies).
    fn _assert_send_sync_clone<T: Send + Sync + Clone>() {}

    fn _compile_assertions() {
        _assert_send_sync_clone::<MessageSender<u32>>();
        _assert_send_sync_clone::<MessageSender<String>>();
    }

    #[tokio::test]
    async fn test_send_round_trip() {
        let (tx, mut rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        sender.send(42).await.expect("send succeeds");

        let received = rx.recv().await;
        assert_eq!(received, Some(42));
    }

    #[tokio::test]
    async fn test_try_send_full() {
        // Channel with capacity 1; fill it, then try_send should return Full.
        let (tx, _rx) = mpsc::channel::<u32>(1);
        let sender = MessageSender::new(tx);

        // Fill the buffer without draining.
        sender.try_send(1).expect("first try_send succeeds");
        // Second try_send should return Full (buffer full).
        let err = sender.try_send(2).expect_err("second try_send is Full");
        assert!(matches!(err, TrySendError::Full(2)));
    }

    #[tokio::test]
    async fn test_send_closed() {
        let (tx, rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        drop(rx);

        let err = sender.send(99).await.expect_err("send fails");
        assert_eq!(err.0, 99);
    }

    #[tokio::test]
    async fn test_try_send_closed() {
        let (tx, rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        drop(rx);

        let err = sender.try_send(99).expect_err("try_send fails");
        assert!(matches!(err, TrySendError::Closed(99)));
    }

    #[tokio::test]
    async fn test_is_closed() {
        let (tx, rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        assert!(!sender.is_closed());
        drop(rx);
        assert!(sender.is_closed());
    }

    #[tokio::test]
    async fn test_capacity_and_max_capacity() {
        let (tx, _rx) = mpsc::channel::<u32>(4);
        let sender = MessageSender::new(tx);

        assert_eq!(sender.max_capacity(), 4);
        assert_eq!(sender.capacity(), 4);

        // After filling one slot, capacity decreases.
        sender.try_send(1).expect("first try_send");
        assert_eq!(sender.capacity(), 3);
        assert_eq!(sender.max_capacity(), 4); // unchanged
    }

    #[tokio::test]
    async fn test_clone_shares_channel() {
        let (tx, mut rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);
        let cloned = sender.clone();

        sender.send(1).await.expect("original send");
        cloned.send(2).await.expect("cloned send");

        assert_eq!(rx.recv().await, Some(1));
        assert_eq!(rx.recv().await, Some(2));
    }

    #[tokio::test]
    async fn test_into_inner_escape_hatch() {
        let (tx, mut rx) = mpsc::channel::<u32>(16);
        let sender = MessageSender::new(tx);

        let inner: mpsc::Sender<u32> = sender.into_inner();
        inner.send(42).await.expect("inner tokio send succeeds");

        assert_eq!(rx.recv().await, Some(42));
    }

    #[test]
    fn test_try_send_error_into_inner() {
        let err = TrySendError::Full(42u32);
        assert_eq!(err.into_inner(), 42);

        let err = TrySendError::Closed(99u32);
        assert_eq!(err.into_inner(), 99);
    }

    #[test]
    fn test_message_send_error_message_recovered() {
        let err = MessageSendError(42u32);
        assert_eq!(err.0, 42);
    }

    #[test]
    fn test_display_impls() {
        let msg_err: MessageSendError<u32> = MessageSendError(42);
        assert_eq!(msg_err.to_string(), "message sender: receiver dropped");

        let full: TrySendError<u32> = TrySendError::Full(1);
        assert_eq!(full.to_string(), "message sender: channel full");

        let closed: TrySendError<u32> = TrySendError::Closed(2);
        assert_eq!(closed.to_string(), "message sender: receiver dropped");
    }
}
