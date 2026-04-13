//! SearchableList example -- filterable list with type-to-search.
//!
//! Demonstrates the SearchableList compound component with text filtering,
//! keyboard navigation, and item selection from filtered results.
//!
//! Run with: cargo run --example searchable_list --features compound-components

use envision::prelude::*;

/// Application marker type.
struct SearchableListApp;

/// Application state.
#[derive(Clone)]
struct State {
    list: SearchableListState<String>,
    selections: Vec<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    List(SearchableListMessage),
    Quit,
}

impl App for SearchableListApp {
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
            "Elixir".to_string(),
            "Ruby".to_string(),
            "Swift".to_string(),
            "Kotlin".to_string(),
            "Scala".to_string(),
            "Zig".to_string(),
            "OCaml".to_string(),
            "Clojure".to_string(),
        ];

        let list = SearchableListState::new(items);

        let state = State {
            list,
            selections: Vec::new(),
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::List(m) => {
                if let Some(SearchableListOutput::Selected(item)) =
                    SearchableList::update(&mut state.list, m)
                {
                    state.selections.push(item);
                }
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(area);

        SearchableList::view(
            &state.list,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        // Show selection history
        let log_lines: Vec<Line> = state
            .selections
            .iter()
            .map(|s| Line::from(format!("  Selected: {}", s)))
            .collect();
        let log = ratatui::widgets::Paragraph::new(log_lines).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Selections"),
        );
        frame.render_widget(log, chunks[1]);

        let filtered = state.list.filtered_items().len();
        let status = format!(
            " {} items shown | Type to filter, Up/Down: navigate, Enter: select, q: quit",
            filtered
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.code == Key::Esc {
                return Some(Msg::Quit);
            }
        }
        SearchableList::handle_event(&state.list, event, &EventContext::new().focused(true))
            .map(Msg::List)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<SearchableListApp, _>::virtual_builder(50, 22).build()?;

    println!("=== SearchableList Example ===\n");

    // Initial render: all items shown
    vt.tick()?;
    println!("Initial state (all items):");
    println!("{}\n", vt.display());

    // Filter by typing "r"
    vt.dispatch(Msg::List(SearchableListMessage::FilterChar('r')));
    vt.tick()?;
    println!("After typing 'r' (filtering):");
    println!("{}\n", vt.display());

    // Clear filter
    vt.dispatch(Msg::List(SearchableListMessage::FilterClear));
    vt.tick()?;
    println!("After clearing filter:");
    println!("{}\n", vt.display());

    // Navigate and select
    vt.dispatch(Msg::List(SearchableListMessage::Down));
    vt.dispatch(Msg::List(SearchableListMessage::Down));
    vt.dispatch(Msg::List(SearchableListMessage::Select));
    vt.tick()?;
    println!("After selecting third item:");
    println!("{}\n", vt.display());

    Ok(())
}
