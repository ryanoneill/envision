//! AlertPanel example -- threshold-based metric alerting.
//!
//! Demonstrates the AlertPanel component with metrics at various alert
//! levels, keyboard navigation, and state transitions.
//!
//! Run with: cargo run --example alert_panel --features compound-components

use envision::prelude::*;

/// Application marker type.
struct AlertPanelApp;

/// Application state.
#[derive(Clone)]
struct State {
    panel: AlertPanelState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Panel(AlertPanelMessage),
    Quit,
}

impl App for AlertPanelApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let panel = AlertPanelState::new()
            .with_metrics(vec![
                AlertMetric::new("cpu", "CPU Usage", AlertThreshold::new(70.0, 90.0))
                    .with_units("%")
                    .with_value(45.2),
                AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
                    .with_units("%")
                    .with_value(82.5),
                AlertMetric::new("disk", "Disk I/O", AlertThreshold::new(100.0, 200.0))
                    .with_units("MB/s")
                    .with_value(12.0),
                AlertMetric::new("errors", "Error Rate", AlertThreshold::new(1.0, 5.0))
                    .with_units("%")
                    .with_value(6.2),
                AlertMetric::new("latency", "API Latency", AlertThreshold::new(200.0, 500.0))
                    .with_units("ms")
                    .with_value(150.0),
                AlertMetric::new("conns", "Connections", AlertThreshold::new(800.0, 1000.0))
                    .with_value(420.0),
            ])
            .with_columns(3)
            .with_title("Infrastructure Alerts");

        (State { panel }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Panel(m) => {
                state.panel.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        AlertPanel::view(
            &state.panel,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let selected = state
            .panel
            .selected_metric()
            .map(|m| m.name().to_string())
            .unwrap_or_else(|| "None".into());
        let status = format!(
            " Selected: {} | Arrow keys: navigate, Enter: select, q: quit",
            selected
        );
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
        AlertPanel::handle_event(&state.panel, event, &EventContext::new().focused(true))
            .map(Msg::Panel)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<AlertPanelApp, _>::virtual_terminal(80, 20)?;

    println!("=== AlertPanel Example ===\n");

    vt.tick()?;
    println!("Alert panel with 6 metrics (3 columns):");
    println!("{}\n", vt.display());

    // Navigate right to Memory
    vt.dispatch(Msg::Panel(AlertPanelMessage::SelectNext));
    vt.tick()?;
    println!("After navigating to Memory:");
    println!("{}\n", vt.display());

    // Navigate down
    vt.dispatch(Msg::Panel(AlertPanelMessage::SelectDown));
    vt.tick()?;
    println!("After navigating down:");
    println!("{}\n", vt.display());

    // Update CPU to trigger a state change
    vt.dispatch(Msg::Panel(AlertPanelMessage::UpdateMetric {
        id: "cpu".into(),
        value: 92.0,
    }));
    vt.tick()?;
    println!("After CPU goes critical (92%):");
    println!("{}\n", vt.display());

    Ok(())
}
