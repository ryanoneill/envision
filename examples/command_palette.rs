//! CommandPalette example -- searchable, fuzzy-filtered action picker.
//!
//! Demonstrates the CommandPalette navigation component with fuzzy matching,
//! keyboard navigation, shortcuts display, and item selection.
//!
//! Run with: cargo run --example command_palette --features navigation-components

use envision::prelude::*;

/// Application marker type.
struct CommandPaletteApp;

/// Application state.
#[derive(Clone)]
struct State {
    palette: CommandPaletteState,
    selections: Vec<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Palette(CommandPaletteMessage),
    TogglePalette,
    Quit,
}

impl App for CommandPaletteApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let items = vec![
            PaletteItem::new("open", "Open File")
                .with_shortcut("Ctrl+O")
                .with_category("File"),
            PaletteItem::new("save", "Save File")
                .with_shortcut("Ctrl+S")
                .with_category("File"),
            PaletteItem::new("save-as", "Save As...")
                .with_shortcut("Ctrl+Shift+S")
                .with_category("File"),
            PaletteItem::new("close", "Close Tab")
                .with_shortcut("Ctrl+W")
                .with_category("File"),
            PaletteItem::new("find", "Find in Files")
                .with_shortcut("Ctrl+Shift+F")
                .with_category("Search"),
            PaletteItem::new("replace", "Find and Replace")
                .with_shortcut("Ctrl+H")
                .with_category("Search"),
            PaletteItem::new("sidebar", "Toggle Sidebar")
                .with_shortcut("Ctrl+B")
                .with_category("View"),
            PaletteItem::new("terminal", "Toggle Terminal")
                .with_shortcut("Ctrl+`")
                .with_category("View"),
            PaletteItem::new("zoom-in", "Zoom In")
                .with_shortcut("Ctrl++")
                .with_category("View"),
            PaletteItem::new("zoom-out", "Zoom Out")
                .with_shortcut("Ctrl+-")
                .with_category("View"),
            PaletteItem::new("settings", "Open Settings")
                .with_shortcut("Ctrl+,")
                .with_category("Preferences"),
            PaletteItem::new("theme", "Change Theme").with_category("Preferences"),
            PaletteItem::new("quit", "Quit Application")
                .with_shortcut("Ctrl+Q")
                .with_category("Application"),
        ];

        let mut palette = CommandPaletteState::new(items)
            .with_title("Command Palette")
            .with_placeholder("Type a command...")
            .with_max_visible(8);
        palette.set_visible(true);

        let state = State {
            palette,
            selections: Vec::new(),
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Palette(m) => {
                if let Some(output) = CommandPalette::update(&mut state.palette, m) {
                    match output {
                        CommandPaletteOutput::Selected(item) => {
                            state
                                .selections
                                .push(format!("{} ({})", item.label, item.id));
                        }
                        CommandPaletteOutput::Dismissed => {
                            state.selections.push("-- Dismissed --".to_string());
                        }
                        CommandPaletteOutput::QueryChanged(_) => {}
                    }
                }
            }
            Msg::TogglePalette => {
                if state.palette.is_visible() {
                    state.palette.dismiss();
                } else {
                    state.palette.show();
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
            Constraint::Min(0),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(area);

        // Main area: render the command palette overlay
        CommandPalette::view(
            &state.palette,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        // Show selection history
        let log_lines: Vec<Line> = state
            .selections
            .iter()
            .rev()
            .take(3)
            .map(|s| Line::from(format!("  {}", s)))
            .collect();
        let log = ratatui::widgets::Paragraph::new(log_lines).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Selection History"),
        );
        frame.render_widget(log, chunks[1]);

        let status = " Ctrl+P: toggle palette | Esc: dismiss | q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        // Global key bindings
        if let Some(key) = event.as_key() {
            if key.code == Key::Esc && !state.palette.is_visible() {
                return Some(Msg::Quit);
            }
            if key.code == Key::Char('p') && key.modifiers.ctrl() && !state.palette.is_visible() {
                return Some(Msg::TogglePalette);
            }
        }

        // Delegate to palette when visible
        if state.palette.is_visible() {
            CommandPalette::handle_event(&state.palette, event, &EventContext::new().focused(true))
                .map(Msg::Palette)
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<CommandPaletteApp, _>::virtual_builder(60, 24).build()?;

    println!("=== CommandPalette Example ===\n");

    // Initial render: palette visible with all items
    vt.tick()?;
    println!("Initial state (palette open):");
    println!("{}\n", vt.display());

    // Type to filter
    vt.dispatch(Msg::Palette(CommandPaletteMessage::TypeChar('f')));
    vt.tick()?;
    println!("After typing 'f' (filtering):");
    println!("{}\n", vt.display());

    // Navigate down and select
    vt.dispatch(Msg::Palette(CommandPaletteMessage::SelectNext));
    vt.dispatch(Msg::Palette(CommandPaletteMessage::Confirm));
    vt.tick()?;
    println!("After selecting an item:");
    println!("{}\n", vt.display());

    // Re-open the palette
    vt.dispatch(Msg::TogglePalette);
    vt.tick()?;
    println!("Palette re-opened:");
    println!("{}\n", vt.display());

    Ok(())
}
