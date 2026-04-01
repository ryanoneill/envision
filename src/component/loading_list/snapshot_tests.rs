use super::*;
use crate::component::test_utils;

#[derive(Clone, Debug, PartialEq)]
struct Task {
    id: u32,
    name: String,
}

fn make_tasks() -> Vec<Task> {
    vec![
        Task {
            id: 1,
            name: "Build project".to_string(),
        },
        Task {
            id: 2,
            name: "Run tests".to_string(),
        },
        Task {
            id: 3,
            name: "Deploy".to_string(),
        },
    ]
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state: LoadingListState<String> = LoadingListState::new();
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_items() {
    let tasks = make_tasks();
    let state = LoadingListState::with_items(tasks, |t| t.name.clone());
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_selected() {
    let tasks = make_tasks();
    let state = LoadingListState::with_items(tasks, |t| t.name.clone()).with_selected(1);
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_loading_item() {
    let tasks = make_tasks();
    let mut state = LoadingListState::with_items(tasks, |t| t.name.clone());
    state.set_loading(1);
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_error_item() {
    let tasks = make_tasks();
    let mut state = LoadingListState::with_items(tasks, |t| t.name.clone());
    state.set_error(2, "Connection failed");
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_mixed_states() {
    let tasks = make_tasks();
    let mut state = LoadingListState::with_items(tasks, |t| t.name.clone());
    state.set_loading(0);
    state.set_error(2, "Timeout");
    state.set_selected(Some(1));
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let tasks = make_tasks();
    let state = LoadingListState::with_items(tasks, |t| t.name.clone()).with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(50, 10);
    terminal
        .draw(|frame| {
            LoadingList::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
