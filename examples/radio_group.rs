//! RadioGroup example -- mutually exclusive option selection.
//!
//! Demonstrates the RadioGroup component with navigation that
//! immediately changes the selection, following radio button behavior.
//!
//! Run with: cargo run --example radio_group --features input-components

use envision::prelude::*;

/// Application marker type.
struct RadioGroupApp;

/// Application state.
#[derive(Clone)]
struct State {
    size: RadioGroupState<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Size(RadioGroupMessage),
    Quit,
}

impl App for RadioGroupApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let options = vec![
            "Small (8 oz)".to_string(),
            "Medium (12 oz)".to_string(),
            "Large (16 oz)".to_string(),
            "Extra Large (20 oz)".to_string(),
        ];
        let size = RadioGroupState::new(options);

        (State { size }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Size(m) => {
                RadioGroup::<String>::update(&mut state.size, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(6),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        RadioGroup::<String>::view(
            &state.size,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let selected = state
            .size
            .selected_item()
            .cloned()
            .unwrap_or_else(|| "None".into());
        let content = ratatui::widgets::Paragraph::new(format!("  Your selection: {}", selected))
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Order Summary"),
            );
        frame.render_widget(content, chunks[1]);

        let status = " Up/Down: choose, Enter: confirm, q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }
        RadioGroup::<String>::handle_event(&state.size, event, &EventContext::new().focused(true))
            .map(Msg::Size)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<RadioGroupApp, _>::virtual_terminal(50, 14)?;

    println!("=== RadioGroup Example ===\n");

    vt.tick()?;
    println!("Initial state (Small selected):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Size(RadioGroupMessage::Down));
    vt.tick()?;
    println!("After selecting Medium:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Size(RadioGroupMessage::Down));
    vt.tick()?;
    println!("After selecting Large:");
    println!("{}\n", vt.display());

    Ok(())
}
