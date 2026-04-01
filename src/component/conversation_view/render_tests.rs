use super::*;
use crate::component::test_utils;
use crate::component::Focusable;

fn focused_state() -> ConversationViewState {
    let mut state = ConversationViewState::new();
    ConversationView::set_focused(&mut state, true);
    state
}

fn state_with_messages() -> ConversationViewState {
    let mut state = focused_state();
    state.push_system("Welcome to the conversation.");
    state.push_user("Hello, can you help me?");
    state.push_assistant("Of course! What do you need?");
    state
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = ConversationViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_messages() {
    let state = state_with_messages();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let state = focused_state();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let state = ConversationViewState::new().with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_title() {
    let state = ConversationViewState::new().with_title("Session 1");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_with_timestamps() {
    let mut state = ConversationViewState::new().with_timestamps(true);
    state.push_message(
        ConversationMessage::new(ConversationRole::User, "Hello").with_timestamp("14:30"),
    );
    state.push_message(
        ConversationMessage::new(ConversationRole::Assistant, "Hi!").with_timestamp("14:31"),
    );
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_without_role_labels() {
    let mut state = ConversationViewState::new().with_role_labels(false);
    state.push_user("Hello");
    state.push_assistant("Hi!");
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_code_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::text("Here is the code:"),
            MessageBlock::code("fn main() {\n    println!(\"hello\");\n}", Some("rust")),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_tool_use_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::text("I'll search for that."),
            MessageBlock::tool_use("web_search").with_input("query: rust TUI frameworks"),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_thinking_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::thinking("Let me reason through this problem..."),
            MessageBlock::text("The answer is 42."),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_error_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Tool,
        vec![MessageBlock::error("Connection timeout")],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_streaming_message() {
    let mut state = ConversationViewState::new();
    state.push_message(
        ConversationMessage::new(ConversationRole::Assistant, "Generating...").with_streaming(true),
    );
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_collapsed_thinking() {
    let mut state = ConversationViewState::new();
    state.collapse("thinking");
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::thinking("Hidden reasoning"),
            MessageBlock::text("Visible answer"),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_collapsed_tool_use() {
    let mut state = ConversationViewState::new();
    state.collapse("tool:search");
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![MessageBlock::tool_use("search").with_input("query: test")],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let state = state_with_messages();
    let (mut terminal, theme) = test_utils::setup_render(60, 4);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_tiny_area_no_panic() {
    let state = state_with_messages();
    let (mut terminal, theme) = test_utils::setup_render(4, 2);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_mixed_blocks() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::thinking("Analyzing the problem..."),
            MessageBlock::text("I found the answer."),
            MessageBlock::code("x = 42", Some("python")),
            MessageBlock::tool_use("calculator").with_input("42 * 2"),
            MessageBlock::error("Rate limit exceeded"),
        ],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 30);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// Empty code/tool blocks
// =============================================================================

#[test]
fn test_render_empty_code_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![MessageBlock::code("", None)],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_empty_tool_input() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![MessageBlock::tool_use("noop")],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

#[test]
fn test_render_empty_text_block() {
    let mut state = ConversationViewState::new();
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::User,
        vec![MessageBlock::text("")],
    ));
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// =============================================================================
// Annotation
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let state = ConversationViewState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 20);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                ConversationView::view(&state, frame, frame.area(), &theme);
            })
            .unwrap();
    });
    assert!(registry.get_by_id("conversation_view").is_some());
}
