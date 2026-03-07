//! LineInput example — single-line text input with visual wrapping.
//!
//! Demonstrates the LineInput component with text insertion, cursor
//! navigation, deletion, and the submit workflow. Shows placeholder
//! text and how the buffer updates in response to messages.
//!
//! Run with: cargo run --example line_input

use envision::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Application marker type.
struct LineInputApp;

/// Application state with a single LineInput and a submitted-values log.
#[derive(Clone)]
struct State {
    input: LineInputState,
    submissions: Vec<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Input(LineInputMessage),
    Quit,
}

impl App for LineInputApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut input = LineInputState::new().with_placeholder("Type something and press Enter...");
        input.set_focused(true);

        let state = State {
            input,
            submissions: Vec::new(),
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Input(m) => {
                if let Some(output) = state.input.update(m) {
                    if let LineInputOutput::Submitted(value) = output {
                        state.submissions.push(value);
                    }
                }
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        let theme = Theme::default();
        LineInput::view(&state.input, frame, chunks[0], &theme);

        // Show submissions log
        let log_lines: Vec<Line> = state
            .submissions
            .iter()
            .enumerate()
            .map(|(i, s)| Line::from(format!("  {}. {}", i + 1, s)))
            .collect();
        let log = Paragraph::new(log_lines)
            .block(Block::default().borders(Borders::ALL).title("Submissions"));
        frame.render_widget(log, chunks[1]);

        let status = Paragraph::new(format!(
            " Buffer: \"{}\" | Cursor: {} | Submissions: {} | Esc quit",
            state.input.value(),
            state.input.cursor_byte_offset(),
            state.submissions.len(),
        ))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status, chunks[2]);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Esc) {
                return Some(Msg::Quit);
            }
        }

        state.input.handle_event(event).map(Msg::Input)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<LineInputApp, _>::virtual_terminal(60, 16)?;

    println!("=== LineInput Example ===\n");

    // Initial render — shows placeholder
    vt.tick()?;
    println!("Initial view (placeholder visible):");
    println!("{}\n", vt.display());

    // Type some text
    for ch in "Hello, world!".chars() {
        vt.dispatch(Msg::Input(LineInputMessage::Insert(ch)));
    }
    vt.tick()?;
    println!("After typing \"Hello, world!\":");
    println!("{}\n", vt.display());

    // Move cursor left 6 positions to land before "world!"
    for _ in 0..6 {
        vt.dispatch(Msg::Input(LineInputMessage::Left));
    }
    for ch in "beautiful ".chars() {
        vt.dispatch(Msg::Input(LineInputMessage::Insert(ch)));
    }
    vt.tick()?;
    println!("After inserting \"beautiful \" before \"world!\":");
    println!("{}\n", vt.display());

    // Submit the text
    vt.dispatch(Msg::Input(LineInputMessage::Submit));
    vt.tick()?;
    println!("After submitting (buffer cleared, value in log):");
    println!("{}\n", vt.display());

    // Type and submit another entry
    for ch in "Second entry".chars() {
        vt.dispatch(Msg::Input(LineInputMessage::Insert(ch)));
    }
    vt.dispatch(Msg::Input(LineInputMessage::Submit));
    vt.tick()?;
    println!("After second submission:");
    println!("{}\n", vt.display());

    println!("Submissions: {:?}", vt.state().submissions);

    Ok(())
}
