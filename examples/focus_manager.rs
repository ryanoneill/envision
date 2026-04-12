//! FocusManager example -- focus coordination between components.
//!
//! Demonstrates FocusManager tracking keyboard focus across multiple
//! components. Tab and Shift+Tab cycle focus; each component renders
//! with a highlighted border when it owns focus.
//!
//! Run with: `cargo run --example focus_manager`

use envision::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Identifies each focusable panel in the layout.
#[derive(Clone, PartialEq, Debug)]
enum Panel {
    Sidebar,
    Content,
    ButtonBar,
}

/// Application marker type.
struct FocusManagerApp;

/// Application state.
#[derive(Clone)]
struct State {
    focus: FocusManager<Panel>,
    sidebar_item: usize,
    last_action: String,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    FocusNext,
    FocusPrev,
    SidebarDown,
    SidebarUp,
    ButtonAction,
    Quit,
}

const SIDEBAR_ITEMS: &[&str] = &["Dashboard", "Reports", "Settings", "Help"];

impl App for FocusManagerApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let focus = FocusManager::with_initial_focus(vec![
            Panel::Sidebar,
            Panel::Content,
            Panel::ButtonBar,
        ]);
        let state = State {
            focus,
            sidebar_item: 0,
            last_action: "Ready -- Tab to move focus".to_string(),
        };
        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::FocusNext => {
                state.focus.focus_next();
                state.last_action = format!("Focus: {}", panel_name(state.focus.focused()));
            }
            Msg::FocusPrev => {
                state.focus.focus_prev();
                state.last_action = format!("Focus: {}", panel_name(state.focus.focused()));
            }
            Msg::SidebarDown => {
                state.sidebar_item =
                    (state.sidebar_item + 1).min(SIDEBAR_ITEMS.len().saturating_sub(1));
                state.last_action = format!("Selected: {}", SIDEBAR_ITEMS[state.sidebar_item]);
            }
            Msg::SidebarUp => {
                state.sidebar_item = state.sidebar_item.saturating_sub(1);
                state.last_action = format!("Selected: {}", SIDEBAR_ITEMS[state.sidebar_item]);
            }
            Msg::ButtonAction => {
                state.last_action = "Button activated!".to_string();
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let rows = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);
        let cols = Layout::horizontal([Constraint::Length(20), Constraint::Min(0)]).split(rows[0]);

        // Sidebar: highlight selected item in cyan when focused
        let sidebar_focused = state.focus.is_focused(&Panel::Sidebar);
        let items: Vec<Line> = SIDEBAR_ITEMS
            .iter()
            .enumerate()
            .map(|(i, &name)| {
                let prefix = if i == state.sidebar_item { "> " } else { "  " };
                let style = if i == state.sidebar_item && sidebar_focused {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                Line::from(format!("{}{}", prefix, name)).style(style)
            })
            .collect();
        frame.render_widget(
            Paragraph::new(items).block(focused_block("Sidebar", sidebar_focused)),
            cols[0],
        );

        // Content: shows last action
        let content_focused = state.focus.is_focused(&Panel::Content);
        frame.render_widget(
            Paragraph::new(format!("  Action: {}", state.last_action))
                .block(focused_block("Content", content_focused)),
            cols[1],
        );

        // Button bar: hint changes when focused
        let bar_focused = state.focus.is_focused(&Panel::ButtonBar);
        let bar_label = if bar_focused {
            "[ Apply ]  [ Cancel ]  [ Help ]  <-- press Enter"
        } else {
            "[ Apply ]  [ Cancel ]  [ Help ]"
        };
        let bar_style = if bar_focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        frame.render_widget(
            Paragraph::new(bar_label)
                .style(bar_style)
                .block(focused_block("Button Bar", bar_focused)),
            rows[1],
        );

        // Status line: shows which panel owns focus and the total count
        let focused_idx = state
            .focus
            .order()
            .iter()
            .position(|p| Some(p) == state.focus.focused())
            .map(|i| i + 1)
            .unwrap_or(0);
        let status = format!(
            " Focused: {} ({}/{}) | Tab/Shift+Tab: cycle, Arrows: navigate, q: quit",
            panel_name(state.focus.focused()),
            focused_idx,
            state.focus.len(),
        );
        frame.render_widget(
            Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            rows[2],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.key {
                Key::Char('q') | Key::Esc => return Some(Msg::Quit),
                Key::Tab if key.modifiers.shift() => return Some(Msg::FocusPrev),

                Key::Tab => return Some(Msg::FocusNext),
                Key::Up if state.focus.is_focused(&Panel::Sidebar) => {
                    return Some(Msg::SidebarUp);
                }
                Key::Down if state.focus.is_focused(&Panel::Sidebar) => {
                    return Some(Msg::SidebarDown);
                }
                Key::Enter | Key::Char(' ') if state.focus.is_focused(&Panel::ButtonBar) => {
                    return Some(Msg::ButtonAction);
                }
                _ => {}
            }
        }
        None
    }
}

fn focused_block(title: &str, focused: bool) -> Block<'_> {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style)
}

fn panel_name(panel: Option<&Panel>) -> &'static str {
    match panel {
        Some(Panel::Sidebar) => "Sidebar",
        Some(Panel::Content) => "Content",
        Some(Panel::ButtonBar) => "Button Bar",
        None => "None",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<FocusManagerApp, _>::virtual_terminal(80, 18)?;

    println!("=== FocusManager Example ===\n");

    // Initial: Sidebar focused (focus 1/3)
    vt.tick()?;
    println!("Initial state (Sidebar focused):");
    println!("{}\n", vt.display());

    // Tab cycles through all 3 panels
    vt.dispatch(Msg::FocusNext); // -> Content (2/3)
    vt.dispatch(Msg::FocusNext); // -> Button Bar (3/3)
    vt.tick()?;
    println!("After Tab x2 -- Button Bar focused:");
    println!("{}\n", vt.display());

    // Activate the button bar, then wrap focus back to Sidebar
    vt.dispatch(Msg::ButtonAction);
    vt.dispatch(Msg::FocusNext); // wraps -> Sidebar (1/3)
    vt.dispatch(Msg::SidebarDown);
    vt.dispatch(Msg::SidebarDown);
    vt.tick()?;
    println!("After activating button, Tab wrap, Down x2 in Sidebar:");
    println!("{}", vt.display());

    Ok(())
}
