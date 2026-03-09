//! Table example -- data table with row selection and column sorting.
//!
//! Demonstrates the Table component with custom row types,
//! keyboard navigation, and sortable columns.
//!
//! Run with: cargo run --example table --features data-components

use envision::prelude::*;

/// A row in the table.
#[derive(Clone, Debug, PartialEq)]
struct Language {
    name: String,
    year: String,
    paradigm: String,
}

impl TableRow for Language {
    fn cells(&self) -> Vec<String> {
        vec![self.name.clone(), self.year.clone(), self.paradigm.clone()]
    }
}

/// Application marker type.
struct TableApp;

/// Application state.
#[derive(Clone)]
struct State {
    table: TableState<Language>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Table(TableMessage),
    Quit,
}

impl App for TableApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let languages = vec![
            Language {
                name: "Rust".into(),
                year: "2015".into(),
                paradigm: "Systems".into(),
            },
            Language {
                name: "Python".into(),
                year: "1991".into(),
                paradigm: "Scripting".into(),
            },
            Language {
                name: "Haskell".into(),
                year: "1990".into(),
                paradigm: "Functional".into(),
            },
            Language {
                name: "Go".into(),
                year: "2012".into(),
                paradigm: "Systems".into(),
            },
            Language {
                name: "TypeScript".into(),
                year: "2012".into(),
                paradigm: "Scripting".into(),
            },
        ];

        let columns = vec![
            Column::new("Language", Constraint::Percentage(30)).sortable(),
            Column::new("Year", Constraint::Percentage(20)).sortable(),
            Column::new("Paradigm", Constraint::Percentage(50)),
        ];

        let mut table = TableState::new(languages, columns);
        table.set_focused(true);

        (State { table }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Table(m) => {
                Table::<Language>::update(&mut state.table, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        Table::<Language>::view(&state.table, frame, chunks[0], &theme);

        let selected = state
            .table
            .selected_item()
            .map(|l| l.name.clone())
            .unwrap_or_else(|| "None".into());
        let status = format!(
            " Selected: {} | j/k: navigate, s: sort by name, q: quit",
            selected
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
                KeyCode::Char('s') => return Some(Msg::Table(TableMessage::SortBy(0))),
                _ => {}
            }
        }
        state.table.handle_event(event).map(Msg::Table)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TableApp, _>::virtual_terminal(60, 12)?;

    println!("=== Table Example ===\n");

    vt.tick()?;
    println!("Initial table:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Table(TableMessage::Down));
    vt.dispatch(Msg::Table(TableMessage::Down));
    vt.tick()?;
    println!("After navigating to Haskell:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Table(TableMessage::SortBy(0)));
    vt.tick()?;
    println!("After sorting by Language (ascending):");
    println!("{}\n", vt.display());

    Ok(())
}
