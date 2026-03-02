use super::*;
use crate::input::{Event, KeyCode};

#[test]
fn test_init_empty() {
    let state = SelectableList::<String>::init();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected_item(), None);
}

#[test]
fn test_new() {
    let state = SelectableListState::new(vec!["a", "b", "c"]);
    assert_eq!(state.len(), 3);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item(), Some(&"a"));
}

#[test]
fn test_new_empty() {
    let state = SelectableListState::<String>::new(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_with_items() {
    let state = SelectableListState::with_items(vec!["a", "b", "c"]);
    assert_eq!(state.len(), 3);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item(), Some(&"a"));
}

#[test]
fn test_set_items() {
    let mut state = SelectableList::<String>::init();
    state.set_items(vec!["x".into(), "y".into()]);
    assert_eq!(state.len(), 2);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_set_items_preserves_selection() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    state.select(Some(1));
    state.set_items(vec!["x", "y", "z", "w"]);
    // Selection should be preserved at index 1
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_set_items_clamps_selection() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    state.select(Some(2));
    state.set_items(vec!["x"]); // Only one item now
                                // Selection should be clamped to last valid index
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_navigate_down() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(1)));

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::Down);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(2)));

    // At the end, should stay at last item
    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::Down);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(output, None);
}

#[test]
fn test_navigate_up() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    state.select(Some(2));

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::Up);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(1)));

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::Up);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(0)));

    // At the beginning, should stay at first item
    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::Up);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, None);
}

#[test]
fn test_navigate_first_last() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e"]);
    state.select(Some(2));

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::Last);
    assert_eq!(state.selected_index(), Some(4));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(4)));

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(0)));
}

#[test]
fn test_page_navigation() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e", "f", "g"]);

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::PageDown(3));
    assert_eq!(state.selected_index(), Some(3));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(3)));

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::PageDown(10));
    assert_eq!(state.selected_index(), Some(6)); // Clamped to last
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(6)));

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::PageUp(4));
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(2)));
}

#[test]
fn test_select() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    state.select(Some(1));

    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::Select);
    assert_eq!(output, Some(SelectableListOutput::Selected("b")));
}

#[test]
fn test_empty_list_navigation() {
    let mut state = SelectableList::<String>::init();

    assert_eq!(
        SelectableList::<String>::update(&mut state, SelectableListMessage::Down),
        None
    );
    assert_eq!(
        SelectableList::<String>::update(&mut state, SelectableListMessage::Up),
        None
    );
    assert_eq!(
        SelectableList::<String>::update(&mut state, SelectableListMessage::Select),
        None
    );
}

#[test]
fn test_select_method() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);

    state.select(Some(2));
    assert_eq!(state.selected_index(), Some(2));

    state.select(None);
    assert_eq!(state.selected_index(), None);

    // Out of bounds should be ignored
    state.select(Some(0));
    state.select(Some(100));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_items_accessor() {
    let state = SelectableListState::with_items(vec![1, 2, 3]);
    assert_eq!(state.items(), &[1, 2, 3]);
}

#[test]
fn test_set_items_to_empty() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    assert_eq!(state.selected_index(), Some(0));

    // Set items to empty list
    state.set_items(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected_item(), None);
}

#[test]
fn test_select_by_index() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);

    // Select a specific index
    state.select(Some(2));
    assert_eq!(state.selected_index(), Some(2));

    // Out-of-bounds index is ignored
    state.select(Some(10));
    assert_eq!(state.selected_index(), Some(2));

    // Deselect
    state.select(None);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_first_when_already_at_first() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    // Already at first item
    assert_eq!(state.selected_index(), Some(0));

    // First should return None when already at first
    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::First);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_last_when_already_at_last() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    state.select(Some(2));
    assert_eq!(state.selected_index(), Some(2));

    // Last should return None when already at last
    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::Last);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_page_up_when_at_first() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e"]);
    assert_eq!(state.selected_index(), Some(0));

    // PageUp at first should return None
    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::PageUp(3));
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_page_down_when_at_last() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e"]);
    state.select(Some(4));
    assert_eq!(state.selected_index(), Some(4));

    // PageDown at last should return None
    let output = SelectableList::<&str>::update(&mut state, SelectableListMessage::PageDown(3));
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(4));
}

#[test]
fn test_view() {
    let mut state = SelectableListState::with_items(vec!["Item 1", "Item 2", "Item 3"]);
    state.focused = true;
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            SelectableList::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_unfocused() {
    let mut state = SelectableListState::with_items(vec!["A", "B", "C"]);
    state.focused = false; // Explicitly unfocused
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            SelectableList::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_with_items_empty() {
    let state = SelectableListState::<String>::with_items(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_default_state() {
    let state: SelectableListState<i32> = SelectableListState::default();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
    assert_eq!(state.selected_index(), None);
    assert!(!state.focused);
}

#[test]
fn test_large_list_navigation() {
    let items: Vec<String> = (0..1000).map(|i| format!("Item {}", i)).collect();
    let mut state = SelectableListState::new(items);

    // Start at 0
    assert_eq!(state.selected_index(), Some(0));

    // Navigate to middle
    for _ in 0..500 {
        SelectableList::<String>::update(&mut state, SelectableListMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(500));

    // First jumps to beginning
    SelectableList::<String>::update(&mut state, SelectableListMessage::First);
    assert_eq!(state.selected_index(), Some(0));

    // Last jumps to end
    SelectableList::<String>::update(&mut state, SelectableListMessage::Last);
    assert_eq!(state.selected_index(), Some(999));

    // PageUp from end
    SelectableList::<String>::update(&mut state, SelectableListMessage::PageUp(100));
    assert_eq!(state.selected_index(), Some(899));

    // PageDown back
    SelectableList::<String>::update(&mut state, SelectableListMessage::PageDown(100));
    assert_eq!(state.selected_index(), Some(999));
}

// handle_event tests

#[test]
fn test_handle_event_up() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, Some(SelectableListMessage::Up));
}

#[test]
fn test_handle_event_down() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, Some(SelectableListMessage::Down));
}

#[test]
fn test_handle_event_home() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::key(KeyCode::Home));
    assert_eq!(msg, Some(SelectableListMessage::First));
}

#[test]
fn test_handle_event_end() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::key(KeyCode::End));
    assert_eq!(msg, Some(SelectableListMessage::Last));
}

#[test]
fn test_handle_event_enter() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(SelectableListMessage::Select));
}

#[test]
fn test_handle_event_page_up() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::key(KeyCode::PageUp));
    assert_eq!(msg, Some(SelectableListMessage::PageUp(10)));
}

#[test]
fn test_handle_event_page_down() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::key(KeyCode::PageDown));
    assert_eq!(msg, Some(SelectableListMessage::PageDown(10)));
}

#[test]
fn test_handle_event_vim_k() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::char('k'));
    assert_eq!(msg, Some(SelectableListMessage::Up));
}

#[test]
fn test_handle_event_vim_j() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::char('j'));
    assert_eq!(msg, Some(SelectableListMessage::Down));
}

#[test]
fn test_handle_event_vim_g() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::char('g'));
    assert_eq!(msg, Some(SelectableListMessage::First));
}

#[test]
fn test_handle_event_vim_shift_g() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = SelectableList::<String>::handle_event(&state, &Event::char('G'));
    assert_eq!(msg, Some(SelectableListMessage::Last));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    let msg = SelectableList::<String>::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, None);
}

#[test]
fn test_dispatch_event_selectable_list() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let output = SelectableList::<String>::dispatch_event(&mut state, &Event::key(KeyCode::Down));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_instance_is_focused() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
}

#[test]
fn test_instance_handle_event() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let msg = state.handle_event(&Event::key(KeyCode::Down));
    assert_eq!(msg, Some(SelectableListMessage::Down));
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    state.set_focused(true);
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(1)));
}

#[test]
fn test_instance_update() {
    let mut state = SelectableListState::new(vec!["one".to_string(), "two".to_string()]);
    let output = state.update(SelectableListMessage::Down);
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(1)));
}

// Filter tests

#[test]
fn test_filter_text_default() {
    let state = SelectableListState::with_items(vec!["a", "b", "c"]);
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 3);
}

#[test]
fn test_set_filter_text() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
        "Apricot".to_string(),
    ]);
    state.set_filter_text("ap");
    assert_eq!(state.filter_text(), "ap");
    assert_eq!(state.visible_count(), 2); // Apple, Apricot
}

#[test]
fn test_filter_case_insensitive() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "banana".to_string(),
        "CHERRY".to_string(),
    ]);
    state.set_filter_text("APPLE");
    assert_eq!(state.visible_count(), 1);
    assert_eq!(state.selected_item(), Some(&"Apple".to_string()));
}

#[test]
fn test_filter_no_matches() {
    let mut state =
        SelectableListState::with_items(vec!["Apple".to_string(), "Banana".to_string()]);
    state.set_filter_text("xyz");
    assert_eq!(state.visible_count(), 0);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected_item(), None);
}

#[test]
fn test_clear_filter() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]);
    state.set_filter_text("ap");
    assert_eq!(state.visible_count(), 1);

    state.clear_filter();
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 3);
}

#[test]
fn test_filter_preserves_selection() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Apricot".to_string(),
    ]);
    // Select Apricot (index 2)
    state.select(Some(2));
    assert_eq!(state.selected_item(), Some(&"Apricot".to_string()));

    // Filter to "ap" - Apple (0) and Apricot (2)
    state.set_filter_text("ap");
    assert_eq!(state.visible_count(), 2);
    // Apricot should still be selected
    assert_eq!(state.selected_item(), Some(&"Apricot".to_string()));
    assert_eq!(state.selected_index(), Some(2)); // original index
}

#[test]
fn test_filter_resets_selection_when_item_hidden() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]);
    // Select Banana (index 1)
    state.select(Some(1));

    // Filter to "ap" - only Apple visible
    state.set_filter_text("ap");
    assert_eq!(state.visible_count(), 1);
    // Banana is filtered out, selection moves to first visible (Apple)
    assert_eq!(state.selected_item(), Some(&"Apple".to_string()));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_filter_navigation() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Apricot".to_string(),
        "Avocado".to_string(),
    ]);
    state.focused = true;
    state.set_filter_text("ap");
    // Filtered: Apple(0), Apricot(2) -- "ap" matches Apple and Apricot
    assert_eq!(state.visible_count(), 2);
    assert_eq!(state.selected_index(), Some(0)); // Apple

    // Navigate down to Apricot
    let output = SelectableList::<String>::update(&mut state, SelectableListMessage::Down);
    assert_eq!(state.selected_item(), Some(&"Apricot".to_string()));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(2))); // original index

    // At end, stay
    let output = SelectableList::<String>::update(&mut state, SelectableListMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_item(), Some(&"Apricot".to_string()));
}

#[test]
fn test_filter_select_returns_original_item() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Apricot".to_string(),
    ]);
    state.set_filter_text("ap");
    // Filtered: Apple(0), Apricot(2)

    // Navigate to Apricot
    SelectableList::<String>::update(&mut state, SelectableListMessage::Down);

    // Select it
    let output = SelectableList::<String>::update(&mut state, SelectableListMessage::Select);
    assert_eq!(
        output,
        Some(SelectableListOutput::Selected("Apricot".to_string()))
    );
}

#[test]
fn test_filter_message_set_filter() {
    let mut state = SelectableListState::with_items(vec![
        "Alpha".to_string(),
        "Beta".to_string(),
        "Gamma".to_string(),
    ]);
    let output = SelectableList::<String>::update(
        &mut state,
        SelectableListMessage::SetFilter("eta".to_string()),
    );
    assert_eq!(state.filter_text(), "eta");
    assert_eq!(state.visible_count(), 1);
    assert_eq!(
        output,
        Some(SelectableListOutput::FilterChanged("eta".to_string()))
    );
}

#[test]
fn test_filter_message_clear_filter() {
    let mut state = SelectableListState::with_items(vec!["Alpha".to_string(), "Beta".to_string()]);
    state.set_filter_text("alpha");
    assert_eq!(state.visible_count(), 1);

    let output = SelectableList::<String>::update(&mut state, SelectableListMessage::ClearFilter);
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 2);
    assert_eq!(
        output,
        Some(SelectableListOutput::FilterChanged(String::new()))
    );
}

#[test]
fn test_filter_empty_string_shows_all() {
    let mut state =
        SelectableListState::with_items(vec!["Apple".to_string(), "Banana".to_string()]);
    state.set_filter_text("");
    assert_eq!(state.visible_count(), 2);
}

#[test]
fn test_filter_set_items_clears_filter() {
    let mut state =
        SelectableListState::with_items(vec!["Apple".to_string(), "Banana".to_string()]);
    state.set_filter_text("ap");
    assert_eq!(state.visible_count(), 1);

    state.set_items(vec!["X".to_string(), "Y".to_string(), "Z".to_string()]);
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 3);
}

#[test]
fn test_filter_first_last_navigation() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Apricot".to_string(),
        "Avocado".to_string(),
    ]);
    state.set_filter_text("a");
    // Filtered: Apple(0), Apricot(2), Avocado(3)

    let output = SelectableList::<String>::update(&mut state, SelectableListMessage::Last);
    assert_eq!(state.selected_item(), Some(&"Avocado".to_string()));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(3)));

    let output = SelectableList::<String>::update(&mut state, SelectableListMessage::First);
    assert_eq!(state.selected_item(), Some(&"Apple".to_string()));
    assert_eq!(output, Some(SelectableListOutput::SelectionChanged(0)));
}

#[test]
fn test_filter_select_by_original_index() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Apricot".to_string(),
    ]);
    state.set_filter_text("ap");
    // Filtered: Apple(0), Apricot(2)

    // Select by original index 2 (Apricot)
    state.select(Some(2));
    assert_eq!(state.selected_item(), Some(&"Apricot".to_string()));

    // Try to select filtered-out item (Banana at index 1) - should be ignored
    state.select(Some(1));
    assert_eq!(state.selected_item(), Some(&"Apricot".to_string()));
}

#[test]
fn test_filter_view() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Apricot".to_string(),
        "Cherry".to_string(),
    ]);
    state.focused = true;
    state.set_filter_text("ap");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            SelectableList::<String>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_filter_disabled_navigation() {
    let mut state = SelectableListState::with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Apricot".to_string(),
    ]);
    state.set_disabled(true);
    state.set_filter_text("ap");

    // Navigation should be blocked when disabled
    let output = SelectableList::<String>::update(&mut state, SelectableListMessage::Down);
    assert_eq!(output, None);
}

#[test]
fn test_filter_disabled_still_allows_filter_change() {
    let mut state =
        SelectableListState::with_items(vec!["Apple".to_string(), "Banana".to_string()]);
    state.set_disabled(true);

    // SetFilter should work even when disabled
    let output = SelectableList::<String>::update(
        &mut state,
        SelectableListMessage::SetFilter("ap".to_string()),
    );
    assert_eq!(
        output,
        Some(SelectableListOutput::FilterChanged("ap".to_string()))
    );
    assert_eq!(state.visible_count(), 1);
}
