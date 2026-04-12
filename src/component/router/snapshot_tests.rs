use super::*;
use crate::component::RenderContext;
use crate::component::test_utils;

/// Router is a state-only component with no visual output.
/// These tests verify that `Router::view()` is a no-op and
/// does not panic on various state configurations.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Screen {
    #[default]
    Home,
    Settings,
    Profile,
}

// =============================================================================
// Snapshot tests (Router view is a no-op, so these verify no-panic rendering)
// =============================================================================

#[test]
fn test_snapshot_initial_screen() {
    let state = RouterState::new(Screen::Home);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Router::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    // Router renders nothing -- the output should be blank
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_after_navigation() {
    let mut state = RouterState::new(Screen::Home);
    Router::update(&mut state, RouterMessage::Navigate(Screen::Settings));
    Router::update(&mut state, RouterMessage::Navigate(Screen::Profile));
    assert_eq!(state.current(), &Screen::Profile);
    assert_eq!(state.history_len(), 2);

    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Router::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    // Router renders nothing -- the output should be blank
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_after_back() {
    let mut state = RouterState::new(Screen::Home);
    Router::update(&mut state, RouterMessage::Navigate(Screen::Settings));
    Router::update(&mut state, RouterMessage::Back);
    assert_eq!(state.current(), &Screen::Home);

    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Router::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
