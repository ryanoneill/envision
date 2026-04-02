use super::*;
use crate::component::test_utils;

fn focused_state() -> MarkdownRendererState {
    let mut state = MarkdownRendererState::new();
    MarkdownRenderer::set_focused(&mut state, true);
    state
}

fn content_state() -> MarkdownRendererState {
    let mut state = focused_state();
    state.set_source(
        "# Heading\n\nParagraph one.\n\nParagraph two.\n\n- item 1\n- item 2\n\n\
         Line 7\nLine 8\nLine 9\nLine 10",
    );
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = MarkdownRendererState::new();
    assert!(state.source().is_empty());
    assert_eq!(state.scroll_offset(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert_eq!(state.title(), None);
    assert!(!state.show_source());
}

#[test]
fn test_default() {
    let state = MarkdownRendererState::default();
    assert!(state.source().is_empty());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_with_source() {
    let state = MarkdownRendererState::new().with_source("# Hello");
    assert_eq!(state.source(), "# Hello");
}

#[test]
fn test_with_title() {
    let state = MarkdownRendererState::new().with_title("Preview");
    assert_eq!(state.title(), Some("Preview"));
}

#[test]
fn test_with_show_source() {
    let state = MarkdownRendererState::new().with_show_source(true);
    assert!(state.show_source());
}

#[test]
fn test_with_disabled() {
    let state = MarkdownRendererState::new().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Source management
// =============================================================================

#[test]
fn test_set_source() {
    let mut state = MarkdownRendererState::new();
    state.set_scroll_offset(5);
    state.set_source("# New content");
    assert_eq!(state.source(), "# New content");
    assert_eq!(state.scroll_offset(), 0); // Reset on set_source
}

#[test]
fn test_set_title() {
    let mut state = MarkdownRendererState::new();
    state.set_title(Some("Title".to_string()));
    assert_eq!(state.title(), Some("Title"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_set_show_source() {
    let mut state = MarkdownRendererState::new();
    state.set_show_source(true);
    assert!(state.show_source());
    state.set_show_source(false);
    assert!(!state.show_source());
}

// =============================================================================
// Scroll operations
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = content_state();
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = content_state();
    state.set_scroll_offset(5);
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 4);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = content_state();
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_page_up() {
    let mut state = content_state();
    state.set_scroll_offset(9);
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::PageUp(5));
    assert_eq!(state.scroll_offset(), 4);
}

#[test]
fn test_page_up_at_top() {
    let mut state = content_state();
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::PageUp(10));
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_page_down() {
    let mut state = content_state();
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::PageDown(5));
    assert_eq!(state.scroll_offset(), 5);
}

#[test]
fn test_home() {
    let mut state = content_state();
    state.set_scroll_offset(5);
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::Home);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_end() {
    let mut state = content_state();
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::End);
    assert!(state.scroll_offset() > 0);
}

#[test]
fn test_set_source_message() {
    let mut state = content_state();
    state.set_scroll_offset(5);
    MarkdownRenderer::update(
        &mut state,
        MarkdownRendererMessage::SetSource("# New".to_string()),
    );
    assert_eq!(state.source(), "# New");
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_toggle_source() {
    let mut state = content_state();
    assert!(!state.show_source());
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::ToggleSource);
    assert!(state.show_source());
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::ToggleSource);
    assert!(!state.show_source());
}

#[test]
fn test_toggle_source_resets_scroll() {
    let mut state = content_state();
    state.set_scroll_offset(5);
    MarkdownRenderer::update(&mut state, MarkdownRendererMessage::ToggleSource);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_update_returns_none() {
    let mut state = content_state();
    let output = MarkdownRenderer::update(&mut state, MarkdownRendererMessage::ScrollDown);
    assert_eq!(output, None);
}

// =============================================================================
// Disabled and unfocused guards
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = MarkdownRenderer::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

#[test]
fn test_unfocused_ignores_events() {
    let state = MarkdownRendererState::new();
    let msg = MarkdownRenderer::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_handle_event_up() {
    let state = focused_state();
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(MarkdownRendererMessage::ScrollUp)
    );
}

#[test]
fn test_handle_event_down() {
    let state = focused_state();
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(MarkdownRendererMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_k_j() {
    let state = focused_state();
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::char('k')),
        Some(MarkdownRendererMessage::ScrollUp)
    );
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::char('j')),
        Some(MarkdownRendererMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_page_up_down() {
    let state = focused_state();
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::key(KeyCode::PageUp)),
        Some(MarkdownRendererMessage::PageUp(10))
    );
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::key(KeyCode::PageDown)),
        Some(MarkdownRendererMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_ctrl_u_d() {
    let state = focused_state();
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::ctrl('u')),
        Some(MarkdownRendererMessage::PageUp(10))
    );
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::ctrl('d')),
        Some(MarkdownRendererMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_home_end() {
    let state = focused_state();
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(MarkdownRendererMessage::Home)
    );
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::key(KeyCode::End)),
        Some(MarkdownRendererMessage::End)
    );
}

#[test]
#[allow(non_snake_case)]
fn test_handle_event_g_and_G() {
    let state = focused_state();
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::char('g')),
        Some(MarkdownRendererMessage::Home)
    );
    assert_eq!(
        MarkdownRenderer::handle_event(
            &state,
            &Event::key_with(KeyCode::Char('G'), KeyModifiers::SHIFT)
        ),
        Some(MarkdownRendererMessage::End)
    );
}

#[test]
fn test_handle_event_s_toggle() {
    let state = focused_state();
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::char('s')),
        Some(MarkdownRendererMessage::ToggleSource)
    );
}

#[test]
fn test_handle_event_unrecognized() {
    let state = focused_state();
    assert_eq!(
        MarkdownRenderer::handle_event(&state, &Event::char('x')),
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
    assert_eq!(msg, Some(MarkdownRendererMessage::ScrollUp));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = content_state();
    state.set_scroll_offset(5);
    state.dispatch_event(&Event::key(KeyCode::Up));
    assert_eq!(state.scroll_offset(), 4);
}

#[test]
fn test_instance_update() {
    let mut state = content_state();
    state.update(MarkdownRendererMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = MarkdownRenderer::init();
    assert!(!MarkdownRenderer::is_focused(&state));

    MarkdownRenderer::focus(&mut state);
    assert!(MarkdownRenderer::is_focused(&state));

    MarkdownRenderer::blur(&mut state);
    assert!(!MarkdownRenderer::is_focused(&state));
}

// =============================================================================
// Disableable trait
// =============================================================================

#[test]
fn test_disableable_trait() {
    let mut state = MarkdownRenderer::init();
    assert!(!MarkdownRenderer::is_disabled(&state));

    MarkdownRenderer::disable(&mut state);
    assert!(MarkdownRenderer::is_disabled(&state));

    MarkdownRenderer::enable(&mut state);
    assert!(!MarkdownRenderer::is_disabled(&state));
}

// =============================================================================
// Snapshot tests (view)
// =============================================================================

#[test]
fn test_view_empty() {
    let state = MarkdownRendererState::new();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            MarkdownRenderer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_heading() {
    let state = MarkdownRendererState::new().with_source("# Hello World");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            MarkdownRenderer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_paragraph() {
    let state =
        MarkdownRendererState::new().with_source("Hello, world!\nThis is a test.\nLine three.");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            MarkdownRenderer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_code_block() {
    let state =
        MarkdownRendererState::new().with_source("```rust\nlet x = 42;\nprintln!(\"{}\", x);\n```");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            MarkdownRenderer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_list() {
    let state = MarkdownRendererState::new().with_source("- Apple\n- Banana\n- Cherry");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            MarkdownRenderer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            MarkdownRenderer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let state = MarkdownRendererState::new()
        .with_source("# Disabled")
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            MarkdownRenderer::view(
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
fn test_view_with_title() {
    let state = MarkdownRendererState::new()
        .with_source("# Hello")
        .with_title("Document");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            MarkdownRenderer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_source_mode() {
    let state = MarkdownRendererState::new()
        .with_source("# Hello\n\nSome **bold** text.")
        .with_title("Preview")
        .with_show_source(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            MarkdownRenderer::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Annotation test
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = MarkdownRendererState::new().with_source("# Title");
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                MarkdownRenderer::view(
                    &state,
                    frame,
                    frame.area(),
                    &theme,
                    &ViewContext::default(),
                );
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Custom("MarkdownRenderer".to_string()));
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}
