//! FileBrowser example -- navigable directory listing with filtering.
//!
//! Demonstrates the FileBrowser component with a virtual filesystem,
//! showing directory navigation, filtering, and selection.
//!
//! Run with: cargo run --example file_browser --features compound-components

use envision::prelude::*;

/// Application marker type.
struct FileBrowserApp;

/// Application state.
#[derive(Clone)]
struct State {
    browser: FileBrowserState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Browser(FileBrowserMessage),
    Quit,
}

impl App for FileBrowserApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let entries = vec![
            file_browser::FileEntry::directory("src", "/project/src"),
            file_browser::FileEntry::directory("tests", "/project/tests"),
            file_browser::FileEntry::directory("docs", "/project/docs"),
            file_browser::FileEntry::file("Cargo.toml", "/project/Cargo.toml").with_size(1250),
            file_browser::FileEntry::file("README.md", "/project/README.md").with_size(4096),
            file_browser::FileEntry::file(".gitignore", "/project/.gitignore").with_size(128),
            file_browser::FileEntry::file("LICENSE", "/project/LICENSE").with_size(1060),
            file_browser::FileEntry::file("rustfmt.toml", "/project/rustfmt.toml").with_size(45),
        ];

        let mut browser = FileBrowserState::new("/project", entries);
        browser.set_focused(true);

        (State { browser }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Browser(m) => {
                FileBrowser::update(&mut state.browser, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        FileBrowser::view(&state.browser, frame, chunks[0], &theme);

        let selected = state
            .browser
            .selected_entry()
            .map(|e| e.name().to_string())
            .unwrap_or_else(|| "None".into());
        let filter = state.browser.filter_text();
        let filter_display = if filter.is_empty() {
            String::new()
        } else {
            format!(" | Filter: {}", filter)
        };
        let status = format!(
            " Selected: {} | Entries: {}{}",
            selected,
            state.browser.filtered_entries().len(),
            filter_display
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                && state.browser.filter_text().is_empty()
            {
                return Some(Msg::Quit);
            }
        }
        state.browser.handle_event(event).map(Msg::Browser)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<FileBrowserApp, _>::virtual_terminal(50, 16)?;

    println!("=== FileBrowser Example ===\n");

    // Initial render: project root listing
    vt.tick()?;
    println!("Initial state (project root):");
    println!("{}\n", vt.display());

    // Navigate down to select different entries
    vt.dispatch(Msg::Browser(FileBrowserMessage::Down));
    vt.dispatch(Msg::Browser(FileBrowserMessage::Down));
    vt.dispatch(Msg::Browser(FileBrowserMessage::Down));
    vt.tick()?;
    println!("After navigating down 3 times:");
    println!("{}\n", vt.display());

    // Type a filter to narrow results
    vt.dispatch(Msg::Browser(FileBrowserMessage::FilterChar('r')));
    vt.tick()?;
    println!("After typing filter 'r':");
    println!("{}\n", vt.display());

    // Clear filter
    vt.dispatch(Msg::Browser(FileBrowserMessage::FilterClear));
    vt.tick()?;
    println!("After clearing filter:");
    println!("{}\n", vt.display());

    // Toggle hidden files
    vt.dispatch(Msg::Browser(FileBrowserMessage::ToggleHidden));
    vt.tick()?;
    println!("After toggling hidden files (now hidden):");
    println!("{}\n", vt.display());

    Ok(())
}
