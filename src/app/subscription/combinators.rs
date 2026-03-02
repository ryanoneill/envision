use std::pin::Pin;
use std::time::Duration;

use tokio_stream::Stream;
use tokio_util::sync::CancellationToken;

use super::Subscription;

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

/// A subscription that filters messages from an inner subscription.
///
/// Only messages for which the predicate returns `true` are emitted.
///
/// # Example
///
/// ```rust
/// use envision::app::{SubscriptionExt, tick};
/// use std::time::Duration;
///
/// let sub = tick(Duration::from_secs(1))
///     .with_message(|| 42i32)
///     .filter(|n| *n > 0);
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
/// ```rust
/// use envision::app::{SubscriptionExt, tick};
/// use std::time::Duration;
///
/// let sub = tick(Duration::from_secs(1))
///     .with_message(|| "tick")
///     .take(5);
/// ```
pub struct TakeSubscription<M, S>
where
    S: Subscription<M>,
{
    inner: Box<S>,
    pub(crate) count: usize,
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
/// ```rust
/// use envision::app::{SubscriptionExt, tick};
/// use std::time::Duration;
///
/// // Only emit after 300ms of no new messages
/// let sub = tick(Duration::from_millis(100))
///     .with_message(|| "tick")
///     .debounce(Duration::from_millis(300));
/// ```
pub struct DebounceSubscription<M, S>
where
    S: Subscription<M>,
{
    inner: Box<S>,
    pub(crate) duration: Duration,
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
/// ```rust
/// use envision::app::{SubscriptionExt, tick};
/// use std::time::Duration;
///
/// // Emit at most once every 100ms
/// let sub = tick(Duration::from_millis(50))
///     .with_message(|| "tick")
///     .throttle(Duration::from_millis(100));
/// ```
pub struct ThrottleSubscription<M, S>
where
    S: Subscription<M>,
{
    inner: Box<S>,
    pub(crate) duration: Duration,
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
