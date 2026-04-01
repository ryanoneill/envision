//! ChatView example -- chat interface with messages and scrolling.
//!
//! Demonstrates the ChatView component with multi-role messages,
//! input handling, and message submission.
//!
//! Run with: cargo run --example chat_view --features compound-components

use envision::prelude::*;

/// Application marker type.
struct ChatViewApp;

/// Application state with a chat view.
#[derive(Clone)]
struct State {
    chat: ChatViewState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Chat(ChatViewMessage),
    Quit,
}

impl App for ChatViewApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut chat = ChatViewState::new()
            .with_placeholder("Type a message...")
            .with_input_height(3);

        chat.set_focused(true);
        chat.push_system("Welcome to the chat!");
        chat.push_assistant("Hello! How can I help you today?");

        let state = State { chat };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Chat(m) => {
                if let Some(ChatViewOutput::Submitted(text)) = ChatView::update(&mut state.chat, m)
                {
                    // Add the user message
                    state.chat.push_user(&text);
                    // Simulate an assistant reply
                    let reply = format!("You said: \"{}\"", text);
                    state.chat.push_assistant(&reply);
                }
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        ChatView::view(
            &state.chat,
            frame,
            chunks[0],
            &theme,
            &ViewContext::default(),
        );

        let status = format!(
            " Messages: {} | Ctrl+Enter: send, Tab: toggle focus, Esc: quit",
            state.chat.message_count()
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.code == KeyCode::Esc {
                return Some(Msg::Quit);
            }
        }
        state.chat.handle_event(event).map(Msg::Chat)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ChatViewApp, _>::virtual_terminal(60, 20)?;

    println!("=== ChatView Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state (system + assistant messages):");
    println!("{}\n", vt.display());

    // Type a message
    vt.dispatch(Msg::Chat(ChatViewMessage::Input('H')));
    vt.dispatch(Msg::Chat(ChatViewMessage::Input('e')));
    vt.dispatch(Msg::Chat(ChatViewMessage::Input('l')));
    vt.dispatch(Msg::Chat(ChatViewMessage::Input('l')));
    vt.dispatch(Msg::Chat(ChatViewMessage::Input('o')));
    vt.tick()?;
    println!("After typing 'Hello':");
    println!("{}\n", vt.display());

    // Submit the message
    vt.dispatch(Msg::Chat(ChatViewMessage::Submit));
    vt.tick()?;
    println!("After submitting the message:");
    println!("{}\n", vt.display());

    // Toggle to history view
    vt.dispatch(Msg::Chat(ChatViewMessage::ToggleFocus));
    vt.tick()?;
    println!("After toggling to history view:");
    println!("{}\n", vt.display());

    Ok(())
}
