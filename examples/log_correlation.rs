//! LogCorrelation example -- side-by-side time-aligned log streams.
//!
//! Demonstrates the LogCorrelation compound component with two simulated
//! services whose log entries are aligned by timestamp, with independent
//! per-stream filtering and synchronized scrolling.
//!
//! Run with: cargo run --example log_correlation --features compound-components

use envision::prelude::*;

/// Application marker type.
struct LogCorrelationApp;

/// Application state.
#[derive(Clone)]
struct State {
    correlation: LogCorrelationState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Correlation(LogCorrelationMessage),
    Quit,
}

impl App for LogCorrelationApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let api = LogStream::new("API Server")
            .with_color(Color::Cyan)
            .with_entry(CorrelationEntry::new(
                43201.0,
                CorrelationLevel::Info,
                "Request received: GET /api/users",
            ))
            .with_entry(CorrelationEntry::new(
                43201.0,
                CorrelationLevel::Debug,
                "Parsing request headers",
            ))
            .with_entry(CorrelationEntry::new(
                43202.0,
                CorrelationLevel::Info,
                "Forwarding query to database",
            ))
            .with_entry(CorrelationEntry::new(
                43203.0,
                CorrelationLevel::Info,
                "Response sent: 200 OK",
            ))
            .with_entry(CorrelationEntry::new(
                43203.0,
                CorrelationLevel::Warning,
                "Slow response: 1200ms",
            ))
            .with_entry(CorrelationEntry::new(
                43205.0,
                CorrelationLevel::Info,
                "Request received: POST /api/orders",
            ))
            .with_entry(CorrelationEntry::new(
                43206.0,
                CorrelationLevel::Error,
                "Payment gateway timeout",
            ))
            .with_entry(CorrelationEntry::new(
                43207.0,
                CorrelationLevel::Info,
                "Retrying payment request",
            ))
            .with_entry(CorrelationEntry::new(
                43208.0,
                CorrelationLevel::Info,
                "Payment processed on retry",
            ));

        let db = LogStream::new("Database")
            .with_color(Color::Green)
            .with_entry(CorrelationEntry::new(
                43201.0,
                CorrelationLevel::Info,
                "Connection established",
            ))
            .with_entry(CorrelationEntry::new(
                43202.0,
                CorrelationLevel::Info,
                "Query start: SELECT * FROM users",
            ))
            .with_entry(CorrelationEntry::new(
                43202.0,
                CorrelationLevel::Debug,
                "Query plan: sequential scan",
            ))
            .with_entry(CorrelationEntry::new(
                43203.0,
                CorrelationLevel::Info,
                "Query complete: 42 rows",
            ))
            .with_entry(CorrelationEntry::new(
                43203.0,
                CorrelationLevel::Warning,
                "Slow query: 800ms",
            ))
            .with_entry(CorrelationEntry::new(
                43205.0,
                CorrelationLevel::Info,
                "INSERT INTO orders",
            ))
            .with_entry(CorrelationEntry::new(
                43206.0,
                CorrelationLevel::Info,
                "Transaction committed",
            ))
            .with_entry(CorrelationEntry::new(
                43208.0,
                CorrelationLevel::Info,
                "Connection pool: 5 active",
            ));

        let correlation = LogCorrelationState::new()
            .with_title("Log Correlation")
            .with_streams(vec![api, db]);

        (State { correlation }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Correlation(m) => {
                LogCorrelation::update(&mut state.correlation, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        LogCorrelation::view(
            &state.correlation,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let status = " Tab: switch stream | j/k: scroll | s: toggle sync | q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.code == Key::Char('q') {
                return Some(Msg::Quit);
            }
        }
        LogCorrelation::handle_event(
            &state.correlation,
            event,
            &EventContext::new().focused(true),
        )
        .map(Msg::Correlation)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<LogCorrelationApp, _>::virtual_terminal(85, 20)?;

    println!("=== LogCorrelation Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state (two services, time-aligned):");
    println!("{}\n", vt.display());

    // Scroll down
    vt.dispatch(Msg::Correlation(LogCorrelationMessage::ScrollDown));
    vt.dispatch(Msg::Correlation(LogCorrelationMessage::ScrollDown));
    vt.tick()?;
    println!("After scrolling down:");
    println!("{}\n", vt.display());

    // Switch active stream
    vt.dispatch(Msg::Correlation(LogCorrelationMessage::FocusNextStream));
    vt.tick()?;
    println!("After switching to Database stream:");
    println!("{}\n", vt.display());

    // Toggle sync scroll
    vt.dispatch(Msg::Correlation(LogCorrelationMessage::ToggleSyncScroll));
    vt.tick()?;
    println!("After toggling sync scroll OFF:");
    println!("{}\n", vt.display());

    Ok(())
}
