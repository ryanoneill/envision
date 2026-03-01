use super::*;

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
    // Use a longer timeout to handle coverage instrumentation overhead
    let msg = tokio::time::timeout(Duration::from_millis(500), stream.next()).await;
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
fn test_debounce_subscription_new() {
    let values = vec![TestMsg::Value(1)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = DebounceSubscription::new(inner, Duration::from_millis(100));
    assert_eq!(sub.duration, Duration::from_millis(100));
}

#[test]
fn test_throttle_subscription_new() {
    let values = vec![TestMsg::Value(1)];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = ThrottleSubscription::new(inner, Duration::from_millis(200));
    assert_eq!(sub.duration, Duration::from_millis(200));
}

#[tokio::test]
async fn test_debounce_empty_stream() {
    let cancel = CancellationToken::new();
    let values: Vec<TestMsg> = vec![];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(DebounceSubscription::new(inner, Duration::from_millis(50)));

    let mut stream = sub.into_stream(cancel);

    // Empty stream should end immediately with no pending message
    let msg = stream.next().await;
    assert_eq!(msg, None);
}

#[tokio::test]
async fn test_throttle_empty_stream() {
    let cancel = CancellationToken::new();
    let values: Vec<TestMsg> = vec![];
    let inner = StreamSubscription::new(tokio_stream::iter(values));
    let sub = Box::new(ThrottleSubscription::new(inner, Duration::from_millis(50)));

    let mut stream = sub.into_stream(cancel);

    // Empty stream ends immediately
    let msg = stream.next().await;
    assert_eq!(msg, None);
}
