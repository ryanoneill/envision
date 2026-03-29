//! HelpPanel example — scrollable keybinding display.
//!
//! Demonstrates the HelpPanel component with categorized keybindings
//! and keyboard-driven scrolling. Exercises scroll-up, scroll-down,
//! page navigation, and home/end jumps.
//!
//! Run with: cargo run --example help_panel

use envision::prelude::*;
use ratatui::widgets::Paragraph;

/// Application marker type.
struct HelpPanelApp;

/// Application state wrapping a single HelpPanel.
#[derive(Clone)]
struct State {
    help: HelpPanelState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Help(HelpPanelMessage),
    Quit,
}

impl App for HelpPanelApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut help = HelpPanelState::new()
            .with_title("Keybindings")
            .with_groups(vec![
                KeyBindingGroup::new(
                    "Navigation",
                    vec![
                        KeyBinding::new("Up/k", "Move up"),
                        KeyBinding::new("Down/j", "Move down"),
                        KeyBinding::new("PgUp/Ctrl+u", "Page up"),
                        KeyBinding::new("PgDn/Ctrl+d", "Page down"),
                        KeyBinding::new("Home/g", "Go to top"),
                        KeyBinding::new("End/G", "Go to bottom"),
                    ],
                ),
                KeyBindingGroup::new(
                    "Editing",
                    vec![
                        KeyBinding::new("i", "Insert mode"),
                        KeyBinding::new("a", "Append after cursor"),
                        KeyBinding::new("o", "Open line below"),
                        KeyBinding::new("dd", "Delete line"),
                        KeyBinding::new("yy", "Yank line"),
                        KeyBinding::new("p", "Paste"),
                    ],
                ),
                KeyBindingGroup::new(
                    "Search",
                    vec![
                        KeyBinding::new("/", "Search forward"),
                        KeyBinding::new("?", "Search backward"),
                        KeyBinding::new("n", "Next match"),
                        KeyBinding::new("N", "Previous match"),
                    ],
                ),
                KeyBindingGroup::new(
                    "General",
                    vec![
                        KeyBinding::new(":", "Command mode"),
                        KeyBinding::new("Ctrl+S", "Save"),
                        KeyBinding::new("Ctrl+Z", "Undo"),
                        KeyBinding::new("Ctrl+Y", "Redo"),
                        KeyBinding::new("q/Esc", "Quit"),
                    ],
                ),
            ]);
        help.set_focused(true);

        (State { help }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Help(m) => {
                state.help.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        let theme = Theme::default();
        HelpPanel::view(&state.help, frame, chunks[0], &theme);

        let status = Paragraph::new(format!(
            " Scroll: {} | Up/Down | PgUp/PgDn | Home/End | Esc quit",
            state.help.scroll_offset()
        ))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status, chunks[1]);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                return Some(Msg::Quit);
            }
        }

        state.help.handle_event(event).map(Msg::Help)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<HelpPanelApp, _>::virtual_terminal(50, 24)?;

    println!("=== HelpPanel Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial view:");
    println!("{}\n", vt.display());

    // Scroll down
    vt.dispatch(Msg::Help(HelpPanelMessage::ScrollDown));
    vt.dispatch(Msg::Help(HelpPanelMessage::ScrollDown));
    vt.dispatch(Msg::Help(HelpPanelMessage::ScrollDown));
    vt.tick()?;
    println!("After scrolling down 3 lines:");
    println!("{}\n", vt.display());

    // Page down
    vt.dispatch(Msg::Help(HelpPanelMessage::PageDown(10)));
    vt.tick()?;
    println!("After page down:");
    println!("{}\n", vt.display());

    // Jump to end
    vt.dispatch(Msg::Help(HelpPanelMessage::End));
    vt.tick()?;
    println!("At the end:");
    println!("{}\n", vt.display());

    // Jump back to top
    vt.dispatch(Msg::Help(HelpPanelMessage::Home));
    vt.tick()?;
    println!("Back at the top:");
    println!("{}\n", vt.display());

    println!("Final scroll offset: {}", vt.state().help.scroll_offset());

    Ok(())
}
