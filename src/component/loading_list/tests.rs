use super::*;

#[derive(Clone, Debug, PartialEq)]
struct TestItem {
    id: u32,
    name: String,
}

fn make_items() -> Vec<TestItem> {
    vec![
        TestItem {
            id: 1,
            name: "Item One".to_string(),
        },
        TestItem {
            id: 2,
            name: "Item Two".to_string(),
        },
        TestItem {
            id: 3,
            name: "Item Three".to_string(),
        },
    ]
}

// ========================================
// ItemState Tests
// ========================================

#[test]
fn test_item_state_error_message() {
    let state = ItemState::Error("Test error".to_string());
    assert_eq!(state.error_message(), Some("Test error"));

    let ready = ItemState::Ready;
    assert!(ready.error_message().is_none());
}

#[test]
fn test_item_state_symbols() {
    assert_eq!(ItemState::Ready.symbol(0), " ");
    assert_eq!(ItemState::Error("".to_string()).symbol(0), "✗");
    // Loading has animated symbols
    assert!(!ItemState::Loading.symbol(0).is_empty());
}

#[test]
fn test_item_state_styles() {
    let theme = Theme::default();
    assert_eq!(ItemState::Ready.style(&theme), theme.normal_style());
    assert_eq!(ItemState::Loading.style(&theme), theme.warning_style());
    assert_eq!(
        ItemState::Error("".to_string()).style(&theme),
        theme.error_style()
    );
}

// ========================================
// LoadingListItem Tests
// ========================================

#[test]
fn test_list_item_new() {
    let item = LoadingListItem::new("data", "Label");
    assert_eq!(item.data(), &"data");
    assert_eq!(item.label(), "Label");
    assert!(item.is_ready());
}

#[test]
fn test_list_item_set_label() {
    let mut item = LoadingListItem::new("data", "Old");
    item.set_label("New");
    assert_eq!(item.label(), "New");
}

#[test]
fn test_list_item_set_state() {
    let mut item = LoadingListItem::new("data", "Label");
    item.set_state(ItemState::Loading);
    assert!(item.is_loading());

    item.set_state(ItemState::Error("err".to_string()));
    assert!(item.is_error());
}

#[test]
fn test_list_item_data_mut() {
    let mut item = LoadingListItem::new("original", "Label");
    *item.data_mut() = "modified";
    assert_eq!(item.data(), &"modified");
}

// ========================================
// LoadingListState Tests
// ========================================

#[test]
fn test_state_new() {
    let state: LoadingListState<String> = LoadingListState::new();
    assert!(state.is_empty());
    assert!(state.selected().is_none());
    assert!(state.show_indicators());
}

#[test]
fn test_state_with_items() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());

    assert_eq!(state.len(), 3);
    assert_eq!(state.items()[0].label(), "Item One");
    assert_eq!(state.items()[1].label(), "Item Two");
    assert_eq!(state.items()[2].label(), "Item Three");
}

#[test]
fn test_state_with_title() {
    let state: LoadingListState<String> = LoadingListState::new().with_title("My List");
    assert_eq!(state.title(), Some("My List"));
}

#[test]
fn test_state_with_indicators() {
    let state: LoadingListState<String> = LoadingListState::new().with_indicators(false);
    assert!(!state.show_indicators());
}

#[test]
fn test_state_set_loading() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_loading(0);
    assert!(state.items()[0].is_loading());
}

#[test]
fn test_state_set_ready() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_loading(0);
    state.set_ready(0);
    assert!(state.items()[0].is_ready());
}

#[test]
fn test_state_set_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_error(0, "Failed to load");
    assert!(state.items()[0].is_error());
    assert_eq!(
        state.items()[0].state().error_message(),
        Some("Failed to load")
    );
}

#[test]
fn test_state_counts() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_loading(0);
    state.set_loading(1);
    state.set_error(2, "Error");

    assert_eq!(state.loading_count(), 2);
    assert_eq!(state.error_count(), 1);
    assert!(state.has_loading());
    assert!(state.has_errors());
}

#[test]
fn test_state_selected() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_selected(Some(1));
    assert_eq!(state.selected(), Some(1));
    assert_eq!(state.selected_item().unwrap().label(), "Item Two");
    assert_eq!(state.selected_data().unwrap().id, 2);
}

#[test]
fn test_state_selected_clamped() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_selected(Some(100)); // Too high
    assert_eq!(state.selected(), Some(2)); // Clamped to last
}

#[test]
fn test_state_get() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());

    assert!(state.get(0).is_some());
    assert!(state.get(100).is_none());
}

#[test]
fn test_state_clear() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(0));

    state.clear();
    assert!(state.is_empty());
    assert!(state.selected().is_none());
}

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

#[test]
fn test_get_mut() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    if let Some(item) = state.get_mut(0) {
        item.set_label("Modified");
    }
    assert_eq!(state.items()[0].label(), "Modified");

    assert!(state.get_mut(100).is_none());
}

#[test]
fn test_items_mut() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.items_mut().push(LoadingListItem::new(
        TestItem {
            id: 4,
            name: "Item Four".to_string(),
        },
        "Item Four",
    ));

    assert_eq!(state.len(), 4);
}

#[test]
fn test_set_title() {
    let mut state: LoadingListState<String> = LoadingListState::new();
    assert!(state.title().is_none());

    state.set_title(Some("New Title".to_string()));
    assert_eq!(state.title(), Some("New Title"));

    state.set_title(None);
    assert!(state.title().is_none());
}

#[test]
fn test_set_show_indicators() {
    let mut state: LoadingListState<String> = LoadingListState::new();
    assert!(state.show_indicators());

    state.set_show_indicators(false);
    assert!(!state.show_indicators());
}

#[test]
fn test_set_loading_invalid_index() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // This should not panic
    state.set_loading(100);
    state.set_ready(100);
    state.set_error(100, "Error");
}

#[test]
fn test_spinner_animation_frames() {
    let state = ItemState::Loading;
    // Test all 10 spinner frames (Braille dots matching SpinnerStyle::Dots)
    let expected = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    for (i, &expected_frame) in expected.iter().enumerate() {
        assert_eq!(state.symbol(i), expected_frame);
    }
    // Frame 10 should wrap to frame 0
    assert_eq!(state.symbol(10), state.symbol(0));
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

// ========================================
// handle_event Tests
// ========================================

use crate::input::{Event, KeyCode};

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
