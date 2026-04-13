//! ConfirmDialog example -- interactive modal confirmation with button navigation.
//!
//! Demonstrates the ConfirmDialog overlay component with Yes/No and Ok/Cancel
//! button configurations, keyboard navigation between buttons, and result handling.
//! Shows how to layer a modal dialog over background content and route events
//! conditionally based on dialog visibility.
//!
//! Controls:
//!   d           Show "Delete File?" dialog (Yes/No)
//!   s           Show "Save Changes?" dialog (Ok/Cancel)
//!   Left/Right  Navigate between dialog buttons
//!   Enter       Confirm selected button
//!   Esc/q       Quit (when no dialog is open)
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
                if let Some(ConfirmDialogOutput::Confirmed(result)) =
                    ConfirmDialog::update(&mut state.dialog, m)
                {
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

        // Background content showing dialog results
        let mut log_lines: Vec<Line> = vec![Line::from(
            "  Press 'd' for a delete dialog, 's' for a save dialog.",
        )];
        log_lines.push(Line::from(""));
        for result in &state.results {
            log_lines.push(Line::from(format!("  {}", result)));
        }
        let log = ratatui::widgets::Paragraph::new(log_lines).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Dialog Results"),
        );
        frame.render_widget(log, chunks[0]);

        // Overlay dialog when visible
        if ConfirmDialog::is_visible(&state.dialog) {
            ConfirmDialog::view(&state.dialog, &mut RenderContext::new(frame, area, &theme));
        }

        let status = " d: delete dialog | s: save dialog | q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        // If dialog is visible, route events to it
        if ConfirmDialog::is_visible(&state.dialog) {
            return ConfirmDialog::handle_event(
                &state.dialog,
                event,
                &EventContext::new().focused(true),
            )
            .map(Msg::Dialog);
        }

        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                Key::Char('d') => Some(Msg::ShowDeleteDialog),
                Key::Char('s') => Some(Msg::ShowSaveDialog),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> envision::Result<()> {
    let _final_state = TerminalRuntime::<ConfirmDialogApp>::terminal_builder()?
        .build()?
        .run_terminal()
        .await?;
    Ok(())
}
