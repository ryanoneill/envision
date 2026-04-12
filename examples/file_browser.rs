//! FileBrowser example -- interactive navigable directory listing with filtering.
//!
//! Demonstrates the FileBrowser compound component with a virtual filesystem,
//! showing directory navigation, entry filtering, selection, and hidden file
//! toggling. The browser uses in-memory file entries rather than the real
//! filesystem so the example is self-contained.
//!
//! Controls:
//!   Up/Down     Navigate entries
//!   Enter       Open selected directory / select file
//!   Backspace   Navigate to parent directory
//!   h           Toggle hidden files
//!   /           Start typing a filter
//!   Esc         Clear filter (if active) or quit
//!   q           Quit (when filter is empty)
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
            file_browser::FileEntry::directory("examples", "/project/examples"),
            file_browser::FileEntry::file("Cargo.toml", "/project/Cargo.toml").with_size(1250),
            file_browser::FileEntry::file("Cargo.lock", "/project/Cargo.lock").with_size(48_200),
            file_browser::FileEntry::file("README.md", "/project/README.md").with_size(4096),
            file_browser::FileEntry::file(".gitignore", "/project/.gitignore").with_size(128),
            file_browser::FileEntry::file("LICENSE", "/project/LICENSE").with_size(1060),
            file_browser::FileEntry::file("rustfmt.toml", "/project/rustfmt.toml").with_size(45),
        ];

        let browser = FileBrowserState::new("/project", entries);

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

        FileBrowser::view(
            &state.browser,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

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
            " Selected: {} | Entries: {}{} | q: quit",
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
            if matches!(key.code, Key::Char('q') | Key::Esc)
                && state.browser.filter_text().is_empty()
            {
                return Some(Msg::Quit);
            }
        }
        FileBrowser::handle_event(&state.browser, event, &EventContext::new().focused(true))
            .map(Msg::Browser)
    }
}

#[tokio::main]
async fn main() -> envision::Result<()> {
    let _final_state = TerminalRuntime::<FileBrowserApp>::new_terminal()?
        .run_terminal()
        .await?;
    Ok(())
}
