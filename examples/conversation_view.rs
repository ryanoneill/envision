//! Conversation view example demonstrating AI-style conversation display.
//!
//! Shows a read-only conversation with various message block types
//! including text, code, tool use, thinking, and error blocks.
//!
//! Run with: cargo run --example conversation_view

use envision::component::{
    Component, ConversationMessage, ConversationRole, ConversationView, ConversationViewMessage,
    ConversationViewState, MessageBlock,
};
use envision::prelude::*;

struct ConversationApp;

#[derive(Clone)]
struct State {
    conversation: ConversationViewState,
}

#[derive(Clone, Debug)]
enum Msg {
    Conversation(ConversationViewMessage),
    Quit,
}

impl App for ConversationApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut conversation = ConversationViewState::new()
            .with_title("AI Conversation")
            .with_show_timestamps(true);

        // System message
        conversation.push_message(
            ConversationMessage::new(
                ConversationRole::System,
                "You are a helpful coding assistant.",
            )
            .with_timestamp("14:00"),
        );

        // User message
        conversation.push_message(
            ConversationMessage::new(
                ConversationRole::User,
                "Can you write a function to compute the Fibonacci sequence in Rust?",
            )
            .with_timestamp("14:01"),
        );

        // Assistant response with thinking, text, and code
        conversation.push_message(
            ConversationMessage::with_blocks(
                ConversationRole::Assistant,
                vec![
                    MessageBlock::thinking(
                        "The user wants a Fibonacci function in Rust.\n\
                         I can provide both iterative and recursive approaches.",
                    ),
                    MessageBlock::text("Here is an iterative Fibonacci function:"),
                    MessageBlock::code(
                        "fn fibonacci(n: u64) -> u64 {\n    \
                         if n <= 1 {\n        \
                         return n;\n    \
                         }\n    \
                         let mut a = 0;\n    \
                         let mut b = 1;\n    \
                         for _ in 2..=n {\n        \
                         let temp = a + b;\n        \
                         a = b;\n        \
                         b = temp;\n    \
                         }\n    \
                         b\n\
                         }",
                        Some("rust"),
                    ),
                    MessageBlock::text("This runs in O(n) time with O(1) space."),
                ],
            )
            .with_timestamp("14:01"),
        );

        // User follow-up
        conversation.push_message(
            ConversationMessage::new(
                ConversationRole::User,
                "Can you test that with a few values?",
            )
            .with_timestamp("14:02"),
        );

        // Assistant uses a tool
        conversation.push_message(
            ConversationMessage::with_blocks(
                ConversationRole::Assistant,
                vec![
                    MessageBlock::text("Let me run some test values."),
                    MessageBlock::tool_use("code_runner").with_input("fibonacci(0) = 0\nfibonacci(1) = 1\nfibonacci(10) = 55\nfibonacci(20) = 6765"),
                ],
            )
            .with_timestamp("14:02"),
        );

        // Tool result
        conversation.push_message(
            ConversationMessage::with_blocks(
                ConversationRole::Tool,
                vec![MessageBlock::text("All tests passed successfully.")],
            )
            .with_timestamp("14:02"),
        );

        // Error example
        conversation.push_message(
            ConversationMessage::with_blocks(
                ConversationRole::Tool,
                vec![MessageBlock::error("Rate limit exceeded (example error)")],
            )
            .with_timestamp("14:03"),
        );

        // Final assistant message
        conversation.push_message(
            ConversationMessage::new(
                ConversationRole::Assistant,
                "All test cases passed! The function works correctly.",
            )
            .with_timestamp("14:03"),
        );

        let state = State { conversation };
        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Conversation(m) => {
                ConversationView::update(&mut state.conversation, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        ConversationView::view(
            &state.conversation,
            &mut RenderContext::new(frame, frame.area(), &theme),
        );
    }

    fn handle_event_with_state(state: &Self::State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, Key::Esc) {
                return Some(Msg::Quit);
            }
        }

        ConversationView::handle_event(
            &state.conversation,
            event,
            &EventContext::new().focused(true),
        )
        .map(Msg::Conversation)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ConversationApp, _>::virtual_builder(80, 30).build()?;

    println!("=== Conversation View Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state:");
    println!("{}\n", vt.display());

    // Scroll up
    vt.send(Event::char('k'));
    vt.tick()?;
    println!("After scrolling up:");
    println!("{}\n", vt.display());

    // Scroll to top
    vt.send(Event::char('g'));
    vt.tick()?;
    println!("At the top:");
    println!("{}\n", vt.display());

    // Scroll to bottom
    vt.send(Event::char('G'));
    vt.tick()?;
    println!("At the bottom:");
    println!("{}\n", vt.display());

    println!(
        "Total messages: {}",
        vt.state().conversation.message_count()
    );

    Ok(())
}
