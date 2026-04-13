//! Enhanced chart example -- area charts, scatter plots, and threshold lines.
//!
//! Demonstrates the new Chart component features: area charts, scatter plots,
//! threshold/reference lines, and manual Y-axis scaling.
//!
//! Run with: cargo run --example chart_enhanced --features compound-components

use envision::prelude::*;

/// Application marker type.
struct ChartEnhancedApp;

/// Application state with area and scatter charts.
#[derive(Clone)]
struct State {
    area_chart: ChartState,
    scatter_chart: ChartState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

impl App for ChartEnhancedApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        // Area chart: CPU usage with SLO threshold lines
        let cpu_series = DataSeries::new(
            "CPU",
            vec![45.0, 52.0, 48.0, 65.0, 72.0, 58.0, 61.0, 55.0, 80.0, 92.0],
        )
        .with_color(Color::Cyan);
        let mem_series = DataSeries::new(
            "Memory",
            vec![30.0, 32.0, 35.0, 38.0, 36.0, 40.0, 42.0, 41.0, 44.0, 46.0],
        )
        .with_color(Color::Magenta);

        let area_chart = ChartState::area(vec![cpu_series, mem_series])
            .with_title("System Metrics (Area)")
            .with_threshold(90.0, "Warning", Color::Yellow)
            .with_threshold(95.0, "Critical", Color::Red)
            .with_y_range(0.0, 100.0);

        // Scatter chart: request latency with SLO threshold
        let latency_series = DataSeries::new(
            "Latency",
            vec![50.0, 120.0, 80.0, 200.0, 90.0, 150.0, 110.0, 75.0],
        )
        .with_color(Color::Green);

        let scatter_chart = ChartState::scatter(vec![latency_series])
            .with_title("Request Latency (Scatter)")
            .with_threshold(100.0, "SLO", Color::Yellow)
            .with_y_range(0.0, 250.0);

        let state = State {
            area_chart,
            scatter_chart,
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
        let chunks =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

        Chart::view(
            &state.area_chart,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );
        Chart::view(
            &state.scatter_chart,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ChartEnhancedApp, _>::virtual_builder(70, 30).build()?;

    println!("=== Enhanced Chart Example ===\n");

    // Render charts
    vt.tick()?;
    println!("Area chart with thresholds and scatter plot:");
    println!("{}\n", vt.display());

    Ok(())
}
