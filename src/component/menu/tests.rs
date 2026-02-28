use super::*;

#[test]
fn test_menu_item_new() {
    let item = MenuItem::new("File");
    assert_eq!(item.label, "File");
    assert!(item.enabled);
}

#[test]
fn test_menu_item_disabled() {
    let item = MenuItem::disabled("Save");
    assert_eq!(item.label, "Save");
    assert!(!item.enabled);
}

#[test]
fn test_menu_item_set_enabled() {
    let mut item = MenuItem::new("Edit");
    item.set_enabled(false);
    assert!(!item.enabled);

    item.set_enabled(true);
    assert!(item.enabled);
}

#[test]
fn test_new() {
    let state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
    assert_eq!(state.items().len(), 2);
    assert_eq!(state.selected_index(), 0);
    assert!(!Menu::is_focused(&state));
}

#[test]
fn test_default() {
    let state = MenuState::default();
    assert_eq!(state.items().len(), 0);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_set_items() {
    let mut state = MenuState::new(vec![MenuItem::new("A")]);
    state.set_items(vec![MenuItem::new("X"), MenuItem::new("Y")]);
    assert_eq!(state.items().len(), 2);
    assert_eq!(state.items()[0].label, "X");
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
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_add_item() {
    let mut state = MenuState::new(vec![MenuItem::new("File")]);
    state.add_item(MenuItem::new("Edit"));
    assert_eq!(state.items().len(), 2);
}

#[test]
fn test_selected_index() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);

    state.set_selected_index(1);
    assert_eq!(state.selected_index(), 1);

    state.set_selected_index(2);
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_selected_index_clamps() {
    let mut state = MenuState::new(vec![MenuItem::new("A"), MenuItem::new("B")]);

    state.set_selected_index(10);
    assert_eq!(state.selected_index(), 1);
}

#[test]
fn test_selected_item() {
    let state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);

    let item = state.selected_item().unwrap();
    assert_eq!(item.label, "File");
}

#[test]
fn test_select_next() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);

    Menu::update(&mut state, MenuMessage::SelectNext);
    assert_eq!(state.selected_index(), 1);

    Menu::update(&mut state, MenuMessage::SelectNext);
    assert_eq!(state.selected_index(), 2);

    // Wrap around
    Menu::update(&mut state, MenuMessage::SelectNext);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_previous() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);

    // Wrap around from start
    Menu::update(&mut state, MenuMessage::SelectPrevious);
    assert_eq!(state.selected_index(), 2);

    Menu::update(&mut state, MenuMessage::SelectPrevious);
    assert_eq!(state.selected_index(), 1);

    Menu::update(&mut state, MenuMessage::SelectPrevious);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_item() {
    let mut state = MenuState::new(vec![
        MenuItem::new("A"),
        MenuItem::new("B"),
        MenuItem::new("C"),
    ]);

    Menu::update(&mut state, MenuMessage::SelectItem(2));
    assert_eq!(state.selected_index(), 2);

    Menu::update(&mut state, MenuMessage::SelectItem(0));
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_item_out_of_bounds() {
    let mut state = MenuState::new(vec![MenuItem::new("A"), MenuItem::new("B")]);

    Menu::update(&mut state, MenuMessage::SelectItem(10));
    // Should remain at 0
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_activate_enabled() {
    let mut state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);

    let output = Menu::update(&mut state, MenuMessage::Activate);
    assert_eq!(output, Some(MenuOutput::ItemActivated(0)));
}

#[test]
fn test_activate_disabled() {
    let mut state = MenuState::new(vec![MenuItem::disabled("File"), MenuItem::new("Edit")]);

    let output = Menu::update(&mut state, MenuMessage::Activate);
    assert_eq!(output, None);
}

#[test]
fn test_activate_empty() {
    let mut state = MenuState::new(vec![]);

    let output = Menu::update(&mut state, MenuMessage::Activate);
    assert_eq!(output, None);
}

#[test]
fn test_empty_menu_ignores_navigation() {
    let mut state = MenuState::new(vec![]);

    let output = Menu::update(&mut state, MenuMessage::SelectNext);
    assert_eq!(output, None);

    let output = Menu::update(&mut state, MenuMessage::SelectPrevious);
    assert_eq!(output, None);
}

#[test]
fn test_focusable() {
    let mut state = MenuState::new(vec![MenuItem::new("Test")]);

    assert!(!Menu::is_focused(&state));

    Menu::focus(&mut state);
    assert!(Menu::is_focused(&state));

    Menu::blur(&mut state);
    assert!(!Menu::is_focused(&state));
}

#[test]
fn test_init() {
    let state = Menu::init();
    assert_eq!(state.items().len(), 0);
    assert!(!Menu::is_focused(&state));
}

#[test]
fn test_clone() {
    let state = MenuState::new(vec![MenuItem::new("File"), MenuItem::new("Edit")]);
    let cloned = state.clone();
    assert_eq!(cloned.items().len(), 2);
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

    let output = terminal.backend().to_string();
    assert!(output.contains("File"));
    assert!(output.contains("Edit"));
    assert!(output.contains("View"));
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

    let output = terminal.backend().to_string();
    // Should have brackets around selected item when focused
    assert!(output.contains("[File]"));
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

    let output = terminal.backend().to_string();
    assert!(output.contains("[Edit]"));
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

    // Should not panic with empty menu
    let output = terminal.backend().to_string();
    assert!(!output.is_empty());
}
