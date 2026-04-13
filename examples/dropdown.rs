//! Dropdown example — searchable/filterable select with type-to-filter.
//!
//! Demonstrates the Dropdown component with keyboard-driven selection,
//! type-to-filter functionality, and open/close toggling.
//!
//! Run with: cargo run --example dropdown

use envision::prelude::*;

/// Application marker type.
struct DropdownApp;

/// Application state.
#[derive(Clone)]
struct State {
    language: DropdownState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Language(DropdownMessage),
    Quit,
}

impl App for DropdownApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let languages = vec![
            "Rust",
            "Python",
            "TypeScript",
            "Go",
            "Java",
            "C++",
            "Haskell",
            "Elixir",
            "Zig",
        ];

        let language = DropdownState::new(languages).with_placeholder("Select language...");

        (State { language }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Language(m) => {
                Dropdown::update(&mut state.language, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

        Dropdown::view(
            &state.language,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let selected = state.language.selected_value().unwrap_or("None");
        let status = format!(" Selected: {}", selected);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.code == Key::Esc && !state.language.is_open() {
                return Some(Msg::Quit);
            }
        }
        Dropdown::handle_event(&state.language, event, &EventContext::new().focused(true))
            .map(Msg::Language)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<DropdownApp, _>::virtual_builder(40, 15).build()?;

    println!("=== Dropdown Example ===\n");

    // Initial render: closed dropdown
    vt.tick()?;
    println!("Initial (closed, no selection):");
    println!("{}\n", vt.display());

    // Open the dropdown
    vt.dispatch(Msg::Language(DropdownMessage::Toggle));
    vt.tick()?;
    println!("After Toggle (open, showing options):");
    println!("{}\n", vt.display());

    // Navigate down and select
    vt.dispatch(Msg::Language(DropdownMessage::Down));
    vt.dispatch(Msg::Language(DropdownMessage::Down));
    vt.dispatch(Msg::Language(DropdownMessage::Confirm));
    vt.tick()?;
    println!("After selecting 'TypeScript':");
    println!("{}\n", vt.display());

    Ok(())
}
