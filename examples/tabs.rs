//! Tabs example -- horizontal tab navigation.
//!
//! Demonstrates the Tabs component with keyboard-driven tab
//! switching and content panels.
//!
//! Run with: cargo run --example tabs --features navigation-components

use envision::prelude::*;

/// Application marker type.
struct TabsApp;

/// Application state.
#[derive(Clone)]
struct State {
    tabs: TabsState<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Tabs(TabsMessage),
    Quit,
}

impl App for TabsApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let tabs = TabsState::new(vec![
            "Overview".to_string(),
            "Details".to_string(),
            "Settings".to_string(),
            "Logs".to_string(),
        ]);

        (State { tabs }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Tabs(m) => {
                Tabs::<String>::update(&mut state.tabs, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        Tabs::<String>::view(
            &state.tabs,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let tab_name = state
            .tabs
            .selected_item()
            .cloned()
            .unwrap_or_else(|| "None".into());
        let content_text = match tab_name.as_str() {
            "Overview" => "  System status: All services running",
            "Details" => "  CPU: 45% | Memory: 2.1 GB / 8 GB",
            "Settings" => "  Theme: Dark | Language: English",
            "Logs" => "  [INFO] Application started successfully",
            _ => "  Unknown tab",
        };
        let content = ratatui::widgets::Paragraph::new(content_text).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(tab_name),
        );
        frame.render_widget(content, chunks[1]);

        let status = " Left/Right: switch tabs, q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.key, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }
        Tabs::handle_event(&state.tabs, event, &EventContext::new().focused(true)).map(Msg::Tabs)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TabsApp, _>::virtual_terminal(60, 10)?;

    println!("=== Tabs Example ===\n");

    vt.tick()?;
    println!("Initial tabs (Overview selected):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Tabs(TabsMessage::Right));
    vt.tick()?;
    println!("After switching to Details:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::Tabs(TabsMessage::Right));
    vt.tick()?;
    println!("After switching to Settings:");
    println!("{}\n", vt.display());

    Ok(())
}
