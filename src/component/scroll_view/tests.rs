use super::*;
use crate::component::test_utils;
use crate::input::{Event, KeyCode, KeyModifiers};

fn focused_state() -> ScrollViewState {
    let mut state = ScrollViewState::new();
    ScrollView::set_focused(&mut state, true);
    state
}

fn scrollable_state() -> ScrollViewState {
    let mut state = focused_state();
    state.set_content_height(100);
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = ScrollViewState::new();
    assert_eq!(state.content_height(), 0);
    assert_eq!(state.scroll_offset(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert!(state.show_scrollbar());
    assert_eq!(state.title(), None);
}

#[test]
fn test_default() {
    let state = ScrollViewState::default();
    assert_eq!(state.content_height(), 0);
    assert_eq!(state.scroll_offset(), 0);
    assert!(state.show_scrollbar());
}

#[test]
fn test_with_content_height() {
    let state = ScrollViewState::new().with_content_height(100);
    assert_eq!(state.content_height(), 100);
}

#[test]
fn test_with_title() {
    let state = ScrollViewState::new().with_title("Preview");
    assert_eq!(state.title(), Some("Preview"));
}

#[test]
fn test_with_show_scrollbar() {
    let state = ScrollViewState::new().with_show_scrollbar(false);
    assert!(!state.show_scrollbar());
}

#[test]
fn test_with_disabled() {
    let state = ScrollViewState::new().with_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_builder_chaining() {
    let state = ScrollViewState::new()
        .with_content_height(50)
        .with_title("Log")
        .with_show_scrollbar(false)
        .with_disabled(true);
    assert_eq!(state.content_height(), 50);
    assert_eq!(state.title(), Some("Log"));
    assert!(!state.show_scrollbar());
    assert!(state.is_disabled());
}

// =============================================================================
// Accessors
// =============================================================================

#[test]
fn test_set_content_height() {
    let mut state = ScrollViewState::new();
    state.set_content_height(75);
    assert_eq!(state.content_height(), 75);
    assert_eq!(state.scroll_state().content_length(), 75);
}

#[test]
fn test_set_title() {
    let mut state = ScrollViewState::new();
    state.set_title(Some("Title".to_string()));
    assert_eq!(state.title(), Some("Title"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_set_show_scrollbar() {
    let mut state = ScrollViewState::new();
    state.set_show_scrollbar(false);
    assert!(!state.show_scrollbar());
    state.set_show_scrollbar(true);
    assert!(state.show_scrollbar());
}

#[test]
fn test_scroll_offset() {
    let mut state = ScrollViewState::new().with_content_height(100);
    state.set_scroll_offset(50);
    assert_eq!(state.scroll_offset(), 50);
}

#[test]
fn test_scroll_state() {
    let state = ScrollViewState::new().with_content_height(50);
    assert_eq!(state.scroll_state().content_length(), 50);
}

#[test]
fn test_is_focused() {
    let state = ScrollViewState::new();
    assert!(!state.is_focused());
}

#[test]
fn test_set_focused() {
    let mut state = ScrollViewState::new();
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());
}

#[test]
fn test_is_disabled() {
    let state = ScrollViewState::new();
    assert!(!state.is_disabled());
}

#[test]
fn test_set_disabled() {
    let mut state = ScrollViewState::new();
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

// =============================================================================
// content_area
// =============================================================================

#[test]
fn test_content_area_basic() {
    let state = ScrollViewState::new().with_content_height(100);
    let area = Rect::new(0, 0, 40, 20);
    let content = state.content_area(area);
    // Inner area: x+1, y+1, width-2, height-2
    assert_eq!(content.x, 1);
    assert_eq!(content.y, 1);
    assert_eq!(content.width, 38);
    assert_eq!(content.height, 18);
}

#[test]
fn test_content_area_with_offset() {
    let state = ScrollViewState::new().with_content_height(100);
    let area = Rect::new(5, 10, 30, 15);
    let content = state.content_area(area);
    assert_eq!(content.x, 6);
    assert_eq!(content.y, 11);
    assert_eq!(content.width, 28);
    assert_eq!(content.height, 13);
}

#[test]
fn test_content_area_too_small_width() {
    let state = ScrollViewState::new();
    let area = Rect::new(0, 0, 1, 10);
    let content = state.content_area(area);
    assert_eq!(content.width, 0);
    assert_eq!(content.height, 0);
}

#[test]
fn test_content_area_too_small_height() {
    let state = ScrollViewState::new();
    let area = Rect::new(0, 0, 40, 1);
    let content = state.content_area(area);
    assert_eq!(content.width, 0);
    assert_eq!(content.height, 0);
}

#[test]
fn test_content_area_zero_area() {
    let state = ScrollViewState::new();
    let area = Rect::new(0, 0, 0, 0);
    let content = state.content_area(area);
    assert_eq!(content.width, 0);
    assert_eq!(content.height, 0);
}

#[test]
fn test_content_area_minimum_viable() {
    let state = ScrollViewState::new();
    // Minimum: 3x3 gives 1x1 inner
    let area = Rect::new(0, 0, 3, 3);
    let content = state.content_area(area);
    assert_eq!(content.x, 1);
    assert_eq!(content.y, 1);
    assert_eq!(content.width, 1);
    assert_eq!(content.height, 1);
}

// =============================================================================
// viewport_height
// =============================================================================

#[test]
fn test_viewport_height() {
    let state = ScrollViewState::new();
    let area = Rect::new(0, 0, 40, 20);
    assert_eq!(state.viewport_height(area), 18);
}

#[test]
fn test_viewport_height_small_area() {
    let state = ScrollViewState::new();
    let area = Rect::new(0, 0, 40, 3);
    assert_eq!(state.viewport_height(area), 1);
}

#[test]
fn test_viewport_height_zero_area() {
    let state = ScrollViewState::new();
    let area = Rect::new(0, 0, 0, 0);
    assert_eq!(state.viewport_height(area), 0);
}

// =============================================================================
// Scroll operations
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = scrollable_state();
    let output = ScrollView::update(&mut state, ScrollViewMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
    assert_eq!(output, Some(()));
}

#[test]
fn test_scroll_up() {
    let mut state = scrollable_state();
    state.set_scroll_offset(5);
    let output = ScrollView::update(&mut state, ScrollViewMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 4);
    assert_eq!(output, Some(()));
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = scrollable_state();
    let output = ScrollView::update(&mut state, ScrollViewMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, None);
}

#[test]
fn test_scroll_down_clamped() {
    // With content_height=5, viewport_height=0, max_offset=5
    let mut state = ScrollViewState::new().with_content_height(5);
    state.set_scroll_offset(5);
    let output = ScrollView::update(&mut state, ScrollViewMessage::ScrollDown);
    assert_eq!(output, None);
}

#[test]
fn test_page_up() {
    let mut state = scrollable_state();
    // Set viewport so page size is known
    state.scroll.set_viewport_height(10);
    state.set_scroll_offset(20);
    let output = ScrollView::update(&mut state, ScrollViewMessage::PageUp);
    assert_eq!(state.scroll_offset(), 10);
    assert_eq!(output, Some(()));
}

#[test]
fn test_page_up_at_top() {
    let mut state = scrollable_state();
    state.scroll.set_viewport_height(10);
    let output = ScrollView::update(&mut state, ScrollViewMessage::PageUp);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, None);
}

#[test]
fn test_page_down() {
    let mut state = scrollable_state();
    state.scroll.set_viewport_height(10);
    let output = ScrollView::update(&mut state, ScrollViewMessage::PageDown);
    assert_eq!(state.scroll_offset(), 10);
    assert_eq!(output, Some(()));
}

#[test]
fn test_home() {
    let mut state = scrollable_state();
    state.set_scroll_offset(50);
    let output = ScrollView::update(&mut state, ScrollViewMessage::Home);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, Some(()));
}

#[test]
fn test_home_already_at_top() {
    let mut state = scrollable_state();
    let output = ScrollView::update(&mut state, ScrollViewMessage::Home);
    assert_eq!(output, None);
}

#[test]
fn test_end() {
    let mut state = scrollable_state();
    state.scroll.set_viewport_height(10);
    let output = ScrollView::update(&mut state, ScrollViewMessage::End);
    assert_eq!(state.scroll_offset(), 90); // 100 - 10
    assert_eq!(output, Some(()));
}

#[test]
fn test_end_already_at_bottom() {
    let mut state = scrollable_state();
    state.scroll.set_viewport_height(10);
    state.set_scroll_offset(90);
    let output = ScrollView::update(&mut state, ScrollViewMessage::End);
    assert_eq!(output, None);
}

#[test]
fn test_set_content_height_message() {
    let mut state = ScrollViewState::new();
    let output = ScrollView::update(&mut state, ScrollViewMessage::SetContentHeight(200));
    assert_eq!(state.content_height(), 200);
    assert_eq!(state.scroll_state().content_length(), 200);
    assert_eq!(output, None);
}

// =============================================================================
// Disabled guards
// =============================================================================

#[test]
fn test_disabled_ignores_scroll_down() {
    let mut state = scrollable_state();
    state.set_disabled(true);
    let output = ScrollView::update(&mut state, ScrollViewMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_scroll_up() {
    let mut state = scrollable_state();
    state.set_scroll_offset(5);
    state.set_disabled(true);
    let output = ScrollView::update(&mut state, ScrollViewMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 5);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_page_up() {
    let mut state = scrollable_state();
    state.set_scroll_offset(20);
    state.set_disabled(true);
    let output = ScrollView::update(&mut state, ScrollViewMessage::PageUp);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_page_down() {
    let mut state = scrollable_state();
    state.set_disabled(true);
    let output = ScrollView::update(&mut state, ScrollViewMessage::PageDown);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_home() {
    let mut state = scrollable_state();
    state.set_scroll_offset(50);
    state.set_disabled(true);
    let output = ScrollView::update(&mut state, ScrollViewMessage::Home);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_end() {
    let mut state = scrollable_state();
    state.set_disabled(true);
    let output = ScrollView::update(&mut state, ScrollViewMessage::End);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_allows_set_content_height() {
    // SetContentHeight is a data mutation, not interactive -- but our
    // component guards all updates when disabled (consistent with Collapsible).
    let mut state = ScrollViewState::new().with_disabled(true);
    let output = ScrollView::update(&mut state, ScrollViewMessage::SetContentHeight(50));
    assert_eq!(output, None);
    assert_eq!(state.content_height(), 0); // unchanged
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_handle_event_up() {
    let state = focused_state();
    assert_eq!(
        ScrollView::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(ScrollViewMessage::ScrollUp)
    );
}

#[test]
fn test_handle_event_down() {
    let state = focused_state();
    assert_eq!(
        ScrollView::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(ScrollViewMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_k_j() {
    let state = focused_state();
    assert_eq!(
        ScrollView::handle_event(&state, &Event::char('k')),
        Some(ScrollViewMessage::ScrollUp)
    );
    assert_eq!(
        ScrollView::handle_event(&state, &Event::char('j')),
        Some(ScrollViewMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_page_up_down() {
    let state = focused_state();
    assert_eq!(
        ScrollView::handle_event(&state, &Event::key(KeyCode::PageUp)),
        Some(ScrollViewMessage::PageUp)
    );
    assert_eq!(
        ScrollView::handle_event(&state, &Event::key(KeyCode::PageDown)),
        Some(ScrollViewMessage::PageDown)
    );
}

#[test]
fn test_handle_event_ctrl_u_d() {
    let state = focused_state();
    assert_eq!(
        ScrollView::handle_event(&state, &Event::ctrl('u')),
        Some(ScrollViewMessage::PageUp)
    );
    assert_eq!(
        ScrollView::handle_event(&state, &Event::ctrl('d')),
        Some(ScrollViewMessage::PageDown)
    );
}

#[test]
fn test_handle_event_home_end() {
    let state = focused_state();
    assert_eq!(
        ScrollView::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(ScrollViewMessage::Home)
    );
    assert_eq!(
        ScrollView::handle_event(&state, &Event::key(KeyCode::End)),
        Some(ScrollViewMessage::End)
    );
}

#[test]
#[allow(non_snake_case)]
fn test_handle_event_g_and_G() {
    let state = focused_state();
    assert_eq!(
        ScrollView::handle_event(&state, &Event::char('g')),
        Some(ScrollViewMessage::Home)
    );
    assert_eq!(
        ScrollView::handle_event(
            &state,
            &Event::key_with(KeyCode::Char('G'), KeyModifiers::SHIFT)
        ),
        Some(ScrollViewMessage::End)
    );
}

#[test]
fn test_handle_event_unrecognized() {
    let state = focused_state();
    assert_eq!(ScrollView::handle_event(&state, &Event::char('x')), None);
}

#[test]
fn test_handle_event_unfocused_ignores() {
    let state = ScrollViewState::new();
    assert_eq!(
        ScrollView::handle_event(&state, &Event::key(KeyCode::Up)),
        None
    );
    assert_eq!(
        ScrollView::handle_event(&state, &Event::key(KeyCode::Down)),
        None
    );
}

#[test]
fn test_handle_event_disabled_ignores() {
    let mut state = focused_state();
    state.set_disabled(true);
    assert_eq!(
        ScrollView::handle_event(&state, &Event::key(KeyCode::Up)),
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
    assert_eq!(msg, Some(ScrollViewMessage::ScrollUp));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = scrollable_state();
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(()));
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_instance_dispatch_event_no_change() {
    let mut state = scrollable_state();
    let output = state.dispatch_event(&Event::key(KeyCode::Up));
    assert_eq!(output, None);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_instance_update() {
    let mut state = scrollable_state();
    let output = state.update(ScrollViewMessage::ScrollDown);
    assert_eq!(output, Some(()));
    assert_eq!(state.scroll_offset(), 1);
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_is_focused() {
    let state = ScrollView::init();
    assert!(!ScrollView::is_focused(&state));
}

#[test]
fn test_focusable_set_focused() {
    let mut state = ScrollView::init();
    ScrollView::set_focused(&mut state, true);
    assert!(ScrollView::is_focused(&state));
}

#[test]
fn test_focusable_focus() {
    let mut state = ScrollView::init();
    ScrollView::focus(&mut state);
    assert!(ScrollView::is_focused(&state));
}

#[test]
fn test_focusable_blur() {
    let mut state = ScrollView::init();
    ScrollView::focus(&mut state);
    ScrollView::blur(&mut state);
    assert!(!ScrollView::is_focused(&state));
}

// =============================================================================
// Disableable trait
// =============================================================================

#[test]
fn test_disableable_is_disabled() {
    let state = ScrollView::init();
    assert!(!ScrollView::is_disabled(&state));
}

#[test]
fn test_disableable_set_disabled() {
    let mut state = ScrollView::init();
    ScrollView::set_disabled(&mut state, true);
    assert!(ScrollView::is_disabled(&state));
}

#[test]
fn test_disableable_disable() {
    let mut state = ScrollView::init();
    ScrollView::disable(&mut state);
    assert!(ScrollView::is_disabled(&state));
}

#[test]
fn test_disableable_enable() {
    let mut state = ScrollView::init();
    ScrollView::disable(&mut state);
    ScrollView::enable(&mut state);
    assert!(!ScrollView::is_disabled(&state));
}

// =============================================================================
// Init
// =============================================================================

#[test]
fn test_init() {
    let state = ScrollView::init();
    assert_eq!(state.content_height(), 0);
    assert_eq!(state.scroll_offset(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert!(state.show_scrollbar());
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_view_empty() {
    let state = ScrollViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_title() {
    let state = ScrollViewState::new().with_title("Preview");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            ScrollView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = ScrollViewState::new()
        .with_title("Disabled")
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_scrollbar() {
    let mut state = ScrollViewState::new()
        .with_content_height(100)
        .with_title("Scrollable");
    state.set_scroll_offset(10);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_no_scrollbar_when_disabled() {
    let state = ScrollViewState::new()
        .with_content_height(100)
        .with_show_scrollbar(false);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_no_scrollbar_when_content_fits() {
    let state = ScrollViewState::new().with_content_height(5);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            ScrollView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_zero_area() {
    let state = ScrollViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            let zero_area = Rect::new(0, 0, 0, 0);
            ScrollView::view(&state, frame, zero_area, &theme, &ViewContext::default());
        })
        .unwrap();
    // Should not panic
}

#[test]
fn test_view_minimal_area() {
    let state = ScrollViewState::new().with_title("T");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            let small_area = Rect::new(0, 0, 3, 3);
            ScrollView::view(&state, frame, small_area, &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Annotation tests
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;

    let state = ScrollViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                ScrollView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.regions();
    assert_eq!(regions.len(), 1);
    let annotation = &regions[0].annotation;
    assert!(!annotation.focused);
    assert!(!annotation.disabled);
}

#[test]
fn test_annotation_reflects_focus_and_disabled() {
    use crate::annotation::with_annotations;

    let mut state = ScrollViewState::new().with_disabled(true);
    ScrollView::focus(&mut state);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                ScrollView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.regions();
    let annotation = &regions[0].annotation;
    assert!(annotation.focused);
    assert!(annotation.disabled);
}

// =============================================================================
// Page size uses viewport
// =============================================================================

#[test]
fn test_page_down_uses_viewport_height() {
    let mut state = scrollable_state();
    state.scroll.set_viewport_height(15);
    ScrollView::update(&mut state, ScrollViewMessage::PageDown);
    assert_eq!(state.scroll_offset(), 15);
}

#[test]
fn test_page_up_uses_viewport_height() {
    let mut state = scrollable_state();
    state.scroll.set_viewport_height(15);
    state.set_scroll_offset(30);
    ScrollView::update(&mut state, ScrollViewMessage::PageUp);
    assert_eq!(state.scroll_offset(), 15);
}

#[test]
fn test_page_with_zero_viewport_uses_one() {
    // When viewport_height is 0, page_up/page_down should use 1 as minimum
    let mut state = scrollable_state();
    ScrollView::update(&mut state, ScrollViewMessage::PageDown);
    assert_eq!(state.scroll_offset(), 1);
}
