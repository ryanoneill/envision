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

mod core;
mod debounce_throttle;
mod filter_take;
mod subscription_ext;
mod terminal_events;
