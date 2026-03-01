use super::*;
use crate::input::{Event, KeyCode};

#[test]
fn test_menu_item_new() {
    let item = MenuItem::new("File");
    assert_eq!(item.label(), "File");
    assert!(item.is_enabled());
}

#[test]
fn test_menu_item_disabled() {
    let item = MenuItem::disabled("Save");
    assert_eq!(item.label(), "Save");
    assert!(!item.is_enabled());
}

#[test]
fn test_menu_item_set_enabled() {
    let mut item = MenuItem::new("Edit");
    item.set_enabled(false);
    assert!(!item.is_enabled());

    item.set_enabled(true);
    assert!(item.is_enabled());
}

#[test]
fn test_new() {
    let state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
    assert_eq!(state.items().len(), 2);
    assert_eq!(state.selected_index(), Some(0));
    assert!(!Menu::is_focused(&state));
}

#[test]
fn test_new_empty() {
    let state = MenuState::new(vec![]);
    assert_eq!(state.items().len(), 0);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_default() {
    let state = MenuState::default();
    assert_eq!(state.items().len(), 0);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_items() {
    let mut state = MenuState::new(vec![MenuItem::new("A")]);
    state.set_items(vec![MenuItem::new("X"), MenuItem::new("Y")]);
    assert_eq!(state.items().len(), 2);
    assert_eq!(state.items()[0].label(), "X");
}

#[test]
fn test_set_items_resets_invalid_selection() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);
    state.set_selected_index(2);

    state.set_items(vec![MenuItem::new("X")]);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_set_items_to_empty() {
    let mut state = MenuState::new(vec![MenuItem::new("A")]);
    state.set_items(vec![]);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_items_preserves_valid_selection() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);
    state.set_selected_index(1);
    state.set_items(vec![
        MenuItem::new("X"),
        MenuItem::new("Y"),
        MenuItem::new("Z"),
    ]);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_add_item() {
    let mut state = MenuState::new(vec![MenuItem::new("File")]);
    state.add_item(MenuItem::new("Edit"));
    assert_eq!(state.items().len(), 2);
}

#[test]
fn test_add_item_to_empty() {
    let mut state = MenuState::new(vec![]);
    assert_eq!(state.selected_index(), None);

    state.add_item(MenuItem::new("File"));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_remove_item() {
    let mut state = MenuState::new(vec![
        MenuItem::new("File"),
        MenuItem::new("Edit"),
        MenuItem::new("View"),
    ]);
    state.remove_item(1);
    assert_eq!(state.items().len(), 2);
    assert_eq!(state.items()[0].label(), "File");
    assert_eq!(state.items()[1].label(), "View");
}

#[test]
fn test_remove_item_adjusts_selection() {
    let mut state = MenuState::new(vec![
        MenuItem::new("File"),
        MenuItem::new("Edit"),
        MenuItem::new("View"),
    ]);
    state.set_selected_index(2);

    // Remove last item, selection should clamp
    state.remove_item(2);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_remove_item_to_empty() {
    let mut state = MenuState::new(vec![MenuItem::new("File")]);
    state.remove_item(0);
    assert!(state.items().is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_remove_item_out_of_bounds() {
    let mut state = MenuState::new(vec![MenuItem::new("File")]);
    state.remove_item(5);
    assert_eq!(state.items().len(), 1); // Unchanged
}

#[test]
fn test_selected_index() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);

    state.set_selected_index(1);
    assert_eq!(state.selected_index(), Some(1));

    state.set_selected_index(2);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_selected_index_clamps() {
    let mut state = MenuState::new(vec![MenuItem::new("A"), MenuItem::new("B")]);

    state.set_selected_index(10);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_selected_item() {
    let state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);

    let item = state.selected_item().unwrap();
    assert_eq!(item.label(), "File");
}

#[test]
fn test_selected_item_empty() {
    let state = MenuState::new(vec![]);
    assert_eq!(state.selected_item(), None);
}

#[test]
fn test_select_next() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);

    let output = Menu::update(&mut state, MenuMessage::Right);
    assert_eq!(output, Some(MenuOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    let output = Menu::update(&mut state, MenuMessage::Right);
    assert_eq!(output, Some(MenuOutput::SelectionChanged(2)));
    assert_eq!(state.selected_index(), Some(2));

    // Wrap around
    let output = Menu::update(&mut state, MenuMessage::Right);
    assert_eq!(output, Some(MenuOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_select_previous() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);

    // Wrap around from start
    let output = Menu::update(&mut state, MenuMessage::Left);
    assert_eq!(output, Some(MenuOutput::SelectionChanged(2)));
    assert_eq!(state.selected_index(), Some(2));

    let output = Menu::update(&mut state, MenuMessage::Left);
    assert_eq!(output, Some(MenuOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    let output = Menu::update(&mut state, MenuMessage::Left);
    assert_eq!(output, Some(MenuOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_select_item() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);

    let output = Menu::update(&mut state, MenuMessage::SelectIndex(2));
    assert_eq!(output, Some(MenuOutput::SelectionChanged(2)));
    assert_eq!(state.selected_index(), Some(2));

    let output = Menu::update(&mut state, MenuMessage::SelectIndex(0));
    assert_eq!(output, Some(MenuOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_select_item_same() {
    let mut state = MenuState::new(vec![MenuItem::new("A"), MenuItem::new("B")]);

    let output = Menu::update(&mut state, MenuMessage::SelectIndex(0));
    assert_eq!(output, None); // Already selected
}

#[test]
fn test_select_item_out_of_bounds() {
    let mut state = MenuState::new(vec![MenuItem::new("A"), MenuItem::new("B")]);

    let output = Menu::update(&mut state, MenuMessage::SelectIndex(10));
    assert_eq!(output, None);
    // Should remain at 0
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_activate_enabled() {
    let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);

    let output = Menu::update(&mut state, MenuMessage::Select);
    assert_eq!(output, Some(MenuOutput::Selected(0)));
}

#[test]
fn test_activate_disabled() {
    let mut state = MenuState::new(vec![MenuItem::disabled("File"), MenuItem::new("Edit")]);

    let output = Menu::update(&mut state, MenuMessage::Select);
    assert_eq!(output, None);
}

#[test]
fn test_activate_empty() {
    let mut state = MenuState::new(vec![]);

    let output = Menu::update(&mut state, MenuMessage::Select);
    assert_eq!(output, None);
}

#[test]
fn test_empty_menu_ignores_navigation() {
    let mut state = MenuState::new(vec![]);

    let output = Menu::update(&mut state, MenuMessage::Right);
    assert_eq!(output, None);

    let output = Menu::update(&mut state, MenuMessage::Left);
    assert_eq!(output, None);
}

#[test]
fn test_init() {
    let state = Menu::init();
    assert_eq!(state.items().len(), 0);
    assert!(!Menu::is_focused(&state));
}

#[test]
fn test_view() {
    let state = MenuState::new(vec![
        MenuItem::new("File"),
        MenuItem::new("Edit"),
        MenuItem::new("View"),
    ]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Menu::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
    Menu::focus(&mut state);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Menu::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_selected() {
    let mut state = MenuState::new(vec![
        MenuItem::new("File"),
        MenuItem::new("Edit"),
        MenuItem::new("View"),
    ]);
    Menu::focus(&mut state);
    state.set_selected_index(1);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Menu::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_empty() {
    let state = MenuState::new(vec![]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(80, 24);

    terminal
        .draw(|frame| {
            Menu::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_large_menu_navigation() {
    let items: Vec<MenuItem> = (0..100)
        .map(|i| MenuItem::new(format!("Item {}", i)))
        .collect();
    let mut state = MenuState::new(items);

    // Navigate to middle using Right
    for _ in 0..50 {
        Menu::update(&mut state, MenuMessage::Right);
    }
    assert_eq!(state.selected_index(), Some(50));

    // Navigate to last by wrapping: 50 more to reach 100, which wraps to 0
    for _ in 0..50 {
        Menu::update(&mut state, MenuMessage::Right);
    }
    assert_eq!(state.selected_index(), Some(0));

    // Left from 0 wraps to last
    Menu::update(&mut state, MenuMessage::Left);
    assert_eq!(state.selected_index(), Some(99));
}

#[test]
fn test_unicode_labels() {
    let items = vec![
        MenuItem::new("日本語メニュー"), // Japanese
        MenuItem::new("Кириллица"),      // Cyrillic
        MenuItem::new("العربية"),        // Arabic
    ];
    let mut state = MenuState::new(items);

    // Navigation works with unicode labels
    Menu::update(&mut state, MenuMessage::Right);
    assert_eq!(state.selected_index(), Some(1));

    Menu::update(&mut state, MenuMessage::Right);
    assert_eq!(state.selected_index(), Some(2));
}

// ========== handle_event Tests ==========

#[test]
fn test_handle_event_left_when_focused() {
    let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
    Menu::focus(&mut state);

    let msg = Menu::handle_event(&state, &Event::key(KeyCode::Left));
    assert_eq!(msg, Some(MenuMessage::Left));
}

#[test]
fn test_handle_event_right_when_focused() {
    let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
    Menu::focus(&mut state);

    let msg = Menu::handle_event(&state, &Event::key(KeyCode::Right));
    assert_eq!(msg, Some(MenuMessage::Right));
}

#[test]
fn test_handle_event_select_when_focused() {
    let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
    Menu::focus(&mut state);

    let msg = Menu::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(MenuMessage::Select));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
    // Not focused by default

    let msg = Menu::handle_event(&state, &Event::key(KeyCode::Right));
    assert_eq!(msg, None);

    let msg = Menu::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);

    let msg = Menu::handle_event(&state, &Event::key(KeyCode::Left));
    assert_eq!(msg, None);
}

// ========== dispatch_event Tests ==========

#[test]
fn test_dispatch_event() {
    let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
    Menu::focus(&mut state);

    // Dispatch Right: should move selection from 0 to 1
    let output = Menu::dispatch_event(&mut state, &Event::key(KeyCode::Right));
    assert_eq!(output, Some(MenuOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    // Dispatch Enter: should select the current item
    let output = Menu::dispatch_event(&mut state, &Event::key(KeyCode::Enter));
    assert_eq!(output, Some(MenuOutput::Selected(1)));
}

// ========== Instance Method Tests ==========

#[test]
fn test_instance_methods() {
    let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);

    // is_focused / set_focused
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());

    // dispatch_event via instance method
    state.set_focused(true);
    let output = state.dispatch_event(&Event::key(KeyCode::Right));
    assert_eq!(output, Some(MenuOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    // update via instance method
    let output = state.update(MenuMessage::Left);
    assert_eq!(output, Some(MenuOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));

    // handle_event via instance method
    let msg = state.handle_event(&Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(MenuMessage::Select));
}
