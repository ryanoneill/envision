//! UsageDisplay example -- metric display component.
//!
//! Demonstrates the UsageDisplay component for showing system metrics
//! in horizontal, vertical, and grid layouts.
//!
//! Run with: cargo run --example usage_display --features display-components

use envision::prelude::*;

/// Application marker type.
struct UsageDisplayApp;

/// Application state.
#[derive(Clone)]
struct State {
    horizontal: UsageDisplayState,
    vertical: UsageDisplayState,
    grid: UsageDisplayState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

fn system_metrics() -> Vec<UsageMetric> {
    vec![
        UsageMetric::new("CPU", "45%").with_color(Color::Green),
        UsageMetric::new("Memory", "3.2 GB").with_color(Color::Yellow),
        UsageMetric::new("Disk", "120 GB").with_color(Color::Cyan),
        UsageMetric::new("Network", "1.5 Mbps").with_color(Color::Magenta),
    ]
}

impl App for UsageDisplayApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let metrics = system_metrics();
        let state = State {
            horizontal: UsageDisplayState::with_metrics(metrics.clone())
                .with_layout(UsageLayout::Horizontal),
            vertical: UsageDisplayState::with_metrics(metrics.clone())
                .with_layout(UsageLayout::Vertical)
                .with_title("System Metrics"),
            grid: UsageDisplayState::with_metrics(metrics)
                .with_layout(UsageLayout::Grid(2))
                .with_title("Grid View"),
        };
        (state, Command::none())
    }

    fn update(_state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => Command::quit(),
        }
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

        // Title
        let title = ratatui::widgets::Paragraph::new("UsageDisplay Component Demo").style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(title, chunks[0]);

        // Horizontal layout
        UsageDisplay::view(&state.horizontal, frame, chunks[1], &theme);

        // Vertical layout
        UsageDisplay::view(&state.vertical, frame, chunks[2], &theme);

        // Spacer line
        let spacer = ratatui::widgets::Paragraph::new("");
        frame.render_widget(spacer, chunks[3]);

        // Grid layout
        UsageDisplay::view(&state.grid, frame, chunks[4], &theme);
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<UsageDisplayApp, _>::virtual_terminal(60, 20)?;

    println!("=== UsageDisplay Example ===\n");

    vt.tick()?;
    println!("All three layouts:");
    println!("{}\n", vt.display());

    Ok(())
}
