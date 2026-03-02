use std::pin::Pin;

use tokio_stream::Stream;
use tokio_util::sync::CancellationToken;

use super::{BoxedSubscription, Subscription};

/// A batch of subscriptions combined into one.
pub struct BatchSubscription<M> {
    pub(crate) subscriptions: Vec<BoxedSubscription<M>>,
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
