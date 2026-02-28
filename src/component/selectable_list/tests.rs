use super::*;

#[test]
fn test_init_empty() {
    let state = SelectableList::<String>::init();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.selected_item(), None);
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

    let output = SelectableList::<&str>::update(&mut state, ListMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(ListOutput::SelectionChanged(1)));

    let output = SelectableList::<&str>::update(&mut state, ListMessage::Down);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(output, Some(ListOutput::SelectionChanged(2)));

    // At the end, should stay at last item
    let output = SelectableList::<&str>::update(&mut state, ListMessage::Down);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(output, None);
}

#[test]
fn test_navigate_up() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    state.select(Some(2));

    let output = SelectableList::<&str>::update(&mut state, ListMessage::Up);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(ListOutput::SelectionChanged(1)));

    let output = SelectableList::<&str>::update(&mut state, ListMessage::Up);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(ListOutput::SelectionChanged(0)));

    // At the beginning, should stay at first item
    let output = SelectableList::<&str>::update(&mut state, ListMessage::Up);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, None);
}

#[test]
fn test_navigate_first_last() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e"]);
    state.select(Some(2));

    let output = SelectableList::<&str>::update(&mut state, ListMessage::Last);
    assert_eq!(state.selected_index(), Some(4));
    assert_eq!(output, Some(ListOutput::SelectionChanged(4)));

    let output = SelectableList::<&str>::update(&mut state, ListMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(ListOutput::SelectionChanged(0)));
}

#[test]
fn test_page_navigation() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e", "f", "g"]);

    let output = SelectableList::<&str>::update(&mut state, ListMessage::PageDown(3));
    assert_eq!(state.selected_index(), Some(3));
    assert_eq!(output, Some(ListOutput::SelectionChanged(3)));

    let output = SelectableList::<&str>::update(&mut state, ListMessage::PageDown(10));
    assert_eq!(state.selected_index(), Some(6)); // Clamped to last
    assert_eq!(output, Some(ListOutput::SelectionChanged(6)));

    let output = SelectableList::<&str>::update(&mut state, ListMessage::PageUp(4));
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(output, Some(ListOutput::SelectionChanged(2)));
}

#[test]
fn test_select() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    state.select(Some(1));

    let output = SelectableList::<&str>::update(&mut state, ListMessage::Select);
    assert_eq!(output, Some(ListOutput::Selected("b")));
}

#[test]
fn test_empty_list_navigation() {
    let mut state = SelectableList::<String>::init();

    assert_eq!(
        SelectableList::<String>::update(&mut state, ListMessage::Down),
        None
    );
    assert_eq!(
        SelectableList::<String>::update(&mut state, ListMessage::Up),
        None
    );
    assert_eq!(
        SelectableList::<String>::update(&mut state, ListMessage::Select),
        None
    );
}

#[test]
fn test_focusable() {
    let mut state = SelectableList::<String>::init();

    assert!(!SelectableList::<String>::is_focused(&state));

    SelectableList::<String>::set_focused(&mut state, true);
    assert!(SelectableList::<String>::is_focused(&state));

    SelectableList::<String>::blur(&mut state);
    assert!(!SelectableList::<String>::is_focused(&state));
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
fn test_list_state_mut() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);

    // Access and modify the internal list state
    let list_state = state.list_state_mut();
    list_state.select(Some(2));

    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_first_when_already_at_first() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    // Already at first item
    assert_eq!(state.selected_index(), Some(0));

    // First should return None when already at first
    let output = SelectableList::<&str>::update(&mut state, ListMessage::First);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_last_when_already_at_last() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c"]);
    state.select(Some(2));
    assert_eq!(state.selected_index(), Some(2));

    // Last should return None when already at last
    let output = SelectableList::<&str>::update(&mut state, ListMessage::Last);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_page_up_when_at_first() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e"]);
    assert_eq!(state.selected_index(), Some(0));

    // PageUp at first should return None
    let output = SelectableList::<&str>::update(&mut state, ListMessage::PageUp(3));
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_page_down_when_at_last() {
    let mut state = SelectableListState::with_items(vec!["a", "b", "c", "d", "e"]);
    state.select(Some(4));
    assert_eq!(state.selected_index(), Some(4));

    // PageDown at last should return None
    let output = SelectableList::<&str>::update(&mut state, ListMessage::PageDown(3));
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(4));
}

#[test]
fn test_view() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let mut state = SelectableListState::with_items(vec!["Item 1", "Item 2", "Item 3"]);
    state.focused = true;
    let theme = Theme::default();

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            SelectableList::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Item 1"));
    assert!(output.contains("Item 2"));
    assert!(output.contains("Item 3"));
}

#[test]
fn test_view_unfocused() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let mut state = SelectableListState::with_items(vec!["A", "B", "C"]);
    state.focused = false; // Explicitly unfocused
    let theme = Theme::default();

    let backend = CaptureBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            SelectableList::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("A"));
    assert!(output.contains("B"));
    assert!(output.contains("C"));
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
fn test_state_debug() {
    let state = SelectableListState::with_items(vec!["a", "b"]);
    let debug = format!("{:?}", state);
    assert!(debug.contains("SelectableListState"));
}

#[test]
fn test_list_message_eq() {
    assert_eq!(ListMessage::Up, ListMessage::Up);
    assert_eq!(ListMessage::Down, ListMessage::Down);
    assert_eq!(ListMessage::First, ListMessage::First);
    assert_eq!(ListMessage::Last, ListMessage::Last);
    assert_eq!(ListMessage::PageUp(5), ListMessage::PageUp(5));
    assert_ne!(ListMessage::PageUp(5), ListMessage::PageUp(10));
    assert_eq!(ListMessage::PageDown(3), ListMessage::PageDown(3));
    assert_eq!(ListMessage::Select, ListMessage::Select);
}

#[test]
fn test_list_message_debug() {
    let debug = format!("{:?}", ListMessage::Up);
    assert_eq!(debug, "Up");
}

#[test]
fn test_list_output_eq() {
    let out1: ListOutput<&str> = ListOutput::Selected("a");
    let out2: ListOutput<&str> = ListOutput::Selected("a");
    assert_eq!(out1, out2);

    let out3: ListOutput<i32> = ListOutput::SelectionChanged(5);
    let out4: ListOutput<i32> = ListOutput::SelectionChanged(5);
    assert_eq!(out3, out4);
}

#[test]
fn test_list_output_debug() {
    let out: ListOutput<&str> = ListOutput::Selected("x");
    let debug = format!("{:?}", out);
    assert!(debug.contains("Selected"));
}
