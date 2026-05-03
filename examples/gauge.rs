//! Gauge example -- ratio and measurement display.
//!
//! Demonstrates the Gauge component with full and line variants,
//! different threshold zones, and units display.
//!
//! Run with: cargo run --example gauge --features display-components

use envision::prelude::*;

/// Application marker type.
struct GaugeApp;

/// Application state with multiple gauges.
#[derive(Clone)]
struct State {
    cpu: GaugeState,
    memory: GaugeState,
    disk: GaugeState,
    network: GaugeState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    UpdateCpu(f64),
    UpdateMemory(f64),
    Quit,
}

impl App for GaugeApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let state = State {
            cpu: GaugeState::new(45.0, 100.0)
                .with_units("%")
                .with_title("CPU Usage"),
            memory: GaugeState::new(6144.0, 16384.0)
                .with_units("MB")
                .with_title("Memory"),
            disk: GaugeState::new(750.0, 1000.0)
                .with_units("GB")
                .with_title("Disk")
                .with_variant(GaugeVariant::Line),
            network: GaugeState::new(85.0, 100.0)
                .with_units("Mbps")
                .with_title("Network")
                .with_variant(GaugeVariant::Line)
                .with_thresholds(vec![
                    ThresholdZone {
                        above: 0.0,
                        color: Color::Red,
                    },
                    ThresholdZone {
                        above: 0.3,
                        color: Color::Yellow,
                    },
                    ThresholdZone {
                        above: 0.6,
                        color: Color::Green,
                    },
                ]),
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::UpdateCpu(value) => {
                Gauge::update(&mut state.cpu, GaugeMessage::SetValue(value));
            }
            Msg::UpdateMemory(value) => {
                Gauge::update(&mut state.memory, GaugeMessage::SetValue(value));
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

        Gauge::view(
            &state.cpu,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );
        Gauge::view(
            &state.memory,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
        Gauge::view(
            &state.disk,
            &mut RenderContext::new(frame, chunks[2], &theme),
        );
        Gauge::view(
            &state.network,
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
    let mut vt = Runtime::<GaugeApp, _>::virtual_builder(60, 14).build()?;

    println!("=== Gauge Example ===\n");

    vt.tick()?;
    println!("Initial state:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::UpdateCpu(78.0));
    vt.tick()?;
    println!("After CPU spike to 78% (yellow zone):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::UpdateCpu(95.0));
    vt.dispatch(Msg::UpdateMemory(14000.0));
    vt.tick()?;
    println!("After CPU critical (95%, red) and memory high:");
    println!("{}\n", vt.display());

    Ok(())
}
