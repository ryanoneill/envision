//! LoadingList example -- list items with loading and error states.
//!
//! Demonstrates the LoadingList component with per-item loading indicators,
//! error states, and keyboard navigation through items.
//!
//! Run with: cargo run --example loading_list --features data-components

use envision::prelude::*;

/// A task item to display in the list.
#[derive(Clone, Debug)]
struct Task {
    name: String,
}

/// Application marker type.
struct LoadingListApp;

/// Application state.
#[derive(Clone)]
struct State {
    list: LoadingListState<Task>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    List(LoadingListMessage<Task>),
    Quit,
}

impl App for LoadingListApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let tasks = vec![
            Task {
                name: "Compile project".into(),
            },
            Task {
                name: "Run test suite".into(),
            },
            Task {
                name: "Deploy to staging".into(),
            },
            Task {
                name: "Run integration tests".into(),
            },
            Task {
                name: "Deploy to production".into(),
            },
        ];

        let list = LoadingListState::with_items(tasks, |t| t.name.clone());

        (State { list }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::List(m) => {
                LoadingList::update(&mut state.list, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        LoadingList::view(
            &state.list,
            frame,
            chunks[0],
            &theme,
            &ViewContext::default(),
        );

        let selected = state
            .list
            .selected_item()
            .map(|item| item.data().name.clone())
            .unwrap_or_else(|| "None".into());
        let status = format!(" Selected: {} | Up/Down: navigate, q: quit", selected);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                return Some(Msg::Quit);
            }
        }
        LoadingList::<Task>::handle_event(&state.list, event, &ViewContext::new().focused(true))
            .map(Msg::List)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<LoadingListApp, _>::virtual_terminal(55, 12)?;

    println!("=== LoadingList Example ===\n");

    // Initial render: all items ready
    vt.tick()?;
    println!("Initial state (all items ready):");
    println!("{}\n", vt.display());

    // Set first item to loading
    vt.dispatch(Msg::List(LoadingListMessage::SetLoading(0)));
    vt.tick()?;
    println!("After setting 'Compile project' to loading:");
    println!("{}\n", vt.display());

    // Complete first item, set second to loading
    vt.dispatch(Msg::List(LoadingListMessage::SetReady(0)));
    vt.dispatch(Msg::List(LoadingListMessage::SetLoading(1)));
    vt.tick()?;
    println!("After completing first, loading second:");
    println!("{}\n", vt.display());

    // Set third item to error
    vt.dispatch(Msg::List(LoadingListMessage::SetReady(1)));
    vt.dispatch(Msg::List(LoadingListMessage::SetError {
        index: 2,
        message: "Connection timeout".to_string(),
    }));
    vt.tick()?;
    println!("After setting 'Deploy to staging' to error:");
    println!("{}\n", vt.display());

    // Navigate down
    vt.dispatch(Msg::List(LoadingListMessage::Down));
    vt.dispatch(Msg::List(LoadingListMessage::Down));
    vt.tick()?;
    println!("After navigating down:");
    println!("{}\n", vt.display());

    Ok(())
}
