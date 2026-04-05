use super::*;
use crate::component::test_utils;

fn sample_state() -> EventStreamState {
    let mut state = EventStreamState::new();

    let id = state.next_id;
    state.next_id += 1;
    state.events.push(
        StreamEvent::new(id, 1000.0, EventLevel::Info, "Request received")
            .with_source("api")
            .with_field("path", "/users"),
    );

    let id = state.next_id;
    state.next_id += 1;
    state.events.push(
        StreamEvent::new(id, 1001.0, EventLevel::Debug, "Query executed")
            .with_source("db")
            .with_field("ms", "12"),
    );

    let id = state.next_id;
    state.next_id += 1;
    state.events.push(
        StreamEvent::new(id, 1002.0, EventLevel::Warning, "Cache miss")
            .with_source("cache")
            .with_field("key", "usr"),
    );

    let id = state.next_id;
    state.next_id += 1;
    state.events.push(
        StreamEvent::new(id, 1003.0, EventLevel::Error, "Timeout")
            .with_source("api")
            .with_field("ms", "5000"),
    );

    state
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_default_empty() {
    let state = EventStreamState::new();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            EventStream::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_populated() {
    let state = sample_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            EventStream::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused() {
    let state = sample_state();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_title() {
    let mut state = EventStreamState::new().with_title("System Events");
    state.push_event(EventLevel::Info, "Started successfully");
    state.push_event(EventLevel::Warning, "High memory usage");
    state.push_event(EventLevel::Error, "Failed to connect");
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            EventStream::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let state = EventStreamState::new();
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_filtered() {
    let mut state = sample_state();
    state.level_filter = Some(EventLevel::Warning);
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            EventStream::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_search_active() {
    let mut state = sample_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    EventStream::update(&mut state, EventStreamMessage::SearchInput('a'));
    EventStream::update(&mut state, EventStreamMessage::SearchInput('p'));
    EventStream::update(&mut state, EventStreamMessage::SearchInput('i'));
    let (mut terminal, theme) = test_utils::setup_render(80, 20);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
