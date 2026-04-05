use super::*;

// ========================================
// handle_event Tests
// ========================================

#[test]
fn test_handle_event_scroll_up() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    // Up arrow -> ScrollUp
    let msg = MultiProgress::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(MultiProgressMessage::ScrollUp));

    // Vim 'k' -> ScrollUp
    let msg =
        MultiProgress::handle_event(&state, &Event::char('k'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(MultiProgressMessage::ScrollUp));
}

#[test]
fn test_handle_event_scroll_down() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    // Down arrow -> ScrollDown
    let msg = MultiProgress::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(MultiProgressMessage::ScrollDown));

    // Vim 'j' -> ScrollDown
    let msg =
        MultiProgress::handle_event(&state, &Event::char('j'), &ViewContext::new().focused(true));
    assert_eq!(msg, Some(MultiProgressMessage::ScrollDown));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = MultiProgressState::new();
    let msg =
        MultiProgress::handle_event(&state, &Event::key(KeyCode::Up), &ViewContext::default());
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
    state.set_selected(Some(5));

    // Down arrow dispatches ScrollDown
    MultiProgress::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(state.selected(), Some(6));
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
    state.set_selected(Some(3));

    // instance handle_event
    let msg = state.handle_event(&Event::key(KeyCode::Up));
    assert_eq!(msg, Some(MultiProgressMessage::ScrollUp));

    // instance update
    state.update(MultiProgressMessage::ScrollUp);
    assert_eq!(state.selected(), Some(2));

    // instance dispatch_event
    state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(state.selected(), Some(3));
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

    let msg = MultiProgress::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = MultiProgress::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = MultiProgress::handle_event(
        &state,
        &Event::char('k'),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = MultiProgress::handle_event(
        &state,
        &Event::char('j'),
        &ViewContext::new().focused(true).disabled(true),
    );
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
    state.set_selected(Some(1));
    state.set_disabled(true);

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollUp);
    assert_eq!(state.selected(), Some(1)); // Should not change
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
    state.set_selected(Some(5));
    state.set_disabled(true);

    MultiProgress::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(state.selected(), Some(5)); // Should not change
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

// ========================================
// Unrecognized Event Tests
// ========================================

#[test]
fn test_handle_event_unrecognized_key() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    let msg = MultiProgress::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, None);

    let msg = MultiProgress::handle_event(
        &state,
        &Event::key(KeyCode::Esc),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, None);

    let msg = MultiProgress::handle_event(
        &state,
        &Event::key(KeyCode::Home),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, None);

    let msg = MultiProgress::handle_event(
        &state,
        &Event::key(KeyCode::End),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_unrecognized_char() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    let msg =
        MultiProgress::handle_event(&state, &Event::char('a'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);

    let msg =
        MultiProgress::handle_event(&state, &Event::char('z'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);

    let msg =
        MultiProgress::handle_event(&state, &Event::char('q'), &ViewContext::new().focused(true));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_instance_unrecognized() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    let msg = state.handle_event(&Event::key(KeyCode::Tab));
    assert_eq!(msg, None);
}

// ========================================
// Dispatch Event Returns None for Unrecognized
// ========================================

#[test]
fn test_dispatch_event_unrecognized_returns_none() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    let output = MultiProgress::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Tab),
        &ViewContext::new().focused(true),
    );
    assert!(output.is_none());
}

#[test]
fn test_dispatch_event_unfocused_returns_none() {
    let mut state = MultiProgressState::new();
    // Not focused

    let output = MultiProgress::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Up),
        &ViewContext::default(),
    );
    assert!(output.is_none());
}

// ========================================
// Dispatch Event Scroll Up
// ========================================

#[test]
fn test_dispatch_event_scroll_up() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_selected(Some(5));

    MultiProgress::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(state.selected(), Some(4));
}

#[test]
fn test_dispatch_event_scroll_up_vim_k() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_selected(Some(5));

    MultiProgress::dispatch_event(
        &mut state,
        &Event::char('k'),
        &ViewContext::new().focused(true),
    );
    assert_eq!(state.selected(), Some(4));
}

#[test]
fn test_dispatch_event_scroll_down_vim_j() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_selected(Some(3));

    MultiProgress::dispatch_event(
        &mut state,
        &Event::char('j'),
        &ViewContext::new().focused(true),
    );
    assert_eq!(state.selected(), Some(4));
}

#[test]
fn test_instance_dispatch_event_scroll_up() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    for i in 0..10 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_selected(Some(5));

    state.dispatch_event(&Event::key(KeyCode::Up));
    assert_eq!(state.selected(), Some(4));
}

// ========================================
// Disabled Prevents All Message Variants
// ========================================

#[test]
fn test_update_set_progress_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_disabled(true);

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetProgress {
            id: "id1".to_string(),
            progress: 0.5,
        },
    );
    assert!(output.is_none());
    assert_eq!(state.find("id1").unwrap().progress(), 0.0);
}

#[test]
fn test_update_set_status_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_disabled(true);

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetStatus {
            id: "id1".to_string(),
            status: ProgressItemStatus::Active,
        },
    );
    assert!(output.is_none());
    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Pending
    );
}

#[test]
fn test_update_set_message_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_disabled(true);

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::SetMessage {
            id: "id1".to_string(),
            message: Some("Message".to_string()),
        },
    );
    assert!(output.is_none());
    assert!(state.find("id1").unwrap().message().is_none());
}

#[test]
fn test_update_fail_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_disabled(true);

    let output = MultiProgress::update(
        &mut state,
        MultiProgressMessage::Fail {
            id: "id1".to_string(),
            message: Some("Error".to_string()),
        },
    );
    assert!(output.is_none());
    assert_eq!(
        state.find("id1").unwrap().status(),
        ProgressItemStatus::Pending
    );
}

#[test]
fn test_update_remove_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_disabled(true);

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Remove("id1".to_string()));
    assert!(output.is_none());
    assert_eq!(state.len(), 1);
}

#[test]
fn test_update_scroll_down_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    for i in 0..5 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_disabled(true);

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_update_scroll_to_top_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    for i in 0..5 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_selected(Some(3));
    state.set_disabled(true);

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollToTop);
    assert_eq!(state.selected(), Some(3));
}

#[test]
fn test_update_scroll_to_bottom_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    for i in 0..5 {
        state.add(format!("id{}", i), format!("Item {}", i));
    }
    state.set_disabled(true);

    MultiProgress::update(&mut state, MultiProgressMessage::ScrollToBottom);
    assert_eq!(state.scroll_offset(), 0);
}

// ========================================
// Select (Enter) Tests
// ========================================

#[test]
fn test_handle_event_enter_produces_select() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    let msg = MultiProgress::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(MultiProgressMessage::Select));
}

#[test]
fn test_update_select_emits_selected_output() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");
    state.add("id3", "Item 3");
    state.set_selected(Some(1));

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Select);
    assert_eq!(output, Some(MultiProgressOutput::Selected(1)));
}

#[test]
fn test_update_select_first_item() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Select);
    assert_eq!(output, Some(MultiProgressOutput::Selected(0)));
}

#[test]
fn test_update_select_last_item() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");
    state.add("id3", "Item 3");
    state.set_selected(Some(2));

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Select);
    assert_eq!(output, Some(MultiProgressOutput::Selected(2)));
}

#[test]
fn test_update_select_empty_returns_none() {
    let mut state = MultiProgressState::new();

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Select);
    assert!(output.is_none());
}

#[test]
fn test_update_select_ignored_when_disabled() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    state.set_disabled(true);

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Select);
    assert!(output.is_none());
}

#[test]
fn test_dispatch_event_enter_selects_item() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    state.add("id1", "Item 1");
    state.add("id2", "Item 2");
    state.set_selected(Some(1));

    let output = MultiProgress::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(MultiProgressOutput::Selected(1)));
}

#[test]
fn test_instance_handle_event_enter() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);

    let msg = state.handle_event(&Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(MultiProgressMessage::Select));
}

#[test]
fn test_instance_dispatch_event_enter() {
    let mut state = MultiProgressState::new();
    state.set_focused(true);
    state.add("id1", "Item 1");

    let output = state.dispatch_event(&Event::key(KeyCode::Enter));
    assert_eq!(output, Some(MultiProgressOutput::Selected(0)));
}

#[test]
fn test_select_clamps_to_last_item() {
    let mut state = MultiProgressState::new();
    state.add("id1", "Item 1");
    // scroll_offset is clamped by set_scroll_offset, but test the Select logic
    // by directly accessing
    state.set_scroll_offset(10); // Will be clamped to 0 (last valid index)

    let output = MultiProgress::update(&mut state, MultiProgressMessage::Select);
    assert_eq!(output, Some(MultiProgressOutput::Selected(0)));
}
