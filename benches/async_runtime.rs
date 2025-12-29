//! Benchmarks for async runtime performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use envision::app::{
    App, AsyncCommandHandler, AsyncRuntime, AsyncRuntimeConfig, Command, TickSubscription,
    TimerSubscription,
};
use ratatui::widgets::Paragraph;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

// Test application for benchmarking
struct BenchApp;

#[derive(Clone, Default)]
struct BenchState {
    count: i32,
    quit: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum BenchMsg {
    Increment,
    IncrementBy(i32),
    AsyncResult(i32),
    Tick,
    Timer,
    Quit,
}

impl App for BenchApp {
    type State = BenchState;
    type Message = BenchMsg;

    fn init() -> (Self::State, Command<Self::Message>) {
        (BenchState::default(), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            BenchMsg::Increment => state.count += 1,
            BenchMsg::IncrementBy(n) => state.count += n,
            BenchMsg::AsyncResult(n) => state.count += n,
            BenchMsg::Tick => {}
            BenchMsg::Timer => {}
            BenchMsg::Quit => state.quit = true,
        }
        Command::none()
    }

    fn view(state: &Self::State, frame: &mut ratatui::Frame) {
        let text = format!("Count: {}", state.count);
        frame.render_widget(Paragraph::new(text), frame.area());
    }

    fn should_quit(state: &Self::State) -> bool {
        state.quit
    }
}

/// Benchmark AsyncRuntime creation.
fn bench_async_runtime_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_runtime_creation");

    for (width, height) in [(80, 24), (120, 40), (200, 60)] {
        group.bench_with_input(
            BenchmarkId::new("headless", format!("{}x{}", width, height)),
            &(width, height),
            |b, &(w, h)| {
                b.iter(|| {
                    let runtime: AsyncRuntime<BenchApp, _> =
                        AsyncRuntime::headless(black_box(w), black_box(h)).unwrap();
                    runtime
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_history", format!("{}x{}", width, height)),
            &(width, height),
            |b, &(w, h)| {
                let config = AsyncRuntimeConfig::new().with_history(10);
                b.iter(|| {
                    let runtime: AsyncRuntime<BenchApp, _> = AsyncRuntime::headless_with_config(
                        black_box(w),
                        black_box(h),
                        config.clone(),
                    )
                    .unwrap();
                    runtime
                });
            },
        );
    }

    group.finish();
}

/// Benchmark AsyncRuntime dispatch operations.
fn bench_async_runtime_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_runtime_dispatch");

    group.bench_function("single_message", |b| {
        let mut runtime: AsyncRuntime<BenchApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        b.iter(|| {
            runtime.dispatch(black_box(BenchMsg::Increment));
        });
    });

    group.bench_function("batch_10", |b| {
        let mut runtime: AsyncRuntime<BenchApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let messages: Vec<_> = (0..10).map(|_| BenchMsg::Increment).collect();
        b.iter(|| {
            runtime.dispatch_all(black_box(messages.clone()));
        });
    });

    group.bench_function("batch_100", |b| {
        let mut runtime: AsyncRuntime<BenchApp, _> = AsyncRuntime::headless(80, 24).unwrap();
        let messages: Vec<_> = (0..100).map(|_| BenchMsg::Increment).collect();
        b.iter(|| {
            runtime.dispatch_all(black_box(messages.clone()));
        });
    });

    group.finish();
}

/// Benchmark AsyncRuntime tick and render.
fn bench_async_runtime_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_runtime_tick");

    for (width, height) in [(80, 24), (200, 60)] {
        group.bench_with_input(
            BenchmarkId::new("tick", format!("{}x{}", width, height)),
            &(width, height),
            |b, &(w, h)| {
                let mut runtime: AsyncRuntime<BenchApp, _> = AsyncRuntime::headless(w, h).unwrap();
                b.iter(|| {
                    runtime.tick().unwrap();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("render_only", format!("{}x{}", width, height)),
            &(width, height),
            |b, &(w, h)| {
                let mut runtime: AsyncRuntime<BenchApp, _> = AsyncRuntime::headless(w, h).unwrap();
                b.iter(|| {
                    runtime.render().unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark AsyncCommandHandler operations.
fn bench_async_command_handler(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_command_handler");

    group.bench_function("creation", |b| {
        b.iter(|| {
            let handler: AsyncCommandHandler<BenchMsg> = AsyncCommandHandler::new();
            black_box(handler)
        });
    });

    group.bench_function("execute_sync", |b| {
        let mut handler: AsyncCommandHandler<BenchMsg> = AsyncCommandHandler::new();
        b.iter(|| {
            handler.execute(black_box(Command::message(BenchMsg::Increment)));
            let _ = handler.take_messages();
        });
    });

    group.bench_function("execute_batch", |b| {
        let mut handler: AsyncCommandHandler<BenchMsg> = AsyncCommandHandler::new();
        b.iter(|| {
            handler.execute(black_box(Command::batch(
                (0..10).map(|_| BenchMsg::Increment),
            )));
            let _ = handler.take_messages();
        });
    });

    group.bench_function("execute_callback", |b| {
        let mut handler: AsyncCommandHandler<BenchMsg> = AsyncCommandHandler::new();
        b.iter(|| {
            handler.execute(black_box(Command::perform(|| Some(BenchMsg::Increment))));
            let _ = handler.take_messages();
        });
    });

    group.bench_function("execute_async_collect", |b| {
        let mut handler: AsyncCommandHandler<BenchMsg> = AsyncCommandHandler::new();
        b.iter(|| {
            handler.execute(black_box(Command::perform_async(async {
                Some(BenchMsg::AsyncResult(42))
            })));
        });
        // Note: We don't spawn here, just measure collection
    });

    group.finish();
}

/// Benchmark subscription creation.
fn bench_subscription_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("subscription_creation");

    group.bench_function("tick_subscription", |b| {
        b.iter(|| {
            let sub =
                TickSubscription::new(black_box(Duration::from_millis(100)), || BenchMsg::Tick);
            black_box(sub)
        });
    });

    group.bench_function("timer_subscription", |b| {
        b.iter(|| {
            let sub = TimerSubscription::after(black_box(Duration::from_secs(1)), BenchMsg::Timer);
            black_box(sub)
        });
    });

    group.finish();
}

/// Benchmark Command creation.
fn bench_command_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_creation");

    group.bench_function("none", |b| {
        b.iter(|| {
            let cmd: Command<BenchMsg> = Command::none();
            black_box(cmd)
        });
    });

    group.bench_function("message", |b| {
        b.iter(|| {
            let cmd = Command::message(black_box(BenchMsg::Increment));
            black_box(cmd)
        });
    });

    group.bench_function("batch_10", |b| {
        let messages: Vec<_> = (0..10).map(|_| BenchMsg::Increment).collect();
        b.iter(|| {
            let cmd = Command::batch(black_box(messages.clone()));
            black_box(cmd)
        });
    });

    group.bench_function("quit", |b| {
        b.iter(|| {
            let cmd: Command<BenchMsg> = Command::quit();
            black_box(cmd)
        });
    });

    group.bench_function("perform", |b| {
        b.iter(|| {
            let cmd: Command<BenchMsg> = Command::perform(|| Some(BenchMsg::Increment));
            black_box(cmd)
        });
    });

    group.bench_function("perform_async", |b| {
        b.iter(|| {
            let cmd: Command<BenchMsg> =
                Command::perform_async(async { Some(BenchMsg::AsyncResult(42)) });
            black_box(cmd)
        });
    });

    group.finish();
}

/// Benchmark CancellationToken operations.
fn bench_cancellation_token(c: &mut Criterion) {
    let mut group = c.benchmark_group("cancellation_token");

    group.bench_function("create", |b| {
        b.iter(|| {
            let token = CancellationToken::new();
            black_box(token)
        });
    });

    group.bench_function("clone", |b| {
        let token = CancellationToken::new();
        b.iter(|| {
            let cloned = token.clone();
            black_box(cloned)
        });
    });

    group.bench_function("is_cancelled", |b| {
        let token = CancellationToken::new();
        b.iter(|| {
            let result = token.is_cancelled();
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark AsyncRuntimeConfig creation and builders.
fn bench_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("config");

    group.bench_function("default", |b| {
        b.iter(|| {
            let config = AsyncRuntimeConfig::default();
            black_box(config)
        });
    });

    group.bench_function("builder_full", |b| {
        b.iter(|| {
            let config = AsyncRuntimeConfig::new()
                .tick_rate(Duration::from_millis(50))
                .frame_rate(Duration::from_millis(16))
                .with_history(10)
                .max_messages(100)
                .channel_capacity(256);
            black_box(config)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_async_runtime_creation,
    bench_async_runtime_dispatch,
    bench_async_runtime_tick,
    bench_async_command_handler,
    bench_subscription_creation,
    bench_command_creation,
    bench_cancellation_token,
    bench_config,
);

criterion_main!(benches);
