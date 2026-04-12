//! Toast example -- notification system with severity levels.
//!
//! Demonstrates the Toast component with info, success, warning,
//! and error notifications, plus auto-dismiss via tick.
//!
//! Run with: cargo run --example toast --features display-components

use envision::prelude::*;

/// Application marker type.
struct ToastApp;

/// Application state.
#[derive(Clone)]
struct State {
    toasts: ToastState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Toast(ToastMessage),
    AddSuccess,
    AddWarning,
    AddError,
    Quit,
}

impl App for ToastApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let toasts = ToastState::with_duration(5000);
        (State { toasts }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Toast(m) => {
                Toast::update(&mut state.toasts, m);
            }
            Msg::AddSuccess => {
                state.toasts.success("Deployment completed successfully!");
            }
            Msg::AddWarning => {
                state.toasts.warning("Disk usage above 80%");
            }
            Msg::AddError => {
                state.toasts.error("Connection to database lost");
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        let content =
            ratatui::widgets::Paragraph::new(format!("  Active toasts: {}", state.toasts.len()))
                .block(
                    ratatui::widgets::Block::default()
                        .borders(ratatui::widgets::Borders::ALL)
                        .title("Toast Demo"),
                );
        frame.render_widget(content, chunks[0]);

        Toast::view(&state.toasts, &mut RenderContext::new(frame, area, &theme));

        let status = " s: success, w: warning, e: error, q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                Key::Char('s') => Some(Msg::AddSuccess),
                Key::Char('w') => Some(Msg::AddWarning),
                Key::Char('e') => Some(Msg::AddError),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ToastApp, _>::virtual_terminal(60, 16)?;

    println!("=== Toast Example ===\n");

    vt.tick()?;
    println!("Initial state (no toasts):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::AddSuccess);
    vt.dispatch(Msg::AddWarning);
    vt.dispatch(Msg::AddError);
    vt.tick()?;
    println!("After adding success, warning, and error toasts:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Toast(ToastMessage::Tick(5000)));
    vt.tick()?;
    println!("After auto-dismiss (5s elapsed):");
    println!("{}\n", vt.display());

    Ok(())
}
