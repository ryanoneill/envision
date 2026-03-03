use super::*;

// ========================================
// LoadingListState Tests
// ========================================

#[test]
fn test_state_new() {
    let state: LoadingListState<String> = LoadingListState::new();
    assert!(state.is_empty());
    assert!(state.selected_index().is_none());
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
fn test_state_selected_index() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_selected(Some(1));
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_item().unwrap().label(), "Item Two");
    assert_eq!(state.selected_data().unwrap().id, 2);
}

#[test]
fn test_selected_returns_item() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    // No selection returns None
    assert!(state.selected().is_none());

    // With selection returns the item
    state.set_selected(Some(0));
    let item = state.selected().unwrap();
    assert_eq!(item.label(), "Item One");
    assert_eq!(item.data().id, 1);

    // selected() and selected_item() return the same thing
    state.set_selected(Some(2));
    assert_eq!(
        state.selected().unwrap().label(),
        state.selected_item().unwrap().label()
    );
}

#[test]
fn test_state_selected_clamped() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    state.set_selected(Some(100)); // Too high
    assert_eq!(state.selected_index(), Some(2)); // Clamped to last
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
    assert!(state.selected_index().is_none());
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

// with_selected tests

#[test]
fn test_state_with_selected() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_selected(1);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_item().unwrap().label(), "Item Two");
}

#[test]
fn test_state_with_selected_first() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_selected(0);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item().unwrap().label(), "Item One");
}

#[test]
fn test_state_with_selected_last() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_selected(2);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(state.selected_item().unwrap().label(), "Item Three");
}

#[test]
fn test_state_with_selected_clamped() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_selected(100);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(state.selected_item().unwrap().label(), "Item Three");
}

#[test]
fn test_state_with_selected_empty() {
    let state: LoadingListState<String> = LoadingListState::new().with_selected(0);
    assert_eq!(state.selected_index(), None);
    assert!(state.selected_item().is_none());
}

#[test]
fn test_state_with_selected_chained() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone())
        .with_selected(1)
        .with_title("My List")
        .with_disabled(true);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.title(), Some("My List"));
    assert!(state.is_disabled());
}

// ========================================
// Default Trait Tests
// ========================================

#[test]
fn test_state_default() {
    let state: LoadingListState<String> = LoadingListState::default();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
    assert_eq!(state.selected_index(), None);
    assert!(state.selected().is_none());
    assert!(state.selected_item().is_none());
    assert!(state.selected_data().is_none());
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert_eq!(state.spinner_frame(), 0);
    assert!(state.title().is_none());
    assert!(state.show_indicators());
}

// ========================================
// PartialEq Tests
// ========================================

#[test]
fn test_state_partial_eq_empty() {
    let state_a: LoadingListState<String> = LoadingListState::new();
    let state_b: LoadingListState<String> = LoadingListState::new();
    assert_eq!(state_a, state_b);
}

#[test]
fn test_state_partial_eq_with_items() {
    let items_a = make_items();
    let items_b = make_items();
    let state_a = LoadingListState::with_items(items_a, |i| i.name.clone());
    let state_b = LoadingListState::with_items(items_b, |i| i.name.clone());
    assert_eq!(state_a, state_b);
}

#[test]
fn test_state_partial_eq_different_selected() {
    let items_a = make_items();
    let items_b = make_items();
    let state_a = LoadingListState::with_items(items_a, |i| i.name.clone()).with_selected(0);
    let state_b = LoadingListState::with_items(items_b, |i| i.name.clone()).with_selected(1);
    assert_ne!(state_a, state_b);
}

#[test]
fn test_state_partial_eq_different_focused() {
    let items_a = make_items();
    let items_b = make_items();
    let mut state_a = LoadingListState::with_items(items_a, |i| i.name.clone());
    let state_b = LoadingListState::with_items(items_b, |i| i.name.clone());
    state_a.set_focused(true);
    assert_ne!(state_a, state_b);
}

#[test]
fn test_state_partial_eq_different_disabled() {
    let items_a = make_items();
    let items_b = make_items();
    let state_a = LoadingListState::with_items(items_a, |i| i.name.clone()).with_disabled(true);
    let state_b = LoadingListState::with_items(items_b, |i| i.name.clone());
    assert_ne!(state_a, state_b);
}

#[test]
fn test_state_partial_eq_different_title() {
    let state_a: LoadingListState<String> = LoadingListState::new().with_title("A");
    let state_b: LoadingListState<String> = LoadingListState::new().with_title("B");
    assert_ne!(state_a, state_b);
}

#[test]
fn test_state_partial_eq_different_indicators() {
    let state_a: LoadingListState<String> = LoadingListState::new().with_indicators(true);
    let state_b: LoadingListState<String> = LoadingListState::new().with_indicators(false);
    assert_ne!(state_a, state_b);
}

// ========================================
// set_selected Edge Case Tests
// ========================================

#[test]
fn test_set_selected_none() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(1));
    assert_eq!(state.selected_index(), Some(1));

    state.set_selected(None);
    assert_eq!(state.selected_index(), None);
    assert!(state.selected().is_none());
    assert!(state.selected_item().is_none());
    assert!(state.selected_data().is_none());
}

#[test]
fn test_set_selected_on_empty_list() {
    let mut state: LoadingListState<TestItem> = LoadingListState::new();
    // set_selected with Some on empty list should clamp to 0 but saturating_sub(1) on 0 == 0
    state.set_selected(Some(0));
    // With empty list, index 0 is clamped via min(0.saturating_sub(1)) = min(0, 0) = Some(0)
    // But get will return None since items is empty
    assert!(state.selected().is_none());
}

// ========================================
// selected_data Edge Cases
// ========================================

#[test]
fn test_selected_data_none_when_no_selection() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    assert!(state.selected_data().is_none());
}

#[test]
fn test_selected_data_returns_data() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(0));
    let data = state.selected_data().unwrap();
    assert_eq!(data.id, 1);
    assert_eq!(data.name, "Item One");
}

// ========================================
// Count Edge Cases
// ========================================

#[test]
fn test_counts_on_empty_list() {
    let state: LoadingListState<TestItem> = LoadingListState::new();
    assert_eq!(state.loading_count(), 0);
    assert_eq!(state.error_count(), 0);
    assert!(!state.has_loading());
    assert!(!state.has_errors());
}

#[test]
fn test_counts_all_ready() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    assert_eq!(state.loading_count(), 0);
    assert_eq!(state.error_count(), 0);
    assert!(!state.has_loading());
    assert!(!state.has_errors());
}

#[test]
fn test_counts_all_loading() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_loading(0);
    state.set_loading(1);
    state.set_loading(2);
    assert_eq!(state.loading_count(), 3);
    assert_eq!(state.error_count(), 0);
    assert!(state.has_loading());
    assert!(!state.has_errors());
}

#[test]
fn test_counts_all_error() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_error(0, "err1");
    state.set_error(1, "err2");
    state.set_error(2, "err3");
    assert_eq!(state.loading_count(), 0);
    assert_eq!(state.error_count(), 3);
    assert!(!state.has_loading());
    assert!(state.has_errors());
}

// ========================================
// Single Item List Tests
// ========================================

#[test]
fn test_single_item_list() {
    let items = vec![TestItem {
        id: 1,
        name: "Only".to_string(),
    }];
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    assert_eq!(state.len(), 1);
    assert!(!state.is_empty());
}

#[test]
fn test_single_item_selected() {
    let items = vec![TestItem {
        id: 1,
        name: "Only".to_string(),
    }];
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_selected(0);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item().unwrap().label(), "Only");
}

// ========================================
// Clear After Various State Operations
// ========================================

#[test]
fn test_clear_resets_everything() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone())
        .with_selected(1)
        .with_title("Title");
    state.set_loading(0);
    state.set_error(2, "err");

    state.clear();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.loading_count(), 0);
    assert_eq!(state.error_count(), 0);
    // Title is preserved through clear (only items/selection are cleared)
    assert_eq!(state.title(), Some("Title"));
}

// ========================================
// with_items Label Function Tests
// ========================================

#[test]
fn test_with_items_custom_label_fn() {
    let items = vec![
        TestItem {
            id: 1,
            name: "Alpha".to_string(),
        },
        TestItem {
            id: 2,
            name: "Beta".to_string(),
        },
    ];
    let state = LoadingListState::with_items(items, |i| format!("{}: {}", i.id, i.name));
    assert_eq!(state.items()[0].label(), "1: Alpha");
    assert_eq!(state.items()[1].label(), "2: Beta");
}

#[test]
fn test_with_items_empty() {
    let items: Vec<TestItem> = vec![];
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

// ========================================
// get and get_mut Tests
// ========================================

#[test]
fn test_get_all_indices() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    assert_eq!(state.get(0).unwrap().label(), "Item One");
    assert_eq!(state.get(1).unwrap().label(), "Item Two");
    assert_eq!(state.get(2).unwrap().label(), "Item Three");
    assert!(state.get(3).is_none());
}

#[test]
fn test_get_mut_modify_state() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());

    if let Some(item) = state.get_mut(1) {
        item.set_state(ItemState::Loading);
    }
    assert!(state.items()[1].is_loading());
    assert_eq!(state.loading_count(), 1);
}
