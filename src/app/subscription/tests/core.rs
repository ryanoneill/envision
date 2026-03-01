use super::*;

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
    let channel = Box::new(ChannelSubscription::new(rx)) as BoxedSubscription<TestMsg>;

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
    let sub =
        Box::new(interval_immediate(Duration::from_millis(100)).with_message(|| TestMsg::Tick));

    let mut stream = sub.into_stream(cancel.clone());

    // Should fire immediately
    let start = std::time::Instant::now();
    let msg = stream.next().await;
    let elapsed = start.elapsed();

    assert_eq!(msg, Some(TestMsg::Tick));
    assert!(elapsed < Duration::from_millis(50));

    cancel.cancel();
}

#[tokio::test(start_paused = true)]
async fn test_interval_immediate_vs_tick() {
    // Both subscriptions produce their first message, but IntervalImmediate
    // yields before any async machinery while Tick goes through interval.tick().
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

    // Both produce their first tick
    let immediate_first = immediate_stream.next().await;
    assert!(immediate_first.is_some());

    let regular_first = regular_stream.next().await;
    assert!(regular_first.is_some());

    // After the first tick, both require waiting for the interval
    // Advance time by the interval duration to get the second tick
    tokio::time::advance(Duration::from_millis(50)).await;

    let immediate_second = immediate_stream.next().await;
    assert!(immediate_second.is_some());

    let regular_second = regular_stream.next().await;
    assert!(regular_second.is_some());

    cancel1.cancel();
    cancel2.cancel();
}

#[tokio::test]
async fn test_empty_batch_subscription() {
    let cancel = CancellationToken::new();
    let sub = Box::new(batch::<TestMsg>(vec![]));

    let mut stream = sub.into_stream(cancel);

    // Empty batch should end immediately
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_channel_subscription_cancellation() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);
    let sub = Box::new(ChannelSubscription::new(rx));

    let mut stream = sub.into_stream(cancel.clone());

    // Send a message
    tx.send(TestMsg::Value(1)).await.unwrap();
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    // Cancel the subscription
    cancel.cancel();

    // Stream should end
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_stream_subscription_cancellation() {
    let cancel = CancellationToken::new();
    let (tx, rx) = mpsc::channel(10);
    let receiver_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    let sub = Box::new(StreamSubscription::new(receiver_stream));

    let mut stream = sub.into_stream(cancel.clone());

    // Send a message
    tx.send(TestMsg::Value(1)).await.unwrap();
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Value(1)));

    // Cancel the subscription
    cancel.cancel();

    // Stream should end
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_interval_immediate_cancellation() {
    let cancel = CancellationToken::new();
    let sub = Box::new(IntervalImmediateSubscription::new(
        Duration::from_millis(10),
        || TestMsg::Tick,
    ));

    let mut stream = sub.into_stream(cancel.clone());

    // Get the immediate first message
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Tick));

    // Get the second message after interval
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Tick));

    // Cancel the subscription
    cancel.cancel();

    // Yield to let cancellation propagate
    tokio::task::yield_now().await;

    // Stream should eventually end (might get one more buffered message on some platforms)
    let mut ended = false;
    for _ in 0..3 {
        let msg = stream.next().await;
        if msg.is_none() {
            ended = true;
            break;
        }
    }
    assert!(ended, "Stream should have ended after cancellation");
}

#[tokio::test]
async fn test_mapped_subscription_empty_stream() {
    let cancel = CancellationToken::new();
    let values: Vec<i32> = vec![];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(MappedSubscription::new(inner, TestMsg::Value));

    let mut stream = sub.into_stream(cancel);

    // Mapped empty stream should end immediately
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[test]
fn test_mapped_subscription_new() {
    let values = vec![42i32];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let _sub = MappedSubscription::new(inner, TestMsg::Value);
    // Construction test - subscription created successfully
}

#[test]
fn test_batch_subscription_new() {
    let subs: Vec<BoxedSubscription<TestMsg>> = vec![];
    let sub = BatchSubscription::new(subs);
    assert!(sub.subscriptions.is_empty());
}

#[test]
fn test_interval_immediate_builder_every() {
    let builder = IntervalImmediateBuilder::every(Duration::from_secs(2));
    let sub = builder.with_message(|| TestMsg::Tick);
    assert_eq!(sub.interval, Duration::from_secs(2));
}

#[tokio::test]
async fn test_tick_cancellation() {
    let cancel = CancellationToken::new();
    let sub = Box::new(TickSubscription::new(Duration::from_millis(10), || {
        TestMsg::Tick
    }));

    let mut stream = sub.into_stream(cancel.clone());

    // Get first tick
    let msg = stream.next().await;
    assert_eq!(msg, Some(TestMsg::Tick));

    // Cancel before next tick
    cancel.cancel();

    // Yield to let cancellation propagate
    tokio::task::yield_now().await;

    // Stream should eventually end (might get one more buffered message on some platforms)
    let mut ended = false;
    for _ in 0..3 {
        let msg = stream.next().await;
        if msg.is_none() {
            ended = true;
            break;
        }
    }
    assert!(ended, "Stream should have ended after cancellation");
}
