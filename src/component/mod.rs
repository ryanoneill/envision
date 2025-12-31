//! Composable UI components for TUI applications.
//!
//! This module provides traits for building reusable UI components that
//! follow the TEA (The Elm Architecture) pattern at a granular level.
//!
//! While the [`App`](crate::app::App) trait defines the top-level application,
//! components are smaller, composable pieces that can be combined to build
//! complex interfaces.
//!
//! # Core Traits
//!
//! - [`Component`]: The base trait for all components
//! - [`Focusable`]: Components that can receive keyboard focus
//! - [`Toggleable`]: Components that can be shown or hidden
//!
//! # Built-in Components
//!
//! - [`SelectableList`]: A scrollable list with keyboard navigation
//! - [`InputField`]: A text input field with cursor navigation
//!
//! # Component vs App
//!
//! | Aspect | App | Component |
//! |--------|-----|-----------|
//! | Scope | Entire application | Part of the UI |
//! | State | Owns all state | Owns its own state |
//! | Messages | All app messages | Component-specific messages |
//! | Output | Commands (side effects) | Messages to parent |
//! | Rendering | Full frame | Specific area |
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Component, Focusable};
//! use ratatui::prelude::*;
//!
//! struct Counter;
//!
//! #[derive(Clone, Default)]
//! struct CounterState {
//!     value: i32,
//!     focused: bool,
//! }
//!
//! #[derive(Clone)]
//! enum CounterMsg {
//!     Increment,
//!     Decrement,
//! }
//!
//! #[derive(Clone)]
//! enum CounterOutput {
//!     ValueChanged(i32),
//! }
//!
//! impl Component for Counter {
//!     type State = CounterState;
//!     type Message = CounterMsg;
//!     type Output = CounterOutput;
//!
//!     fn init() -> Self::State {
//!         CounterState::default()
//!     }
//!
//!     fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
//!         match msg {
//!             CounterMsg::Increment => state.value += 1,
//!             CounterMsg::Decrement => state.value -= 1,
//!         }
//!         Some(CounterOutput::ValueChanged(state.value))
//!     }
//!
//!     fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
//!         let style = if state.focused {
//!             Style::default().fg(Color::Yellow)
//!         } else {
//!             Style::default()
//!         };
//!         let text = format!("Count: {}", state.value);
//!         frame.render_widget(
//!             ratatui::widgets::Paragraph::new(text).style(style),
//!             area,
//!         );
//!     }
//! }
//!
//! impl Focusable for Counter {
//!     fn is_focused(state: &Self::State) -> bool {
//!         state.focused
//!     }
//!
//!     fn set_focused(state: &mut Self::State, focused: bool) {
//!         state.focused = focused;
//!     }
//! }
//! ```

use ratatui::prelude::*;

mod input_field;
mod selectable_list;

pub use input_field::{InputField, InputFieldState, InputMessage, InputOutput};
pub use selectable_list::{ListMessage, ListOutput, SelectableList, SelectableListState};

/// A composable UI component with its own state and message handling.
///
/// Components are the building blocks of complex TUI applications. Each
/// component manages its own state, handles its own messages, and renders
/// to a specific area of the screen.
///
/// # Associated Types
///
/// - `State`: The component's internal state. Should be `Clone` for testing.
/// - `Message`: Messages the component can receive from its parent or from
///   user interaction.
/// - `Output`: Messages the component emits to communicate with its parent.
///   Use `()` if the component doesn't need to communicate outward.
///
/// # Design Pattern
///
/// Components follow the same TEA pattern as [`App`](crate::app::App), but
/// at a smaller scale:
///
/// 1. Parent sends `Message` to component
/// 2. Component updates its `State`
/// 3. Component optionally emits `Output` to parent
/// 4. Component renders itself to its designated area
pub trait Component: Sized {
    /// The component's internal state type.
    ///
    /// This should contain all data needed to render the component.
    /// Derive `Clone` for testing and state snapshots.
    type State: Clone;

    /// Messages this component can receive.
    ///
    /// These typically come from user input or parent components.
    type Message: Clone;

    /// Messages this component can emit to its parent.
    ///
    /// Use `()` if the component doesn't need to communicate upward.
    /// This enables child-to-parent communication without tight coupling.
    type Output: Clone;

    /// Initialize the component state.
    ///
    /// Returns the initial state for this component.
    fn init() -> Self::State;

    /// Update component state based on a message.
    ///
    /// Returns an optional output message for the parent to handle.
    /// Return `None` if no parent notification is needed.
    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output>;

    /// Render the component to the given area.
    ///
    /// Unlike [`App::view`](crate::app::App::view) which renders to the full
    /// frame, components render to a specific `Rect` area provided by their
    /// parent.
    fn view(state: &Self::State, frame: &mut Frame, area: Rect);
}

/// A component that can receive keyboard focus.
///
/// Focus is used to determine which component receives keyboard input
/// when multiple components are on screen. Only one component should
/// typically be focused at a time.
///
/// # Visual Feedback
///
/// Components should provide visual feedback when focused, such as:
/// - Highlighted borders
/// - Changed colors
/// - Cursor visibility
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, Focusable};
/// use ratatui::prelude::*;
///
/// struct TextInput;
///
/// #[derive(Clone, Default)]
/// struct TextInputState {
///     value: String,
///     focused: bool,
/// }
///
/// # #[derive(Clone)]
/// # enum TextInputMsg {}
/// # #[derive(Clone)]
/// # enum TextInputOutput {}
/// #
/// # impl Component for TextInput {
/// #     type State = TextInputState;
/// #     type Message = TextInputMsg;
/// #     type Output = TextInputOutput;
/// #     fn init() -> Self::State { TextInputState::default() }
/// #     fn update(_: &mut Self::State, _: Self::Message) -> Option<Self::Output> { None }
/// #     fn view(_: &Self::State, _: &mut Frame, _: Rect) {}
/// # }
/// #
/// impl Focusable for TextInput {
///     fn is_focused(state: &Self::State) -> bool {
///         state.focused
///     }
///
///     fn set_focused(state: &mut Self::State, focused: bool) {
///         state.focused = focused;
///     }
/// }
/// ```
pub trait Focusable: Component {
    /// Returns true if this component is currently focused.
    fn is_focused(state: &Self::State) -> bool;

    /// Sets the focus state of this component.
    fn set_focused(state: &mut Self::State, focused: bool);

    /// Gives focus to this component.
    ///
    /// Convenience method equivalent to `set_focused(state, true)`.
    fn focus(state: &mut Self::State) {
        Self::set_focused(state, true);
    }

    /// Removes focus from this component.
    ///
    /// Convenience method equivalent to `set_focused(state, false)`.
    fn blur(state: &mut Self::State) {
        Self::set_focused(state, false);
    }
}

/// A component that can be shown or hidden.
///
/// This is useful for panels, dialogs, and other UI elements that
/// can be toggled on and off. When hidden, the component should not
/// be rendered or receive input.
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, Toggleable};
/// use ratatui::prelude::*;
///
/// struct HelpPanel;
///
/// #[derive(Clone)]
/// struct HelpPanelState {
///     visible: bool,
///     content: String,
/// }
///
/// # #[derive(Clone)]
/// # enum HelpPanelMsg {}
/// # #[derive(Clone)]
/// # enum HelpPanelOutput {}
/// #
/// # impl Component for HelpPanel {
/// #     type State = HelpPanelState;
/// #     type Message = HelpPanelMsg;
/// #     type Output = HelpPanelOutput;
/// #     fn init() -> Self::State { HelpPanelState { visible: false, content: String::new() } }
/// #     fn update(_: &mut Self::State, _: Self::Message) -> Option<Self::Output> { None }
/// #     fn view(_: &Self::State, _: &mut Frame, _: Rect) {}
/// # }
/// #
/// impl Toggleable for HelpPanel {
///     fn is_visible(state: &Self::State) -> bool {
///         state.visible
///     }
///
///     fn set_visible(state: &mut Self::State, visible: bool) {
///         state.visible = visible;
///     }
/// }
///
/// // Usage:
/// let mut state = HelpPanel::init();
/// assert!(!HelpPanel::is_visible(&state));
///
/// HelpPanel::toggle(&mut state);
/// assert!(HelpPanel::is_visible(&state));
///
/// HelpPanel::hide(&mut state);
/// assert!(!HelpPanel::is_visible(&state));
/// ```
pub trait Toggleable: Component {
    /// Returns true if this component is currently visible.
    fn is_visible(state: &Self::State) -> bool;

    /// Sets the visibility of this component.
    fn set_visible(state: &mut Self::State, visible: bool);

    /// Toggles the visibility of this component.
    fn toggle(state: &mut Self::State) {
        let visible = Self::is_visible(state);
        Self::set_visible(state, !visible);
    }

    /// Shows this component.
    ///
    /// Convenience method equivalent to `set_visible(state, true)`.
    fn show(state: &mut Self::State) {
        Self::set_visible(state, true);
    }

    /// Hides this component.
    ///
    /// Convenience method equivalent to `set_visible(state, false)`.
    fn hide(state: &mut Self::State) {
        Self::set_visible(state, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Paragraph;

    // Test component implementation
    struct TestCounter;

    #[derive(Clone, Default)]
    struct TestCounterState {
        value: i32,
        focused: bool,
        visible: bool,
    }

    #[derive(Clone)]
    enum TestCounterMsg {
        Increment,
        Decrement,
    }

    #[derive(Clone, PartialEq, Debug)]
    enum TestCounterOutput {
        Changed(i32),
    }

    impl Component for TestCounter {
        type State = TestCounterState;
        type Message = TestCounterMsg;
        type Output = TestCounterOutput;

        fn init() -> Self::State {
            TestCounterState {
                value: 0,
                focused: false,
                visible: true,
            }
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
            match msg {
                TestCounterMsg::Increment => state.value += 1,
                TestCounterMsg::Decrement => state.value -= 1,
            }
            Some(TestCounterOutput::Changed(state.value))
        }

        fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
            let text = format!("Count: {}", state.value);
            frame.render_widget(Paragraph::new(text), area);
        }
    }

    impl Focusable for TestCounter {
        fn is_focused(state: &Self::State) -> bool {
            state.focused
        }

        fn set_focused(state: &mut Self::State, focused: bool) {
            state.focused = focused;
        }
    }

    impl Toggleable for TestCounter {
        fn is_visible(state: &Self::State) -> bool {
            state.visible
        }

        fn set_visible(state: &mut Self::State, visible: bool) {
            state.visible = visible;
        }
    }

    // Component trait tests

    #[test]
    fn test_component_init() {
        let state = TestCounter::init();
        assert_eq!(state.value, 0);
        assert!(!state.focused);
        assert!(state.visible);
    }

    #[test]
    fn test_component_update() {
        let mut state = TestCounter::init();

        let output = TestCounter::update(&mut state, TestCounterMsg::Increment);
        assert_eq!(state.value, 1);
        assert_eq!(output, Some(TestCounterOutput::Changed(1)));

        let output = TestCounter::update(&mut state, TestCounterMsg::Increment);
        assert_eq!(state.value, 2);
        assert_eq!(output, Some(TestCounterOutput::Changed(2)));

        let output = TestCounter::update(&mut state, TestCounterMsg::Decrement);
        assert_eq!(state.value, 1);
        assert_eq!(output, Some(TestCounterOutput::Changed(1)));
    }

    #[test]
    fn test_component_view() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state = TestCounter::init();
        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                TestCounter::view(&state, frame, frame.area());
            })
            .unwrap();

        let text = terminal.backend().to_string();
        assert!(text.contains("Count: 0"));
    }

    #[test]
    fn test_state_clone() {
        let mut state = TestCounter::init();
        TestCounter::update(&mut state, TestCounterMsg::Increment);

        let snapshot = state.clone();
        TestCounter::update(&mut state, TestCounterMsg::Increment);

        assert_eq!(snapshot.value, 1);
        assert_eq!(state.value, 2);
    }

    // Focusable trait tests

    #[test]
    fn test_focusable_is_focused() {
        let state = TestCounter::init();
        assert!(!TestCounter::is_focused(&state));
    }

    #[test]
    fn test_focusable_set_focused() {
        let mut state = TestCounter::init();

        TestCounter::set_focused(&mut state, true);
        assert!(TestCounter::is_focused(&state));

        TestCounter::set_focused(&mut state, false);
        assert!(!TestCounter::is_focused(&state));
    }

    #[test]
    fn test_focusable_focus() {
        let mut state = TestCounter::init();

        TestCounter::focus(&mut state);
        assert!(TestCounter::is_focused(&state));
    }

    #[test]
    fn test_focusable_blur() {
        let mut state = TestCounter::init();
        TestCounter::set_focused(&mut state, true);

        TestCounter::blur(&mut state);
        assert!(!TestCounter::is_focused(&state));
    }

    // Toggleable trait tests

    #[test]
    fn test_toggleable_is_visible() {
        let state = TestCounter::init();
        assert!(TestCounter::is_visible(&state));
    }

    #[test]
    fn test_toggleable_set_visible() {
        let mut state = TestCounter::init();

        TestCounter::set_visible(&mut state, false);
        assert!(!TestCounter::is_visible(&state));

        TestCounter::set_visible(&mut state, true);
        assert!(TestCounter::is_visible(&state));
    }

    #[test]
    fn test_toggleable_toggle() {
        let mut state = TestCounter::init();
        assert!(TestCounter::is_visible(&state));

        TestCounter::toggle(&mut state);
        assert!(!TestCounter::is_visible(&state));

        TestCounter::toggle(&mut state);
        assert!(TestCounter::is_visible(&state));
    }

    #[test]
    fn test_toggleable_show() {
        let mut state = TestCounter::init();
        TestCounter::set_visible(&mut state, false);

        TestCounter::show(&mut state);
        assert!(TestCounter::is_visible(&state));
    }

    #[test]
    fn test_toggleable_hide() {
        let mut state = TestCounter::init();

        TestCounter::hide(&mut state);
        assert!(!TestCounter::is_visible(&state));
    }

    // Test component with unit Output type
    struct NoOutputComponent;

    #[derive(Clone, Default)]
    struct NoOutputState {
        data: String,
    }

    #[derive(Clone)]
    enum NoOutputMsg {
        SetData(String),
    }

    impl Component for NoOutputComponent {
        type State = NoOutputState;
        type Message = NoOutputMsg;
        type Output = ();

        fn init() -> Self::State {
            NoOutputState::default()
        }

        fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
            match msg {
                NoOutputMsg::SetData(data) => state.data = data,
            }
            None // No output needed
        }

        fn view(_state: &Self::State, _frame: &mut Frame, _area: Rect) {}
    }

    #[test]
    fn test_component_no_output() {
        let mut state = NoOutputComponent::init();
        let output = NoOutputComponent::update(&mut state, NoOutputMsg::SetData("test".into()));
        assert!(output.is_none());
        assert_eq!(state.data, "test");
    }
}
