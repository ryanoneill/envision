use super::*;
use crate::component::test_utils;
use crate::component::Focusable;
use crate::input::{KeyCode, KeyModifiers};

fn focused_state() -> ChatViewState {
    let mut state = ChatViewState::new();
    ChatView::set_focused(&mut state, true);
    state
}

fn state_with_messages() -> ChatViewState {
    let mut state = focused_state();
    state.push_system("Welcome!");
    state.push_user("Hello");
    state.push_assistant("Hi there!");
    state
}

// =============================================================================
// Construction
// =============================================================================

#[test]
fn test_new() {
    let state = ChatViewState::new();
    assert_eq!(state.message_count(), 0);
    assert!(state.input_value().is_empty());
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert!(state.is_input_focused());
}

#[test]
fn test_default() {
    let state = ChatViewState::default();
    assert_eq!(state.max_messages(), 1000);
    assert!(!state.show_timestamps());
    assert!(state.auto_scroll());
    assert_eq!(state.input_height(), 3);
}

#[test]
fn test_with_max_messages() {
    let state = ChatViewState::new().with_max_messages(50);
    assert_eq!(state.max_messages(), 50);
}

#[test]
fn test_with_timestamps() {
    let state = ChatViewState::new().with_timestamps(true);
    assert!(state.show_timestamps());
}

#[test]
fn test_with_input_height() {
    let state = ChatViewState::new().with_input_height(5);
    assert_eq!(state.input_height(), 5);
}

#[test]
fn test_with_input_height_minimum() {
    let state = ChatViewState::new().with_input_height(0);
    assert_eq!(state.input_height(), 1);
}

#[test]
fn test_with_disabled() {
    let state = ChatViewState::new().with_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_with_placeholder() {
    let state = ChatViewState::new().with_placeholder("Enter text...");
    assert_eq!(state.input.placeholder(), "Enter text...");
}

// =============================================================================
// ChatRole
// =============================================================================

#[test]
fn test_role_prefix() {
    assert_eq!(ChatRole::User.prefix(), "You");
    assert_eq!(ChatRole::System.prefix(), "System");
    assert_eq!(ChatRole::Assistant.prefix(), "Assistant");
}

#[test]
fn test_role_color() {
    assert_eq!(ChatRole::User.color(), Color::Cyan);
    assert_eq!(ChatRole::System.color(), Color::DarkGray);
    assert_eq!(ChatRole::Assistant.color(), Color::Green);
}

// =============================================================================
// ChatMessage
// =============================================================================

#[test]
fn test_chat_message_new() {
    let msg = ChatMessage::new(ChatRole::User, "Hello");
    assert_eq!(msg.role(), ChatRole::User);
    assert_eq!(msg.content(), "Hello");
    assert_eq!(msg.timestamp(), None);
    assert_eq!(msg.display_name(), "You");
}

#[test]
fn test_chat_message_with_timestamp() {
    let msg = ChatMessage::new(ChatRole::User, "Hello").with_timestamp("12:00");
    assert_eq!(msg.timestamp(), Some("12:00"));
}

#[test]
fn test_chat_message_with_username() {
    let msg = ChatMessage::new(ChatRole::User, "Hello").with_username("Alice");
    assert_eq!(msg.display_name(), "Alice");
}

#[test]
fn test_chat_message_display_name_default() {
    let msg = ChatMessage::new(ChatRole::Assistant, "Hi");
    assert_eq!(msg.display_name(), "Assistant");
}

// =============================================================================
// Message manipulation
// =============================================================================

#[test]
fn test_push_user() {
    let mut state = ChatViewState::new();
    state.push_user("Hello");
    assert_eq!(state.message_count(), 1);
    assert_eq!(state.messages()[0].role(), ChatRole::User);
    assert_eq!(state.messages()[0].content(), "Hello");
}

#[test]
fn test_push_system() {
    let mut state = ChatViewState::new();
    state.push_system("Welcome");
    assert_eq!(state.messages()[0].role(), ChatRole::System);
}

#[test]
fn test_push_assistant() {
    let mut state = ChatViewState::new();
    state.push_assistant("How can I help?");
    assert_eq!(state.messages()[0].role(), ChatRole::Assistant);
}

#[test]
fn test_push_with_timestamps() {
    let mut state = ChatViewState::new();
    state.push_user_with_timestamp("Hi", "12:00");
    state.push_system_with_timestamp("Info", "12:01");
    state.push_assistant_with_timestamp("Hello", "12:02");
    assert_eq!(state.messages()[0].timestamp(), Some("12:00"));
    assert_eq!(state.messages()[1].timestamp(), Some("12:01"));
    assert_eq!(state.messages()[2].timestamp(), Some("12:02"));
}

#[test]
fn test_push_message_custom() {
    let mut state = ChatViewState::new();
    let msg = ChatMessage::new(ChatRole::User, "Custom")
        .with_username("Bob")
        .with_timestamp("09:00");
    state.push_message(msg);
    assert_eq!(state.messages()[0].display_name(), "Bob");
}

#[test]
fn test_clear_messages() {
    let mut state = state_with_messages();
    state.clear_messages();
    assert!(state.is_empty());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_eviction() {
    let mut state = ChatViewState::new().with_max_messages(3);
    state.push_user("a");
    state.push_user("b");
    state.push_user("c");
    state.push_user("d");
    assert_eq!(state.message_count(), 3);
    assert_eq!(state.messages()[0].content(), "b");
}

#[test]
fn test_set_max_messages() {
    let mut state = ChatViewState::new();
    state.push_user("a");
    state.push_user("b");
    state.push_user("c");
    state.set_max_messages(2);
    assert_eq!(state.message_count(), 2);
    assert_eq!(state.messages()[0].content(), "b");
}

// =============================================================================
// Input editing
// =============================================================================

#[test]
fn test_type_input() {
    let mut state = focused_state();
    let output = ChatView::update(&mut state, ChatViewMessage::Input('H'));
    ChatView::update(&mut state, ChatViewMessage::Input('i'));
    assert_eq!(state.input_value(), "Hi");
    assert!(matches!(output, Some(ChatViewOutput::InputChanged(_))));
}

#[test]
fn test_newline() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('a'));
    ChatView::update(&mut state, ChatViewMessage::NewLine);
    ChatView::update(&mut state, ChatViewMessage::Input('b'));
    assert_eq!(state.input_value(), "a\nb");
}

#[test]
fn test_backspace() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('a'));
    ChatView::update(&mut state, ChatViewMessage::Input('b'));
    let output = ChatView::update(&mut state, ChatViewMessage::Backspace);
    assert_eq!(state.input_value(), "a");
    assert!(matches!(output, Some(ChatViewOutput::InputChanged(_))));
}

#[test]
fn test_backspace_at_start() {
    let mut state = focused_state();
    let output = ChatView::update(&mut state, ChatViewMessage::Backspace);
    assert_eq!(output, None);
}

#[test]
fn test_delete() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('a'));
    ChatView::update(&mut state, ChatViewMessage::Input('b'));
    ChatView::update(&mut state, ChatViewMessage::Home);
    let output = ChatView::update(&mut state, ChatViewMessage::Delete);
    assert_eq!(state.input_value(), "b");
    assert!(matches!(output, Some(ChatViewOutput::InputChanged(_))));
}

#[test]
fn test_delete_at_end() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('a'));
    let output = ChatView::update(&mut state, ChatViewMessage::Delete);
    assert_eq!(output, None);
}

#[test]
fn test_set_input_value() {
    let mut state = ChatViewState::new();
    state.set_input_value("preset text");
    assert_eq!(state.input_value(), "preset text");
}

#[test]
fn test_clear_input() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('a'));
    let output = ChatView::update(&mut state, ChatViewMessage::ClearInput);
    assert!(state.input_value().is_empty());
    assert_eq!(output, Some(ChatViewOutput::InputChanged(String::new())));
}

#[test]
fn test_delete_to_end() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('a'));
    ChatView::update(&mut state, ChatViewMessage::Input('b'));
    ChatView::update(&mut state, ChatViewMessage::Home);
    let output = ChatView::update(&mut state, ChatViewMessage::DeleteToEnd);
    assert!(state.input_value().is_empty());
    assert!(matches!(output, Some(ChatViewOutput::InputChanged(_))));
}

#[test]
fn test_delete_to_start() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('a'));
    ChatView::update(&mut state, ChatViewMessage::Input('b'));
    let output = ChatView::update(&mut state, ChatViewMessage::DeleteToStart);
    assert!(state.input_value().is_empty());
    assert!(matches!(output, Some(ChatViewOutput::InputChanged(_))));
}

// =============================================================================
// Submit
// =============================================================================

#[test]
fn test_submit() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('H'));
    ChatView::update(&mut state, ChatViewMessage::Input('i'));
    let output = ChatView::update(&mut state, ChatViewMessage::Submit);
    assert_eq!(output, Some(ChatViewOutput::Submitted("Hi".into())));
    assert!(state.input_value().is_empty());
    assert_eq!(state.message_count(), 1);
    assert_eq!(state.messages()[0].content(), "Hi");
}

#[test]
fn test_submit_empty() {
    let mut state = focused_state();
    let output = ChatView::update(&mut state, ChatViewMessage::Submit);
    assert_eq!(output, None);
    assert_eq!(state.message_count(), 0);
}

#[test]
fn test_submit_whitespace_only() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input(' '));
    ChatView::update(&mut state, ChatViewMessage::Input(' '));
    let output = ChatView::update(&mut state, ChatViewMessage::Submit);
    assert_eq!(output, None);
}

// =============================================================================
// Focus management
// =============================================================================

#[test]
fn test_toggle_focus() {
    let mut state = focused_state();
    assert!(state.is_input_focused());

    ChatView::update(&mut state, ChatViewMessage::ToggleFocus);
    assert!(state.is_history_focused());

    ChatView::update(&mut state, ChatViewMessage::ToggleFocus);
    assert!(state.is_input_focused());
}

#[test]
fn test_focus_input() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::FocusHistory);
    ChatView::update(&mut state, ChatViewMessage::FocusInput);
    assert!(state.is_input_focused());
}

#[test]
fn test_focus_history() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::FocusHistory);
    assert!(state.is_history_focused());
}

// =============================================================================
// Scrolling
// =============================================================================

#[test]
fn test_scroll_down() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    state.scroll.set_offset(0);
    ChatView::update(&mut state, ChatViewMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    state.scroll.set_content_length(state.messages().len());
    state.scroll.set_offset(2);
    ChatView::update(&mut state, ChatViewMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 1);
}

#[test]
fn test_scroll_up_at_top() {
    let mut state = state_with_messages();
    state.scroll.set_offset(0);
    ChatView::update(&mut state, ChatViewMessage::ScrollUp);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_scroll_to_top() {
    let mut state = state_with_messages();
    state.scroll.set_content_length(state.messages().len());
    state.scroll.set_offset(2);
    ChatView::update(&mut state, ChatViewMessage::ScrollToTop);
    assert_eq!(state.scroll_offset(), 0);
    assert!(!state.auto_scroll());
}

#[test]
fn test_scroll_to_bottom() {
    let mut state = state_with_messages();
    state.set_auto_scroll(false);
    state.scroll.set_offset(0);
    ChatView::update(&mut state, ChatViewMessage::ScrollToBottom);
    assert!(state.scroll.at_end());
    assert!(state.auto_scroll());
}

#[test]
fn test_auto_scroll_on_new_message() {
    let mut state = focused_state();
    state.push_user("a");
    state.push_user("b");
    // auto_scroll should keep us at the bottom
    assert!(state.auto_scroll());
}

#[test]
fn test_scroll_up_disables_auto_scroll() {
    let mut state = state_with_messages();
    ChatView::update(&mut state, ChatViewMessage::ScrollUp);
    assert!(!state.auto_scroll());
}

#[test]
fn test_set_auto_scroll() {
    let mut state = ChatViewState::new();
    state.set_auto_scroll(false);
    assert!(!state.auto_scroll());
    state.set_auto_scroll(true);
    assert!(state.auto_scroll());
}

// =============================================================================
// Disabled state
// =============================================================================

#[test]
fn test_disabled_ignores_messages() {
    let mut state = focused_state();
    state.set_disabled(true);
    let output = ChatView::update(&mut state, ChatViewMessage::Input('a'));
    assert_eq!(output, None);
}

#[test]
fn test_disabled_ignores_events() {
    let mut state = focused_state();
    state.set_disabled(true);
    let msg = ChatView::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);
}

// =============================================================================
// Unfocused state
// =============================================================================

#[test]
fn test_unfocused_ignores_events() {
    let state = ChatViewState::new();
    let msg = ChatView::handle_event(&state, &Event::char('a'));
    assert_eq!(msg, None);
}

// =============================================================================
// Event mapping — input mode
// =============================================================================

#[test]
fn test_input_mode_char() {
    let state = focused_state();
    assert_eq!(
        ChatView::handle_event(&state, &Event::char('a')),
        Some(ChatViewMessage::Input('a'))
    );
}

#[test]
fn test_input_mode_enter() {
    let state = focused_state();
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Enter)),
        Some(ChatViewMessage::NewLine)
    );
}

#[test]
fn test_input_mode_ctrl_enter() {
    let state = focused_state();
    assert_eq!(
        ChatView::handle_event(
            &state,
            &Event::key_with(KeyCode::Enter, KeyModifiers::CONTROL)
        ),
        Some(ChatViewMessage::Submit)
    );
}

#[test]
fn test_input_mode_tab() {
    let state = focused_state();
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Tab)),
        Some(ChatViewMessage::ToggleFocus)
    );
}

#[test]
fn test_input_mode_backspace() {
    let state = focused_state();
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Backspace)),
        Some(ChatViewMessage::Backspace)
    );
}

#[test]
fn test_input_mode_delete() {
    let state = focused_state();
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Delete)),
        Some(ChatViewMessage::Delete)
    );
}

#[test]
fn test_input_mode_arrows() {
    let state = focused_state();
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Left)),
        Some(ChatViewMessage::Left)
    );
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Right)),
        Some(ChatViewMessage::Right)
    );
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(ChatViewMessage::Up)
    );
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(ChatViewMessage::Down)
    );
}

#[test]
fn test_input_mode_home_end() {
    let state = focused_state();
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(ChatViewMessage::Home)
    );
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::End)),
        Some(ChatViewMessage::End)
    );
}

#[test]
fn test_input_mode_ctrl_home_end() {
    let state = focused_state();
    assert_eq!(
        ChatView::handle_event(
            &state,
            &Event::key_with(KeyCode::Home, KeyModifiers::CONTROL)
        ),
        Some(ChatViewMessage::InputStart)
    );
    assert_eq!(
        ChatView::handle_event(
            &state,
            &Event::key_with(KeyCode::End, KeyModifiers::CONTROL)
        ),
        Some(ChatViewMessage::InputEnd)
    );
}

// =============================================================================
// Event mapping — history mode
// =============================================================================

#[test]
fn test_history_mode_up_down() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::FocusHistory);
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(ChatViewMessage::ScrollUp)
    );
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(ChatViewMessage::ScrollDown)
    );
    assert_eq!(
        ChatView::handle_event(&state, &Event::char('k')),
        Some(ChatViewMessage::ScrollUp)
    );
    assert_eq!(
        ChatView::handle_event(&state, &Event::char('j')),
        Some(ChatViewMessage::ScrollDown)
    );
}

#[test]
fn test_history_mode_home_end() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::FocusHistory);
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(ChatViewMessage::ScrollToTop)
    );
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::End)),
        Some(ChatViewMessage::ScrollToBottom)
    );
}

#[test]
fn test_history_mode_tab() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::FocusHistory);
    assert_eq!(
        ChatView::handle_event(&state, &Event::key(KeyCode::Tab)),
        Some(ChatViewMessage::ToggleFocus)
    );
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = focused_state();
    let msg = state.handle_event(&Event::char('a'));
    assert_eq!(msg, Some(ChatViewMessage::Input('a')));
}

#[test]
fn test_instance_update() {
    let mut state = focused_state();
    state.update(ChatViewMessage::Input('a'));
    assert_eq!(state.input_value(), "a");
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = focused_state();
    state.dispatch_event(&Event::char('a'));
    assert_eq!(state.input_value(), "a");
}

// =============================================================================
// Accessors
// =============================================================================

#[test]
fn test_set_show_timestamps() {
    let mut state = ChatViewState::new();
    state.set_show_timestamps(true);
    assert!(state.show_timestamps());
}

#[test]
fn test_set_input_height() {
    let mut state = ChatViewState::new();
    state.set_input_height(5);
    assert_eq!(state.input_height(), 5);
}

#[test]
fn test_set_input_height_minimum() {
    let mut state = ChatViewState::new();
    state.set_input_height(0);
    assert_eq!(state.input_height(), 1);
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = ChatViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_messages() {
    let state = state_with_messages();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_focused_input() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_focused_history() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::FocusHistory);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = ChatViewState::new().with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_timestamps() {
    let mut state = ChatViewState::new().with_timestamps(true);
    state.push_user_with_timestamp("Hello", "12:00");
    state.push_assistant_with_timestamp("Hi!", "12:01");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = state_with_messages();
    let (mut terminal, theme) = test_utils::setup_render(60, 3);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_input_text() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('H'));
    ChatView::update(&mut state, ChatViewMessage::Input('e'));
    ChatView::update(&mut state, ChatViewMessage::Input('l'));
    ChatView::update(&mut state, ChatViewMessage::Input('l'));
    ChatView::update(&mut state, ChatViewMessage::Input('o'));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// Focusable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = ChatView::init();
    assert!(!ChatView::is_focused(&state));

    ChatView::focus(&mut state);
    assert!(ChatView::is_focused(&state));

    ChatView::blur(&mut state);
    assert!(!ChatView::is_focused(&state));
}

// =============================================================================
// PartialEq
// =============================================================================

#[test]
fn test_partial_eq() {
    let state1 = state_with_messages();
    let state2 = state_with_messages();
    assert_eq!(state1, state2);
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_submit_clears_input() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('x'));
    ChatView::update(&mut state, ChatViewMessage::Submit);
    assert!(state.input_value().is_empty());
}

#[test]
fn test_scroll_empty_history() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::ScrollDown);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_input_cursor_movement() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('a'));
    ChatView::update(&mut state, ChatViewMessage::Input('b'));
    ChatView::update(&mut state, ChatViewMessage::Input('c'));
    ChatView::update(&mut state, ChatViewMessage::Home);
    ChatView::update(&mut state, ChatViewMessage::Right);
    ChatView::update(&mut state, ChatViewMessage::Delete);
    assert_eq!(state.input_value(), "ac");
}

#[test]
fn test_multiline_submit() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('l'));
    ChatView::update(&mut state, ChatViewMessage::Input('1'));
    ChatView::update(&mut state, ChatViewMessage::NewLine);
    ChatView::update(&mut state, ChatViewMessage::Input('l'));
    ChatView::update(&mut state, ChatViewMessage::Input('2'));
    let output = ChatView::update(&mut state, ChatViewMessage::Submit);
    assert_eq!(output, Some(ChatViewOutput::Submitted("l1\nl2".into())));
}

#[test]
fn test_input_start_end() {
    let mut state = focused_state();
    ChatView::update(&mut state, ChatViewMessage::Input('a'));
    ChatView::update(&mut state, ChatViewMessage::NewLine);
    ChatView::update(&mut state, ChatViewMessage::Input('b'));
    ChatView::update(&mut state, ChatViewMessage::InputStart);
    // Cursor should be at (0,0) now
    ChatView::update(&mut state, ChatViewMessage::Input('!'));
    assert_eq!(state.input_value(), "!a\nb");
}

// =============================================================================
// Role style inheritance
// =============================================================================

#[test]
fn test_default_role_style() {
    let state = ChatViewState::new();
    assert_eq!(
        state.role_style(&ChatRole::User),
        Style::default().fg(Color::Cyan)
    );
    assert_eq!(
        state.role_style(&ChatRole::System),
        Style::default().fg(Color::DarkGray)
    );
    assert_eq!(
        state.role_style(&ChatRole::Assistant),
        Style::default().fg(Color::Green)
    );
}

#[test]
fn test_custom_role_style() {
    let mut state = ChatViewState::new();
    let custom = Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::ITALIC);
    state.set_role_style(ChatRole::User, custom);
    assert_eq!(state.role_style(&ChatRole::User), custom);
    // Other roles unchanged
    assert_eq!(
        state.role_style(&ChatRole::Assistant),
        Style::default().fg(Color::Green)
    );
}

#[test]
fn test_with_role_style_builder() {
    let custom = Style::default().fg(Color::Yellow);
    let state = ChatViewState::new().with_role_style(ChatRole::Assistant, custom);
    assert_eq!(state.role_style(&ChatRole::Assistant), custom);
}

#[test]
fn test_clear_role_styles() {
    let mut state = ChatViewState::new();
    state.set_role_style(ChatRole::User, Style::default().fg(Color::Red));
    state.clear_role_styles();
    assert_eq!(
        state.role_style(&ChatRole::User),
        Style::default().fg(Color::Cyan)
    );
}

#[test]
fn test_role_style_per_role() {
    let state = ChatViewState::new()
        .with_role_style(ChatRole::User, Style::default().fg(Color::Red))
        .with_role_style(ChatRole::System, Style::default().fg(Color::Blue))
        .with_role_style(ChatRole::Assistant, Style::default().fg(Color::Yellow));
    assert_eq!(
        state.role_style(&ChatRole::User),
        Style::default().fg(Color::Red)
    );
    assert_eq!(
        state.role_style(&ChatRole::System),
        Style::default().fg(Color::Blue)
    );
    assert_eq!(
        state.role_style(&ChatRole::Assistant),
        Style::default().fg(Color::Yellow)
    );
}

#[test]
fn test_format_message_uses_base_style() {
    let msg = ChatMessage::new(ChatRole::User, "Hello");
    let custom_style = Style::default().fg(Color::Red);
    let state = ChatViewState::new().with_role_style(ChatRole::User, custom_style);
    let theme = crate::theme::Theme::default();
    let lines = super::render_helpers::format_message(&msg, &state, 40, &theme);
    // Header line should use bold variant of custom style
    let (header, _) = &lines[0];
    let header_span = &header.spans[0]; // "You:" span
    assert_eq!(header_span.style, custom_style.add_modifier(Modifier::BOLD));
    // Content line should use custom style
    let (content, _) = &lines[1];
    let content_span = &content.spans[0]; // "  Hello" span
    assert_eq!(content_span.style, custom_style);
}

#[cfg(feature = "markdown")]
#[test]
fn test_markdown_format_message_uses_role_style() {
    let msg = ChatMessage::new(ChatRole::Assistant, "**bold** and plain");
    let role_style = Style::default().fg(Color::Green);
    let state = ChatViewState::new()
        .with_markdown(true)
        .with_role_style(ChatRole::Assistant, role_style);
    let theme = crate::theme::Theme::default();
    let lines = super::render_helpers::format_message(&msg, &state, 40, &theme);
    // Header uses role style with bold modifier
    let (header, _) = &lines[0];
    let header_span = &header.spans[0]; // "Assistant:" span
    assert_eq!(header_span.style, role_style.add_modifier(Modifier::BOLD));
    // Content lines use role style as base
    // Line 1: indent + "bold" (bold) + " and " (plain) + ...
    let (content, _) = &lines[1];
    // First span is the 2-space indent
    assert_eq!(content.spans[0].content.as_ref(), "  ");
    // Second span is "bold" with role_style + BOLD modifier
    assert_eq!(
        content.spans[1].style,
        role_style.add_modifier(Modifier::BOLD)
    );
    // Third span is " and plain" with role_style (plain text)
    assert_eq!(content.spans[2].style, role_style);
}

#[test]
fn test_render_with_custom_role_styles() {
    let mut state = ChatViewState::new()
        .with_role_style(ChatRole::User, Style::default().fg(Color::Red))
        .with_role_style(ChatRole::Assistant, Style::default().fg(Color::Yellow));
    ChatView::set_focused(&mut state, true);
    state.push_user("Hello");
    state.push_assistant("Hi!");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ChatView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
    // Just verify it renders without panicking
}

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = ChatViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                ChatView::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    assert!(registry.get_by_id("chat_view").is_some());
}
