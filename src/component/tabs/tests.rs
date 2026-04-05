use super::*;
use crate::input::{Event, KeyCode};

// State Tests

#[test]
fn test_new() {
    let state = TabsState::new(vec!["Tab1", "Tab2", "Tab3"]);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.len(), 3);
    assert!(!state.is_empty());
}

#[test]
fn test_new_empty() {
    let state: TabsState<&str> = TabsState::new(vec![]);
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.len(), 0);
    assert!(state.is_empty());
}

#[test]
fn test_default() {
    let state: TabsState<String> = TabsState::default();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_with_selected() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_item(), Some(&"B"));
}

#[test]
fn test_with_selected_clamps() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 10);
    assert_eq!(state.selected_index(), Some(2)); // Clamped to last valid index
}

#[test]
fn test_with_selected_empty() {
    let state: TabsState<&str> = TabsState::with_selected(vec![], 5);
    assert_eq!(state.selected_index(), None);
}

// Accessors

#[test]
fn test_selected_index() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 2);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_selected() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    assert_eq!(state.selected_item(), Some(&"B"));
}

#[test]
fn test_selected_empty() {
    let state: TabsState<&str> = TabsState::new(vec![]);
    assert_eq!(state.selected_item(), None);
}

#[test]
fn test_set_selected() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    state.set_selected(Some(2));
    assert_eq!(state.selected_index(), Some(2));

    // Test clamping
    state.set_selected(Some(100));
    assert_eq!(state.selected_index(), Some(2));
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
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Left);
    assert_eq!(output, Some(TabsOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_left_at_first() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Left);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_right() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Right);
    assert_eq!(output, Some(TabsOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_right_at_last() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Right);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_select() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Select(2));
    assert_eq!(output, Some(TabsOutput::SelectionChanged(2)));
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_select_same() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Select(0));
    assert_eq!(output, None); // Already selected
}

#[test]
fn test_select_clamps() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Select(100));
    assert_eq!(output, Some(TabsOutput::SelectionChanged(2))); // Clamped to last
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_first() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::First);
    assert_eq!(output, Some(TabsOutput::SelectionChanged(0)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_first_already_first() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::First);
    assert_eq!(output, None);
}

#[test]
fn test_last() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Last);
    assert_eq!(output, Some(TabsOutput::SelectionChanged(2)));
    assert_eq!(state.selected_index(), Some(2));
}

#[test]
fn test_last_already_last() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Last);
    assert_eq!(output, None);
}

// Confirm

#[test]
fn test_confirm() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Confirm);
    assert_eq!(output, Some(TabsOutput::Confirmed("B")));
}

#[test]
fn test_confirm_empty() {
    let mut state: TabsState<&str> = TabsState::new(vec![]);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Confirm);
    assert_eq!(output, None);
}

// Empty State

#[test]
fn test_empty_navigation() {
    let mut state: TabsState<&str> = TabsState::new(vec![]);

    assert_eq!(Tabs::<&str>::update(&mut state, TabsMessage::Right), None);
    assert_eq!(Tabs::<&str>::update(&mut state, TabsMessage::Left), None);
    assert_eq!(
        Tabs::<&str>::update(&mut state, TabsMessage::Select(0)),
        None
    );
    assert_eq!(Tabs::<&str>::update(&mut state, TabsMessage::First), None);
    assert_eq!(Tabs::<&str>::update(&mut state, TabsMessage::Last), None);
}

#[test]
fn test_empty_confirm() {
    let mut state: TabsState<&str> = TabsState::new(vec![]);
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Confirm);
    assert_eq!(output, None);
}

// View Tests

#[test]
fn test_view_renders() {
    let state = TabsState::new(vec!["Home", "Settings", "Help"]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Tabs::<&str>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = TabsState::new(vec!["Tab1", "Tab2"]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Tabs::<&str>::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}
#[test]
fn test_view_empty() {
    let state: TabsState<&str> = TabsState::new(vec![]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 5);

    terminal
        .draw(|frame| {
            Tabs::<&str>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// Integration

#[test]
fn test_init() {
    let state: TabsState<String> = Tabs::<String>::init();
    assert!(state.is_empty());
}

#[test]
fn test_full_workflow() {
    let mut state = TabsState::new(vec!["Home", "Settings", "Profile", "Help"]);

    // Start at first tab
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item(), Some(&"Home"));

    // Navigate right twice
    Tabs::<&str>::update(&mut state, TabsMessage::Right);
    Tabs::<&str>::update(&mut state, TabsMessage::Right);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(state.selected_item(), Some(&"Profile"));

    // Navigate left once
    Tabs::<&str>::update(&mut state, TabsMessage::Left);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_item(), Some(&"Settings"));

    // Jump to last
    Tabs::<&str>::update(&mut state, TabsMessage::Last);
    assert_eq!(state.selected_index(), Some(3));
    assert_eq!(state.selected_item(), Some(&"Help"));

    // Jump to first
    Tabs::<&str>::update(&mut state, TabsMessage::First);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_item(), Some(&"Home"));

    // Confirm selection
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Confirm);
    assert_eq!(output, Some(TabsOutput::Confirmed("Home")));
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

    let output = Tabs::<Page>::update(&mut state, TabsMessage::Right);
    assert_eq!(output, Some(TabsOutput::SelectionChanged(1)));

    let output = Tabs::<Page>::update(&mut state, TabsMessage::Confirm);
    assert_eq!(output, Some(TabsOutput::Confirmed(Page::Settings)));
}

#[test]
fn test_with_string_tabs() {
    let mut state = TabsState::new(vec![
        "Dashboard".to_string(),
        "Analytics".to_string(),
        "Reports".to_string(),
    ]);

    let output = Tabs::<String>::update(&mut state, TabsMessage::Select(1));
    assert_eq!(output, Some(TabsOutput::SelectionChanged(1)));
}

#[test]
fn test_single_tab() {
    let mut state = TabsState::new(vec!["Only"]);

    // Can't navigate anywhere
    assert_eq!(Tabs::<&str>::update(&mut state, TabsMessage::Left), None);
    assert_eq!(Tabs::<&str>::update(&mut state, TabsMessage::Right), None);

    // But can confirm
    let output = Tabs::<&str>::update(&mut state, TabsMessage::Confirm);
    assert_eq!(output, Some(TabsOutput::Confirmed("Only")));
}

#[test]
fn test_large_tabs_navigation() {
    let labels: Vec<String> = (0..50).map(|i| format!("Tab {}", i)).collect();
    let mut state = TabsState::new(labels);

    // Navigate to middle using Right
    for _ in 0..25 {
        Tabs::<String>::update(&mut state, TabsMessage::Right);
    }
    assert_eq!(state.selected_index(), Some(25));

    // First/Last
    Tabs::<String>::update(&mut state, TabsMessage::First);
    assert_eq!(state.selected_index(), Some(0));

    Tabs::<String>::update(&mut state, TabsMessage::Last);
    assert_eq!(state.selected_index(), Some(49));
}

#[test]
fn test_unicode_tab_labels() {
    let tabs = vec!["首页", "设置", "帮助"];
    let mut state = TabsState::new(tabs);

    Tabs::<&str>::update(&mut state, TabsMessage::Right);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_item(), Some(&"设置"));
}

// ========== handle_event Tests ==========

#[test]
fn test_handle_event_left_when_focused() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 1);

    let msg = Tabs::<&str>::handle_event(
        &state,
        &Event::key(KeyCode::Left),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TabsMessage::Left));
}

#[test]
fn test_handle_event_right_when_focused() {
    let state = TabsState::new(vec!["A", "B", "C"]);

    let msg = Tabs::<&str>::handle_event(
        &state,
        &Event::key(KeyCode::Right),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TabsMessage::Right));
}

#[test]
fn test_handle_event_first_when_focused() {
    let state = TabsState::new(vec!["A", "B", "C"]);

    let msg = Tabs::<&str>::handle_event(
        &state,
        &Event::key(KeyCode::Home),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TabsMessage::First));
}

#[test]
fn test_handle_event_last_when_focused() {
    let state = TabsState::new(vec!["A", "B", "C"]);

    let msg = Tabs::<&str>::handle_event(
        &state,
        &Event::key(KeyCode::End),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TabsMessage::Last));
}

#[test]
fn test_handle_event_confirm_when_focused() {
    let state = TabsState::new(vec!["A", "B", "C"]);

    let msg = Tabs::<&str>::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TabsMessage::Confirm));
}

#[test]
fn test_handle_event_vim_keys() {
    let state = TabsState::new(vec!["A", "B", "C"]);

    let msg_h =
        Tabs::<&str>::handle_event(&state, &Event::char('h'), &ViewContext::new().focused(true));
    assert_eq!(msg_h, Some(TabsMessage::Left));

    let msg_l =
        Tabs::<&str>::handle_event(&state, &Event::char('l'), &ViewContext::new().focused(true));
    assert_eq!(msg_l, Some(TabsMessage::Right));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = TabsState::new(vec!["A", "B", "C"]);
    // focused is false by default

    let msg =
        Tabs::<&str>::handle_event(&state, &Event::key(KeyCode::Right), &ViewContext::default());
    assert_eq!(msg, None);

    let msg =
        Tabs::<&str>::handle_event(&state, &Event::key(KeyCode::Enter), &ViewContext::default());
    assert_eq!(msg, None);

    let msg = Tabs::<&str>::handle_event(&state, &Event::char('l'), &ViewContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_ignored_when_disabled() {
    let state = TabsState::new(vec!["A", "B", "C"]);

    let msg = Tabs::<&str>::handle_event(
        &state,
        &Event::key(KeyCode::Right),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Tabs::<&str>::handle_event(
        &state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

// ========== dispatch_event Tests ==========

#[test]
fn test_dispatch_event() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);

    // Dispatch Right: should move selection from 0 to 1
    let output = Tabs::<&str>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Right),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(TabsOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    // Dispatch Enter: should confirm the current selection
    let output = Tabs::<&str>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Enter),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(TabsOutput::Confirmed("B")));
}

// ========== Instance Method Tests ==========

#[test]
fn test_instance_methods() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);

    // dispatch_event via static method
    let output = Tabs::<&str>::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Right),
        &ViewContext::new().focused(true),
    );
    assert_eq!(output, Some(TabsOutput::SelectionChanged(1)));
    assert_eq!(state.selected_index(), Some(1));

    // update via instance method
    let output = state.update(TabsMessage::Right);
    assert_eq!(output, Some(TabsOutput::SelectionChanged(2)));
    assert_eq!(state.selected_index(), Some(2));

    // handle_event via static method
    let msg = Tabs::<&str>::handle_event(
        &state,
        &Event::key(KeyCode::Left),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(TabsMessage::Left));
}
#[test]
fn test_selected_item() {
    let state = TabsState::with_selected(vec!["A", "B", "C"], 1);
    assert_eq!(state.selected_item(), Some(&"B"));
    assert_eq!(state.selected_item(), Some(&"B"));
}

#[test]
fn test_selected_item_empty() {
    let state: TabsState<&str> = TabsState::new(vec![]);
    assert_eq!(state.selected_item(), None);
}

// ========== set_tabs Tests ==========

#[test]
fn test_set_tabs_updates_tabs() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    state.set_tabs(vec!["X", "Y", "Z"]);
    assert_eq!(state.tabs(), &["X", "Y", "Z"]);
    assert_eq!(state.len(), 3);
}

#[test]
fn test_set_tabs_preserves_valid_selection() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    state.set_selected(Some(1));
    state.set_tabs(vec!["X", "Y", "Z"]);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_set_tabs_clamps_selection() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C", "D", "E"], 4);
    assert_eq!(state.selected_index(), Some(4));

    state.set_tabs(vec!["X", "Y"]);
    assert_eq!(state.selected_index(), Some(1)); // Clamped to last valid index
    assert_eq!(state.selected_item(), Some(&"Y"));
}

#[test]
fn test_set_tabs_empty_clears_selection() {
    let mut state = TabsState::new(vec!["A", "B", "C"]);
    assert_eq!(state.selected_index(), Some(0));

    state.set_tabs(Vec::<&str>::new());
    assert_eq!(state.selected_index(), None);
    assert!(state.is_empty());
}

#[test]
fn test_set_tabs_from_empty_to_non_empty() {
    let mut state = TabsState::<&str>::new(vec![]);
    assert_eq!(state.selected_index(), None);

    state.set_tabs(vec!["X", "Y"]);
    assert_eq!(state.tabs(), &["X", "Y"]);
    // Selection remains None since there was no prior selection
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_tabs_selection_at_boundary() {
    let mut state = TabsState::with_selected(vec!["A", "B", "C"], 2);
    assert_eq!(state.selected_index(), Some(2));

    // Set tabs to exactly 3 items - index 2 is still valid
    state.set_tabs(vec!["X", "Y", "Z"]);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(state.selected_item(), Some(&"Z"));
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = TabsState::new(vec!["Tab1".to_string(), "Tab2".to_string()]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 3);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Tabs::<String>::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::TabBar);
    assert_eq!(regions.len(), 1);
}
