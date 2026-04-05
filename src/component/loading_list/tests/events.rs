use super::*;

// ========================================
// handle_event Tests
// ========================================

#[test]
fn test_handle_event_up() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());

    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(LoadingListMessage::Up));
}

#[test]
fn test_handle_event_down() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());

    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(LoadingListMessage::Down));
}

#[test]
fn test_handle_event_select() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());

    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(LoadingListMessage::Select));
}

#[test]
fn test_handle_event_vim_keys() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());

    // 'k' -> Up
    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::char('k'),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(LoadingListMessage::Up));

    // 'j' -> Down
    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::char('j'),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(LoadingListMessage::Down));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    // Not focused by default
    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::key(KeyCode::Up),
        &ViewContext::default(),
    );
    assert_eq!(msg, None);
}

// ========================================
// dispatch_event Tests
// ========================================

#[test]
fn test_dispatch_event() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // Down dispatches Down message, which selects the first item
    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
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

    // instance update
    let output = state.update(LoadingListMessage::Down);
    assert!(matches!(
        output,
        Some(LoadingListOutput::SelectionChanged(0))
    ));
    assert_eq!(state.selected_index(), Some(0));
}

// ========================================
// Disabled State Tests
// ========================================

#[test]
fn test_disabled_prevents_handle_event() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_disabled_allows_programmatic_state_changes() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

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

// ========================================
// Unrecognized Key Tests
// ========================================

#[test]
fn test_handle_event_unrecognized_key() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());

    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::char('a'),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, None);

    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::char('x'),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, None);

    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::key(KeyCode::Tab),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, None);

    let msg = LoadingList::<TestItem>::handle_event(
        &state,
        &Event::key(KeyCode::Esc),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, None);
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
        LoadingList::<TestItem>::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &ViewContext::default()
        ),
        None
    );
    assert_eq!(
        LoadingList::<TestItem>::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &ViewContext::default()
        ),
        None
    );
    assert_eq!(
        LoadingList::<TestItem>::handle_event(
            &state,
            &Event::key(KeyCode::Enter),
            &ViewContext::default()
        ),
        None
    );
    assert_eq!(
        LoadingList::<TestItem>::handle_event(&state, &Event::char('k'), &ViewContext::default()),
        None
    );
    assert_eq!(
        LoadingList::<TestItem>::handle_event(&state, &Event::char('j'), &ViewContext::default()),
        None
    );
}

// ========================================
// Disabled State Allows SetError via Update
// ========================================

#[test]
fn test_disabled_allows_set_error_via_update() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

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
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_error(0, "err");

    let output = LoadingList::<TestItem>::update(&mut state, LoadingListMessage::ClearError(0));
    assert!(output.is_some());
    assert!(state.get(0).unwrap().is_ready());
}

#[test]
fn test_disabled_allows_set_items_via_update() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

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
// Dispatch Event Chained Navigation Tests
// ========================================

#[test]
fn test_dispatch_event_chained_navigation() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // Navigate down 3 times, wrapping around
    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(0)));

    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(1)));

    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(2)));

    // Wraps to top
    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(0)));
}

#[test]
fn test_dispatch_event_up_navigation() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_selected(Some(2));

    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(1)));

    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(0)));

    // Wraps to bottom
    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Up),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(2)));
}

#[test]
fn test_dispatch_event_enter_selects() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_selected(Some(1));

    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert!(matches!(
        output,
        Some(LoadingListOutput::Selected(item)) if item.id == 2
    ));
}

#[test]
fn test_dispatch_event_vim_keys() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // 'j' moves down
    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::char('j'),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(0)));

    // 'k' moves up (wraps)
    let output = LoadingList::<TestItem>::dispatch_event(
        &mut state,
        &Event::char('k'),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(LoadingListOutput::SelectionChanged(2)));
}
