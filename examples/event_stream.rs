//! EventStream example -- structured event feed with severity filtering.
//!
//! Demonstrates the EventStream compound component with structured events
//! from simulated services, level filtering, and text search.
//!
//! Run with: cargo run --example event_stream --features compound-components

use envision::prelude::*;

/// Application marker type.
struct EventStreamApp;

/// Application state.
#[derive(Clone)]
struct State {
    stream: EventStreamState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Stream(EventStreamMessage),
    Quit,
}

impl App for EventStreamApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let mut stream = EventStreamState::new()
            .with_title("System Events")
            .with_visible_columns(vec![]);

        // Simulate structured events from multiple services
        push_event(
            &mut stream,
            100.0,
            EventLevel::Info,
            "Service starting",
            "api",
            vec![("version", "2.1.0")],
        );
        push_event(
            &mut stream,
            101.0,
            EventLevel::Debug,
            "Loading configuration",
            "api",
            vec![("path", "/etc/app/config.toml")],
        );
        push_event(
            &mut stream,
            102.0,
            EventLevel::Info,
            "Database connected",
            "db",
            vec![("host", "localhost"), ("port", "5432")],
        );
        push_event(
            &mut stream,
            200.0,
            EventLevel::Info,
            "Request received",
            "api",
            vec![("method", "GET"), ("path", "/api/users")],
        );
        push_event(
            &mut stream,
            201.0,
            EventLevel::Debug,
            "Query executed",
            "db",
            vec![("ms", "12"), ("table", "users")],
        );
        push_event(
            &mut stream,
            202.0,
            EventLevel::Info,
            "Response sent",
            "api",
            vec![("status", "200"), ("ms", "45")],
        );
        push_event(
            &mut stream,
            300.0,
            EventLevel::Warning,
            "Cache miss",
            "cache",
            vec![("key", "user:123")],
        );
        push_event(
            &mut stream,
            301.0,
            EventLevel::Info,
            "Request received",
            "api",
            vec![("method", "POST"), ("path", "/api/orders")],
        );
        push_event(
            &mut stream,
            302.0,
            EventLevel::Warning,
            "Slow query detected",
            "db",
            vec![("ms", "1200"), ("table", "orders")],
        );
        push_event(
            &mut stream,
            400.0,
            EventLevel::Error,
            "Connection timeout",
            "api",
            vec![("ms", "5000"), ("endpoint", "payment")],
        );
        push_event(
            &mut stream,
            401.0,
            EventLevel::Info,
            "Retry attempt",
            "api",
            vec![("attempt", "1"), ("endpoint", "payment")],
        );
        push_event(
            &mut stream,
            402.0,
            EventLevel::Info,
            "Payment processed",
            "api",
            vec![("order", "1234"), ("amount", "$99.00")],
        );
        push_event(
            &mut stream,
            500.0,
            EventLevel::Warning,
            "Memory usage high",
            "monitor",
            vec![("percent", "85"), ("threshold", "80")],
        );
        push_event(
            &mut stream,
            501.0,
            EventLevel::Trace,
            "GC cycle completed",
            "runtime",
            vec![("freed_mb", "128")],
        );

        (State { stream }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Stream(m) => {
                EventStream::update(&mut state.stream, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        EventStream::view(
            &state.stream,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let visible = state.stream.visible_events().len();
        let total = state.stream.event_count();
        let status = format!(
            " {}/{} events | /: search, 1-5: level filter, f: auto-scroll, Up/Down: scroll, q: quit",
            visible, total
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.code == Key::Char('q') && !state.stream.is_search_focused() {
                return Some(Msg::Quit);
            }
        }
        EventStream::handle_event(&state.stream, event, &EventContext::new().focused(true))
            .map(Msg::Stream)
    }
}

/// Helper to push a structured event with source and fields.
fn push_event(
    stream: &mut EventStreamState,
    timestamp: f64,
    level: EventLevel,
    message: &str,
    source: &str,
    fields: Vec<(&str, &str)>,
) {
    let id = stream.event_count() as u64;
    let event = StreamEvent::new(id, timestamp, level, message).with_source(source);
    let event = fields
        .into_iter()
        .fold(event, |e, (k, v)| e.with_field(k, v));
    EventStream::update(stream, EventStreamMessage::PushEvent(event));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<EventStreamApp, _>::virtual_builder(90, 22).build()?;

    println!("=== EventStream Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state (all events visible):");
    println!("{}\n", vt.display());

    // Filter to warnings and above
    vt.dispatch(Msg::Stream(EventStreamMessage::QuickLevelFilter(4)));
    vt.tick()?;
    println!("After filtering to >= Warning:");
    println!("{}\n", vt.display());

    // Clear filter and search for "api"
    vt.dispatch(Msg::Stream(EventStreamMessage::QuickLevelFilter(4)));
    vt.dispatch(Msg::Stream(EventStreamMessage::SetFilter(
        "api".to_string(),
    )));
    vt.tick()?;
    println!("After text search for 'api':");
    println!("{}\n", vt.display());

    Ok(())
}
