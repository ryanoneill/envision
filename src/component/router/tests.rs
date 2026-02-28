use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum TestScreen {
    Home,
    Settings,
    Profile,
    About,
}

// ========================================
// NavigationMode Tests
// ========================================

#[test]
fn test_navigation_mode_default() {
    let mode = NavigationMode::default();
    assert_eq!(mode, NavigationMode::Push);
}

// ========================================
// RouterState Tests
// ========================================

#[test]
fn test_state_new() {
    let state = RouterState::new(TestScreen::Home);
    assert_eq!(state.current(), &TestScreen::Home);
    assert!(!state.can_go_back());
    assert_eq!(state.history_len(), 0);
    assert_eq!(state.max_history(), 0);
}

#[test]
fn test_state_with_max_history() {
    let state = RouterState::new(TestScreen::Home).with_max_history(5);
    assert_eq!(state.max_history(), 5);
}

#[test]
fn test_is_at() {
    let state = RouterState::new(TestScreen::Home);
    assert!(state.is_at(&TestScreen::Home));
    assert!(!state.is_at(&TestScreen::Settings));
}

#[test]
fn test_previous() {
    let mut state = RouterState::new(TestScreen::Home);
    assert!(state.previous().is_none());

    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
    assert_eq!(state.previous(), Some(&TestScreen::Home));
}

#[test]
fn test_history() {
    let mut state = RouterState::new(TestScreen::Home);
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Profile));

    let history = state.history();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0], TestScreen::Home);
    assert_eq!(history[1], TestScreen::Settings);
}

#[test]
fn test_clear_history() {
    let mut state = RouterState::new(TestScreen::Home);
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Profile));

    state.clear_history();
    assert!(!state.can_go_back());
    assert_eq!(state.current(), &TestScreen::Profile);
}

#[test]
fn test_set_max_history() {
    let mut state = RouterState::new(TestScreen::Home);
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Profile));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::About));

    assert_eq!(state.history_len(), 3);

    state.set_max_history(2);
    assert_eq!(state.history_len(), 2);
    // Oldest should be removed
    assert_eq!(state.history()[0], TestScreen::Settings);
}

#[test]
fn test_clone() {
    let mut state = RouterState::new(TestScreen::Home);
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));

    let cloned = state.clone();
    assert_eq!(cloned.current(), &TestScreen::Settings);
    assert_eq!(cloned.history_len(), 1);
}

// ========================================
// Navigation Tests
// ========================================

#[test]
fn test_navigate() {
    let mut state = RouterState::new(TestScreen::Home);

    let output = Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));

    assert!(matches!(
        output,
        Some(RouterOutput::ScreenChanged {
            from: TestScreen::Home,
            to: TestScreen::Settings
        })
    ));
    assert_eq!(state.current(), &TestScreen::Settings);
    assert!(state.can_go_back());
}

#[test]
fn test_navigate_same_screen() {
    let mut state = RouterState::new(TestScreen::Home);

    let output = Router::update(&mut state, RouterMessage::Navigate(TestScreen::Home));

    assert!(output.is_none());
    assert_eq!(state.history_len(), 0);
}

#[test]
fn test_navigate_with_push() {
    let mut state = RouterState::new(TestScreen::Home);

    Router::update(
        &mut state,
        RouterMessage::NavigateWith(TestScreen::Settings, NavigationMode::Push),
    );

    assert_eq!(state.current(), &TestScreen::Settings);
    assert!(state.can_go_back());
}

#[test]
fn test_navigate_with_replace() {
    let mut state = RouterState::new(TestScreen::Home);

    Router::update(
        &mut state,
        RouterMessage::NavigateWith(TestScreen::Settings, NavigationMode::Replace),
    );

    assert_eq!(state.current(), &TestScreen::Settings);
    assert!(!state.can_go_back());
}

#[test]
fn test_replace() {
    let mut state = RouterState::new(TestScreen::Home);
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));

    let output = Router::update(&mut state, RouterMessage::Replace(TestScreen::Profile));

    assert!(matches!(
        output,
        Some(RouterOutput::ScreenChanged {
            from: TestScreen::Settings,
            to: TestScreen::Profile
        })
    ));
    assert_eq!(state.current(), &TestScreen::Profile);
    // History should still have Home (Settings was replaced)
    assert_eq!(state.history_len(), 1);
    assert_eq!(state.previous(), Some(&TestScreen::Home));
}

#[test]
fn test_replace_same_screen() {
    let mut state = RouterState::new(TestScreen::Home);
    let output = Router::update(&mut state, RouterMessage::Replace(TestScreen::Home));
    assert!(output.is_none());
}

#[test]
fn test_back() {
    let mut state = RouterState::new(TestScreen::Home);
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Profile));

    let output = Router::update(&mut state, RouterMessage::Back);

    assert!(matches!(
        output,
        Some(RouterOutput::NavigatedBack {
            to: TestScreen::Settings
        })
    ));
    assert_eq!(state.current(), &TestScreen::Settings);
    assert!(state.can_go_back());
}

#[test]
fn test_back_no_history() {
    let mut state = RouterState::new(TestScreen::Home);

    let output = Router::update(&mut state, RouterMessage::Back);

    assert!(matches!(output, Some(RouterOutput::NoPreviousScreen)));
    assert_eq!(state.current(), &TestScreen::Home);
}

#[test]
fn test_back_to_start() {
    let mut state = RouterState::new(TestScreen::Home);
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));

    Router::update(&mut state, RouterMessage::Back);

    assert_eq!(state.current(), &TestScreen::Home);
    assert!(!state.can_go_back());
}

#[test]
fn test_clear_history_message() {
    let mut state = RouterState::new(TestScreen::Home);
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));

    let output = Router::update(&mut state, RouterMessage::ClearHistory);

    assert!(matches!(output, Some(RouterOutput::HistoryCleared)));
    assert!(!state.can_go_back());
}

#[test]
fn test_clear_history_empty() {
    let mut state = RouterState::new(TestScreen::Home);
    let output = Router::update(&mut state, RouterMessage::ClearHistory);
    assert!(output.is_none());
}

#[test]
fn test_reset() {
    let mut state = RouterState::new(TestScreen::Home);
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Profile));

    let output = Router::update(&mut state, RouterMessage::Reset(TestScreen::About));

    assert!(matches!(
        output,
        Some(RouterOutput::Reset(TestScreen::About))
    ));
    assert_eq!(state.current(), &TestScreen::About);
    assert!(!state.can_go_back());
}

// ========================================
// Max History Tests
// ========================================

#[test]
fn test_max_history_enforcement() {
    let mut state = RouterState::new(TestScreen::Home).with_max_history(2);

    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Profile));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::About));

    // Should only keep 2 entries
    assert_eq!(state.history_len(), 2);
    // Oldest (Home) should be removed
    assert_eq!(state.history()[0], TestScreen::Settings);
    assert_eq!(state.history()[1], TestScreen::Profile);
}

#[test]
fn test_max_history_zero() {
    let mut state = RouterState::new(TestScreen::Home);

    for _ in 0..100 {
        Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
        Router::update(&mut state, RouterMessage::Navigate(TestScreen::Profile));
    }

    // Should keep all entries (unlimited)
    assert_eq!(state.history_len(), 200);
}

// ========================================
// View Test
// ========================================

#[test]
fn test_view_is_noop() {
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    let state = RouterState::new(TestScreen::Home);
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| Router::view(&state, frame, frame.area(), &Theme::default()))
        .unwrap();

    // View should do nothing - output should be empty
    // (This is intentional - Router is state-only)
}

// ========================================
// Complex Navigation Scenarios
// ========================================

#[test]
fn test_navigation_round_trip() {
    let mut state = RouterState::new(TestScreen::Home);

    // Navigate forward
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Profile));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::About));

    assert_eq!(state.current(), &TestScreen::About);
    assert_eq!(state.history_len(), 3);

    // Navigate back to start
    Router::update(&mut state, RouterMessage::Back);
    assert_eq!(state.current(), &TestScreen::Profile);

    Router::update(&mut state, RouterMessage::Back);
    assert_eq!(state.current(), &TestScreen::Settings);

    Router::update(&mut state, RouterMessage::Back);
    assert_eq!(state.current(), &TestScreen::Home);

    Router::update(&mut state, RouterMessage::Back);
    assert_eq!(state.current(), &TestScreen::Home); // Still Home
}

#[test]
fn test_mixed_navigate_and_replace() {
    let mut state = RouterState::new(TestScreen::Home);

    Router::update(&mut state, RouterMessage::Navigate(TestScreen::Settings));
    Router::update(&mut state, RouterMessage::Replace(TestScreen::Profile));
    Router::update(&mut state, RouterMessage::Navigate(TestScreen::About));

    assert_eq!(state.current(), &TestScreen::About);
    assert_eq!(state.history_len(), 2);

    // Go back should skip Settings (it was replaced)
    Router::update(&mut state, RouterMessage::Back);
    assert_eq!(state.current(), &TestScreen::Profile);

    Router::update(&mut state, RouterMessage::Back);
    assert_eq!(state.current(), &TestScreen::Home);
}
