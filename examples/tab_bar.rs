//! TabBar example -- rich tab bar with closable, modified, and icon-decorated tabs.
//!
//! Demonstrates the TabBar component with keyboard-driven tab switching,
//! closing, and adding tabs.
//!
//! Run with: cargo run --example tab_bar --features navigation-components

use envision::prelude::*;

/// Application marker type.
struct TabBarApp;

/// Application state.
#[derive(Clone)]
struct State {
    tab_bar: TabBarState,
    next_id: usize,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    TabBar(TabBarMessage),
    AddNewTab,
    Quit,
}

impl App for TabBarApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let tabs = vec![
            Tab::new("1", "main.rs").with_icon("R").with_closable(true),
            Tab::new("2", "lib.rs")
                .with_icon("R")
                .with_closable(true)
                .with_modified(true),
            Tab::new("3", "style.css")
                .with_icon("C")
                .with_closable(true),
            Tab::new("4", "index.html")
                .with_icon("H")
                .with_closable(true),
        ];
        let tab_bar = TabBarState::new(tabs);

        (
            State {
                tab_bar,
                next_id: 5,
            },
            Command::none(),
        )
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::TabBar(m) => {
                TabBar::update(&mut state.tab_bar, m);
            }
            Msg::AddNewTab => {
                let id = state.next_id.to_string();
                state.next_id += 1;
                let tab = Tab::new(id, format!("new_{}.rs", state.next_id - 1))
                    .with_icon("R")
                    .with_closable(true);
                TabBar::update(&mut state.tab_bar, TabBarMessage::AddTab(tab));
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

        TabBar::view(
            &state.tab_bar,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let tab_name = state
            .tab_bar
            .active_tab()
            .map(|t| t.label().to_string())
            .unwrap_or_else(|| "No tab selected".into());
        let modified_str = state
            .tab_bar
            .active_tab()
            .map(|t| if t.modified() { " [modified]" } else { "" })
            .unwrap_or("");
        let content_text = format!("  Editing: {}{}", tab_name, modified_str);
        let content = ratatui::widgets::Paragraph::new(content_text).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(tab_name),
        );
        frame.render_widget(content, chunks[1]);

        let status = " Left/Right: switch | w: close | n: new | q: quit";
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
            if matches!(key.code, Key::Char('n')) {
                return Some(Msg::AddNewTab);
            }
        }
        TabBar::handle_event(&state.tab_bar, event, &EventContext::new().focused(true))
            .map(Msg::TabBar)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TabBarApp, _>::virtual_builder(70, 10).build()?;

    println!("=== TabBar Example ===\n");

    vt.tick()?;
    println!("Initial tab bar (main.rs active):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::TabBar(TabBarMessage::NextTab));
    vt.tick()?;
    println!("After switching to lib.rs (modified):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::TabBar(TabBarMessage::NextTab));
    vt.tick()?;
    println!("After switching to style.css:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::AddNewTab);
    vt.tick()?;
    println!("After adding a new tab:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::TabBar(TabBarMessage::CloseActiveTab));
    vt.tick()?;
    println!("After closing the new tab:");
    println!("{}\n", vt.display());

    Ok(())
}
