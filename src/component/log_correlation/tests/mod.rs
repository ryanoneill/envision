use super::*;
use crate::component::test_utils;

fn entry(ts: f64, level: CorrelationLevel, msg: &str) -> CorrelationEntry {
    CorrelationEntry::new(ts, level, msg)
}

fn two_stream_state() -> LogCorrelationState {
    use CorrelationLevel::*;
    let api = LogStream::new("API Server")
        .with_color(Color::Cyan)
        .with_entry(entry(1.0, Info, "Request received"))
        .with_entry(entry(1.0, Debug, "Parsing body"))
        .with_entry(entry(2.0, Info, "Query sent"))
        .with_entry(entry(3.0, Info, "Response sent"))
        .with_entry(entry(3.0, Warning, "Slow response"));

    let db = LogStream::new("Database")
        .with_color(Color::Green)
        .with_entry(entry(1.0, Info, "Connected"))
        .with_entry(entry(2.0, Info, "Query start"))
        .with_entry(entry(2.0, Debug, "Query plan"))
        .with_entry(entry(3.0, Info, "Query done"))
        .with_entry(entry(3.0, Warning, "Slow query"));

    LogCorrelationState::new().with_streams(vec![api, db])
}

fn focused_state() -> LogCorrelationState {
    two_stream_state()
}

mod alignment;
mod edge_cases;
mod events;
mod filtering;
mod state;
mod view;
