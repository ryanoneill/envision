//! Box plot example -- statistical distribution comparison.
//!
//! Demonstrates the BoxPlot component with simulated latency data
//! across multiple services, showing median, quartiles, and outliers.
//!
//! Run with: cargo run --example box_plot --features compound-components

use envision::prelude::*;

/// Application marker type.
struct BoxPlotApp;

/// Application state with a box plot.
#[derive(Clone)]
struct State {
    box_plot: BoxPlotState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

impl App for BoxPlotApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        // Simulated P50/P95/P99 latency data across services
        let datasets = vec![
            BoxPlotData::new("Auth API", 5.0, 12.0, 18.0, 28.0, 45.0)
                .with_color(Color::Cyan)
                .with_outliers(vec![2.0, 65.0, 80.0]),
            BoxPlotData::new("User API", 8.0, 15.0, 22.0, 35.0, 52.0)
                .with_color(Color::Green)
                .with_outliers(vec![3.0, 70.0]),
            BoxPlotData::new("Search API", 12.0, 25.0, 40.0, 60.0, 85.0)
                .with_color(Color::Yellow)
                .with_outliers(vec![5.0, 120.0, 150.0]),
            BoxPlotData::new("Payment API", 15.0, 30.0, 50.0, 75.0, 100.0)
                .with_color(Color::Red)
                .with_outliers(vec![200.0]),
        ];

        let box_plot = BoxPlotState::new(datasets)
            .with_title("Service Latency Distribution (ms)")
            .with_show_outliers(true);

        (State { box_plot }, Command::none())
    }

    fn update(_state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => Command::quit(),
        }
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        BoxPlot::view(
            &state.box_plot,
            &mut RenderContext::new(frame, area, &theme),
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
    let mut vt = Runtime::<BoxPlotApp, _>::virtual_terminal(70, 22)?;

    println!("=== Box Plot Example ===\n");

    // Render box plot
    vt.tick()?;
    println!("Service latency comparison (simulated data):");
    println!("{}\n", vt.display());

    Ok(())
}
