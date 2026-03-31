//! Chat application composing ChatView message display with LineInput.
//!
//! Demonstrates component composition: ChatView renders the message
//! history while LineInput provides a single-line input with visual
//! wrapping at the bottom. Enter submits a user message, and a
//! simulated assistant echo response appears.
//!
//! Run with: cargo run --example chat_app

use envision::component::{
    ChatView, ChatViewState, Component, LineInput, LineInputMessage, LineInputOutput,
    LineInputState,
};
use envision::prelude::*;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

struct ChatApp;

#[derive(Clone)]
struct State {
    chat: ChatViewState,
    input: LineInputState,
    input_focused: bool,
}

#[derive(Clone, Debug)]
enum Msg {
    /// A LineInput message to process.
    Input(LineInputMessage),
    /// LineInput produced output (submitted, changed, etc.)
    InputOutput(LineInputOutput),
    /// Quit the application.
    Quit,
}

impl App for ChatApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut chat = ChatViewState::new();
        chat.push_system("Welcome! Type a message and press Enter.");

        let input = LineInputState::new().with_placeholder("Type a message...");

        let state = State {
            chat,
            input,
            input_focused: true,
        };
        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Input(m) => {
                if let Some(output) = LineInput::update(&mut state.input, m) {
                    return Self::update(state, Msg::InputOutput(output));
                }
            }
            Msg::InputOutput(output) => match output {
                LineInputOutput::Submitted(text) => {
                    if !text.trim().is_empty() {
                        state.chat.push_user(&text);
                        // Simulated echo response
                        state.chat.push_assistant(format!("You said: \"{}\"", text));
                    }
                }
                LineInputOutput::Changed(_) | LineInputOutput::Copied(_) => {}
                _ => {}
            },
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Min(0),    // Chat history
            Constraint::Length(3), // LineInput
            Constraint::Length(1), // Status bar
        ])
        .split(area);

        let theme = Theme::default();

        // Render chat history (using ChatView but without its built-in input)
        ChatView::view(&state.chat, frame, chunks[0], &theme);

        // Render LineInput
        LineInput::view(&state.input, frame, chunks[1], &theme);

        // Status bar
        let status = Paragraph::new(format!(
            " Messages: {} | Enter: send | Esc: quit",
            state.chat.message_count()
        ))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status, chunks[2]);
    }

    fn handle_event_with_state(state: &Self::State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Esc) {
                return Some(Msg::Quit);
            }
        }

        if state.input_focused {
            state.input.handle_event(event).map(Msg::Input)
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ChatApp, _>::virtual_terminal(60, 20)?;

    println!("=== Chat App Example ===\n");

    // Initial render — shows welcome message
    vt.tick()?;
    println!("Initial state:");
    println!("{}\n", vt.display());

    // Simulate typing "Hello, world!"
    for c in "Hello, world!".chars() {
        vt.dispatch(Msg::Input(LineInputMessage::Insert(c)));
    }
    vt.tick()?;
    println!("After typing a message:");
    println!("{}\n", vt.display());

    // Submit
    vt.dispatch(Msg::Input(LineInputMessage::Submit));
    vt.tick()?;
    println!("After submitting:");
    println!("{}\n", vt.display());

    // Type and submit another message
    for c in "How are you?".chars() {
        vt.dispatch(Msg::Input(LineInputMessage::Insert(c)));
    }
    vt.dispatch(Msg::Input(LineInputMessage::Submit));
    vt.tick()?;
    println!("After second message:");
    println!("{}\n", vt.display());

    println!("Total messages: {}", vt.state().chat.message_count());

    Ok(())
}
