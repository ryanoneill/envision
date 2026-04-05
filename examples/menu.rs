//! Menu example -- horizontal menu bar with keyboard navigation.
//!
//! Demonstrates the Menu component with navigable items,
//! disabled entries, and selection handling.
//!
//! Run with: cargo run --example menu --features navigation-components

use envision::prelude::*;

/// Application marker type.
struct MenuApp;

/// Application state.
#[derive(Clone)]
struct State {
    menu: MenuState,
    last_selected: Option<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Menu(MenuMessage),
    Quit,
}

impl App for MenuApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let menu = MenuState::new(vec![
            MenuItem::new("File"),
            MenuItem::new("Edit"),
            MenuItem::new("View"),
            MenuItem::disabled("Tools"),
            MenuItem::new("Help"),
        ]);

        let state = State {
            menu,
            last_selected: None,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Menu(m) => {
                if let Some(MenuOutput::Selected(idx)) = Menu::update(&mut state.menu, m) {
                    let label = state.menu.items()[idx].label().to_string();
                    state.last_selected = Some(label);
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
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        Menu::view(
            &state.menu,
            frame,
            chunks[0],
            &theme,
            &ViewContext::default(),
        );

        let selected_text = state.last_selected.as_deref().unwrap_or("None");
        let content =
            ratatui::widgets::Paragraph::new(format!("  Last activated: {}", selected_text)).block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Menu Demo"),
            );
        frame.render_widget(content, chunks[1]);

        let status = " Left/Right: navigate, Enter: select, q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                return Some(Msg::Quit);
            }
        }
        Menu::handle_event(&state.menu, event, &ViewContext::new().focused(true)).map(Msg::Menu)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<MenuApp, _>::virtual_terminal(60, 10)?;

    println!("=== Menu Example ===\n");

    vt.tick()?;
    println!("Initial menu (File selected):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Menu(MenuMessage::Right));
    vt.tick()?;
    println!("After navigating to Edit:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Menu(MenuMessage::Select));
    vt.tick()?;
    println!("After activating Edit:");
    println!("{}\n", vt.display());

    Ok(())
}
