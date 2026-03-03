use super::*;

// ========================================
// handle_event Tests
// ========================================

#[test]
fn test_handle_event_up() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);

    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, Some(LoadingListMessage::Up));
}

#[test]
fn test_handle_event_down() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);

    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, Some(LoadingListMessage::Down));
}

#[test]
fn test_handle_event_select() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);

    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(LoadingListMessage::Select));
}

#[test]
fn test_handle_event_vim_keys() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);

    // 'k' -> Up
    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::char('k'));
    assert_eq!(msg, Some(LoadingListMessage::Up));

    // 'j' -> Down
    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::char('j'));
    assert_eq!(msg, Some(LoadingListMessage::Down));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    // Not focused by default
    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

// ========================================
// dispatch_event Tests
// ========================================

#[test]
fn test_dispatch_event() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);

    // Down dispatches Down message, which selects the first item
    let output = LoadingList::<TestItem>::dispatch_event(&mut state, &Event::key(KeyCode::Down));
    assert!(matches!(
        output,
        Some(LoadingListOutput::SelectionChanged(0))
    ));
    assert_eq!(state.selected_index(), Some(0));
}

// ========================================
// Instance Method Tests
// ========================================

#[test]
fn test_instance_methods() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);

    // instance handle_event
    let msg = state.handle_event(&Event::key(KeyCode::Down));
    assert_eq!(msg, Some(LoadingListMessage::Down));

    // instance update
    let output = state.update(LoadingListMessage::Down);
    assert!(matches!(
        output,
        Some(LoadingListOutput::SelectionChanged(0))
    ));
    assert_eq!(state.selected_index(), Some(0));

    // instance dispatch_event
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert!(matches!(
        output,
        Some(LoadingListOutput::SelectionChanged(1))
    ));
    assert_eq!(state.selected_index(), Some(1));
}

// ========================================
// Disabled State Tests
// ========================================

#[test]
fn test_with_disabled() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_disabled(true);
    assert!(state.is_disabled());

    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_set_disabled() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    assert!(!state.is_disabled());
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

#[test]
fn test_disabled_prevents_handle_event() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);
    state.set_disabled(true);
    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

#[test]
fn test_disabled_prevents_navigation() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_disabled(true);
    state.set_focused(true);

    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), None);

    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::Up);
    assert_eq!(output, None);

    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::First);
    assert_eq!(output, None);

    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::Last);
    assert_eq!(output, None);

    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::Select);
    assert_eq!(output, None);
}

#[test]
fn test_disabled_allows_programmatic_state_changes() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_disabled(true);

    // SetLoading should still work when disabled
    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::SetLoading(0));
    assert!(output.is_some());
    assert!(state.get(0).unwrap().is_loading());

    // SetReady should still work when disabled
    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::SetReady(0));
    assert!(output.is_some());
    assert!(state.get(0).unwrap().is_ready());

    // Tick should still work when disabled
    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::Tick);
    assert!(output.is_none());
}

#[test]
fn test_disabled_dispatch_event_returns_none() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_disabled(true);
    state.set_focused(true);

    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, None);
}

#[test]
fn test_disabled_default_is_false() {
    let state = LoadingListState::<TestItem>::new();
    assert!(!state.is_disabled());
}

#[test]
fn test_builder_chaining_with_disabled() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone())
        .with_title("Test")
        .with_indicators(true)
        .with_disabled(true);
    assert!(state.is_disabled());
    assert_eq!(state.title(), Some("Test"));
    assert!(state.show_indicators());
}

// ========================================
// Unrecognized Key Tests
// ========================================

#[test]
fn test_handle_event_unrecognized_key() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);

    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);

    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::char('x'));
    assert_eq!(msg, None);

    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Tab));
    assert_eq!(msg, None);

    let msg = LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Esc));
    assert_eq!(msg, None);
}

// ========================================
// Focusable Trait Tests
// ========================================

#[test]
fn test_focusable_trait_is_focused() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    assert!(!LoadingList::<TestItem>::is_focused(&state));
}

#[test]
fn test_focusable_trait_set_focused() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    LoadingList::<TestItem>::set_focused(&mut state, true);
    assert!(LoadingList::<TestItem>::is_focused(&state));
    LoadingList::<TestItem>::set_focused(&mut state, false);
    assert!(!LoadingList::<TestItem>::is_focused(&state));
}

#[test]
fn test_focusable_trait_focus_blur() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    LoadingList::<TestItem>::focus(&mut state);
    assert!(state.is_focused());

    LoadingList::<TestItem>::blur(&mut state);
    assert!(!state.is_focused());
}

// ========================================
// Focus / Unfocus Event Gating
// ========================================

#[test]
fn test_handle_event_all_keys_ignored_when_unfocused() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    // Not focused (default)

    assert_eq!(
        LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Up)),
        None
    );
    assert_eq!(
        LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Down)),
        None
    );
    assert_eq!(
        LoadingList::<TestItem>::handle_event(&state, &Event::key(KeyCode::Enter)),
        None
    );
    assert_eq!(
        LoadingList::<TestItem>::handle_event(&state, &Event::char('k')),
        None
    );
    assert_eq!(
        LoadingList::<TestItem>::handle_event(&state, &Event::char('j')),
        None
    );
}

#[test]
fn test_dispatch_event_ignored_when_unfocused() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    // Not focused (default)

    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), None); // No change
}

// ========================================
// Disabled State Allows SetError via Update
// ========================================

#[test]
fn test_disabled_allows_set_error_via_update() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_disabled(true);

    let output = LoadingList::<TestItem>::update(
        &mut state,
        LoadingListMessage::SetError {
            index: 1,
            message: "timeout".to_string(),
        },
    );
    assert!(output.is_some());
    assert!(state.get(1).unwrap().is_error());
}

#[test]
fn test_disabled_allows_clear_error_via_update() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_disabled(true);
    state.set_error(0, "err");

    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::ClearError(0));
    assert!(output.is_some());
    assert!(state.get(0).unwrap().is_ready());
}

#[test]
fn test_disabled_allows_set_items_via_update() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_disabled(true);

    let new_items = vec![TestItem {
        id: 10,
        name: "New".to_string(),
    }];
    let output =
        LoadingList::<TestItem>::update(&mut state, LoadingListMessage::SetItems(new_items));
    assert!(output.is_none());
    assert_eq!(state.len(), 1);
}

// ========================================
// Instance Method Focused/Disabled Tests
// ========================================

#[test]
fn test_instance_is_focused_default() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    assert!(!state.is_focused());
}

#[test]
fn test_instance_set_focused() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());
}

// ========================================
// Dispatch Event Chained Navigation Tests
// ========================================

#[test]
fn test_dispatch_event_chained_navigation() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);

    // Navigate down 3 times, wrapping around
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(0)));

    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(1)));

    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(2)));

    // Wraps to top
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(0)));
}

#[test]
fn test_dispatch_event_up_navigation() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);
    state.set_selected(Some(2));

    let output = state.dispatch_event(&Event::key(KeyCode::Up));
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(1)));

    let output = state.dispatch_event(&Event::key(KeyCode::Up));
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(0)));

    // Wraps to bottom
    let output = state.dispatch_event(&Event::key(KeyCode::Up));
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(2)));
}

#[test]
fn test_dispatch_event_enter_selects() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);
    state.set_selected(Some(1));

    let output = state.dispatch_event(&Event::key(KeyCode::Enter));
    assert!(matches!(
        output,
        Some(LoadingListOutput::Selected(item)) if item.id == 2
    ));
}

#[test]
fn test_dispatch_event_vim_keys() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_focused(true);

    // 'j' moves down
    let output = state.dispatch_event(&Event::char('j'));
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(0)));

    // 'k' moves up (wraps)
    let output = state.dispatch_event(&Event::char('k'));
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(2)));
}
