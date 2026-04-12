//! Spinner example -- animated loading indicators.
//!
//! Demonstrates the Spinner component with multiple animation styles
//! and tick-driven frame advancement.
//!
//! Run with: cargo run --example spinner --features display-components

use envision::prelude::*;

/// Application marker type.
struct SpinnerApp;

/// Application state with multiple spinners.
#[derive(Clone)]
struct State {
    dots: SpinnerState,
    line: SpinnerState,
    circle: SpinnerState,
    stopped: SpinnerState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Tick,
    Quit,
}

impl App for SpinnerApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let state = State {
            dots: SpinnerState::with_label("Loading data..."),
            line: SpinnerState::with_style(SpinnerStyle::Line),
            circle: SpinnerState::with_style(SpinnerStyle::Circle),
            stopped: {
                let mut s = SpinnerState::with_label("Paused");
                s.set_spinning(false);
                s
            },
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Tick => {
                Spinner::update(&mut state.dots, SpinnerMessage::Tick);
                Spinner::update(&mut state.line, SpinnerMessage::Tick);
                Spinner::update(&mut state.circle, SpinnerMessage::Tick);
                Spinner::update(&mut state.stopped, SpinnerMessage::Tick);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

        Spinner::view(
            &state.dots,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );
        Spinner::view(
            &state.line,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
        Spinner::view(
            &state.circle,
            &mut RenderContext::new(frame, chunks[2], &theme),
        );
        Spinner::view(
            &state.stopped,
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
    let mut vt = Runtime::<SpinnerApp, _>::virtual_terminal(40, 6)?;

    println!("=== Spinner Example ===\n");

    vt.tick()?;
    println!("Initial spinners (frame 0):");
    println!("{}\n", vt.display());

    for _ in 0..3 {
        vt.dispatch(Msg::Tick);
    }
    vt.tick()?;
    println!("After 3 ticks (frames advanced):");
    println!("{}\n", vt.display());

    Ok(())
}
