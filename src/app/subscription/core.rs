use std::pin::Pin;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_util::sync::CancellationToken;

/// A subscription that produces messages over time.
///
/// Subscriptions are long-running async streams that emit messages. They're
/// typically used for timers, events from external sources, or any ongoing
/// async operation that produces multiple messages.
pub trait Subscription<M>: Send + 'static {
    /// Converts this subscription into a stream of messages.
    ///
    /// The stream runs until it naturally ends or the cancellation token is triggered.
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>>;
}

/// A boxed subscription.
pub type BoxedSubscription<M> = Box<dyn Subscription<M>>;

/// A subscription that fires at regular intervals.
///
/// Each tick produces a message using the provided function.
///
/// # Example
///
/// ```rust
/// use envision::app::TickSubscription;
/// use std::time::Duration;
///
/// let tick = TickSubscription::new(Duration::from_secs(1), || "tick");
/// ```
pub struct TickSubscription<M, F>
where
    F: Fn() -> M + Send + 'static,
{
    pub(crate) interval: Duration,
    message_fn: F,
}

impl<M, F> TickSubscription<M, F>
where
    F: Fn() -> M + Send + 'static,
{
    /// Creates a new tick subscription with the given interval and message function.
    pub fn new(interval: Duration, message_fn: F) -> Self {
        Self {
            interval,
            message_fn,
        }
    }
}

impl<M: Send + 'static, F: Fn() -> M + Send + 'static> Subscription<M> for TickSubscription<M, F> {
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        let interval_duration = self.interval;
        let message_fn = self.message_fn;

        Box::pin(async_stream::stream! {
            let mut interval = tokio::time::interval(interval_duration);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        yield (message_fn)();
                    }
                    _ = cancel.cancelled() => {
                        break;
                    }
                }
            }
        })
    }
}

/// Builder for tick subscriptions with a fluent API.
pub struct TickSubscriptionBuilder {
    interval: Duration,
}

impl TickSubscriptionBuilder {
    /// Creates a tick subscription builder with the given interval.
    pub fn every(interval: Duration) -> Self {
        Self { interval }
    }

    /// Sets the message to produce on each tick.
    pub fn with_message<M, F>(self, message_fn: F) -> TickSubscription<M, F>
    where
        F: Fn() -> M + Send + 'static,
    {
        TickSubscription::new(self.interval, message_fn)
    }
}

/// Creates a tick subscription builder.
///
/// # Example
///
/// ```rust
/// use envision::app::tick;
/// use std::time::Duration;
///
/// let sub = tick(Duration::from_secs(1)).with_message(|| "tick");
/// ```
pub fn tick(interval: Duration) -> TickSubscriptionBuilder {
    TickSubscriptionBuilder::every(interval)
}

/// A subscription that fires once after a delay.
///
/// # Example
///
/// ```rust
/// use envision::app::TimerSubscription;
/// use std::time::Duration;
///
/// let timer = TimerSubscription::after(Duration::from_secs(5), "timeout");
/// ```
pub struct TimerSubscription<M> {
    pub(crate) delay: Duration,
    pub(crate) message: M,
}

impl<M> TimerSubscription<M> {
    /// Creates a timer that fires the given message after the delay.
    pub fn after(delay: Duration, message: M) -> Self {
        Self { delay, message }
    }
}

impl<M: Send + 'static> Subscription<M> for TimerSubscription<M> {
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        let delay = self.delay;
        let message = self.message;

        Box::pin(async_stream::stream! {
            tokio::select! {
                _ = tokio::time::sleep(delay) => {
                    yield message;
                }
                _ = cancel.cancelled() => {}
            }
        })
    }
}

/// A subscription that receives messages from a channel.
///
/// This is useful for receiving events from external sources like
/// websockets, file watchers, or other async operations.
///
/// # Example
///
/// ```rust
/// use envision::app::ChannelSubscription;
///
/// let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);
/// let subscription = ChannelSubscription::new(rx);
/// ```
pub struct ChannelSubscription<M> {
    receiver: mpsc::Receiver<M>,
}

impl<M> ChannelSubscription<M> {
    /// Creates a subscription from a channel receiver.
    pub fn new(receiver: mpsc::Receiver<M>) -> Self {
        Self { receiver }
    }
}

impl<M: Send + 'static> Subscription<M> for ChannelSubscription<M> {
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        let mut receiver = self.receiver;

        Box::pin(async_stream::stream! {
            loop {
                tokio::select! {
                    msg = receiver.recv() => {
                        match msg {
                            Some(m) => yield m,
                            None => break, // Channel closed
                        }
                    }
                    _ = cancel.cancelled() => {
                        break;
                    }
                }
            }
        })
    }
}

/// A subscription that receives messages from an unbounded channel.
///
/// This is the unbounded counterpart of [`ChannelSubscription`]. Use this
/// when the sender should never block, at the cost of unbounded memory
/// growth if messages are produced faster than consumed.
///
/// # Example
///
/// ```rust
/// use envision::app::UnboundedChannelSubscription;
///
/// let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();
/// let subscription = UnboundedChannelSubscription::new(rx);
/// ```
pub struct UnboundedChannelSubscription<M> {
    receiver: mpsc::UnboundedReceiver<M>,
}

impl<M> UnboundedChannelSubscription<M> {
    /// Creates a subscription from an unbounded channel receiver.
    pub fn new(receiver: mpsc::UnboundedReceiver<M>) -> Self {
        Self { receiver }
    }
}

impl<M: Send + 'static> Subscription<M> for UnboundedChannelSubscription<M> {
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        let mut receiver = self.receiver;

        Box::pin(async_stream::stream! {
            loop {
                tokio::select! {
                    msg = receiver.recv() => {
                        match msg {
                            Some(m) => yield m,
                            None => break, // Channel closed
                        }
                    }
                    _ = cancel.cancelled() => {
                        break;
                    }
                }
            }
        })
    }
}

/// A subscription that wraps a stream directly.
///
/// This allows using any async stream as a subscription.
///
/// # Example
///
/// ```rust
/// use envision::app::StreamSubscription;
///
/// let stream = tokio_stream::pending::<String>();
/// let subscription = StreamSubscription::new(stream);
/// ```
pub struct StreamSubscription<S> {
    stream: S,
}

impl<S> StreamSubscription<S> {
    /// Creates a subscription from any stream.
    pub fn new(stream: S) -> Self {
        Self { stream }
    }
}

impl<M, S> Subscription<M> for StreamSubscription<S>
where
    M: Send + 'static,
    S: Stream<Item = M> + Send + Unpin + 'static,
{
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        use tokio_stream::StreamExt;
        let mut inner = self.stream;

        Box::pin(async_stream::stream! {
            loop {
                tokio::select! {
                    item = inner.next() => {
                        match item {
                            Some(m) => yield m,
                            None => break, // Stream ended
                        }
                    }
                    _ = cancel.cancelled() => {
                        break;
                    }
                }
            }
        })
    }
}
