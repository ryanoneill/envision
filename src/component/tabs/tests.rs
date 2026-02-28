use super::*;

// State Tests

#[test]
fn test_new() {
    let state = TabsState::new(vec!["Tab1", "Tab2", "Tab3"]);
    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.len(), 3);
    assert!(!state.is_empty());
}

#[test]
fn test_new_empty() {
    let state: TabsState<&str> = TabsState::new(vec![]);
    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.len(), 0);
    assert!(state.is_empty());
}

#[test]
fn test_default() {
    let state: TabsState<String> = TabsState::default();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_with_selected() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    assert_eq!(state.selected_index(), 1);
    assert_eq!(state.selected(), Some(&"B"));
}

#[test]
fn test_with_selected_clamps() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 10);
    assert_eq!(state.selected_index(), 2); // Clamped to last valid index
}

#[test]
fn test_with_selected_empty() {
    let state: TabsState<&str> = TabsState::with_selected(vec![], 5);
    assert_eq!(state.selected_index(), 0);
}

// Accessors

#[test]
fn test_selected_index() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 2);
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_selected() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    assert_eq!(state.selected(), Some(&"B"));
}

#[test]
fn test_selected_empty() {
    let state: TabsState<&str> = TabsState::new(vec![]);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_set_selected() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    state.set_selected(2);
    assert_eq!(state.selected_index(), 2);

    // Test clamping
    state.set_selected(100);
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_tabs() {
    let state = TabsState::new(vec!["A", "B", "C"]);
    assert_eq!(state.tabs(), &["A", "B", "C"]);
}

#[test]
fn test_len() {
    let state = TabsState::new(vec!["A", "B", "C"]);
    assert_eq!(state.len(), 3);
}

#[test]
fn test_is_empty() {
    let empty: TabsState<&str> = TabsState::new(vec![]);
    assert!(empty.is_empty());

    let not_empty = TabsState::new(vec!["A"]);
    assert!(!not_empty.is_empty());
}

// Navigation

#[test]
fn test_left() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Left);
    assert_eq!(output, Some(TabOutput::Selected("A")));
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_left_at_first() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Left);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_right() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Right);
    assert_eq!(output, Some(TabOutput::Selected("B")));
    assert_eq!(state.selected_index(), 1);
}

#[test]
fn test_right_at_last() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Right);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_select() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Select(2));
    assert_eq!(output, Some(TabOutput::Selected("C")));
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_select_same() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Select(0));
    assert_eq!(output, None); // Already selected
}

#[test]
fn test_select_clamps() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Select(100));
    assert_eq!(output, Some(TabOutput::Selected("C"))); // Clamped to last
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_first() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
    let output = Tabs::<&str>::update(&mut state, TabMessage::First);
    assert_eq!(output, Some(TabOutput::Selected("A")));
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_first_already_first() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabMessage::First);
    assert_eq!(output, None);
}

#[test]
fn test_last() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Last);
    assert_eq!(output, Some(TabOutput::Selected("C")));
    assert_eq!(state.selected_index(), 2);
}

#[test]
fn test_last_already_last() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Last);
    assert_eq!(output, None);
}

// Confirm

#[test]
fn test_confirm() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
    assert_eq!(output, Some(TabOutput::Confirmed("B")));
}

#[test]
fn test_confirm_empty() {
    let mut state: TabsState<&str> = TabsState::new(vec![]);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
    assert_eq!(output, None);
}

// Disabled State

#[test]
fn test_disabled() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    state.set_disabled(true);

    // All messages should be ignored
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Right), None);
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Left), None);
    assert_eq!(
        Tabs::<&str>::update(&mut state, TabMessage::Select(2)),
        None
    );
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::First), None);
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Last), None);
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Confirm), None);

    // State should not have changed
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_disabled_accessors() {
    let mut state = TabsState::new(vec!["A", "B"]);
    assert!(!state.is_disabled());

    state.set_disabled(true);
    assert!(state.is_disabled());

    state.set_disabled(false);
    assert!(!state.is_disabled());
}

// Empty State

#[test]
fn test_empty_navigation() {
    let mut state: TabsState<&str> = TabsState::new(vec![]);

    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Right), None);
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Left), None);
    assert_eq!(
        Tabs::<&str>::update(&mut state, TabMessage::Select(0)),
        None
    );
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::First), None);
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Last), None);
}

#[test]
fn test_empty_confirm() {
    let mut state: TabsState<&str> = TabsState::new(vec![]);
    let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
    assert_eq!(output, None);
}

// Focus

#[test]
fn test_focusable() {
    let mut state = TabsState::new(vec!["A", "B"]);
    assert!(!Tabs::<&str>::is_focused(&state));

    Tabs::<&str>::set_focused(&mut state, true);
    assert!(Tabs::<&str>::is_focused(&state));

    Tabs::<&str>::blur(&mut state);
    assert!(!Tabs::<&str>::is_focused(&state));

    Tabs::<&str>::focus(&mut state);
    assert!(Tabs::<&str>::is_focused(&state));
}

// View Tests

#[test]
fn test_view_renders() {
    let state = TabsState::new(vec!["Home", "Settings", "Help"]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Tabs::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Home"));
    assert!(output.contains("Settings"));
    assert!(output.contains("Help"));
}

#[test]
fn test_view_focused() {
    let mut state = TabsState::new(vec!["Tab1", "Tab2"]);
    state.focused = true;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Tabs::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Tab1"));
    assert!(output.contains("Tab2"));
}

#[test]
fn test_view_disabled() {
    let mut state = TabsState::new(vec!["Tab1", "Tab2"]);
    state.disabled = true;

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Tabs::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    let output = terminal.backend().to_string();
    assert!(output.contains("Tab1"));
}

#[test]
fn test_view_empty() {
    let state: TabsState<&str> = TabsState::new(vec![]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Tabs::<&str>::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    // Should render without panicking
    let _output = terminal.backend().to_string();
}

// Integration

#[test]
fn test_clone() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    state.focused = true;
    state.disabled = true;

    let cloned = state.clone();
    assert_eq!(cloned.selected_index(), 1);
    assert!(cloned.focused);
    assert!(cloned.disabled);
    assert_eq!(cloned.tabs(), &["A", "B", "C"]);
}

#[test]
fn test_init() {
    let state: TabsState<String> = Tabs::<String>::init();
    assert!(state.is_empty());
    assert!(!state.focused);
    assert!(!state.disabled);
}

#[test]
fn test_full_workflow() {
    let mut state = TabsState::new(vec!["Home", "Settings", "Profile", "Help"]);
    Tabs::<&str>::set_focused(&mut state, true);

    // Start at first tab
    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.selected(), Some(&"Home"));

    // Navigate right twice
    Tabs::<&str>::update(&mut state, TabMessage::Right);
    Tabs::<&str>::update(&mut state, TabMessage::Right);
    assert_eq!(state.selected_index(), 2);
    assert_eq!(state.selected(), Some(&"Profile"));

    // Navigate left once
    Tabs::<&str>::update(&mut state, TabMessage::Left);
    assert_eq!(state.selected_index(), 1);
    assert_eq!(state.selected(), Some(&"Settings"));

    // Jump to last
    Tabs::<&str>::update(&mut state, TabMessage::Last);
    assert_eq!(state.selected_index(), 3);
    assert_eq!(state.selected(), Some(&"Help"));

    // Jump to first
    Tabs::<&str>::update(&mut state, TabMessage::First);
    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.selected(), Some(&"Home"));

    // Confirm selection
    let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
    assert_eq!(output, Some(TabOutput::Confirmed("Home")));
}

#[test]
fn test_with_enum_tabs() {
    #[derive(Clone, Debug, PartialEq)]
    enum Page {
        Home,
        Settings,
        Help,
    }

    impl std::fmt::Display for Page {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Page::Home => write!(f, "Home"),
                Page::Settings => write!(f, "Settings"),
                Page::Help => write!(f, "Help"),
            }
        }
    }

    let mut state = TabsState::new(vec![Page::Home, Page::Settings, Page::Help]);

    let output = Tabs::<Page>::update(&mut state, TabMessage::Right);
    assert_eq!(output, Some(TabOutput::Selected(Page::Settings)));

    let output = Tabs::<Page>::update(&mut state, TabMessage::Confirm);
    assert_eq!(output, Some(TabOutput::Confirmed(Page::Settings)));
}

#[test]
fn test_with_string_tabs() {
    let mut state = TabsState::new(vec![
        "Dashboard".to_string(),
        "Analytics".to_string(),
        "Reports".to_string(),
    ]);

    let output = Tabs::<String>::update(&mut state, TabMessage::Select(1));
    assert_eq!(output, Some(TabOutput::Selected("Analytics".to_string())));
}

#[test]
fn test_single_tab() {
    let mut state = TabsState::new(vec!["Only"]);

    // Can't navigate anywhere
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Left), None);
    assert_eq!(Tabs::<&str>::update(&mut state, TabMessage::Right), None);

    // But can confirm
    let output = Tabs::<&str>::update(&mut state, TabMessage::Confirm);
    assert_eq!(output, Some(TabOutput::Confirmed("Only")));
}
