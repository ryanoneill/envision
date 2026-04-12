use super::*;

// =============================================================================
// Event handling
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = ConversationViewState::new();
    assert_eq!(
        ConversationView::handle_event(&state, &Event::char('k'), &EventContext::default()),
        None
    );
}

#[test]
fn test_disabled_ignores_events() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::char('k'),
            &EventContext::new().focused(true).disabled(true)
        ),
        None
    );
}

#[test]
fn test_scroll_up_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::key(Key::Up),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::ScrollUp)
    );
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::char('k'),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::ScrollUp)
    );
}

#[test]
fn test_scroll_down_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::key(Key::Down),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::ScrollDown)
    );
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::char('j'),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::ScrollDown)
    );
}

#[test]
fn test_scroll_to_top_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::key(Key::Home),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::ScrollToTop)
    );
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::char('g'),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::ScrollToTop)
    );
}

#[test]
fn test_scroll_to_bottom_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::key(Key::End),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::ScrollToBottom)
    );
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::char('G'),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::ScrollToBottom)
    );
}

#[test]
fn test_page_up_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::key(Key::PageUp),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::PageUp)
    );
}

#[test]
fn test_page_down_event() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::key(Key::PageDown),
            &EventContext::new().focused(true)
        ),
        Some(ConversationViewMessage::PageDown)
    );
}

#[test]
fn test_unrecognized_key_ignored() {
    let state = focused_state();
    assert_eq!(
        ConversationView::handle_event(
            &state,
            &Event::char('x'),
            &EventContext::new().focused(true)
        ),
        None
    );
}

// =============================================================================
// Scrolling behavior
// =============================================================================

#[test]
fn test_scroll_up_disables_auto_scroll() {
    let mut state = state_with_messages();
    assert!(state.auto_scroll());
    ConversationView::update(&mut state, ConversationViewMessage::ScrollUp);
    assert!(!state.auto_scroll());
}

#[test]
fn test_scroll_to_bottom_enables_auto_scroll() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollToBottom);
    assert!(state.auto_scroll());
}

#[test]
fn test_scroll_to_top_disables_auto_scroll() {
    let mut state = state_with_messages();
    ConversationView::update(&mut state, ConversationViewMessage::ScrollToTop);
    assert!(!state.auto_scroll());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_down_output() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    let output = ConversationView::update(&mut state, ConversationViewMessage::ScrollDown);
    assert!(matches!(
        output,
        Some(ConversationViewOutput::ScrollChanged { .. })
    ));
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    state.scroll.set_offset(0);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_page_up_disables_auto_scroll() {
    let mut state = state_with_messages();
    ConversationView::update(&mut state, ConversationViewMessage::PageUp);
    assert!(!state.auto_scroll());
}

#[test]
fn test_scroll_empty_conversation() {
    let mut state = focused_state();
    let output = ConversationView::update(&mut state, ConversationViewMessage::ScrollDown);
    assert!(matches!(
        output,
        Some(ConversationViewOutput::ScrollChanged { offset: 0 })
    ));
}

// =============================================================================
// Toggle collapse
// =============================================================================

#[test]
fn test_toggle_collapse_via_message() {
    let mut state = ConversationViewState::new();
    ConversationView::update(
        &mut state,
        ConversationViewMessage::ToggleCollapse("thinking".to_string()),
    );
    assert!(state.is_collapsed("thinking"));
    ConversationView::update(
        &mut state,
        ConversationViewMessage::ToggleCollapse("thinking".to_string()),
    );
    assert!(!state.is_collapsed("thinking"));
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    state.push_user("Hello");
    let output = state.update(ConversationViewMessage::ScrollDown);
    assert!(output.is_some());
}
