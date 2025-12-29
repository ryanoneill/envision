//! Subscriptions for long-running async operations in TEA applications.
//!
//! Subscriptions are async streams that produce messages over time. They're useful
//! for timers, websockets, file watchers, and other ongoing async operations.
//!
//! # Example
//!
//! ```ignore
//! use envision::app::{Subscription, TickSubscription};
//! use std::time::Duration;
//!
//! // Create a subscription that fires every second
//! let tick = TickSubscription::every(Duration::from_secs(1));
//! ```

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
/// ```ignore
/// TickSubscription::every(Duration::from_secs(1))
///     .with_message(|| Msg::Tick)
/// ```
pub struct TickSubscription<M, F>
where
    F: Fn() -> M + Send + 'static,
{
    interval: Duration,
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
/// ```ignore
/// tick(Duration::from_secs(1)).with_message(|| Msg::Tick)
/// ```
pub fn tick(interval: Duration) -> TickSubscriptionBuilder {
    TickSubscriptionBuilder::every(interval)
}

/// A subscription that fires once after a delay.
///
/// # Example
///
/// ```ignore
/// TimerSubscription::after(Duration::from_secs(5), Msg::Timeout)
/// ```
pub struct TimerSubscription<M> {
    delay: Duration,
    message: M,
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
/// ```ignore
/// let (tx, rx) = tokio::sync::mpsc::channel(100);
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

/// A subscription that wraps a stream directly.
///
/// This allows using any async stream as a subscription.
///
/// # Example
///
/// ```ignore
/// let stream = async_stream::stream! {
///     for i in 0..10 {
///         yield Msg::Count(i);
///         tokio::time::sleep(Duration::from_secs(1)).await;
///     }
/// };
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

/// A subscription that maps the messages of an inner subscription.
pub struct MappedSubscription<M, N, F, S>
where
    S: Subscription<M>,
    F: Fn(M) -> N + Send + 'static,
{
    inner: Box<S>,
    map_fn: F,
    _phantom: std::marker::PhantomData<(M, N)>,
}

impl<M, N, F, S> MappedSubscription<M, N, F, S>
where
    S: Subscription<M>,
    F: Fn(M) -> N + Send + 'static,
{
    /// Creates a mapped subscription.
    pub fn new(inner: S, map_fn: F) -> Self {
        Self {
            inner: Box::new(inner),
            map_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M, N, F, S> Subscription<N> for MappedSubscription<M, N, F, S>
where
    M: Send + 'static,
    N: Send + 'static,
    F: Fn(M) -> N + Send + 'static,
    S: Subscription<M>,
{
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = N> + Send>> {
        use tokio_stream::StreamExt;
        let mut inner_stream = self.inner.into_stream(cancel);
        let map_fn = self.map_fn;

        Box::pin(async_stream::stream! {
            while let Some(m) = inner_stream.next().await {
                yield (map_fn)(m);
            }
        })
    }
}

/// Extension trait for subscriptions.
pub trait SubscriptionExt<M>: Subscription<M> + Sized {
    /// Maps the messages of this subscription.
    fn map<N, F>(self, f: F) -> MappedSubscription<M, N, F, Self>
    where
        F: Fn(M) -> N + Send + 'static,
    {
        MappedSubscription::new(self, f)
    }
}

impl<M, S: Subscription<M>> SubscriptionExt<M> for S {}


/// A batch of subscriptions combined into one.
pub struct BatchSubscription<M> {
    subscriptions: Vec<BoxedSubscription<M>>,
}

impl<M> BatchSubscription<M> {
    /// Creates a batch of subscriptions.
    pub fn new(subscriptions: Vec<BoxedSubscription<M>>) -> Self {
        Self { subscriptions }
    }
}

impl<M: Send + 'static> Subscription<M> for BatchSubscription<M> {
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        use futures_util::stream::SelectAll;
        use tokio_stream::StreamExt;

        let mut select_all = SelectAll::new();
        for sub in self.subscriptions {
            select_all.push(sub.into_stream(cancel.clone()));
        }

        Box::pin(async_stream::stream! {
            while let Some(msg) = select_all.next().await {
                yield msg;
            }
        })
    }
}

/// Combines multiple subscriptions into one.
pub fn batch<M: Send + 'static>(subscriptions: Vec<BoxedSubscription<M>>) -> BatchSubscription<M> {
    BatchSubscription::new(subscriptions)
}

/// A subscription that fires immediately, then at regular intervals.
///
/// Unlike [`TickSubscription`], this fires the first message immediately
/// without waiting for the interval.
///
/// # Example
///
/// ```ignore
/// IntervalImmediateSubscription::new(Duration::from_secs(1), || Msg::Tick)
/// ```
pub struct IntervalImmediateSubscription<M, F>
where
    F: Fn() -> M + Send + 'static,
{
    interval: Duration,
    message_fn: F,
}

impl<M, F> IntervalImmediateSubscription<M, F>
where
    F: Fn() -> M + Send + 'static,
{
    /// Creates a new interval immediate subscription.
    pub fn new(interval: Duration, message_fn: F) -> Self {
        Self {
            interval,
            message_fn,
        }
    }
}

impl<M: Send + 'static, F: Fn() -> M + Send + 'static> Subscription<M>
    for IntervalImmediateSubscription<M, F>
{
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        let interval_duration = self.interval;
        let message_fn = self.message_fn;

        Box::pin(async_stream::stream! {
            // Fire immediately
            yield (message_fn)();

            let mut interval = tokio::time::interval(interval_duration);
            // Skip the first tick since we already fired
            interval.tick().await;

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

/// Builder for interval immediate subscriptions with a fluent API.
pub struct IntervalImmediateBuilder {
    interval: Duration,
}

impl IntervalImmediateBuilder {
    /// Creates an interval immediate subscription builder.
    pub fn every(interval: Duration) -> Self {
        Self { interval }
    }

    /// Sets the message to produce on each tick.
    pub fn with_message<M, F>(self, message_fn: F) -> IntervalImmediateSubscription<M, F>
    where
        F: Fn() -> M + Send + 'static,
    {
        IntervalImmediateSubscription::new(self.interval, message_fn)
    }
}

/// Creates an interval immediate subscription builder that fires immediately.
///
/// # Example
///
/// ```ignore
/// interval_immediate(Duration::from_secs(1)).with_message(|| Msg::Tick)
/// ```
pub fn interval_immediate(interval: Duration) -> IntervalImmediateBuilder {
    IntervalImmediateBuilder::every(interval)
}

/// A subscription that filters messages from an inner subscription.
///
/// Only messages for which the predicate returns `true` are emitted.
///
/// # Example
///
/// ```ignore
/// some_subscription.filter(|msg| msg.is_important())
/// ```
pub struct FilterSubscription<M, S, P>
where
    S: Subscription<M>,
    P: Fn(&M) -> bool + Send + 'static,
{
    inner: Box<S>,
    predicate: P,
    _phantom: std::marker::PhantomData<M>,
}

impl<M, S, P> FilterSubscription<M, S, P>
where
    S: Subscription<M>,
    P: Fn(&M) -> bool + Send + 'static,
{
    /// Creates a filtered subscription.
    pub fn new(inner: S, predicate: P) -> Self {
        Self {
            inner: Box::new(inner),
            predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M, S, P> Subscription<M> for FilterSubscription<M, S, P>
where
    M: Send + 'static,
    S: Subscription<M>,
    P: Fn(&M) -> bool + Send + 'static,
{
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        use tokio_stream::StreamExt;

        let predicate = self.predicate;
        let mut inner = self.inner.into_stream(cancel);

        Box::pin(async_stream::stream! {
            while let Some(msg) = inner.next().await {
                if (predicate)(&msg) {
                    yield msg;
                }
            }
        })
    }
}

/// A subscription that takes only the first N messages from an inner subscription.
///
/// After N messages, the subscription ends.
///
/// # Example
///
/// ```ignore
/// some_subscription.take(5)
/// ```
pub struct TakeSubscription<M, S>
where
    S: Subscription<M>,
{
    inner: Box<S>,
    count: usize,
    _phantom: std::marker::PhantomData<M>,
}

impl<M, S> TakeSubscription<M, S>
where
    S: Subscription<M>,
{
    /// Creates a take subscription.
    pub fn new(inner: S, count: usize) -> Self {
        Self {
            inner: Box::new(inner),
            count,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M, S> Subscription<M> for TakeSubscription<M, S>
where
    M: Send + 'static,
    S: Subscription<M>,
{
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        use tokio_stream::StreamExt;

        let count = self.count;
        let mut inner = self.inner.into_stream(cancel);

        Box::pin(async_stream::stream! {
            let mut taken = 0;
            while taken < count {
                match inner.next().await {
                    Some(msg) => {
                        taken += 1;
                        yield msg;
                    }
                    None => break,
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio_stream::StreamExt;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        Tick,
        Timer,
        Value(i32),
    }

    #[tokio::test]
    async fn test_tick_subscription() {
        let cancel = CancellationToken::new();
        let sub = Box::new(TickSubscription::new(Duration::from_millis(10), || {
            TestMsg::Tick
        }));

        let mut stream = sub.into_stream(cancel.clone());

        // Get first tick
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Tick));

        // Cancel and verify stream ends
        cancel.cancel();
    }

    #[tokio::test]
    async fn test_tick_builder() {
        let cancel = CancellationToken::new();
        let sub = Box::new(tick(Duration::from_millis(10)).with_message(|| TestMsg::Tick));

        let mut stream = sub.into_stream(cancel.clone());
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Tick));

        cancel.cancel();
    }

    #[tokio::test]
    async fn test_timer_subscription() {
        let cancel = CancellationToken::new();
        let sub = Box::new(TimerSubscription::after(
            Duration::from_millis(10),
            TestMsg::Timer,
        ));

        let mut stream = sub.into_stream(cancel);

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Timer));

        // Timer should only fire once
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_timer_cancellation() {
        let cancel = CancellationToken::new();
        let sub = Box::new(TimerSubscription::after(
            Duration::from_secs(10),
            TestMsg::Timer,
        ));

        let mut stream = sub.into_stream(cancel.clone());

        // Cancel before timer fires
        cancel.cancel();

        // Stream should end
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_channel_subscription() {
        let cancel = CancellationToken::new();
        let (tx, rx) = mpsc::channel(10);
        let sub = Box::new(ChannelSubscription::new(rx));

        let mut stream = sub.into_stream(cancel.clone());

        // Send messages
        tx.send(TestMsg::Value(1)).await.unwrap();
        tx.send(TestMsg::Value(2)).await.unwrap();

        // Receive messages
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(1)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        // Drop sender to close channel
        drop(tx);

        // Stream should end
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_stream_subscription() {
        let cancel = CancellationToken::new();
        let values = vec![TestMsg::Value(1), TestMsg::Value(2), TestMsg::Value(3)];
        let inner_stream = tokio_stream::iter(values);
        let sub = Box::new(StreamSubscription::new(inner_stream));

        let mut stream = sub.into_stream(cancel);

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(1)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(3)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_mapped_subscription() {
        let cancel = CancellationToken::new();
        let inner = TickSubscription::new(Duration::from_millis(10), || 42i32);
        let sub = Box::new(inner.map(TestMsg::Value));

        let mut stream = sub.into_stream(cancel.clone());

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(42)));

        cancel.cancel();
    }

    #[tokio::test]
    async fn test_batch_subscription() {
        let cancel = CancellationToken::new();
        let (tx, rx) = mpsc::channel(10);

        let timer = Box::new(TimerSubscription::after(
            Duration::from_millis(5),
            TestMsg::Timer,
        )) as BoxedSubscription<TestMsg>;
        let channel =
            Box::new(ChannelSubscription::new(rx)) as BoxedSubscription<TestMsg>;

        let sub = Box::new(batch(vec![timer, channel]));
        let mut stream = sub.into_stream(cancel.clone());

        // Send a channel message
        tx.send(TestMsg::Value(1)).await.unwrap();

        // Collect messages (order may vary)
        let mut received = Vec::new();
        for _ in 0..2 {
            if let Some(msg) = stream.next().await {
                received.push(msg);
            }
        }

        assert!(received.contains(&TestMsg::Timer));
        assert!(received.contains(&TestMsg::Value(1)));

        cancel.cancel();
    }

    #[test]
    fn test_tick_builder_every() {
        let builder = TickSubscriptionBuilder::every(Duration::from_secs(1));
        let sub = builder.with_message(|| TestMsg::Tick);
        assert_eq!(sub.interval, Duration::from_secs(1));
    }

    #[test]
    fn test_timer_after() {
        let timer = TimerSubscription::after(Duration::from_secs(5), TestMsg::Timer);
        assert_eq!(timer.delay, Duration::from_secs(5));
        assert_eq!(timer.message, TestMsg::Timer);
    }

    #[tokio::test]
    async fn test_interval_immediate_subscription() {
        let cancel = CancellationToken::new();
        let sub = Box::new(IntervalImmediateSubscription::new(
            Duration::from_millis(100),
            || TestMsg::Tick,
        ));

        let mut stream = sub.into_stream(cancel.clone());

        // Should fire immediately without waiting for interval
        let start = std::time::Instant::now();
        let msg = stream.next().await;
        let elapsed = start.elapsed();

        assert_eq!(msg, Some(TestMsg::Tick));
        // First message should be immediate (less than the interval)
        assert!(
            elapsed < Duration::from_millis(50),
            "First message should be immediate, took {:?}",
            elapsed
        );

        cancel.cancel();
    }

    #[tokio::test]
    async fn test_interval_immediate_builder() {
        let cancel = CancellationToken::new();
        let sub = Box::new(interval_immediate(Duration::from_millis(100)).with_message(|| TestMsg::Tick));

        let mut stream = sub.into_stream(cancel.clone());

        // Should fire immediately
        let start = std::time::Instant::now();
        let msg = stream.next().await;
        let elapsed = start.elapsed();

        assert_eq!(msg, Some(TestMsg::Tick));
        assert!(elapsed < Duration::from_millis(50));

        cancel.cancel();
    }

    #[tokio::test]
    async fn test_interval_immediate_vs_tick() {
        // Compare immediate subscription vs regular tick subscription
        let cancel1 = CancellationToken::new();
        let cancel2 = CancellationToken::new();

        let immediate = Box::new(IntervalImmediateSubscription::new(
            Duration::from_millis(50),
            || TestMsg::Tick,
        ));
        let regular = Box::new(TickSubscription::new(Duration::from_millis(50), || {
            TestMsg::Tick
        }));

        let mut immediate_stream = immediate.into_stream(cancel1.clone());
        let mut regular_stream = regular.into_stream(cancel2.clone());

        // Time the first message from each
        let immediate_start = std::time::Instant::now();
        let _ = immediate_stream.next().await;
        let immediate_elapsed = immediate_start.elapsed();

        let regular_start = std::time::Instant::now();
        let _ = regular_stream.next().await;
        let regular_elapsed = regular_start.elapsed();

        // Immediate should be much faster for first message
        assert!(
            immediate_elapsed < regular_elapsed,
            "Immediate: {:?}, Regular: {:?}",
            immediate_elapsed,
            regular_elapsed
        );

        cancel1.cancel();
        cancel2.cancel();
    }

    #[tokio::test]
    async fn test_filter_subscription() {
        let cancel = CancellationToken::new();
        let values = vec![
            TestMsg::Value(1),
            TestMsg::Value(2),
            TestMsg::Value(3),
            TestMsg::Value(4),
            TestMsg::Value(5),
        ];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        let sub = Box::new(FilterSubscription::new(inner, |msg| {
            matches!(msg, TestMsg::Value(n) if *n % 2 == 0)
        }));

        let mut stream = sub.into_stream(cancel);

        // Should only get even values
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(4)));

        // Stream should end
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_filter_subscription_all_filtered() {
        let cancel = CancellationToken::new();
        let values = vec![TestMsg::Value(1), TestMsg::Value(3), TestMsg::Value(5)];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        let sub = Box::new(FilterSubscription::new(inner, |msg| {
            matches!(msg, TestMsg::Value(n) if *n % 2 == 0)
        }));

        let mut stream = sub.into_stream(cancel);

        // All values are odd, so nothing should pass through
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_filter_subscription_none_filtered() {
        let cancel = CancellationToken::new();
        let values = vec![TestMsg::Value(2), TestMsg::Value(4)];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        let sub = Box::new(FilterSubscription::new(inner, |_| true));

        let mut stream = sub.into_stream(cancel);

        // All values should pass through
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(4)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_take_subscription() {
        let cancel = CancellationToken::new();
        let values = vec![
            TestMsg::Value(1),
            TestMsg::Value(2),
            TestMsg::Value(3),
            TestMsg::Value(4),
            TestMsg::Value(5),
        ];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        let sub = Box::new(TakeSubscription::new(inner, 3));

        let mut stream = sub.into_stream(cancel);

        // Should only get first 3 values
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(1)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(3)));

        // Stream should end after 3
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_take_subscription_zero() {
        let cancel = CancellationToken::new();
        let values = vec![TestMsg::Value(1), TestMsg::Value(2)];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        let sub = Box::new(TakeSubscription::new(inner, 0));

        let mut stream = sub.into_stream(cancel);

        // Should get nothing
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_take_subscription_more_than_available() {
        let cancel = CancellationToken::new();
        let values = vec![TestMsg::Value(1), TestMsg::Value(2)];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        let sub = Box::new(TakeSubscription::new(inner, 100));

        let mut stream = sub.into_stream(cancel);

        // Should get all available values then end
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(1)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_take_one() {
        let cancel = CancellationToken::new();
        let values = vec![TestMsg::Value(1), TestMsg::Value(2), TestMsg::Value(3)];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        let sub = Box::new(TakeSubscription::new(inner, 1));

        let mut stream = sub.into_stream(cancel);

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(1)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }
}
