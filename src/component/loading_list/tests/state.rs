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
