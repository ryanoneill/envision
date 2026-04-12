use super::*;
use crate::theme::Theme;
use ratatui::prelude::Rect;

// ========================================
// View Tests (moved from component.rs)
// ========================================

#[test]
fn test_view_empty() {
    let state: LoadingListState<String> = LoadingListState::new();
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
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
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_title() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_title("My Items");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
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
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_zero_size_area() {
    let items = make_items();
    let state = LoadingListState::with_items(items, |i| i.name.clone());
    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    // Test with zero width
    terminal
        .draw(|frame| {
            LoadingList::view(
                &state,
                &mut RenderContext::new(frame, Rect::new(0, 0, 0, 10), &theme),
            );
        })
        .unwrap();

    // Test with zero height
    terminal
        .draw(|frame| {
            LoadingList::view(
                &state,
                &mut RenderContext::new(frame, Rect::new(0, 0, 60, 0), &Theme::default()),
            );
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
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
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
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========================================
// Additional View Tests
// ========================================

#[test]
fn test_view_focused() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(0));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            LoadingList::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            )
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_loading_item() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_loading(1);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_mixed_states() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_loading(0);
    state.set_error(2, "Connection refused");
    state.set_selected(Some(1));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_single_item() {
    let items = vec![TestItem {
        id: 1,
        name: "Only Item".to_string(),
    }];
    let state = LoadingListState::with_items(items, |i| i.name.clone()).with_selected(0);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_disabled() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone());
    state.set_selected(Some(1));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            LoadingList::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            )
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_title_and_selection() {
    let items = make_items();
    let mut state = LoadingListState::with_items(items, |i| i.name.clone()).with_title("Tasks");
    state.set_selected(Some(2));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(60, 10);

    terminal
        .draw(|frame| {
            LoadingList::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}
