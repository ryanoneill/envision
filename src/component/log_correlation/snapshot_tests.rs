use super::*;
use crate::component::test_utils;

fn two_stream_state() -> LogCorrelationState {
    let api = LogStream::new("API Server")
        .with_color(Color::Cyan)
        .with_entry(CorrelationEntry::new(
            1.0,
            CorrelationLevel::Info,
            "Request received",
        ))
        .with_entry(CorrelationEntry::new(
            1.0,
            CorrelationLevel::Debug,
            "Parsing body",
        ))
        .with_entry(CorrelationEntry::new(
            2.0,
            CorrelationLevel::Info,
            "Query sent",
        ))
        .with_entry(CorrelationEntry::new(
            3.0,
            CorrelationLevel::Info,
            "Response sent",
        ))
        .with_entry(CorrelationEntry::new(
            3.0,
            CorrelationLevel::Warning,
            "Slow response",
        ));

    let db = LogStream::new("Database")
        .with_color(Color::Green)
        .with_entry(CorrelationEntry::new(
            1.0,
            CorrelationLevel::Info,
            "Connected",
        ))
        .with_entry(CorrelationEntry::new(
            2.0,
            CorrelationLevel::Info,
            "Query start",
        ))
        .with_entry(CorrelationEntry::new(
            2.0,
            CorrelationLevel::Debug,
            "Query plan",
        ))
        .with_entry(CorrelationEntry::new(
            3.0,
            CorrelationLevel::Info,
            "Query done",
        ))
        .with_entry(CorrelationEntry::new(
            3.0,
            CorrelationLevel::Warning,
            "Slow query",
        ));

    LogCorrelationState::new().with_streams(vec![api, db])
}

#[test]
fn test_snapshot_empty() {
    let state = LogCorrelationState::new();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_two_streams() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_title() {
    let state = LogCorrelationState::new()
        .with_title("Log Correlation")
        .with_streams(vec![
            LogStream::new("API")
                .with_color(Color::Cyan)
                .with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "ok")),
            LogStream::new("DB")
                .with_color(Color::Green)
                .with_entry(CorrelationEntry::new(1.0, CorrelationLevel::Info, "ok")),
        ]);
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_filter() {
    let mut state = two_stream_state();
    state.streams[0].filter = "Query".to_string();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            LogCorrelation::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn view_chrome_owned_no_outer_border() {
    let state = two_stream_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            LogCorrelation::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).chrome_owned(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
