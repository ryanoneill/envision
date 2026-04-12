//! Sparkline example -- compact inline data trend display.
//!
//! Demonstrates the Sparkline component with sample data,
//! push operations, and display limit features.
//!
//! Run with: cargo run --example sparkline --features display-components

use envision::prelude::*;

/// Application marker type.
struct SparklineApp;

/// Application state with multiple sparklines.
#[derive(Clone)]
struct State {
    basic: SparklineState,
    titled: SparklineState,
    rtl: SparklineState,
    limited: SparklineState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

impl App for SparklineApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let data = vec![
            0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0,
        ];

        let state = State {
            basic: SparklineState::with_data(data.clone()),
            titled: SparklineState::with_data(data.clone()).with_title("CPU Usage"),
            rtl: SparklineState::with_data(data.clone())
                .with_direction(SparklineDirection::RightToLeft),
            limited: SparklineState::with_data(data).with_max_display_points(8),
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
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

        Sparkline::view(
            &state.basic,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );
        Sparkline::view(
            &state.titled,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
        Sparkline::view(
            &state.rtl,
            &mut RenderContext::new(frame, chunks[2], &theme),
        );
        Sparkline::view(
            &state.limited,
            &mut RenderContext::new(frame, chunks[3], &theme),
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
    let mut vt = Runtime::<SparklineApp, _>::virtual_terminal(40, 8)?;

    println!("=== Sparkline Example ===\n");

    vt.tick()?;
    println!("Basic sparkline (raw data):");
    println!("{}\n", vt.display());

    Ok(())
}
