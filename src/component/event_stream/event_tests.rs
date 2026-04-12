use super::*;
use crate::component::test_utils;

fn sample_state() -> EventStreamState {
    let mut state = EventStreamState::new();
    state.push_event(EventLevel::Info, "Request received");
    state.push_event_with_fields(
        EventLevel::Debug,
        "Query executed",
        vec![("ms".into(), "12".into()), ("table".into(), "users".into())],
    );
    state.push_event(EventLevel::Warning, "Cache miss");
    state.push_event(EventLevel::Error, "Connection timeout");
    state
}

fn sample_state_with_sources() -> EventStreamState {
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

fn focused_state() -> EventStreamState {
    sample_state()
}

// =============================================================================
// Event mapping: list mode
// =============================================================================

#[test]
fn test_list_mode_up_key() {
    let state = focused_state();
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ScrollUp)
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('k'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ScrollUp)
    );
}

#[test]
fn test_list_mode_down_key() {
    let state = focused_state();
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ScrollDown)
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('j'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ScrollDown)
    );
}

#[test]
fn test_list_mode_home_g() {
    let state = focused_state();
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Home),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ScrollToTop)
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('g'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ScrollToTop)
    );
}

#[test]
fn test_list_mode_end_shift_g() {
    let state = focused_state();
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::End),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ScrollToBottom)
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('G'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ScrollToBottom)
    );
}

#[test]
fn test_list_mode_slash() {
    let state = focused_state();
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('/'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::FocusSearch)
    );
}

#[test]
fn test_list_mode_number_keys() {
    let state = focused_state();
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('1'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::QuickLevelFilter(1))
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('2'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::QuickLevelFilter(2))
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('3'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::QuickLevelFilter(3))
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('4'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::QuickLevelFilter(4))
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('5'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::QuickLevelFilter(5))
    );
}

#[test]
fn test_list_mode_f_toggle_auto() {
    let state = focused_state();
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('f'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ToggleAutoScroll)
    );
}

// =============================================================================
// Event mapping: search mode
// =============================================================================

#[test]
fn test_search_mode_char_input() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::char('a'),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::SearchInput('a'))
    );
}

#[test]
fn test_search_mode_esc() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Esc),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::ClearSearch)
    );
}

#[test]
fn test_search_mode_enter() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Enter),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::FocusList)
    );
}

#[test]
fn test_search_mode_backspace() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Backspace),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::SearchBackspace)
    );
}

#[test]
fn test_search_mode_delete() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Delete),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::SearchDelete)
    );
}

#[test]
fn test_search_mode_left_right() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Left),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::SearchLeft)
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Right),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::SearchRight)
    );
}

#[test]
fn test_search_mode_home_end() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::Home),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::SearchHome)
    );
    assert_eq!(
        EventStream::handle_event(
            &state,
            &Event::key(KeyCode::End),
            &EventContext::new().focused(true)
        ),
        Some(EventStreamMessage::SearchEnd)
    );
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = EventStreamState::new();
    let (mut terminal, theme) = test_utils::setup_render(70, 15);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_events() {
    let state = sample_state();
    let (mut terminal, theme) = test_utils::setup_render(70, 15);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(70, 15);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_sources() {
    let state = sample_state_with_sources();
    let (mut terminal, theme) = test_utils::setup_render(80, 15);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = EventStreamState::new();
    let (mut terminal, theme) = test_utils::setup_render(70, 15);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_title() {
    let mut state = EventStreamState::new().with_title("System Events");
    state.push_event(EventLevel::Info, "entry 1");
    state.push_event(EventLevel::Error, "entry 2");
    let (mut terminal, theme) = test_utils::setup_render(70, 15);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_filtered() {
    let mut state = sample_state_with_sources();
    state.level_filter = Some(EventLevel::Warning);
    let (mut terminal, theme) = test_utils::setup_render(80, 15);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = sample_state();
    let (mut terminal, theme) = test_utils::setup_render(70, 2);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_search_focused() {
    let mut state = focused_state();
    EventStream::update(&mut state, EventStreamMessage::FocusSearch);
    EventStream::update(&mut state, EventStreamMessage::SearchInput('r'));
    EventStream::update(&mut state, EventStreamMessage::SearchInput('e'));
    let (mut terminal, theme) = test_utils::setup_render(70, 15);
    terminal
        .draw(|frame| {
            EventStream::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = EventStreamState::new();
    let (mut terminal, theme) = test_utils::setup_render(70, 15);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                EventStream::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
                );
            })
            .unwrap();
    });
    assert!(registry.get_by_id("event_stream").is_some());
}
