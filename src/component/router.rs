//! A component for multi-screen navigation with history.
//!
//! `Router` provides type-safe navigation between screens with back navigation
//! support. Unlike most components, Router is state-only and doesn't implement
//! a view - the parent application renders based on the current screen.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Router, RouterState, RouterMessage, Component};
//!
//! #[derive(Clone, Debug, PartialEq, Eq)]
//! enum Screen {
//!     Home,
//!     Settings,
//!     Profile,
//! }
//!
//! // Create router starting at Home screen
//! let mut state = RouterState::new(Screen::Home);
//!
//! // Navigate to Settings
//! Router::update(&mut state, RouterMessage::Navigate(Screen::Settings));
//! assert_eq!(state.current(), &Screen::Settings);
//! assert!(state.can_go_back());
//!
//! // Go back to Home
//! Router::update(&mut state, RouterMessage::Back);
//! assert_eq!(state.current(), &Screen::Home);
//! ```
//!
//! # Usage Pattern
//!
//! ```rust,ignore
//! // In your app's view function:
//! fn view(state: &AppState, frame: &mut Frame) {
//!     match state.router.current() {
//!         Screen::Home => render_home(state, frame),
//!         Screen::Settings => render_settings(state, frame),
//!         Screen::Profile => render_profile(state, frame),
//!     }
//! }
//! ```

use ratatui::prelude::*;

use super::Component;

/// Navigation mode for screen transitions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationMode {
    /// Push the new screen onto the history stack.
    #[default]
    Push,
    /// Replace the current screen without adding to history.
    Replace,
}

/// Messages for the Router component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouterMessage<S: Clone + PartialEq> {
    /// Navigate to a new screen (pushes to history).
    Navigate(S),
    /// Navigate with a specific mode.
    NavigateWith(S, NavigationMode),
    /// Replace the current screen without adding to history.
    Replace(S),
    /// Go back to the previous screen.
    Back,
    /// Clear all navigation history.
    ClearHistory,
    /// Reset to a specific screen, clearing all history.
    Reset(S),
}

/// Output messages from the Router component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouterOutput<S: Clone + PartialEq> {
    /// Screen changed (from, to).
    ScreenChanged {
        /// The previous screen.
        from: S,
        /// The new current screen.
        to: S,
    },
    /// Successfully navigated back.
    NavigatedBack {
        /// The screen we navigated to.
        to: S,
    },
    /// Tried to go back but there's no history.
    NoPreviousScreen,
    /// Router was reset to a new screen.
    Reset(S),
    /// History was cleared.
    HistoryCleared,
}

/// State for the Router component.
///
/// The type parameter `S` is your screen enum. It must implement `Clone` and `PartialEq`.
///
/// # Example
///
/// ```rust
/// use envision::component::RouterState;
///
/// #[derive(Clone, Debug, PartialEq, Eq)]
/// enum Screen {
///     Home,
///     Settings,
/// }
///
/// let state = RouterState::new(Screen::Home);
/// assert_eq!(state.current(), &Screen::Home);
/// assert!(!state.can_go_back());
/// ```
#[derive(Clone, Debug)]
pub struct RouterState<S: Clone + PartialEq> {
    /// The current screen.
    current: S,
    /// Navigation history (most recent last).
    history: Vec<S>,
    /// Maximum history size (0 = unlimited).
    max_history: usize,
}

impl<S: Clone + PartialEq> RouterState<S> {
    /// Creates a new router state starting at the given screen.
    pub fn new(initial: S) -> Self {
        Self {
            current: initial,
            history: Vec::new(),
            max_history: 0,
        }
    }

    /// Sets the maximum history size.
    ///
    /// When the history exceeds this limit, the oldest entries are removed.
    /// Set to 0 for unlimited history (default).
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Returns the current screen.
    pub fn current(&self) -> &S {
        &self.current
    }

    /// Returns true if we can navigate back.
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    /// Returns the number of screens in history.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Returns the history stack (oldest first).
    pub fn history(&self) -> &[S] {
        &self.history
    }

    /// Returns the maximum history size (0 = unlimited).
    pub fn max_history(&self) -> usize {
        self.max_history
    }

    /// Sets the maximum history size.
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        self.enforce_history_limit();
    }

    /// Returns the previous screen if available.
    pub fn previous(&self) -> Option<&S> {
        self.history.last()
    }

    /// Checks if the current screen is the given screen.
    pub fn is_at(&self, screen: &S) -> bool {
        &self.current == screen
    }

    /// Clears all navigation history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Enforces the max history limit.
    fn enforce_history_limit(&mut self) {
        if self.max_history > 0 && self.history.len() > self.max_history {
            let excess = self.history.len() - self.max_history;
            self.history.drain(0..excess);
        }
    }
}

/// A component for multi-screen navigation with history.
///
/// Router manages screen navigation with a history stack for back navigation.
/// It's designed to be used with an enum representing your application's screens.
///
/// # Note
///
/// Router doesn't implement `view()` - it's a state-only component. Your
/// application should render based on `state.current()`.
///
/// # Example
///
/// ```rust
/// use envision::component::{Router, RouterState, RouterMessage, RouterOutput, Component};
///
/// #[derive(Clone, Debug, PartialEq, Eq)]
/// enum Screen {
///     Home,
///     Settings,
///     About,
/// }
///
/// let mut state = RouterState::new(Screen::Home);
///
/// // Navigate to Settings
/// let output = Router::update(&mut state, RouterMessage::Navigate(Screen::Settings));
/// assert!(matches!(output, Some(RouterOutput::ScreenChanged { .. })));
/// assert_eq!(state.current(), &Screen::Settings);
///
/// // Navigate to About
/// Router::update(&mut state, RouterMessage::Navigate(Screen::About));
/// assert_eq!(state.history_len(), 2);
///
/// // Go back twice
/// Router::update(&mut state, RouterMessage::Back);
/// assert_eq!(state.current(), &Screen::Settings);
/// Router::update(&mut state, RouterMessage::Back);
/// assert_eq!(state.current(), &Screen::Home);
/// ```
pub struct Router<S: Clone + PartialEq>(std::marker::PhantomData<S>);

impl<S: Clone + PartialEq> Component for Router<S> {
    type State = RouterState<S>;
    type Message = RouterMessage<S>;
    type Output = RouterOutput<S>;

    fn init() -> Self::State {
        // This will panic because we need an initial screen.
        // Users should create RouterState directly with RouterState::new(initial_screen)
        panic!("Router requires an initial screen. Use RouterState::new(initial_screen) instead of Router::init()");
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            RouterMessage::Navigate(screen) => {
                if state.current == screen {
                    return None; // Already at this screen
                }

                let from = state.current.clone();
                state.history.push(state.current.clone());
                state.current = screen.clone();
                state.enforce_history_limit();

                Some(RouterOutput::ScreenChanged { from, to: screen })
            }

            RouterMessage::NavigateWith(screen, mode) => match mode {
                NavigationMode::Push => Self::update(state, RouterMessage::Navigate(screen)),
                NavigationMode::Replace => Self::update(state, RouterMessage::Replace(screen)),
            },

            RouterMessage::Replace(screen) => {
                if state.current == screen {
                    return None;
                }

                let from = state.current.clone();
                state.current = screen.clone();

                Some(RouterOutput::ScreenChanged { from, to: screen })
            }

            RouterMessage::Back => {
                if let Some(previous) = state.history.pop() {
                    state.current = previous.clone();
                    Some(RouterOutput::NavigatedBack { to: previous })
                } else {
                    Some(RouterOutput::NoPreviousScreen)
                }
            }

            RouterMessage::ClearHistory => {
                if state.history.is_empty() {
                    None
                } else {
                    state.history.clear();
                    Some(RouterOutput::HistoryCleared)
                }
            }

            RouterMessage::Reset(screen) => {
                state.history.clear();
                state.current = screen.clone();
                Some(RouterOutput::Reset(screen))
            }
        }
    }

    fn view(_state: &Self::State, _frame: &mut Frame, _area: Rect) {
        // Router is state-only - no view implementation.
        // The parent application should render based on state.current()
    }
}

#[cfg(test)]
mod tests {
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
            .draw(|frame| Router::view(&state, frame, frame.area()))
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
}
