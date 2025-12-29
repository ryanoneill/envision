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
///
/// Provides fluent methods for composing and transforming subscriptions.
///
/// # Example
///
/// ```ignore
/// use envision::app::{SubscriptionExt, tick};
/// use std::time::Duration;
///
/// // Create a tick subscription with filtering and limiting
/// let sub = tick(Duration::from_millis(100))
///     .with_message(|| Msg::Tick)
///     .filter(|msg| msg.should_process())
///     .take(10)
///     .throttle(Duration::from_millis(200));
/// ```
pub trait SubscriptionExt<M>: Subscription<M> + Sized {
    /// Maps the messages of this subscription.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sub = tick(Duration::from_secs(1))
    ///     .with_message(|| 42)
    ///     .map(|n| Msg::Value(n));
    /// ```
    fn map<N, F>(self, f: F) -> MappedSubscription<M, N, F, Self>
    where
        F: Fn(M) -> N + Send + 'static,
    {
        MappedSubscription::new(self, f)
    }

    /// Filters messages from this subscription.
    ///
    /// Only messages for which the predicate returns `true` are emitted.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sub = some_subscription
    ///     .filter(|msg| msg.is_important());
    /// ```
    fn filter<P>(self, predicate: P) -> FilterSubscription<M, Self, P>
    where
        P: Fn(&M) -> bool + Send + 'static,
    {
        FilterSubscription::new(self, predicate)
    }

    /// Takes only the first N messages from this subscription.
    ///
    /// After N messages, the subscription ends.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sub = some_subscription.take(5);
    /// ```
    fn take(self, count: usize) -> TakeSubscription<M, Self> {
        TakeSubscription::new(self, count)
    }

    /// Debounces messages from this subscription.
    ///
    /// Only emits a message after a quiet period has passed. If a new message
    /// arrives before the quiet period expires, the timer resets. Only the most
    /// recent message is emitted.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Only emit after 300ms of no new messages
    /// let sub = some_subscription.debounce(Duration::from_millis(300));
    /// ```
    fn debounce(self, duration: Duration) -> DebounceSubscription<M, Self> {
        DebounceSubscription::new(self, duration)
    }

    /// Throttles messages from this subscription.
    ///
    /// Limits the rate of message emission. At most one message is emitted
    /// per duration. The first message passes immediately, subsequent messages
    /// are dropped until the duration has passed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Emit at most once every 100ms
    /// let sub = some_subscription.throttle(Duration::from_millis(100));
    /// ```
    fn throttle(self, duration: Duration) -> ThrottleSubscription<M, Self> {
        ThrottleSubscription::new(self, duration)
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

/// A subscription that debounces messages from an inner subscription.
///
/// Debouncing delays message emission until a quiet period has passed.
/// If a new message arrives before the quiet period expires, the timer resets.
/// Only the most recent message is emitted after the quiet period.
///
/// This is useful for scenarios like search-as-you-type where you want to
/// wait until the user stops typing before triggering a search.
///
/// # Example
///
/// ```ignore
/// // Only emit after 300ms of no new messages
/// some_subscription.debounce(Duration::from_millis(300))
/// ```
pub struct DebounceSubscription<M, S>
where
    S: Subscription<M>,
{
    inner: Box<S>,
    duration: Duration,
    _phantom: std::marker::PhantomData<M>,
}

impl<M, S> DebounceSubscription<M, S>
where
    S: Subscription<M>,
{
    /// Creates a debounced subscription.
    pub fn new(inner: S, duration: Duration) -> Self {
        Self {
            inner: Box::new(inner),
            duration,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M, S> Subscription<M> for DebounceSubscription<M, S>
where
    M: Send + 'static,
    S: Subscription<M>,
{
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        use tokio_stream::StreamExt;

        let duration = self.duration;
        let mut inner = self.inner.into_stream(cancel.clone());

        Box::pin(async_stream::stream! {
            let mut pending: Option<M> = None;
            let mut deadline: Option<tokio::time::Instant> = None;

            loop {
                tokio::select! {
                    biased;

                    // Check for cancellation first
                    _ = cancel.cancelled() => {
                        break;
                    }

                    // Check if deadline has passed
                    _ = async {
                        match deadline {
                            Some(d) => tokio::time::sleep_until(d).await,
                            None => std::future::pending::<()>().await,
                        }
                    } => {
                        if let Some(m) = pending.take() {
                            deadline = None;
                            yield m;
                        }
                    }

                    // Check for new messages
                    msg = inner.next() => {
                        match msg {
                            Some(m) => {
                                pending = Some(m);
                                deadline = Some(tokio::time::Instant::now() + duration);
                            }
                            None => {
                                // Stream ended, emit any pending message
                                if let Some(m) = pending.take() {
                                    yield m;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        })
    }
}

/// A subscription that throttles messages from an inner subscription.
///
/// Throttling limits the rate of message emission. At most one message
/// is emitted per duration. The first message is emitted immediately,
/// and subsequent messages are dropped until the duration has passed.
///
/// This is useful for limiting API calls or expensive operations.
///
/// # Example
///
/// ```ignore
/// // Emit at most once every 100ms
/// some_subscription.throttle(Duration::from_millis(100))
/// ```
pub struct ThrottleSubscription<M, S>
where
    S: Subscription<M>,
{
    inner: Box<S>,
    duration: Duration,
    _phantom: std::marker::PhantomData<M>,
}

impl<M, S> ThrottleSubscription<M, S>
where
    S: Subscription<M>,
{
    /// Creates a throttled subscription.
    pub fn new(inner: S, duration: Duration) -> Self {
        Self {
            inner: Box::new(inner),
            duration,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M, S> Subscription<M> for ThrottleSubscription<M, S>
where
    M: Send + 'static,
    S: Subscription<M>,
{
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        use tokio_stream::StreamExt;

        let duration = self.duration;
        let mut inner = self.inner.into_stream(cancel);

        Box::pin(async_stream::stream! {
            let mut last_emit: Option<tokio::time::Instant> = None;

            while let Some(msg) = inner.next().await {
                let now = tokio::time::Instant::now();
                let should_emit = match last_emit {
                    None => true,
                    Some(last) => now.duration_since(last) >= duration,
                };

                if should_emit {
                    last_emit = Some(now);
                    yield msg;
                }
            }
        })
    }
}

/// A subscription that reads terminal input events from crossterm.
///
/// This subscription uses crossterm's async event stream to read keyboard,
/// mouse, paste, focus, and resize events. Each event is passed through
/// a handler function that can optionally produce a message.
///
/// # Example
///
/// ```ignore
/// use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
///
/// let sub = TerminalEventSubscription::new(|event| {
///     match event {
///         Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
///             Some(Msg::Quit)
///         }
///         Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
///             Some(Msg::MoveUp)
///         }
///         Event::Resize(width, height) => {
///             Some(Msg::Resize(width, height))
///         }
///         _ => None,
///     }
/// });
/// ```
pub struct TerminalEventSubscription<M, F>
where
    F: Fn(crossterm::event::Event) -> Option<M> + Send + 'static,
{
    event_handler: F,
    _phantom: std::marker::PhantomData<M>,
}

impl<M, F> TerminalEventSubscription<M, F>
where
    F: Fn(crossterm::event::Event) -> Option<M> + Send + 'static,
{
    /// Creates a new terminal event subscription.
    pub fn new(event_handler: F) -> Self {
        Self {
            event_handler,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<M, F> Subscription<M> for TerminalEventSubscription<M, F>
where
    M: Send + 'static,
    F: Fn(crossterm::event::Event) -> Option<M> + Send + 'static,
{
    fn into_stream(
        self: Box<Self>,
        cancel: CancellationToken,
    ) -> Pin<Box<dyn Stream<Item = M> + Send>> {
        use crossterm::event::EventStream;
        use tokio_stream::StreamExt;

        let handler = self.event_handler;

        Box::pin(async_stream::stream! {
            let mut reader = EventStream::new();
            loop {
                tokio::select! {
                    maybe_event = reader.next() => {
                        match maybe_event {
                            Some(Ok(event)) => {
                                if let Some(msg) = (handler)(event) {
                                    yield msg;
                                }
                            }
                            Some(Err(_)) => break,
                            None => break,
                        }
                    }
                    _ = cancel.cancelled() => break,
                }
            }
        })
    }
}

/// Creates a terminal event subscription.
///
/// This is a convenience function for creating a [`TerminalEventSubscription`].
///
/// # Example
///
/// ```ignore
/// use crossterm::event::{Event, KeyCode, KeyEvent};
///
/// let sub = terminal_events(|event| {
///     if let Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) = event {
///         Some(Msg::Quit)
///     } else {
///         None
///     }
/// });
/// ```
pub fn terminal_events<M, F>(handler: F) -> TerminalEventSubscription<M, F>
where
    F: Fn(crossterm::event::Event) -> Option<M> + Send + 'static,
{
    TerminalEventSubscription::new(handler)
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
        Quit,
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

    #[tokio::test]
    async fn test_debounce_subscription() {
        let cancel = CancellationToken::new();
        let (tx, rx) = mpsc::channel(10);
        let inner = ChannelSubscription::new(rx);
        let sub = Box::new(DebounceSubscription::new(inner, Duration::from_millis(50)));

        let mut stream = sub.into_stream(cancel.clone());

        // Send multiple messages quickly (should be debounced to just the last one)
        tx.send(TestMsg::Value(1)).await.unwrap();
        tx.send(TestMsg::Value(2)).await.unwrap();
        tx.send(TestMsg::Value(3)).await.unwrap();

        // Give debounce time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should only get the last value
        let msg = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
        assert_eq!(msg.unwrap(), Some(TestMsg::Value(3)));

        // Close channel
        drop(tx);

        // Stream should end
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_debounce_emits_pending_on_stream_end() {
        let cancel = CancellationToken::new();
        let values = vec![TestMsg::Value(1), TestMsg::Value(2)];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        let sub = Box::new(DebounceSubscription::new(inner, Duration::from_secs(10)));

        let mut stream = sub.into_stream(cancel);

        // Even with long debounce, pending message should emit when stream ends
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_debounce_with_slow_messages() {
        let cancel = CancellationToken::new();
        let (tx, rx) = mpsc::channel(10);
        let inner = ChannelSubscription::new(rx);
        // Short debounce window
        let sub = Box::new(DebounceSubscription::new(inner, Duration::from_millis(20)));

        let mut stream = sub.into_stream(cancel.clone());

        // Send first message
        tx.send(TestMsg::Value(1)).await.unwrap();
        // Wait longer than debounce
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should get first message
        let msg = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
        assert_eq!(msg.unwrap(), Some(TestMsg::Value(1)));

        // Send second message
        tx.send(TestMsg::Value(2)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should get second message
        let msg = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
        assert_eq!(msg.unwrap(), Some(TestMsg::Value(2)));

        cancel.cancel();
    }

    #[tokio::test]
    async fn test_debounce_cancellation() {
        let cancel = CancellationToken::new();
        let (tx, rx) = mpsc::channel(10);
        let inner = ChannelSubscription::new(rx);
        let sub = Box::new(DebounceSubscription::new(inner, Duration::from_secs(10)));

        let mut stream = sub.into_stream(cancel.clone());

        // Send a message (won't emit due to long debounce)
        tx.send(TestMsg::Value(1)).await.unwrap();

        // Cancel immediately
        cancel.cancel();

        // Stream should end without emitting
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_throttle_subscription() {
        let cancel = CancellationToken::new();
        let values = vec![
            TestMsg::Value(1),
            TestMsg::Value(2),
            TestMsg::Value(3),
            TestMsg::Value(4),
            TestMsg::Value(5),
        ];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        // Very long throttle - should only get the first message
        let sub = Box::new(ThrottleSubscription::new(inner, Duration::from_secs(10)));

        let mut stream = sub.into_stream(cancel);

        // Should get first message immediately (throttle allows first through)
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(1)));

        // Stream ends (all others were throttled)
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_throttle_allows_spaced_messages() {
        let cancel = CancellationToken::new();
        let (tx, rx) = mpsc::channel(10);
        let inner = ChannelSubscription::new(rx);
        let sub = Box::new(ThrottleSubscription::new(inner, Duration::from_millis(20)));

        let mut stream = sub.into_stream(cancel.clone());

        // First message - should pass
        tx.send(TestMsg::Value(1)).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        let msg = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
        assert_eq!(msg.unwrap(), Some(TestMsg::Value(1)));

        // Wait longer than throttle duration
        tokio::time::sleep(Duration::from_millis(30)).await;

        // Second message after throttle period - should pass
        tx.send(TestMsg::Value(2)).await.unwrap();
        let msg = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
        assert_eq!(msg.unwrap(), Some(TestMsg::Value(2)));

        cancel.cancel();
    }

    #[tokio::test]
    async fn test_throttle_drops_rapid_messages() {
        let cancel = CancellationToken::new();
        // Use a finite stream of values that arrive "instantly"
        let values = vec![
            TestMsg::Value(1),
            TestMsg::Value(2),
            TestMsg::Value(3),
            TestMsg::Value(4),
            TestMsg::Value(5),
        ];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        // With a long throttle, only the first message should pass
        let sub = Box::new(ThrottleSubscription::new(inner, Duration::from_millis(100)));

        let mut stream = sub.into_stream(cancel);

        // Should get first message (allowed through)
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(1)));

        // Stream ends (all others 2,3,4,5 were throttled/dropped)
        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_throttle_zero_duration() {
        let cancel = CancellationToken::new();
        let values = vec![TestMsg::Value(1), TestMsg::Value(2), TestMsg::Value(3)];
        let inner = StreamSubscription::new(tokio_stream::iter(values));
        // Zero throttle - all messages should pass
        let sub = Box::new(ThrottleSubscription::new(inner, Duration::ZERO));

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

    #[test]
    fn test_terminal_event_subscription_creation() {
        use crossterm::event::{Event, KeyCode, KeyEvent};

        // Test that we can create a TerminalEventSubscription
        let _sub = TerminalEventSubscription::new(|event| {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) = event
            {
                Some(TestMsg::Quit)
            } else {
                None
            }
        });

        // Test the convenience function
        let _sub2 = terminal_events(|event| {
            if let Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) = event
            {
                Some(TestMsg::Tick)
            } else {
                None
            }
        });
    }

    #[test]
    fn test_terminal_event_handler_filters_events() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

        // Create handler that only responds to 'q'
        let handler = |event: Event| -> Option<TestMsg> {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) = event
            {
                Some(TestMsg::Quit)
            } else {
                None
            }
        };

        // Test q key
        let q_event = Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert_eq!(handler(q_event), Some(TestMsg::Quit));

        // Test other key (should be None)
        let a_event = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert_eq!(handler(a_event), None);

        // Test resize event (should be None)
        let resize_event = Event::Resize(80, 24);
        assert_eq!(handler(resize_event), None);
    }

    #[test]
    fn test_terminal_event_handler_with_modifiers() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

        // Create handler that responds to Ctrl+C
        let handler = |event: Event| -> Option<TestMsg> {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
                ..
            }) = event
            {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    Some(TestMsg::Quit)
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Test Ctrl+C
        let ctrl_c = Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert_eq!(handler(ctrl_c), Some(TestMsg::Quit));

        // Test plain 'c' (should be None)
        let plain_c = Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert_eq!(handler(plain_c), None);
    }

    #[test]
    fn test_terminal_event_handler_resize() {
        use crossterm::event::Event;

        #[derive(Debug, Clone, PartialEq)]
        enum ResizeMsg {
            Resize(u16, u16),
        }

        let handler = |event: Event| -> Option<ResizeMsg> {
            if let Event::Resize(width, height) = event {
                Some(ResizeMsg::Resize(width, height))
            } else {
                None
            }
        };

        let resize_event = Event::Resize(120, 40);
        assert_eq!(handler(resize_event), Some(ResizeMsg::Resize(120, 40)));

        // Key event should be None
        let key_event = Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Enter,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert_eq!(handler(key_event), None);
    }

    // Note: We can't test TerminalEventSubscription::into_stream in unit tests
    // because crossterm's EventStream requires a real terminal to be attached.
    // The handler logic is tested through the test_terminal_event_* tests above
    // which verify the event handling works correctly.

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsgWithQuit {
        Quit,
        Key(char),
    }

    #[test]
    fn test_terminal_events_convenience_function() {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

        let sub = terminal_events(|event: Event| -> Option<TestMsgWithQuit> {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => Some(TestMsgWithQuit::Quit),
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => Some(TestMsgWithQuit::Key(c)),
                _ => None,
            }
        });

        // Verify the handler works correctly by testing it directly
        let q_event = Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });
        assert_eq!((sub.event_handler)(q_event), Some(TestMsgWithQuit::Quit));
    }

    // Tests for SubscriptionExt fluent methods

    #[tokio::test]
    async fn test_subscription_ext_filter() {
        let cancel = CancellationToken::new();
        let values = vec![
            TestMsg::Value(1),
            TestMsg::Value(2),
            TestMsg::Value(3),
            TestMsg::Value(4),
        ];
        let inner = StreamSubscription::new(tokio_stream::iter(values));

        // Use fluent filter method
        let sub = Box::new(inner.filter(|msg| {
            matches!(msg, TestMsg::Value(n) if *n % 2 == 0)
        }));

        let mut stream = sub.into_stream(cancel);

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(4)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_subscription_ext_take() {
        let cancel = CancellationToken::new();
        let values = vec![
            TestMsg::Value(1),
            TestMsg::Value(2),
            TestMsg::Value(3),
            TestMsg::Value(4),
        ];
        let inner = StreamSubscription::new(tokio_stream::iter(values));

        // Use fluent take method
        let sub = Box::new(inner.take(2));

        let mut stream = sub.into_stream(cancel);

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(1)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_subscription_ext_debounce() {
        let cancel = CancellationToken::new();
        let values = vec![TestMsg::Value(1), TestMsg::Value(2)];
        let inner = StreamSubscription::new(tokio_stream::iter(values));

        // Use fluent debounce method
        let sub = Box::new(inner.debounce(Duration::from_secs(10)));

        let mut stream = sub.into_stream(cancel);

        // Should emit pending on stream end
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_subscription_ext_throttle() {
        let cancel = CancellationToken::new();
        let values = vec![
            TestMsg::Value(1),
            TestMsg::Value(2),
            TestMsg::Value(3),
        ];
        let inner = StreamSubscription::new(tokio_stream::iter(values));

        // Use fluent throttle method with long duration
        let sub = Box::new(inner.throttle(Duration::from_secs(10)));

        let mut stream = sub.into_stream(cancel);

        // Only first should pass
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(1)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_subscription_ext_chaining() {
        let cancel = CancellationToken::new();
        let values = vec![
            TestMsg::Value(1),
            TestMsg::Value(2),
            TestMsg::Value(3),
            TestMsg::Value(4),
            TestMsg::Value(5),
            TestMsg::Value(6),
        ];
        let inner = StreamSubscription::new(tokio_stream::iter(values));

        // Chain multiple extension methods
        let sub = Box::new(
            inner
                .filter(|msg| matches!(msg, TestMsg::Value(n) if *n % 2 == 0))
                .take(2)
        );

        let mut stream = sub.into_stream(cancel);

        // Should filter to even (2, 4, 6) then take 2 (2, 4)
        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(2)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(4)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_subscription_ext_map_and_filter() {
        let cancel = CancellationToken::new();
        let inner = TickSubscription::new(Duration::from_millis(10), || 42i32);

        // Map then filter
        let sub = Box::new(
            inner
                .map(TestMsg::Value)
                .filter(|msg| matches!(msg, TestMsg::Value(n) if *n > 0))
                .take(1)
        );

        let mut stream = sub.into_stream(cancel.clone());

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(42)));

        cancel.cancel();
    }

    #[tokio::test]
    async fn test_subscription_ext_filter_map_take() {
        let cancel = CancellationToken::new();
        let values = vec![1i32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let inner = StreamSubscription::new(tokio_stream::iter(values));

        // Filter, map, then take
        let sub = Box::new(
            inner
                .filter(|n| n % 2 == 0)         // Keep even: 2, 4, 6, 8, 10
                .map(|n| TestMsg::Value(n * 10)) // Multiply by 10
                .take(3)                         // Take first 3
        );

        let mut stream = sub.into_stream(cancel);

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(20)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(40)));

        let msg = stream.next().await;
        assert_eq!(msg, Some(TestMsg::Value(60)));

        let msg = stream.next().await;
        assert_eq!(msg, None);
    }
}
