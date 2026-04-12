use super::*;

#[test]
fn test_worker_progress_new() {
    let progress = WorkerProgress::new(0.5, Some("Working...".to_string()));
    assert_eq!(progress.percentage(), 0.5);
    assert_eq!(progress.status(), Some("Working..."));
}

#[test]
fn test_worker_progress_clamps_high() {
    let progress = WorkerProgress::new(1.5, None);
    assert_eq!(progress.percentage(), 1.0);
}

#[test]
fn test_worker_progress_clamps_low() {
    let progress = WorkerProgress::new(-0.5, None);
    assert_eq!(progress.percentage(), 0.0);
}

#[test]
fn test_worker_progress_no_status() {
    let progress = WorkerProgress::new(0.3, None);
    assert!(progress.status().is_none());
}

#[test]
fn test_worker_progress_clone() {
    let progress = WorkerProgress::new(0.5, Some("test".to_string()));
    let cloned = progress.clone();
    assert_eq!(progress, cloned);
}

#[test]
fn test_worker_handle_cancel() {
    let cancel = CancellationToken::new();
    let handle = WorkerHandle {
        cancel: cancel.clone(),
        id: "test".to_string(),
    };
    assert!(!handle.is_cancelled());
    handle.cancel();
    assert!(handle.is_cancelled());
}

#[test]
fn test_worker_handle_id() {
    let cancel = CancellationToken::new();
    let handle = WorkerHandle {
        cancel,
        id: "my-worker".to_string(),
    };
    assert_eq!(handle.id(), "my-worker");
}

#[test]
fn test_worker_handle_drop_cancels() {
    let cancel = CancellationToken::new();
    let cancel_check = cancel.clone();
    {
        let _handle = WorkerHandle {
            cancel,
            id: "test".to_string(),
        };
        assert!(!cancel_check.is_cancelled());
    }
    // After handle is dropped, cancellation should be triggered
    assert!(cancel_check.is_cancelled());
}

#[test]
fn test_worker_builder_default_capacity() {
    let builder = WorkerBuilder::new("test");
    assert_eq!(builder.channel_capacity, DEFAULT_CHANNEL_CAPACITY);
}

#[test]
fn test_worker_builder_custom_capacity() {
    let builder = WorkerBuilder::new("test").with_channel_capacity(64);
    assert_eq!(builder.channel_capacity, 64);
}

#[tokio::test]
async fn test_progress_sender_send() {
    let (tx, mut rx) = mpsc::channel(8);
    let sender: ProgressSender<WorkerProgress> = ProgressSender { tx };

    sender
        .send(WorkerProgress::new(0.5, Some("halfway".to_string())))
        .await
        .unwrap();

    let received = rx.recv().await.unwrap();
    assert_eq!(received.percentage(), 0.5);
    assert_eq!(received.status(), Some("halfway"));
}

#[tokio::test]
async fn test_progress_sender_custom_type() {
    #[derive(Debug, PartialEq)]
    enum MyProgress {
        Started,
        ChapterCount(usize),
        Finished,
    }

    let (tx, mut rx) = mpsc::channel(8);
    let sender: ProgressSender<MyProgress> = ProgressSender { tx };

    sender.send(MyProgress::Started).await.unwrap();
    sender.send(MyProgress::ChapterCount(12)).await.unwrap();
    sender.send(MyProgress::Finished).await.unwrap();

    assert_eq!(rx.recv().await.unwrap(), MyProgress::Started);
    assert_eq!(rx.recv().await.unwrap(), MyProgress::ChapterCount(12));
    assert_eq!(rx.recv().await.unwrap(), MyProgress::Finished);
}

#[tokio::test]
async fn test_progress_sender_try_send_succeeds() {
    let (tx, mut rx) = mpsc::channel(8);
    let sender: ProgressSender<u32> = ProgressSender { tx };

    sender.try_send(42).unwrap();
    sender.try_send(43).unwrap();

    assert_eq!(rx.recv().await.unwrap(), 42);
    assert_eq!(rx.recv().await.unwrap(), 43);
}

#[tokio::test]
async fn test_progress_sender_try_send_full_channel() {
    // Channel capacity 1 — second try_send should fail
    let (tx, _rx) = mpsc::channel(1);
    let sender: ProgressSender<u32> = ProgressSender { tx };

    sender.try_send(1).unwrap(); // fills the channel
    let result = sender.try_send(2);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_progress_sender_try_send_closed_channel() {
    let (tx, rx) = mpsc::channel(8);
    let sender: ProgressSender<u32> = ProgressSender { tx };
    drop(rx);

    let result = sender.try_send(1);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_spawn_simple_runs_task() {
    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Done(String),
        Failed(String),
    }

    let (cmd, handle) = WorkerBuilder::new("test").spawn_simple(
        |_cancel| async move { Ok::<_, String>("result".to_string()) },
        |result: Result<String, String>| match result {
            Ok(data) => Msg::Done(data),
            Err(e) => Msg::Failed(e),
        },
    );

    assert!(!cmd.is_none());
    assert!(!handle.is_cancelled());
    assert_eq!(handle.id(), "test");
}

#[tokio::test]
async fn test_spawn_with_progress() {
    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    enum Msg {
        Progress(WorkerProgress),
        Done(String),
        Failed(String),
    }

    let (cmd, _subscription, handle) = WorkerBuilder::new("download").spawn(
        |sender: ProgressSender<WorkerProgress>, _cancel| async move {
            sender.send(WorkerProgress::new(0.5, None)).await.ok();
            Ok::<_, String>("done".to_string())
        },
        Msg::Progress,
        |result: Result<String, String>| match result {
            Ok(data) => Msg::Done(data),
            Err(e) => Msg::Failed(e),
        },
    );

    assert!(!cmd.is_none());
    assert!(!handle.is_cancelled());
    assert_eq!(handle.id(), "download");
}

#[tokio::test]
async fn test_spawn_with_custom_progress_type() {
    #[derive(Clone, Debug)]
    enum Progress {
        ChapterFound(String),
        Encoding { percent: f32 },
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    enum Msg {
        Update(Progress),
        Done(String),
    }

    let (cmd, _sub, handle) = WorkerBuilder::new("transcode")
        .with_channel_capacity(128)
        .spawn(
            |sender: ProgressSender<Progress>, _cancel| async move {
                sender
                    .send(Progress::ChapterFound("Chapter 1".into()))
                    .await
                    .ok();
                sender.try_send(Progress::Encoding { percent: 0.5 }).ok();
                Ok::<_, String>("output.m4b".to_string())
            },
            Msg::Update,
            |result: Result<String, String>| Msg::Done(result.unwrap_or_default()),
        );

    assert!(!cmd.is_none());
    assert_eq!(handle.id(), "transcode");
}

#[tokio::test]
async fn test_spawn_simple_error_handling() {
    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Done(String),
        Failed(String),
    }

    let (cmd, _handle) = WorkerBuilder::new("test").spawn_simple(
        |_cancel| async move { Err::<String, String>("something failed".to_string()) },
        |result: Result<String, String>| match result {
            Ok(data) => Msg::Done(data),
            Err(e) => Msg::Failed(e),
        },
    );

    assert!(!cmd.is_none());
}

#[tokio::test]
async fn test_progress_sender_clone() {
    let (tx, mut rx) = mpsc::channel(8);
    let sender: ProgressSender<WorkerProgress> = ProgressSender { tx };
    let sender2 = sender.clone();

    sender.send(WorkerProgress::new(0.25, None)).await.unwrap();
    sender2.send(WorkerProgress::new(0.50, None)).await.unwrap();

    let p1 = rx.recv().await.unwrap();
    let p2 = rx.recv().await.unwrap();
    assert_eq!(p1.percentage(), 0.25);
    assert_eq!(p2.percentage(), 0.50);
}

#[tokio::test]
async fn test_progress_sender_fails_when_receiver_dropped() {
    let (tx, rx) = mpsc::channel(8);
    let sender: ProgressSender<WorkerProgress> = ProgressSender { tx };
    drop(rx);

    let result = sender.send(WorkerProgress::new(0.5, None)).await;
    assert!(result.is_err());
}

#[test]
fn test_worker_builder_id() {
    let builder = WorkerBuilder::new("my-task");
    assert_eq!(builder.id, "my-task");
}
