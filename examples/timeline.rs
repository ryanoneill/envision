//! Timeline example -- horizontal event and span visualization.
//!
//! Demonstrates the Timeline component with point events and duration spans
//! resembling an HTTP request tracing view (Jaeger/Zipkin style).
//!
//! Run with: cargo run --example timeline --features compound-components

use envision::prelude::*;

/// Application marker type.
struct TimelineApp;

/// Application state.
#[derive(Clone)]
struct State {
    timeline: TimelineState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

impl App for TimelineApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        // Simulate an HTTP request trace
        let events = vec![
            TimelineEvent::new("req-start", 0.0, "Request Start").with_color(Color::Green),
            TimelineEvent::new("dns-done", 50.0, "DNS Resolved").with_color(Color::Yellow),
            TimelineEvent::new("tls-done", 150.0, "TLS Handshake").with_color(Color::Yellow),
            TimelineEvent::new("resp-start", 600.0, "First Byte").with_color(Color::Cyan),
            TimelineEvent::new("resp-end", 950.0, "Complete").with_color(Color::Green),
        ];

        let spans = vec![
            TimelineSpan::new("dns", 0.0, 50.0, "DNS Lookup")
                .with_color(Color::Blue)
                .with_lane(0),
            TimelineSpan::new("tls", 50.0, 150.0, "TLS Setup")
                .with_color(Color::Magenta)
                .with_lane(0),
            TimelineSpan::new("request", 150.0, 600.0, "Server Processing")
                .with_color(Color::Cyan)
                .with_lane(0),
            TimelineSpan::new("db-1", 200.0, 350.0, "DB Query 1")
                .with_color(Color::Red)
                .with_lane(1),
            TimelineSpan::new("db-2", 400.0, 550.0, "DB Query 2")
                .with_color(Color::Red)
                .with_lane(1),
            TimelineSpan::new("cache", 180.0, 220.0, "Cache")
                .with_color(Color::Green)
                .with_lane(2),
            TimelineSpan::new("response", 600.0, 950.0, "Response Transfer")
                .with_color(Color::Yellow)
                .with_lane(0),
        ];

        let timeline = TimelineState::new()
            .with_events(events)
            .with_spans(spans)
            .with_title("HTTP Request Trace")
            .with_view_range(-50.0, 1050.0);

        let state = State { timeline };
        (state, Command::none())
    }

    fn update(_state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => Command::quit(),
        }
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        Timeline::view(
            &state.timeline,
            frame,
            frame.area(),
            &theme,
            &ViewContext::default(),
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TimelineApp, _>::virtual_terminal(80, 20)?;

    println!("=== Timeline Example ===\n");

    // Render initial state
    vt.tick()?;
    println!("HTTP Request Trace Timeline:");
    println!("{}\n", vt.display());

    // Select first event
    vt.send(Event::key(KeyCode::Down));
    vt.tick()?;
    println!("After selecting first event:");
    println!("{}\n", vt.display());

    // Navigate to a span
    for _ in 0..5 {
        vt.send(Event::key(KeyCode::Down));
        vt.tick()?;
    }
    println!("After selecting a span:");
    println!("{}\n", vt.display());

    Ok(())
}
