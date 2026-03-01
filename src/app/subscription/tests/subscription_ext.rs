use super::*;

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
    let sub = Box::new(inner.filter(|msg| matches!(msg, TestMsg::Value(n) if *n % 2 == 0)));

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
    let values = vec![TestMsg::Value(1), TestMsg::Value(2), TestMsg::Value(3)];
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
            .take(2),
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
            .take(1),
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
            .filter(|n| n % 2 == 0) // Keep even: 2, 4, 6, 8, 10
            .map(|n| TestMsg::Value(n * 10)) // Multiply by 10
            .take(3), // Take first 3
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
