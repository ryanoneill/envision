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
    assert_eq!(state.selected(), Some(0));
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
    assert_eq!(state.selected(), Some(0));

    // instance dispatch_event
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert!(matches!(
        output,
        Some(LoadingListOutput::SelectionChanged(1))
    ));
    assert_eq!(state.selected(), Some(1));
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
    assert_eq!(state.selected(), None);

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
