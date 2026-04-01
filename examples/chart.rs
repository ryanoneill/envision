//! Chart example -- data visualization with line and bar charts.
//!
//! Demonstrates the Chart component with multiple data series,
//! both line chart (sparkline) and bar chart visualizations.
//!
//! Run with: cargo run --example chart --features compound-components

use envision::prelude::*;

/// Application marker type.
struct ChartApp;

/// Application state with two charts side by side.
#[derive(Clone)]
struct State {
    line_chart: ChartState,
    bar_chart: ChartState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

impl App for ChartApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        // Line chart: CPU and memory usage over time
        let cpu_series =
            DataSeries::new("CPU", vec![45.0, 52.0, 48.0, 65.0, 72.0, 58.0, 61.0, 55.0])
                .with_color(Color::Cyan);
        let mem_series = DataSeries::new(
            "Memory",
            vec![30.0, 32.0, 35.0, 38.0, 36.0, 40.0, 42.0, 41.0],
        )
        .with_color(Color::Magenta);
        let line_chart =
            ChartState::line(vec![cpu_series, mem_series]).with_title("System Metrics (Line)");

        // Bar chart: monthly sales data
        let sales_series = DataSeries::new("Sales", vec![120.0, 95.0, 140.0, 110.0, 165.0, 130.0])
            .with_color(Color::Green);
        let bar_chart =
            ChartState::bar_vertical(vec![sales_series]).with_title("Monthly Sales (Bar)");

        let state = State {
            line_chart,
            bar_chart,
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
            &state.line_chart,
            frame,
            chunks[0],
            &theme,
            &ViewContext::default(),
        );
        Chart::view(
            &state.bar_chart,
            frame,
            chunks[1],
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
    let mut vt = Runtime::<ChartApp, _>::virtual_terminal(70, 20)?;

    println!("=== Chart Example ===\n");

    // Render charts
    vt.tick()?;
    println!("Line chart (CPU/Memory) and bar chart (Sales):");
    println!("{}\n", vt.display());

    Ok(())
}
