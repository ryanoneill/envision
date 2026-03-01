use super::*;

// ========================================
// Component Tests
// ========================================

#[test]
fn test_init() {
    let state: LoadingListState<String> = LoadingList::init();
    assert!(state.is_empty());
}

#[test]
fn test_update_set_loading() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    let output = LoadingList::update(&mut state, LoadingListMessage::SetLoading(0));

    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Loading
        })
    ));
    assert!(state.items()[0].is_loading());
}

#[test]
fn test_update_set_ready() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_loading(0);

    let output = LoadingList::update(&mut state, LoadingListMessage::SetReady(0));

    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Ready
        })
    ));
}

#[test]
fn test_update_set_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    let output = LoadingList::update(
        &mut state,
        LoadingListMessage::SetError {
            index: 0,
            message: "Failed".to_string(),
        },
    );

    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Error(_)
        })
    ));
}

#[test]
fn test_update_clear_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_error(0, "Error");

    let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(0));

    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Ready
        })
    ));
    assert!(state.items()[0].is_ready());
}

#[test]
fn test_update_clear_error_not_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(0));
    assert!(output.is_none()); // Was already ready
}

// ========================================
// Navigation Tests
// ========================================

#[test]
fn test_update_down() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(0));

    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(1));
}

#[test]
fn test_update_down_wrap() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(2)); // Last item

    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(0)); // Wraps
}

#[test]
fn test_update_up() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(2));

    LoadingList::update(&mut state, LoadingListMessage::Up);
    assert_eq!(state.selected(), Some(1));
}

#[test]
fn test_update_up_wrap() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(0));

    LoadingList::update(&mut state, LoadingListMessage::Up);
    assert_eq!(state.selected(), Some(2)); // Wraps
}

#[test]
fn test_update_first() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(2));

    LoadingList::update(&mut state, LoadingListMessage::First);
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_update_last() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    LoadingList::update(&mut state, LoadingListMessage::Last);
    assert_eq!(state.selected(), Some(2));
}

#[test]
fn test_update_select() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items.clone(), |i| i.name.clone());
    state.set_selected(Some(1));

    let output = LoadingList::update(&mut state, LoadingListMessage::Select);

    assert!(matches!(output, Some(LoadingListOutput::Selected(item)) if item.id == 2));
}

#[test]
fn test_update_select_nothing() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    let output = LoadingList::update(&mut state, LoadingListMessage::Select);
    assert!(output.is_none()); // Nothing selected
}

#[test]
fn test_update_tick() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    assert_eq!(state.spinner_frame(), 0);
    LoadingList::update(&mut state, LoadingListMessage::Tick);
    assert_eq!(state.spinner_frame(), 1);
}

#[test]
fn test_navigation_empty_list() {
    let mut state: LoadingListState<TestItem> = LoadingListState::new();

    let output = LoadingList::update(&mut state, LoadingListMessage::Down);
    assert!(output.is_none());

    let output = LoadingList::update(&mut state, LoadingListMessage::Up);
    assert!(output.is_none());
}

// ========================================
// View Tests
// ========================================

#[test]
fn test_view_empty() {
    let state: LoadingListState<String> = LoadingListState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_items() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(1));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_title() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_title("My Items");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_error(0, "Connection failed");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========================================
// Additional Coverage Tests
// ========================================

#[test]
fn test_view_zero_size_area() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    // Test with zero width
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, Rect::new(0, 0, 0, 10), &theme);
        })
        .unwrap();

    // Test with zero height
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, Rect::new(0, 0, 60, 0), &Theme::default());
        })
        .unwrap();
}

#[test]
fn test_view_without_indicators() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_indicators(false);
    state.set_selected(Some(0));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_without_indicators_with_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_indicators(false);
    state.set_error(0, "Failed");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| LoadingList::view(&state, frame, frame.area(), &theme))
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_update_set_items() {
    let mut state: LoadingListState<TestItem> = LoadingListState::new();
    state.set_selected(Some(0));

    let items = make_items();
    LoadingList::update(&mut state, LoadingListMessage::SetItems(items));

    assert_eq!(state.len(), 3);
    assert!(state.selected().is_none()); // Selection cleared
    assert_eq!(state.items()[0].label(), "Item 1"); // Uses default labeling
}

#[test]
fn test_update_invalid_index() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // SetLoading with invalid index
    let output = LoadingList::update(&mut state, LoadingListMessage::SetLoading(100));
    assert!(output.is_none());

    // SetReady with invalid index
    let output = LoadingList::update(&mut state, LoadingListMessage::SetReady(100));
    assert!(output.is_none());

    // SetError with invalid index
    let output = LoadingList::update(
        &mut state,
        LoadingListMessage::SetError {
            index: 100,
            message: "Error".to_string(),
        },
    );
    assert!(output.is_none());

    // ClearError with invalid index
    let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(100));
    assert!(output.is_none());
}

#[test]
fn test_up_no_selection() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    // No selection set

    let output = LoadingList::update(&mut state, LoadingListMessage::Up);
    assert_eq!(state.selected(), Some(2)); // Goes to last item
    assert!(matches!(
        output,
        Some(LoadingListOutput::SelectionChanged(2))
    ));
}

#[test]
fn test_down_no_selection() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    // No selection set

    let output = LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(0)); // Goes to first item
    assert!(matches!(
        output,
        Some(LoadingListOutput::SelectionChanged(0))
    ));
}

#[test]
fn test_first_empty_list() {
    let mut state: LoadingListState<TestItem> = LoadingListState::new();

    let output = LoadingList::update(&mut state, LoadingListMessage::First);
    assert!(output.is_none());
}

#[test]
fn test_last_empty_list() {
    let mut state: LoadingListState<TestItem> = LoadingListState::new();

    let output = LoadingList::update(&mut state, LoadingListMessage::Last);
    assert!(output.is_none());
}

// ========================================
// State Transition Edge Case Tests
// ========================================

#[test]
fn test_rapid_loading_ready_cycles() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // Rapidly cycle items between loading and ready states
    for cycle in 0..5 {
        for idx in 0..3 {
            let output = LoadingList::update(&mut state, LoadingListMessage::SetLoading(idx));
            assert!(
                matches!(
                    output,
                    Some(LoadingListOutput::ItemStateChanged {
                        state: ItemState::Loading,
                        ..
                    })
                ),
                "Cycle {cycle}, index {idx}: expected Loading state change"
            );
            assert!(state.items()[idx].is_loading());
        }

        for idx in 0..3 {
            let output = LoadingList::update(&mut state, LoadingListMessage::SetReady(idx));
            assert!(
                matches!(
                    output,
                    Some(LoadingListOutput::ItemStateChanged {
                        state: ItemState::Ready,
                        ..
                    })
                ),
                "Cycle {cycle}, index {idx}: expected Ready state change"
            );
            assert!(state.items()[idx].is_ready());
        }
    }

    // After all cycles, all items should be ready
    assert_eq!(state.loading_count(), 0);
    assert_eq!(state.error_count(), 0);
}

#[test]
fn test_loading_to_error_to_ready() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // Start in Ready (default), transition to Loading
    assert!(state.items()[0].is_ready());
    LoadingList::update(&mut state, LoadingListMessage::SetLoading(0));
    assert!(state.items()[0].is_loading());

    // Transition from Loading to Error
    let output = LoadingList::update(
        &mut state,
        LoadingListMessage::SetError {
            index: 0,
            message: "Network timeout".to_string(),
        },
    );
    assert!(state.items()[0].is_error());
    assert_eq!(
        state.items()[0].state().error_message(),
        Some("Network timeout")
    );
    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Error(_),
        })
    ));

    // Transition from Error back to Ready via ClearError
    let output = LoadingList::update(&mut state, LoadingListMessage::ClearError(0));
    assert!(state.items()[0].is_ready());
    assert!(matches!(
        output,
        Some(LoadingListOutput::ItemStateChanged {
            index: 0,
            state: ItemState::Ready,
        })
    ));
}

#[test]
fn test_all_items_loading_then_ready() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // Mark all items as loading
    for idx in 0..3 {
        LoadingList::update(&mut state, LoadingListMessage::SetLoading(idx));
    }
    assert_eq!(state.loading_count(), 3);
    assert!(state.has_loading());
    assert!(!state.has_errors());

    // Mark items as ready one by one, verifying counts
    for idx in 0..3 {
        LoadingList::update(&mut state, LoadingListMessage::SetReady(idx));
        assert_eq!(state.loading_count(), 2 - idx);
        assert!(state.items()[idx].is_ready());
    }

    assert_eq!(state.loading_count(), 0);
    assert!(!state.has_loading());
}

#[test]
fn test_mixed_item_states_with_navigation() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // Set each item to a different state
    // Item 0: Loading
    LoadingList::update(&mut state, LoadingListMessage::SetLoading(0));
    // Item 1: Error
    LoadingList::update(
        &mut state,
        LoadingListMessage::SetError {
            index: 1,
            message: "Failed".to_string(),
        },
    );
    // Item 2: Ready (default)

    assert!(state.items()[0].is_loading());
    assert!(state.items()[1].is_error());
    assert!(state.items()[2].is_ready());

    // Navigate through all items regardless of state
    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(0));
    assert!(state.selected_item().unwrap().is_loading());

    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(1));
    assert!(state.selected_item().unwrap().is_error());

    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(2));
    assert!(state.selected_item().unwrap().is_ready());

    // Select works on items in any state
    let output = LoadingList::update(&mut state, LoadingListMessage::Select);
    assert!(matches!(
        output,
        Some(LoadingListOutput::Selected(item)) if item.id == 3
    ));
}

#[test]
fn test_large_loading_list_navigation() {
    let items: Vec<TestItem> = (0..100)
        .map(|i| TestItem {
            id: i,
            name: format!("Item {}", i),
        })
        .collect();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // Navigate through them (wrapping navigation, starts with no selection)
    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(0));

    for _ in 0..50 {
        LoadingList::update(&mut state, LoadingListMessage::Down);
    }
    assert_eq!(state.selected(), Some(50));

    LoadingList::update(&mut state, LoadingListMessage::First);
    assert_eq!(state.selected(), Some(0));

    LoadingList::update(&mut state, LoadingListMessage::Last);
    assert_eq!(state.selected(), Some(99));

    // Down from last wraps to first
    LoadingList::update(&mut state, LoadingListMessage::Down);
    assert_eq!(state.selected(), Some(0));

    // Up from first wraps to last
    LoadingList::update(&mut state, LoadingListMessage::Up);
    assert_eq!(state.selected(), Some(99));
}
