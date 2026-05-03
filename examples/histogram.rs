//! Histogram example -- frequency distribution visualization.
//!
//! Demonstrates the Histogram component with simulated latency data,
//! showing how raw continuous data is automatically binned and displayed.
//!
//! Run with: cargo run --example histogram --features compound-components

use envision::prelude::*;

/// Application marker type.
struct HistogramApp;

/// Application state with a histogram.
#[derive(Clone)]
struct State {
    histogram: HistogramState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

impl App for HistogramApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        // Simulated latency data (ms) with a roughly normal distribution
        let latency_data = vec![
            12.0, 15.0, 18.0, 20.0, 22.0, 23.0, 24.0, 25.0, 25.0, 26.0, 27.0, 28.0, 28.0, 29.0,
            30.0, 30.0, 31.0, 31.0, 32.0, 32.0, 33.0, 33.0, 33.0, 34.0, 34.0, 35.0, 35.0, 35.0,
            36.0, 36.0, 37.0, 38.0, 39.0, 40.0, 42.0, 45.0, 48.0, 52.0, 58.0, 65.0,
        ];

        let histogram = HistogramState::with_data(latency_data)
            .with_bin_count(8)
            .with_title("API Latency Distribution (ms)")
            .with_x_label("Latency (ms)")
            .with_y_label("Frequency")
            .with_color(Color::Cyan)
            .with_show_counts(true);

        (State { histogram }, Command::none())
    }

    fn update(_state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quit => Command::quit(),
        }
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        Histogram::view(
            &state.histogram,
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
    let mut vt = Runtime::<HistogramApp, _>::virtual_builder(70, 20).build()?;

    println!("=== Histogram Example ===\n");

    // Render histogram
    vt.tick()?;
    println!("API latency distribution (simulated data):");
    println!("{}\n", vt.display());

    Ok(())
}
