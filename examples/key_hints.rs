//! KeyHints example -- keyboard shortcut display bar.
//!
//! Demonstrates the KeyHints component for showing available keyboard
//! shortcuts in a compact status bar format.
//!
//! Run with: cargo run --example key_hints --features display-components

use envision::prelude::*;

/// Application marker type.
struct KeyHintsApp;

/// Application state.
#[derive(Clone)]
struct State {
    hints: KeyHintsState,
    mode: AppMode,
}

/// Which mode the application is in.
#[derive(Clone, Debug, PartialEq)]
enum AppMode {
    Normal,
    Edit,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    SwitchToEdit,
    SwitchToNormal,
    Quit,
}

fn normal_hints() -> KeyHintsState {
    KeyHintsState::new()
        .hint("e", "Edit")
        .hint("d", "Delete")
        .hint("/", "Search")
        .hint("?", "Help")
        .hint("q", "Quit")
}

fn edit_hints() -> KeyHintsState {
    KeyHintsState::new()
        .hint("Esc", "Cancel")
        .hint("Ctrl+S", "Save")
        .hint("Tab", "Next field")
        .hint("Shift+Tab", "Prev field")
}

impl App for KeyHintsApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let state = State {
            hints: normal_hints(),
            mode: AppMode::Normal,
        };
        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::SwitchToEdit => {
                state.mode = AppMode::Edit;
                state.hints = edit_hints();
            }
            Msg::SwitchToNormal => {
                state.mode = AppMode::Normal;
                state.hints = normal_hints();
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        // Main content area showing current mode
        let mode_label = match state.mode {
            AppMode::Normal => "NORMAL",
            AppMode::Edit => "EDIT",
        };
        let content = format!(
            "Current mode: {}\n\nPress 'e' to switch to edit mode.\nPress Esc to return to normal mode.\nPress 'q' to quit.",
            mode_label
        );
        let widget = ratatui::widgets::Paragraph::new(content).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Application"),
        );
        frame.render_widget(widget, chunks[0]);

        // Key hints bar at the bottom
        KeyHints::view(
            &state.hints,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') => Some(Msg::Quit),
                Key::Char('e') => Some(Msg::SwitchToEdit),
                Key::Esc => Some(Msg::SwitchToNormal),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<KeyHintsApp, _>::virtual_builder(70, 10).build()?;

    println!("=== KeyHints Example ===\n");

    // Initial render: normal mode hints
    vt.tick()?;
    println!("Normal mode (with navigation hints):");
    println!("{}\n", vt.display());

    // Switch to edit mode
    vt.dispatch(Msg::SwitchToEdit);
    vt.tick()?;
    println!("Edit mode (different hints):");
    println!("{}\n", vt.display());

    // Switch back to normal
    vt.dispatch(Msg::SwitchToNormal);
    vt.tick()?;
    println!("Back to normal mode:");
    println!("{}\n", vt.display());

    Ok(())
}
