//! DiffViewer example -- side-by-side and unified diff display.
//!
//! Demonstrates the DiffViewer component showing differences between
//! two code snippets. Exercises hunk navigation, scrolling, and mode
//! toggling between unified and side-by-side display.
//!
//! Run with: cargo run --example diff_viewer

use envision::prelude::*;
use ratatui::widgets::Paragraph;

/// Application marker type.
struct DiffViewerApp;

/// Application state wrapping a single DiffViewer.
#[derive(Clone)]
struct State {
    viewer: DiffViewerState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Viewer(DiffViewerMessage),
    Quit,
}

impl App for DiffViewerApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let old_code = "\
fn greet(name: &str) {
    println!(\"Hello, {}!\", name);
}

fn main() {
    greet(\"world\");
}";

        let new_code = "\
fn greet(name: &str, greeting: &str) {
    println!(\"{}, {}!\", greeting, name);
    log::info!(\"Greeted {}\", name);
}

fn farewell(name: &str) {
    println!(\"Goodbye, {}!\", name);
}

fn main() {
    greet(\"world\", \"Hello\");
    farewell(\"world\");
}";

        let viewer = DiffViewerState::from_texts(old_code, new_code)
            .with_title("Code Changes")
            .with_old_label("greet.rs (old)")
            .with_new_label("greet.rs (new)");

        (State { viewer }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Viewer(m) => {
                state.viewer.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        let theme = Theme::default();
        DiffViewer::view(
            &state.viewer,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let mode_str = match state.viewer.mode() {
            DiffMode::Unified => "Unified",
            DiffMode::SideBySide => "Side-by-Side",
        };

        let status = Paragraph::new(format!(
            " Hunk {}/{} | {} | +{} -{} | j/k scroll | n/p hunk | m mode | q quit",
            state.viewer.current_hunk() + 1,
            state.viewer.hunk_count().max(1),
            mode_str,
            state.viewer.added_count(),
            state.viewer.removed_count(),
        ))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status, chunks[1]);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }

        DiffViewer::handle_event(&state.viewer, event, &EventContext::new().focused(true))
            .map(Msg::Viewer)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<DiffViewerApp, _>::virtual_terminal(80, 24)?;

    println!("=== DiffViewer Example ===\n");

    // Initial render (unified mode)
    vt.tick()?;
    println!("Unified mode:");
    println!("{}\n", vt.display());

    // Navigate to next hunk
    vt.dispatch(Msg::Viewer(DiffViewerMessage::NextHunk));
    vt.tick()?;
    println!("After next hunk:");
    println!("{}\n", vt.display());

    // Toggle to side-by-side mode
    vt.dispatch(Msg::Viewer(DiffViewerMessage::ToggleMode));
    vt.dispatch(Msg::Viewer(DiffViewerMessage::Home));
    vt.tick()?;
    println!("Side-by-side mode:");
    println!("{}\n", vt.display());

    println!(
        "Total changes: +{} -{}",
        vt.state().viewer.added_count(),
        vt.state().viewer.removed_count()
    );

    Ok(())
}
