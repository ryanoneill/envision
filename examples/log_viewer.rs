//! LogViewer example -- searchable log viewer with severity filtering.
//!
//! Demonstrates the LogViewer compound component with log entries at
//! different severity levels, filtering by level, text search, regex
//! search, follow mode, and search history.
//!
//! Run with: cargo run --example log_viewer --features compound-components

use envision::prelude::*;

/// Application marker type.
struct LogViewerApp;

/// Application state.
#[derive(Clone)]
struct State {
    viewer: LogViewerState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Viewer(LogViewerMessage),
    Quit,
}

impl App for LogViewerApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut viewer = LogViewerState::new();

        // Populate with realistic log entries
        viewer.push_info("Application starting up");
        viewer.push_info("Loading configuration from /etc/myapp/config.toml");
        viewer.push_success("Configuration loaded successfully");
        viewer.push_info("Connecting to database at localhost:5432");
        viewer.push_success("Database connection established");
        viewer.push_info("Starting HTTP server on port 8080");
        viewer.push_warning("TLS certificate expires in 7 days");
        viewer.push_success("HTTP server listening on 0.0.0.0:8080");
        viewer.push_info("Processing incoming request: GET /api/users");
        viewer.push_info("Query executed in 45ms");
        viewer.push_warning("Response time exceeded threshold: 250ms");
        viewer.push_error("Failed to connect to cache server: connection refused");
        viewer.push_info("Falling back to direct database queries");
        viewer.push_warning("Memory usage at 85% of available");
        viewer.push_info("Processing incoming request: POST /api/orders");
        viewer.push_success("Order #1234 created successfully");
        viewer.push_error("Payment gateway timeout after 30s");
        viewer.push_info("Retrying payment for order #1234");
        viewer.push_success("Payment processed on retry");

        (State { viewer }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Viewer(m) => {
                LogViewer::update(&mut state.viewer, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        LogViewer::view(
            &state.viewer,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let visible = state.viewer.visible_entries().len();
        let follow = if state.viewer.follow() { "ON" } else { "OFF" };
        let regex = if state.viewer.use_regex() {
            "ON"
        } else {
            "OFF"
        };
        let status = format!(
            " {} entries | follow:{} regex:{} | /: search, f: follow, 1-4: levels, q: quit",
            visible, follow, regex
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.code == Key::Char('q') && !state.viewer.is_search_focused() {
                return Some(Msg::Quit);
            }
        }
        LogViewer::handle_event(&state.viewer, event, &EventContext::new().focused(true))
            .map(Msg::Viewer)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<LogViewerApp, _>::virtual_builder(75, 18).build()?;

    println!("=== LogViewer Example ===\n");

    // Initial render: all log entries visible
    vt.tick()?;
    println!("Initial state (all entries visible, follow mode ON):");
    println!("{}\n", vt.display());

    // Scroll down (disables follow)
    vt.dispatch(Msg::Viewer(LogViewerMessage::ScrollDown));
    vt.dispatch(Msg::Viewer(LogViewerMessage::ScrollDown));
    vt.dispatch(Msg::Viewer(LogViewerMessage::ScrollDown));
    vt.tick()?;
    println!("After scrolling down (follow mode OFF):");
    println!("{}\n", vt.display());

    // Toggle follow back on
    vt.dispatch(Msg::Viewer(LogViewerMessage::ToggleFollow));
    vt.tick()?;
    println!("After toggling follow mode back ON:");
    println!("{}\n", vt.display());

    // Filter to errors only
    vt.dispatch(Msg::Viewer(LogViewerMessage::ToggleInfo));
    vt.dispatch(Msg::Viewer(LogViewerMessage::ToggleSuccess));
    vt.dispatch(Msg::Viewer(LogViewerMessage::ToggleWarning));
    vt.tick()?;
    println!("After filtering to errors only:");
    println!("{}\n", vt.display());

    // Show all again
    vt.dispatch(Msg::Viewer(LogViewerMessage::ToggleInfo));
    vt.dispatch(Msg::Viewer(LogViewerMessage::ToggleSuccess));
    vt.dispatch(Msg::Viewer(LogViewerMessage::ToggleWarning));
    vt.tick()?;
    println!("After showing all levels again:");
    println!("{}\n", vt.display());

    Ok(())
}
