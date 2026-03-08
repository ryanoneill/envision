//! ConfirmDialog example -- modal confirmation with button navigation.
//!
//! Demonstrates the ConfirmDialog overlay component with Yes/No buttons,
//! keyboard navigation, and result handling.
//!
//! Run with: cargo run --example confirm_dialog --features overlay-components

use envision::prelude::*;

/// Application marker type.
struct ConfirmDialogApp;

/// Application state.
#[derive(Clone)]
struct State {
    dialog: ConfirmDialogState,
    results: Vec<String>,
    dialog_count: usize,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Dialog(ConfirmDialogMessage),
    ShowDeleteDialog,
    ShowSaveDialog,
    Quit,
}

impl App for ConfirmDialogApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let dialog = ConfirmDialogState::yes_no(
            "Delete File?",
            "Are you sure you want to delete 'important.txt'?\nThis action cannot be undone.",
        );

        let state = State {
            dialog,
            results: Vec::new(),
            dialog_count: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Dialog(m) => {
                if let Some(output) = ConfirmDialog::update(&mut state.dialog, m) {
                    if let ConfirmDialogOutput::Confirmed(result) = output {
                        let result_str = match result {
                            ConfirmDialogResult::Yes => "Yes",
                            ConfirmDialogResult::No => "No",
                            ConfirmDialogResult::Ok => "Ok",
                            ConfirmDialogResult::Cancel => "Cancel",
                        };
                        state.results.push(format!(
                            "Dialog {}: {}",
                            state.dialog_count + 1,
                            result_str
                        ));
                        state.dialog_count += 1;
                    }
                }
            }
            Msg::ShowDeleteDialog => {
                state.dialog = ConfirmDialogState::yes_no(
                    "Delete File?",
                    "Are you sure you want to delete 'important.txt'?\nThis action cannot be undone.",
                );
                ConfirmDialog::show(&mut state.dialog);
            }
            Msg::ShowSaveDialog => {
                state.dialog = ConfirmDialogState::ok_cancel(
                    "Save Changes?",
                    "You have unsaved changes.\nWould you like to save before closing?",
                );
                ConfirmDialog::show(&mut state.dialog);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        // Background content
        let log_lines: Vec<Line> = state
            .results
            .iter()
            .map(|s| Line::from(format!("  {}", s)))
            .collect();
        let log = ratatui::widgets::Paragraph::new(log_lines).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Dialog Results"),
        );
        frame.render_widget(log, chunks[0]);

        // Overlay dialog when visible
        if ConfirmDialog::is_visible(&state.dialog) {
            ConfirmDialog::view(&state.dialog, frame, area, &theme);
        }

        let status = " d: delete dialog, s: save dialog, q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        // If dialog is visible, route events to it
        if ConfirmDialog::is_visible(&state.dialog) {
            return state.dialog.handle_event(event).map(Msg::Dialog);
        }

        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
                KeyCode::Char('d') => Some(Msg::ShowDeleteDialog),
                KeyCode::Char('s') => Some(Msg::ShowSaveDialog),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ConfirmDialogApp, _>::virtual_terminal(60, 18)?;

    println!("=== ConfirmDialog Example ===\n");

    // Initial render (no dialog visible)
    vt.tick()?;
    println!("Initial state (no dialog):");
    println!("{}\n", vt.display());

    // Show delete dialog
    vt.dispatch(Msg::ShowDeleteDialog);
    vt.tick()?;
    println!("After showing delete dialog:");
    println!("{}\n", vt.display());

    // Select "Yes"
    vt.dispatch(Msg::Dialog(ConfirmDialogMessage::SelectResult(
        ConfirmDialogResult::Yes,
    )));
    vt.tick()?;
    println!("After confirming Yes:");
    println!("{}\n", vt.display());

    // Show save dialog
    vt.dispatch(Msg::ShowSaveDialog);
    vt.tick()?;
    println!("After showing save dialog:");
    println!("{}\n", vt.display());

    // Select "Cancel"
    vt.dispatch(Msg::Dialog(ConfirmDialogMessage::SelectResult(
        ConfirmDialogResult::Cancel,
    )));
    vt.tick()?;
    println!("After cancelling:");
    println!("{}\n", vt.display());

    Ok(())
}
