use super::*;
use crate::component::test_utils;
use crate::component::Focusable;

fn focused_state() -> ConversationViewState {
    let mut state = ConversationViewState::new();
    ConversationView::set_focused(&mut state, true);
    state
}

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state = ConversationViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_messages() {
    let mut state = focused_state();
    state.push_system("Welcome to the conversation.");
    state.push_user("Hello, can you help me?");
    state.push_assistant("Of course! What do you need?");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_code_block() {
    let mut state = focused_state();
    state.push_user("Show me some code.");
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::text("Here is a code example:"),
            MessageBlock::code("fn main() {\n    println!(\"Hello!\");\n}", Some("rust")),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let mut state = ConversationViewState::new().with_disabled(true);
    state.push_user("Hello");
    state.push_assistant("Hi there!");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_title() {
    let mut state = focused_state().with_title("Session 1");
    state.push_user("Hello!");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_multiple_roles() {
    let mut state = focused_state();
    state.push_system("System initialized.");
    state.push_user("What tools do you have?");
    state.push_tool("search: found 5 results");
    state.push_assistant("I found some results for you.");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
