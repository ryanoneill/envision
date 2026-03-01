use super::*;

// ========================================
// handle_event Tests
// ========================================

#[test]
fn test_handle_event_scroll_up() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    // Up arrow -> ScrollUp
    let msg = MultiProgress::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, Some(MultiProgressMessage::ScrollUp));

    // Vim 'k' -> ScrollUp
    let msg = MultiProgress::handle_event(&state, &Event::char('k'));
    assert_eq!(msg, Some(MultiProgressMessage::ScrollUp));
}

#[test]
fn test_handle_event_scroll_down() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    // Down arrow -> ScrollDown
    let msg = MultiProgress::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, Some(MultiProgressMessage::ScrollDown));

    // Vim 'j' -> ScrollDown
    let msg = MultiProgress::handle_event(&state, &Event::char('j'));
    assert_eq!(msg, Some(MultiProgressMessage::ScrollDown));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = MultiProgressState::new();
    let msg = MultiProgress::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

// ========================================
// dispatch_event Tests
// ========================================

#[test]
fn test_dispatch_event() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_scroll_offset(5);

    // Down arrow dispatches ScrollDown
    MultiProgress::dispatch_event(&mut state, &Event::key(KeyCode::Down));
    assert_eq!(state.scroll_offset(), 6);
}

// ========================================
// Instance Method Tests
// ========================================

#[test]
fn test_instance_methods() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_scroll_offset(3);

    // instance handle_event
    let msg = state.handle_event(&Event::key(KeyCode::Up));
    assert_eq!(msg, Some(MultiProgressMessage::ScrollUp));

    // instance update
    state.update(MultiProgressMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 2);

    // instance dispatch_event
    state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(state.scroll_offset(), 3);
}

// ========================================
// Disabled State Tests
// ========================================

#[test]
fn test_disabled_default_false() {
    let state = MultiProgressState::new();
    assert!(!state.is_disabled());
}

#[test]
fn test_set_disabled() {
    let mut state = MultiProgressState::new();
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());

    state.set_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_with_disabled() {
    let state = MultiProgressState::new().with_disabled(true);
    assert!(state.is_disabled());

    let state = MultiProgressState::new().with_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    state.set_disabled(true);

    let msg = MultiProgress::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);

    let msg = MultiProgress::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);

    let msg = MultiProgress::handle_event(&state, &Event::char('k'));
    assert_eq!(msg, None);

    let msg = MultiProgress::handle_event(&state, &Event::char('j'));
    assert_eq!(msg, None);
}

#[test]
fn test_update_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.set_disabled(true);

    // Add should be ignored
    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Add {
            id: "id1".to_string(),
            label: "Item 1".to_string(),
        },
    );
    assert!(output.is_none());
    assert!(state.is_empty());
}

#[test]
fn test_update_scroll_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");
    state.set_scroll_offset(1);
    state.set_disabled(true);

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 1); // Should not change
}

#[test]
fn test_update_complete_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_disabled(true);

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Complete("id1".to_string()),
    );
    assert!(output.is_none());
    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Pending
    );
}

#[test]
fn test_update_clear_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_disabled(true);

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Clear);
    assert!(output.is_none());
    assert_eq!(state.len(), 1); // Should not be cleared
}

#[test]
fn test_dispatch_event_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    state.set_disabled(true);
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    // Temporarily enable to set scroll offset
    state.set_disabled(false);
    state.set_scroll_offset(5);
    state.set_disabled(true);

    MultiProgress::dispatch_event(&mut state, &Event::key(KeyCode::Down));
    assert_eq!(state.scroll_offset(), 5); // Should not change
}

#[test]
fn test_instance_handle_event_disabled() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    state.set_disabled(true);

    let msg = state.handle_event(&Event::key(KeyCode::Up));
    assert!(msg.is_none());
}

#[test]
fn test_instance_update_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_disabled(true);

    let output = state.update(MultiProgressMessage::Complete("id1".to_string()));
    assert!(output.is_none());
}

#[test]
fn test_instance_dispatch_event_disabled() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    state.set_disabled(true);
    state.add("id1", "Item 1");

    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert!(output.is_none());
}
