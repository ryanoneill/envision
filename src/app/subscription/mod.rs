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
mod tests;
