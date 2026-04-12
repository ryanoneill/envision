//! TextArea example — multi-line text editor with scrolling.
//!
//! Demonstrates the TextArea component with cursor navigation,
//! line editing, and text insertion.
//!
//! Run with: cargo run --example text_area

use envision::prelude::*;

/// Application marker type.
struct TextAreaApp;

/// Application state wrapping a single TextArea.
#[derive(Clone)]
struct State {
    editor: TextAreaState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Editor(TextAreaMessage),
    Quit,
}

impl App for TextAreaApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let initial_text = "Hello, TextArea!\nEdit this content.\nLine three.";
        let editor = TextAreaState::new().with_value(initial_text);

        (State { editor }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Editor(m) => {
                TextArea::update(&mut state.editor, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        TextArea::view(
            &state.editor,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let (row, col) = state.editor.cursor_position();
        let lines = state.editor.line_count();
        let status = format!(" Ln {}, Col {} | {} lines", row + 1, col + 1, lines);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.key == Key::Esc {
                return Some(Msg::Quit);
            }
        }
        TextArea::handle_event(&state.editor, event, &EventContext::new().focused(true))
            .map(Msg::Editor)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TextAreaApp, _>::virtual_terminal(50, 10)?;

    println!("=== TextArea Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial content:");
    println!("{}\n", vt.display());

    // Type some text
    vt.dispatch(Msg::Editor(TextAreaMessage::Insert('!')));
    vt.tick()?;
    println!("After inserting '!' at cursor:");
    println!("{}\n", vt.display());

    // Navigate to start
    vt.dispatch(Msg::Editor(TextAreaMessage::Home));
    vt.tick()?;
    println!("After Home (cursor at line start):");
    println!("{}\n", vt.display());

    Ok(())
}
