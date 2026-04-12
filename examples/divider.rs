//! Divider example -- horizontal and vertical separator lines.
//!
//! Demonstrates the Divider component with different orientations
//! and label configurations.
//!
//! Run with: cargo run --example divider --features display-components

use envision::prelude::*;

/// Application marker type.
struct DividerApp;

/// Application state with multiple dividers.
#[derive(Clone)]
struct State {
    horizontal: DividerState,
    horizontal_labeled: DividerState,
    vertical: DividerState,
    vertical_labeled: DividerState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quit,
}

impl App for DividerApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let state = State {
            horizontal: DividerState::new(),
            horizontal_labeled: DividerState::new().with_label("Settings"),
            vertical: DividerState::vertical(),
            vertical_labeled: DividerState::vertical().with_label("V"),
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

        // Top section: horizontal dividers
        let main_chunks = Layout::vertical([
            Constraint::Length(1), // horizontal, no label
            Constraint::Length(1), // spacer
            Constraint::Length(1), // horizontal, with label
            Constraint::Length(1), // spacer
            Constraint::Min(5),    // vertical dividers section
        ])
        .split(area);

        Divider::view(
            &state.horizontal,
            &mut RenderContext::new(frame, main_chunks[0], &theme),
        );
        Divider::view(
            &state.horizontal_labeled,
            &mut RenderContext::new(frame, main_chunks[2], &theme),
        );

        // Bottom section: vertical dividers side by side
        let vertical_chunks = Layout::horizontal([
            Constraint::Length(1), // vertical, no label
            Constraint::Length(2), // spacer
            Constraint::Length(1), // vertical, with label
            Constraint::Min(0),    // remainder
        ])
        .split(main_chunks[4]);

        Divider::view(
            &state.vertical,
            &mut RenderContext::new(frame, vertical_chunks[0], &theme),
        );
        Divider::view(
            &state.vertical_labeled,
            &mut RenderContext::new(frame, vertical_chunks[2], &theme),
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.key {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<DividerApp, _>::virtual_terminal(40, 10)?;

    println!("=== Divider Example ===\n");

    vt.tick()?;
    println!("Horizontal and vertical dividers:");
    println!("{}\n", vt.display());

    Ok(())
}
