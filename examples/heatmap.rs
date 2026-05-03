//! Heatmap example -- 2D color-intensity grid visualization.
//!
//! Demonstrates the Heatmap component with simulated error rates
//! by hour of day and day of week. Navigate with arrow keys.
//!
//! Run with: cargo run --example heatmap --features compound-components

use envision::prelude::*;

/// Application marker type.
struct HeatmapApp;

/// Application state.
#[derive(Clone)]
struct State {
    heatmap: HeatmapState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

impl App for HeatmapApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        // Simulated error rates: rows = hours (00h, 06h, 12h, 18h), cols = days
        let data = vec![
            vec![0.02, 0.01, 0.03, 0.01, 0.02, 0.08, 0.05],
            vec![0.05, 0.04, 0.06, 0.03, 0.04, 0.02, 0.01],
            vec![0.12, 0.15, 0.18, 0.14, 0.11, 0.03, 0.02],
            vec![0.08, 0.10, 0.09, 0.12, 0.07, 0.04, 0.03],
        ];

        let heatmap = HeatmapState::with_data(data)
            .with_row_labels(vec!["00h".into(), "06h".into(), "12h".into(), "18h".into()])
            .with_col_labels(vec![
                "Mon".into(),
                "Tue".into(),
                "Wed".into(),
                "Thu".into(),
                "Fri".into(),
                "Sat".into(),
                "Sun".into(),
            ])
            .with_color_scale(HeatmapColorScale::GreenToRed)
            .with_range(0.0, 0.20)
            .with_show_values(true)
            .with_title("Error Rates by Hour and Day");

        let state = State { heatmap };
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
        Heatmap::view(&state.heatmap, &mut RenderContext::new(frame, area, &theme));
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => return Some(Msg::Quit),
                _ => {}
            }
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<HeatmapApp, _>::virtual_builder(60, 12).build()?;

    println!("=== Heatmap Example ===\n");

    // Render heatmap
    vt.tick()?;
    println!("Error rates by hour and day of week:");
    println!("{}\n", vt.display());

    // Navigate to a cell
    vt.send(Event::key(Key::Down));
    vt.send(Event::key(Key::Right));
    vt.send(Event::key(Key::Right));
    vt.tick()?;
    println!("After navigating to (1, 2):");
    println!("{}\n", vt.display());

    Ok(())
}
