//! MetricsDashboard example -- grid of metric widgets.
//!
//! Demonstrates the MetricsDashboard component with counters,
//! gauges, status indicators, and keyboard navigation.
//!
//! Run with: cargo run --example metrics_dashboard --features compound-components

use envision::prelude::*;

/// Application marker type.
struct MetricsDashboardApp;

/// Application state.
#[derive(Clone)]
struct State {
    dashboard: MetricsDashboardState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Dashboard(MetricsDashboardMessage),
    Quit,
}

impl App for MetricsDashboardApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut dashboard = MetricsDashboardState::new(
            vec![
                MetricWidget::counter("Requests", 1284),
                MetricWidget::gauge("CPU %", 67, 100),
                MetricWidget::gauge("Memory", 3200, 8192),
                MetricWidget::status("API", true),
                MetricWidget::status("Database", true),
                MetricWidget::text("Version", "2.4.1"),
            ],
            3,
        )
        .with_title("System Metrics");
        dashboard.set_focused(true);

        (State { dashboard }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Dashboard(m) => {
                MetricsDashboard::update(&mut state.dashboard, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        MetricsDashboard::view(&state.dashboard, frame, chunks[0], &theme);

        let selected = state
            .dashboard
            .selected_widget()
            .map(|w| w.label().to_string())
            .unwrap_or_else(|| "None".into());
        let status = format!(" Selected: {} | Arrow keys: navigate, q: quit", selected);
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
        state.dashboard.handle_event(event).map(Msg::Dashboard)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<MetricsDashboardApp, _>::virtual_terminal(66, 14)?;

    println!("=== MetricsDashboard Example ===\n");

    vt.tick()?;
    println!("Dashboard with 6 widgets in 3 columns:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Dashboard(MetricsDashboardMessage::Right));
    vt.tick()?;
    println!("After navigating to CPU %:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Dashboard(MetricsDashboardMessage::Down));
    vt.tick()?;
    println!("After navigating down to Database:");
    println!("{}\n", vt.display());

    Ok(())
}
