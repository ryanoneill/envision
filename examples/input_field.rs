//! InputField example -- text input with placeholder and submit.
//!
//! Demonstrates the InputField component with text editing,
//! placeholder text, and submit behavior.
//!
//! Run with: cargo run --example input_field --features input-components

use envision::prelude::*;

/// Application marker type.
struct InputFieldApp;

/// Application state with an input field and submitted values.
#[derive(Clone)]
struct State {
    input: InputFieldState,
    submitted: Vec<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Input(InputFieldMessage),
    Quit,
}

impl App for InputFieldApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let input = InputFieldState::with_placeholder("Enter your name...");

        let state = State {
            input,
            submitted: Vec::new(),
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Input(m) => {
                if let Some(InputFieldOutput::Submitted(value)) =
                    InputField::update(&mut state.input, m)
                {
                    if !value.is_empty() {
                        state.submitted.push(value);
                        InputField::update(&mut state.input, InputFieldMessage::Clear);
                    }
                }
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        InputField::view(
            &state.input,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        // Show submitted values
        let submitted_text = if state.submitted.is_empty() {
            "  No values submitted yet. Type and press Enter.".to_string()
        } else {
            state
                .submitted
                .iter()
                .enumerate()
                .map(|(i, v)| format!("  {}. {}", i + 1, v))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let submitted_widget = ratatui::widgets::Paragraph::new(submitted_text).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Submitted Values"),
        );
        frame.render_widget(submitted_widget, chunks[1]);

        let status = format!(
            " Value: \"{}\" | Cursor: {} | Enter: submit, q: quit",
            state.input.value(),
            state.input.cursor_position()
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.code == Key::Esc {
                return Some(Msg::Quit);
            }
        }
        InputField::handle_event(&state.input, event, &EventContext::new().focused(true))
            .map(Msg::Input)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<InputFieldApp, _>::virtual_builder(55, 14).build()?;

    println!("=== InputField Example ===\n");

    // Initial render (shows placeholder)
    vt.tick()?;
    println!("Initial state (placeholder shown):");
    println!("{}\n", vt.display());

    // Type some text
    vt.dispatch(Msg::Input(InputFieldMessage::Insert('A')));
    vt.dispatch(Msg::Input(InputFieldMessage::Insert('l')));
    vt.dispatch(Msg::Input(InputFieldMessage::Insert('i')));
    vt.dispatch(Msg::Input(InputFieldMessage::Insert('c')));
    vt.dispatch(Msg::Input(InputFieldMessage::Insert('e')));
    vt.tick()?;
    println!("After typing 'Alice':");
    println!("{}\n", vt.display());

    // Submit
    vt.dispatch(Msg::Input(InputFieldMessage::Submit));
    vt.tick()?;
    println!("After submitting 'Alice':");
    println!("{}\n", vt.display());

    // Type and submit another value
    vt.dispatch(Msg::Input(InputFieldMessage::Insert('B')));
    vt.dispatch(Msg::Input(InputFieldMessage::Insert('o')));
    vt.dispatch(Msg::Input(InputFieldMessage::Insert('b')));
    vt.dispatch(Msg::Input(InputFieldMessage::Submit));
    vt.tick()?;
    println!("After submitting 'Bob':");
    println!("{}\n", vt.display());

    Ok(())
}
