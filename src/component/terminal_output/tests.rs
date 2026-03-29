use super::*;
use crate::component::test_utils;
use crate::input::KeyModifiers;

fn focused_state() -> TerminalOutputState {
    let mut state = TerminalOutputState::new();
    TerminalOutput::set_focused(&mut state, true);
    state
}

fn content_state() -> TerminalOutputState {
    let mut state = focused_state();
    for i in 1..=20 {
        state.push_line(format!("Line {i}"));
    }
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = TerminalOutputState::new();
    assert!(state.lines().is_empty());
    assert_eq!(state.line_count(), 0);
    assert_eq!(state.scroll_offset(), 0);
    assert!(state.auto_scroll());
    assert!(!state.show_line_numbers());
    assert!(!state.running());
    assert_eq!(state.exit_code(), None);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert_eq!(state.title(), None);
    assert_eq!(state.max_lines(), 10_000);
}

#[test]
fn test_default() {
    let state = TerminalOutputState::default();
    assert!(state.lines().is_empty());
    assert!(state.auto_scroll());
}

#[test]
fn test_with_title() {
    let state = TerminalOutputState::new().with_title("Output");
    assert_eq!(state.title(), Some("Output"));
}

#[test]
fn test_with_max_lines() {
    let state = TerminalOutputState::new().with_max_lines(500);
    assert_eq!(state.max_lines(), 500);
}

#[test]
fn test_with_auto_scroll() {
    let state = TerminalOutputState::new().with_auto_scroll(false);
    assert!(!state.auto_scroll());
}

#[test]
fn test_with_line_numbers() {
    let state = TerminalOutputState::new().with_line_numbers(true);
    assert!(state.show_line_numbers());
}

#[test]
fn test_with_running() {
    let state = TerminalOutputState::new().with_running(true);
    assert!(state.running());
}

#[test]
fn test_with_disabled() {
    let state = TerminalOutputState::new().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Line management
// =============================================================================

#[test]
fn test_push_line() {
    let mut state = TerminalOutputState::new();
    state.push_line("hello");
    assert_eq!(state.line_count(), 1);
    assert_eq!(state.lines()[0], "hello");
}

#[test]
fn test_push_line_with_ansi() {
    let mut state = TerminalOutputState::new();
    state.push_line("\x1b[31mred text\x1b[0m");
    assert_eq!(state.line_count(), 1);
    assert_eq!(state.lines()[0], "\x1b[31mred text\x1b[0m");
}

#[test]
fn test_push_lines() {
    let mut state = TerminalOutputState::new();
    state.push_lines(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert_eq!(state.line_count(), 3);
    assert_eq!(state.lines()[2], "c");
}

#[test]
fn test_clear() {
    let mut state = TerminalOutputState::new();
    state.push_line("hello");
    state.push_line("world");
    state.clear();
    assert!(state.lines().is_empty());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_max_lines_enforcement() {
    let mut state = TerminalOutputState::new().with_max_lines(3);
    for i in 1..=5 {
        state.push_line(format!("line {i}"));
    }
    assert_eq!(state.line_count(), 3);
    assert_eq!(state.lines()[0], "line 3");
    assert_eq!(state.lines()[2], "line 5");
}

#[test]
fn test_max_lines_set_after_push() {
    let mut state = TerminalOutputState::new();
    for i in 1..=10 {
        state.push_line(format!("line {i}"));
    }
    state.set_max_lines(5);
    assert_eq!(state.line_count(), 5);
    assert_eq!(state.lines()[0], "line 6");
}

#[test]
fn test_push_lines_enforces_max() {
    let mut state = TerminalOutputState::new().with_max_lines(3);
    state.push_lines(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
        "e".to_string(),
    ]);
    assert_eq!(state.line_count(), 3);
    assert_eq!(state.lines()[0], "c");
}

// =============================================================================
// Auto-scroll behavior
// =============================================================================

#[test]
fn test_auto_scroll_follows_new_lines() {
    let mut state = TerminalOutputState::new().with_auto_scroll(true);
    for i in 1..=50 {
        state.push_line(format!("line {i}"));
    }
    // Auto-scroll should put us at the end
    assert_eq!(state.scroll.offset(), state.scroll.max_offset());
}

#[test]
fn test_auto_scroll_disabled_does_not_follow() {
    let mut state = TerminalOutputState::new().with_auto_scroll(false);
    state.push_line("line 1");
    assert_eq!(state.scroll_offset(), 0);
    state.push_line("line 2");
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_up_disables_auto_scroll() {
    let mut state = content_state();
    assert!(state.auto_scroll());
    TerminalOutput::update(&mut state, TerminalOutputMessage::ScrollUp);
    assert!(!state.auto_scroll());
}

#[test]
fn test_page_up_disables_auto_scroll() {
    let mut state = content_state();
    assert!(state.auto_scroll());
    TerminalOutput::update(&mut state, TerminalOutputMessage::PageUp(5));
    assert!(!state.auto_scroll());
}

#[test]
fn test_home_disables_auto_scroll() {
    let mut state = content_state();
    assert!(state.auto_scroll());
    TerminalOutput::update(&mut state, TerminalOutputMessage::Home);
    assert!(!state.auto_scroll());
}

// =============================================================================
// Scroll operations (via update)
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = content_state();
    state.set_auto_scroll(false);
    state.set_scroll_offset(0);
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
    assert_eq!(output, Some(TerminalOutputOutput::ScrollChanged(1)));
}

#[test]
fn test_scroll_up() {
    let mut state = content_state();
    state.set_auto_scroll(false);
    state.set_scroll_offset(5);
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 4);
    assert_eq!(output, Some(TerminalOutputOutput::ScrollChanged(4)));
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = content_state();
    state.set_auto_scroll(false);
    state.set_scroll_offset(0);
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, None);
}

#[test]
fn test_page_up() {
    let mut state = content_state();
    state.set_auto_scroll(false);
    state.set_scroll_offset(10);
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::PageUp(5));
    assert_eq!(state.scroll_offset(), 5);
    assert_eq!(output, Some(TerminalOutputOutput::ScrollChanged(5)));
}

#[test]
fn test_page_down() {
    let mut state = content_state();
    state.set_auto_scroll(false);
    state.set_scroll_offset(0);
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::PageDown(5));
    assert_eq!(state.scroll_offset(), 5);
    assert_eq!(output, Some(TerminalOutputOutput::ScrollChanged(5)));
}

#[test]
fn test_home() {
    let mut state = content_state();
    state.set_auto_scroll(false);
    state.set_scroll_offset(10);
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::Home);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(output, Some(TerminalOutputOutput::ScrollChanged(0)));
}

#[test]
fn test_end() {
    let mut state = content_state();
    state.set_auto_scroll(false);
    state.set_scroll_offset(0);
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::End);
    assert!(output.is_some());
    assert_eq!(state.scroll_offset(), state.scroll.max_offset());
}

// =============================================================================
// Toggle messages
// =============================================================================

#[test]
fn test_toggle_auto_scroll() {
    let mut state = TerminalOutputState::new();
    assert!(state.auto_scroll());
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ToggleAutoScroll);
    assert!(!state.auto_scroll());
    assert_eq!(output, Some(TerminalOutputOutput::AutoScrollToggled(false)));

    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ToggleAutoScroll);
    assert!(state.auto_scroll());
    assert_eq!(output, Some(TerminalOutputOutput::AutoScrollToggled(true)));
}

#[test]
fn test_toggle_line_numbers() {
    let mut state = TerminalOutputState::new();
    assert!(!state.show_line_numbers());
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ToggleLineNumbers);
    assert!(state.show_line_numbers());
    assert_eq!(output, Some(TerminalOutputOutput::LineNumbersToggled(true)));
}

// =============================================================================
// Running / exit code
// =============================================================================

#[test]
fn test_set_running() {
    let mut state = TerminalOutputState::new();
    TerminalOutput::update(&mut state, TerminalOutputMessage::SetRunning(true));
    assert!(state.running());
}

#[test]
fn test_set_exit_code() {
    let mut state = TerminalOutputState::new().with_running(true);
    TerminalOutput::update(&mut state, TerminalOutputMessage::SetExitCode(Some(0)));
    assert_eq!(state.exit_code(), Some(0));
    assert!(!state.running()); // Setting exit code stops running
}

#[test]
fn test_set_exit_code_nonzero() {
    let mut state = TerminalOutputState::new().with_running(true);
    state.set_exit_code(Some(1));
    assert_eq!(state.exit_code(), Some(1));
    assert!(!state.running());
}

#[test]
fn test_set_exit_code_none() {
    let mut state = TerminalOutputState::new();
    state.set_exit_code(Some(0));
    state.set_exit_code(None);
    assert_eq!(state.exit_code(), None);
}

// =============================================================================
// Push line / push lines via update
// =============================================================================

#[test]
fn test_push_line_via_update() {
    let mut state = TerminalOutputState::new();
    let output = TerminalOutput::update(
        &mut state,
        TerminalOutputMessage::PushLine("hello".to_string()),
    );
    assert_eq!(state.line_count(), 1);
    assert_eq!(output, Some(TerminalOutputOutput::LineAdded(1)));
}

#[test]
fn test_push_lines_via_update() {
    let mut state = TerminalOutputState::new();
    let output = TerminalOutput::update(
        &mut state,
        TerminalOutputMessage::PushLines(vec!["a".to_string(), "b".to_string()]),
    );
    assert_eq!(state.line_count(), 2);
    assert_eq!(output, Some(TerminalOutputOutput::LineAdded(2)));
}

#[test]
fn test_push_empty_lines_via_update() {
    let mut state = TerminalOutputState::new();
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::PushLines(vec![]));
    assert_eq!(output, None);
}

#[test]
fn test_clear_via_update() {
    let mut state = TerminalOutputState::new();
    state.push_line("hello");
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::Clear);
    assert!(state.lines().is_empty());
    assert_eq!(output, Some(TerminalOutputOutput::Cleared));
}

#[test]
fn test_clear_empty_via_update() {
    let mut state = TerminalOutputState::new();
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::Clear);
    assert_eq!(output, None);
}

// =============================================================================
// Disabled and unfocused guards
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = TerminalOutput::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

#[test]
fn test_unfocused_ignores_events() {
    let state = TerminalOutputState::new();
    let msg = TerminalOutput::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_handle_event_up() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(TerminalOutputMessage::ScrollUp)
    );
}

#[test]
fn test_handle_event_down() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(TerminalOutputMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_k_j() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::char('k')),
        Some(TerminalOutputMessage::ScrollUp)
    );
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::char('j')),
        Some(TerminalOutputMessage::ScrollDown)
    );
}

#[test]
fn test_handle_event_page_up_down() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::key(KeyCode::PageUp)),
        Some(TerminalOutputMessage::PageUp(10))
    );
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::key(KeyCode::PageDown)),
        Some(TerminalOutputMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_ctrl_u_d() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::ctrl('u')),
        Some(TerminalOutputMessage::PageUp(10))
    );
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::ctrl('d')),
        Some(TerminalOutputMessage::PageDown(10))
    );
}

#[test]
fn test_handle_event_home_end() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(TerminalOutputMessage::Home)
    );
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::key(KeyCode::End)),
        Some(TerminalOutputMessage::End)
    );
}

#[test]
#[allow(non_snake_case)]
fn test_handle_event_g_and_G() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::char('g')),
        Some(TerminalOutputMessage::Home)
    );
    assert_eq!(
        TerminalOutput::handle_event(
            &state,
            &Event::key_with(KeyCode::Char('G'), KeyModifiers::SHIFT)
        ),
        Some(TerminalOutputMessage::End)
    );
}

#[test]
fn test_handle_event_toggle_auto_scroll() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::char('a')),
        Some(TerminalOutputMessage::ToggleAutoScroll)
    );
}

#[test]
fn test_handle_event_toggle_line_numbers() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::char('n')),
        Some(TerminalOutputMessage::ToggleLineNumbers)
    );
}

#[test]
fn test_handle_event_unrecognized() {
    let state = focused_state();
    assert_eq!(
        TerminalOutput::handle_event(&state, &Event::char('x')),
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
    assert_eq!(msg, Some(TerminalOutputMessage::ScrollUp));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = content_state();
    state.set_auto_scroll(false);
    state.set_scroll_offset(5);
    let output = state.dispatch_event(&Event::key(KeyCode::Up));
    assert_eq!(output, Some(TerminalOutputOutput::ScrollChanged(4)));
    assert_eq!(state.scroll_offset(), 4);
}

#[test]
fn test_instance_update() {
    let mut state = TerminalOutputState::new();
    let output = state.update(TerminalOutputMessage::PushLine("hello".to_string()));
    assert_eq!(output, Some(TerminalOutputOutput::LineAdded(1)));
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = TerminalOutput::init();
    assert!(!TerminalOutput::is_focused(&state));

    TerminalOutput::focus(&mut state);
    assert!(TerminalOutput::is_focused(&state));

    TerminalOutput::blur(&mut state);
    assert!(!TerminalOutput::is_focused(&state));
}

// =============================================================================
// Disableable trait
// =============================================================================

#[test]
fn test_disableable_trait() {
    let mut state = TerminalOutput::init();
    assert!(!TerminalOutput::is_disabled(&state));

    TerminalOutput::disable(&mut state);
    assert!(TerminalOutput::is_disabled(&state));

    TerminalOutput::enable(&mut state);
    assert!(!TerminalOutput::is_disabled(&state));
}

// =============================================================================
// Setters
// =============================================================================

#[test]
fn test_set_title() {
    let mut state = TerminalOutputState::new();
    state.set_title(Some("Title".to_string()));
    assert_eq!(state.title(), Some("Title"));
    state.set_title(None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_set_auto_scroll_directly() {
    let mut state = TerminalOutputState::new();
    state.set_auto_scroll(false);
    assert!(!state.auto_scroll());
}

#[test]
fn test_set_show_line_numbers_directly() {
    let mut state = TerminalOutputState::new();
    state.set_show_line_numbers(true);
    assert!(state.show_line_numbers());
}

#[test]
fn test_set_running_directly() {
    let mut state = TerminalOutputState::new();
    state.set_running(true);
    assert!(state.running());
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_view_empty() {
    let state = TerminalOutputState::new();
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_content() {
    let mut state = TerminalOutputState::new().with_title("Output");
    state.set_auto_scroll(false);
    state.push_line("Line 1: Hello, world!");
    state.push_line("Line 2: Compiling...");
    state.push_line("Line 3: Done.");
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_ansi_colors() {
    let mut state = TerminalOutputState::new();
    state.set_auto_scroll(false);
    state.push_line("\x1b[32m   Compiling\x1b[0m envision v0.7.0");
    state.push_line("\x1b[32m    Finished\x1b[0m in 2.5s");
    state.push_line("\x1b[31merror[E0308]\x1b[0m: mismatched types");
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_line_numbers() {
    let mut state = TerminalOutputState::new().with_line_numbers(true);
    state.set_auto_scroll(false);
    for i in 1..=5 {
        state.push_line(format!("Line content {i}"));
    }
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_running() {
    let mut state = TerminalOutputState::new().with_running(true);
    state.set_auto_scroll(false);
    state.push_line("Starting build...");
    let (mut terminal, theme) = test_utils::setup_render(50, 8);
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_exit_code() {
    let mut state = TerminalOutputState::new();
    state.set_auto_scroll(false);
    state.push_line("Done.");
    state.set_exit_code(Some(0));
    let (mut terminal, theme) = test_utils::setup_render(50, 8);
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let mut state = TerminalOutputState::new().with_disabled(true);
    state.push_line("Disabled content");
    let (mut terminal, theme) = test_utils::setup_render(50, 8);
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

// =============================================================================
// Annotation tests
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = TerminalOutputState::new();
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                TerminalOutput::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::TerminalOutput);
    assert_eq!(regions.len(), 1);
    assert!(!regions[0].annotation.focused);
    assert!(!regions[0].annotation.disabled);
}

#[test]
fn test_annotation_focused() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(30, 5);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                TerminalOutput::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    let regions = registry.find_by_type(&WidgetType::TerminalOutput);
    assert_eq!(regions.len(), 1);
    assert!(regions[0].annotation.focused);
}
