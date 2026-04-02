use super::*;
use crate::component::test_utils;

fn focused_state() -> ScrollableTextState {
    let mut state = ScrollableTextState::new();
    ScrollableText::set_focused(&mut state, true);
    state
}

fn content_state() -> ScrollableTextState {
    let mut state = focused_state();
    state.set_content(
        "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10",
    );
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = ScrollableTextState::new();
    assert!(state.content().is_empty());
    assert_eq!(state.scroll_offset(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert_eq!(state.title(), None);
}

#[test]
fn test_default() {
    let state = ScrollableTextState::default();
    assert!(state.content().is_empty());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_with_content() {
    let state = ScrollableTextState::new().with_content("Hello!");
    assert_eq!(state.content(), "Hello!");
}

#[test]
fn test_with_title() {
    let state = ScrollableTextState::new().with_title("Preview");
    assert_eq!(state.title(), Some("Preview"));
}

#[test]
fn test_with_disabled() {
    let state = ScrollableTextState::new().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Content management
// =============================================================================

#[test]
fn test_set_content() {
    let mut state = ScrollableTextState::new();
    state.set_scroll_offset(5);
    state.set_content("New content");
    assert_eq!(state.content(), "New content");
    assert_eq!(state.scroll_offset(), 0); // Reset on set_content
}

#[test]
fn test_append() {
    let mut state = ScrollableTextState::new().with_content("Hello");
    state.append(", world!");
    assert_eq!(state.content(), "Hello, world!");
}

#[test]
fn test_set_title() {
    let mut state = ScrollableTextState::new();
    state.set_title(Some("Title".to_string()));
    assert_eq!(state.title(), Some("Title"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_line_count() {
    let state = ScrollableTextState::new().with_content("hello world");
    assert_eq!(state.line_count(5), 3);
    assert_eq!(state.line_count(11), 1);
    assert_eq!(state.line_count(20), 1);
}

#[test]
fn test_line_count_with_newlines() {
    let state = ScrollableTextState::new().with_content("a\nb\nc");
    assert_eq!(state.line_count(10), 3);
}

#[test]
fn test_line_count_empty() {
    let state = ScrollableTextState::new();
    assert_eq!(state.line_count(10), 1);
}

// =============================================================================
// Scroll operations
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = content_state();
    let output = ScrollableText::update(&mut state, ScrollableTextMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
    assert_eq!(output, Some(ScrollableTextOutput::ScrollChanged(1)));
}

#[test]
fn test_scroll_up() {
    let mut state = content_state();
    state.set_scroll_offset(5);
    let output = ScrollableText::update(&mut state, ScrollableTextMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 4);
    assert_eq!(output, Some(ScrollableTextOutput::ScrollChanged(4)));
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = content_state();
    let output = ScrollableText::update(&mut state, ScrollableTextMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, None);
}

#[test]
fn test_page_up() {
    let mut state = content_state();
    // content_state has 10 lines; set offset to 9 (max for 10-line content)
    state.set_scroll_offset(9);
    let output = ScrollableText::update(&mut state, ScrollableTextMessage::PageUp(5));
    assert_eq!(state.scroll_offset(), 4);
    assert_eq!(output, Some(ScrollableTextOutput::ScrollChanged(4)));
}

#[test]
fn test_page_up_at_top() {
    let mut state = content_state();
    let output = ScrollableText::update(&mut state, ScrollableTextMessage::PageUp(10));
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, None);
}

#[test]
fn test_page_down() {
    let mut state = content_state();
    let output = ScrollableText::update(&mut state, ScrollableTextMessage::PageDown(10));
    assert_eq!(state.scroll_offset(), 10);
    assert_eq!(output, Some(ScrollableTextOutput::ScrollChanged(10)));
}

#[test]
fn test_home() {
    let mut state = content_state();
    state.set_scroll_offset(5);
    let output = ScrollableText::update(&mut state, ScrollableTextMessage::Home);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, Some(ScrollableTextOutput::ScrollChanged(0)));
}

#[test]
fn test_home_already_at_top() {
    let mut state = content_state();
    let output = ScrollableText::update(&mut state, ScrollableTextMessage::Home);
    assert_eq!(output, None);
}

#[test]
fn test_end() {
    let mut state = content_state();
    // content_state has 10 lines; End scrolls to max_offset (content_length - viewport_height).
    // With content_length=10 and viewport_height=0 (no render yet), max_offset=10.
    let output = ScrollableText::update(&mut state, ScrollableTextMessage::End);
    assert!(output.is_some());
    assert_eq!(state.scroll_offset(), 10);
}

#[test]
fn test_set_content_message() {
    let mut state = content_state();
    state.set_scroll_offset(5);
    let output = ScrollableText::update(
        &mut state,
        ScrollableTextMessage::SetContent("New".to_string()),
    );
    assert_eq!(state.content(), "New");
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, None);
}

// =============================================================================
// Disabled and unfocused guards
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = ScrollableText::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

#[test]
fn test_unfocused_ignores_events() {
    let state = ScrollableTextState::new();
    let msg = ScrollableText::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_handle_event_up() {
    let state = focused_state();
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(ScrollableTextMessage::ScrollUp)
    );
}

#[test]
fn test_handle_event_down() {
    let state = focused_state();
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(ScrollableTextMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_k_j() {
    let state = focused_state();
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::char('k')),
        Some(ScrollableTextMessage::ScrollUp)
    );
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::char('j')),
        Some(ScrollableTextMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_page_up_down() {
    let state = focused_state();
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::key(KeyCode::PageUp)),
        Some(ScrollableTextMessage::PageUp(10))
    );
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::key(KeyCode::PageDown)),
        Some(ScrollableTextMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_ctrl_u_d() {
    let state = focused_state();
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::ctrl('u')),
        Some(ScrollableTextMessage::PageUp(10))
    );
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::ctrl('d')),
        Some(ScrollableTextMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_home_end() {
    let state = focused_state();
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(ScrollableTextMessage::Home)
    );
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::key(KeyCode::End)),
        Some(ScrollableTextMessage::End)
    );
}

#[test]
#[allow(non_snake_case)]
fn test_handle_event_g_and_G() {
    let state = focused_state();
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::char('g')),
        Some(ScrollableTextMessage::Home)
    );
    assert_eq!(
        ScrollableText::handle_event(
            &state,
            &Event::key_with(KeyCode::Char('G'), KeyModifiers::SHIFT)
        ),
        Some(ScrollableTextMessage::End)
    );
}

#[test]
fn test_handle_event_unrecognized() {
    let state = focused_state();
    assert_eq!(
        ScrollableText::handle_event(&state, &Event::char('x')),
        None
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::key(KeyCode::Up));
    assert_eq!(msg, Some(ScrollableTextMessage::ScrollUp));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = content_state();
    state.set_scroll_offset(5);
    let output = state.dispatch_event(&Event::key(KeyCode::Up));
    assert_eq!(output, Some(ScrollableTextOutput::ScrollChanged(4)));
    assert_eq!(state.scroll_offset(), 4);
}

#[test]
fn test_instance_update() {
    let mut state = content_state();
    let output = state.update(ScrollableTextMessage::ScrollDown);
    assert_eq!(output, Some(ScrollableTextOutput::ScrollChanged(1)));
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = ScrollableText::init();
    assert!(!ScrollableText::is_focused(&state));

    ScrollableText::focus(&mut state);
    assert!(ScrollableText::is_focused(&state));

    ScrollableText::blur(&mut state);
    assert!(!ScrollableText::is_focused(&state));
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_view_empty() {
    let state = ScrollableTextState::new();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollableText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_content() {
    let state =
        ScrollableTextState::new().with_content("Hello, world!\nThis is a test.\nLine three.");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollableText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_scrolled() {
    let mut state = ScrollableTextState::new()
        .with_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8");
    state.set_scroll_offset(3);
    let (mut terminal, theme) = test_utils::setup_render(40, 6);
    terminal
        .draw(|frame| {
            ScrollableText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollableText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = ScrollableTextState::new()
        .with_content("Disabled text")
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollableText::view(
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

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = ScrollableTextState::new().with_content("text");
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                ScrollableText::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::ScrollableText);
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}
