//! SelectableList example -- scrollable list with keyboard navigation.
//!
//! Demonstrates the SelectableList component with selection tracking,
//! vim-style navigation, and item activation.
//!
//! Run with: cargo run --example selectable_list --features data-components

use envision::prelude::*;

/// Application marker type.
struct SelectableListApp;

/// Application state.
#[derive(Clone)]
struct State {
    list: SelectableListState<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    List(SelectableListMessage),
    Quit,
}

impl App for SelectableListApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let items = vec![
            "Rust".to_string(),
            "Python".to_string(),
            "TypeScript".to_string(),
            "Go".to_string(),
            "Java".to_string(),
            "C++".to_string(),
            "Haskell".to_string(),
        ];
        let list = SelectableListState::new(items);

        (State { list }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::List(m) => {
                SelectableList::<String>::update(&mut state.list, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        SelectableList::<String>::view(
            &state.list,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let selected = state
            .list
            .selected_item()
            .cloned()
            .unwrap_or_else(|| "None".into());
        let status = format!(
            " Selected: {} | j/k: navigate, Enter: select, q: quit",
            selected
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }
        SelectableList::handle_event(&state.list, event, &EventContext::new().focused(true))
            .map(Msg::List)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<SelectableListApp, _>::virtual_builder(40, 12).build()?;

    println!("=== SelectableList Example ===\n");

    vt.tick()?;
    println!("Initial list (Rust selected):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::List(SelectableListMessage::Down));
    vt.dispatch(Msg::List(SelectableListMessage::Down));
    vt.tick()?;
    println!("After navigating to TypeScript:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::List(SelectableListMessage::Last));
    vt.tick()?;
    println!("After jumping to last (Haskell):");
    println!("{}\n", vt.display());

    Ok(())
}
