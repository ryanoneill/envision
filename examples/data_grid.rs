//! DataGrid example — editable data table with cell navigation.
//!
//! Demonstrates the DataGrid compound component with row selection,
//! cell editing, and keyboard navigation through a tabular data set.
//!
//! Run with: cargo run --example data_grid --features compound-components

use envision::prelude::*;

/// A row in the data grid.
#[derive(Clone, Debug)]
struct Employee {
    name: String,
    department: String,
    role: String,
}

impl TableRow for Employee {
    fn cells(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.department.clone(),
            self.role.clone(),
        ]
    }
}

/// Application marker type.
struct DataGridApp;

/// Application state.
#[derive(Clone)]
struct State {
    grid: DataGridState<Employee>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Grid(DataGridMessage),
    Quit,
}

impl App for DataGridApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let employees = vec![
            Employee {
                name: "Alice Chen".into(),
                department: "Engineering".into(),
                role: "Senior Developer".into(),
            },
            Employee {
                name: "Bob Smith".into(),
                department: "Design".into(),
                role: "UI Designer".into(),
            },
            Employee {
                name: "Carol Davis".into(),
                department: "Engineering".into(),
                role: "Tech Lead".into(),
            },
        ];

        let columns = vec![
            Column::new("Name", Constraint::Percentage(30)),
            Column::new("Department", Constraint::Percentage(30)),
            Column::new("Role", Constraint::Percentage(40)),
        ];

        let mut grid = DataGridState::new(employees, columns);
        grid.set_focused(true);

        (State { grid }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Grid(m) => {
                DataGrid::update(&mut state.grid, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        DataGrid::view(&state.grid, frame, chunks[0], &theme);

        let selected = state
            .grid
            .selected_item()
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "None".into());
        let status = format!(" Selected: {}", selected);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status)
                .style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if !state.grid.is_editing() {
            if let Some(key) = event.as_key() {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    return Some(Msg::Quit);
                }
            }
        }
        state.grid.handle_event(event).map(Msg::Grid)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<DataGridApp, _>::virtual_terminal(70, 12)?;

    println!("=== DataGrid Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial data grid:");
    println!("{}\n", vt.display());

    // Navigate down
    vt.dispatch(Msg::Grid(DataGridMessage::Down));
    vt.tick()?;
    println!("After selecting second row:");
    println!("{}\n", vt.display());

    Ok(())
}
