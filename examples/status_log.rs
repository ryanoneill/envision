//! StatusLog example -- scrolling status messages with severity levels.
//!
//! Demonstrates the StatusLog component for displaying application
//! status messages with Info, Success, Warning, and Error levels.
//!
//! Run with: cargo run --example status_log --features display-components

use envision::prelude::*;

/// Application marker type.
struct StatusLogApp;

/// Application state.
#[derive(Clone)]
struct State {
    log: StatusLogState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Log(StatusLogMessage),
    Quit,
}

impl App for StatusLogApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut log = StatusLogState::new()
            .with_title("Status Log")
            .with_max_entries(20);

        // Add initial messages
        log.info("Application starting...");
        log.info("Loading configuration");
        log.success("Configuration loaded");
        log.info("Connecting to database");

        (State { log }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Log(m) => {
                StatusLog::update(&mut state.log, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        StatusLog::view(
            &state.log,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let status = format!(" Entries: {} | Up/Down: scroll, q: quit", state.log.len());
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                return Some(Msg::Quit);
            }
        }
        StatusLog::handle_event(&state.log, event, &EventContext::new().focused(true)).map(Msg::Log)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<StatusLogApp, _>::virtual_terminal(55, 14)?;

    println!("=== StatusLog Example ===\n");

    // Initial render with some messages
    vt.tick()?;
    println!("Initial state (startup messages):");
    println!("{}\n", vt.display());

    // Add more messages including warning and error
    vt.dispatch(Msg::Log(StatusLogMessage::Push {
        message: "Database connected".to_string(),
        level: StatusLogLevel::Success,
        timestamp: None,
    }));
    vt.dispatch(Msg::Log(StatusLogMessage::Push {
        message: "Slow query detected (2.5s)".to_string(),
        level: StatusLogLevel::Warning,
        timestamp: None,
    }));
    vt.dispatch(Msg::Log(StatusLogMessage::Push {
        message: "Cache miss rate high".to_string(),
        level: StatusLogLevel::Warning,
        timestamp: None,
    }));
    vt.dispatch(Msg::Log(StatusLogMessage::Push {
        message: "Failed to send email notification".to_string(),
        level: StatusLogLevel::Error,
        timestamp: None,
    }));
    vt.dispatch(Msg::Log(StatusLogMessage::Push {
        message: "Retry succeeded for email".to_string(),
        level: StatusLogLevel::Success,
        timestamp: None,
    }));
    vt.tick()?;
    println!("After adding more messages:");
    println!("{}\n", vt.display());

    // Scroll down to see older messages
    vt.dispatch(Msg::Log(StatusLogMessage::ScrollDown));
    vt.dispatch(Msg::Log(StatusLogMessage::ScrollDown));
    vt.dispatch(Msg::Log(StatusLogMessage::ScrollDown));
    vt.tick()?;
    println!("After scrolling down:");
    println!("{}\n", vt.display());

    Ok(())
}
