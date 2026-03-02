use std::pin::Pin;
use std::time::Duration;

use tokio_stream::Stream;
use tokio_util::sync::CancellationToken;

use super::Subscription;

/// A subscription that fires immediately, then at regular intervals.
///
/// Unlike [`TickSubscription`](super::TickSubscription), this fires the first message immediately
/// without waiting for the interval.
///
/// # Example
///
/// ```rust
/// use envision::app::IntervalImmediateSubscription;
/// use std::time::Duration;
///
/// let sub = IntervalImmediateSubscription::new(Duration::from_secs(1), || "tick");
/// ```
pub struct IntervalImmediateSubscription<M, F>
where
    F: Fn() -> M + Send + 'static,
{
    pub(crate) interval: Duration,
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
/// ```rust
/// use envision::app::interval_immediate;
/// use std::time::Duration;
///
/// let sub = interval_immediate(Duration::from_secs(1)).with_message(|| "tick");
/// ```
pub fn interval_immediate(interval: Duration) -> IntervalImmediateBuilder {
    IntervalImmediateBuilder::every(interval)
}
