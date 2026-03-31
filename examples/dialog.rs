//! Dialog example -- modal dialog with configurable buttons.
//!
//! Demonstrates the Dialog component with custom buttons,
//! button navigation, and press handling.
//!
//! Run with: cargo run --example dialog --features overlay-components

use envision::prelude::*;

/// Application marker type.
struct DialogApp;

/// Application state.
#[derive(Clone)]
struct State {
    dialog: DialogState,
    last_result: Option<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Dialog(DialogMessage),
    ShowDialog,
    Quit,
}

impl App for DialogApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let dialog = DialogState::new(
            "Unsaved Changes",
            "You have unsaved changes.\nWhat would you like to do?",
            vec![
                DialogButton::new("discard", "Discard"),
                DialogButton::new("save", "Save"),
                DialogButton::new("cancel", "Cancel"),
            ],
        );

        let state = State {
            dialog,
            last_result: None,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Dialog(m) => {
                if let Some(output) = Dialog::update(&mut state.dialog, m) {
                    match output {
                        DialogOutput::ButtonPressed(id) => {
                            state.last_result = Some(format!("Pressed: {}", id));
                        }
                        DialogOutput::Closed => {
                            state.last_result = Some("Dialog closed".into());
                        }
                        _ => {}
                    }
                }
            }
            Msg::ShowDialog => {
                Dialog::show(&mut state.dialog);
                state.dialog.set_focused(true);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        let result_text = state.last_result.as_deref().unwrap_or("No action yet");
        let content = ratatui::widgets::Paragraph::new(format!("  Result: {}", result_text)).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Dialog Demo"),
        );
        frame.render_widget(content, chunks[0]);

        if Dialog::is_visible(&state.dialog) {
            Dialog::view(&state.dialog, frame, area, &theme);
        }

        let status = " d: show dialog, q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if Dialog::is_visible(&state.dialog) {
            return state.dialog.handle_event(event).map(Msg::Dialog);
        }
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
                KeyCode::Char('d') => Some(Msg::ShowDialog),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<DialogApp, _>::virtual_terminal(60, 14)?;

    println!("=== Dialog Example ===\n");

    vt.tick()?;
    println!("Initial state (no dialog):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::ShowDialog);
    vt.tick()?;
    println!("After showing dialog:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Dialog(DialogMessage::FocusNext));
    vt.dispatch(Msg::Dialog(DialogMessage::Press));
    vt.tick()?;
    println!("After pressing Save:");
    println!("{}\n", vt.display());

    Ok(())
}
