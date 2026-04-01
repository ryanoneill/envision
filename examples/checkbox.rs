//! Checkbox example -- toggleable checkboxes with focus management.
//!
//! Demonstrates the Checkbox component with multiple checkboxes,
//! manual focus cycling, and toggle behavior.
//!
//! Run with: cargo run --example checkbox --features input-components

use envision::prelude::*;

/// Application marker type.
struct CheckboxApp;

/// Application state with multiple checkboxes.
#[derive(Clone)]
struct State {
    notifications: CheckboxState,
    dark_mode: CheckboxState,
    auto_save: CheckboxState,
    focus_index: usize,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Notifications(CheckboxMessage),
    DarkMode(CheckboxMessage),
    AutoSave(CheckboxMessage),
    FocusNext,
    FocusPrev,
    Quit,
}

impl App for CheckboxApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut notifications = CheckboxState::new("Enable notifications");
        notifications.set_focused(true);
        notifications.set_checked(true);

        let dark_mode = CheckboxState::new("Dark mode");
        let auto_save = CheckboxState::new("Auto-save documents");

        let state = State {
            notifications,
            dark_mode,
            auto_save,
            focus_index: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Notifications(m) => {
                Checkbox::update(&mut state.notifications, m);
            }
            Msg::DarkMode(m) => {
                Checkbox::update(&mut state.dark_mode, m);
            }
            Msg::AutoSave(m) => {
                Checkbox::update(&mut state.auto_save, m);
            }
            Msg::FocusNext => {
                // Blur current
                match state.focus_index {
                    0 => state.notifications.set_focused(false),
                    1 => state.dark_mode.set_focused(false),
                    _ => state.auto_save.set_focused(false),
                }
                state.focus_index = (state.focus_index + 1) % 3;
                // Focus next
                match state.focus_index {
                    0 => state.notifications.set_focused(true),
                    1 => state.dark_mode.set_focused(true),
                    _ => state.auto_save.set_focused(true),
                }
            }
            Msg::FocusPrev => {
                // Blur current
                match state.focus_index {
                    0 => state.notifications.set_focused(false),
                    1 => state.dark_mode.set_focused(false),
                    _ => state.auto_save.set_focused(false),
                }
                state.focus_index = (state.focus_index + 2) % 3;
                // Focus prev
                match state.focus_index {
                    0 => state.notifications.set_focused(true),
                    1 => state.dark_mode.set_focused(true),
                    _ => state.auto_save.set_focused(true),
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
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        Checkbox::view(
            &state.notifications,
            frame,
            chunks[0],
            &theme,
            &ViewContext::default(),
        );
        Checkbox::view(
            &state.dark_mode,
            frame,
            chunks[1],
            &theme,
            &ViewContext::default(),
        );
        Checkbox::view(
            &state.auto_save,
            frame,
            chunks[2],
            &theme,
            &ViewContext::default(),
        );

        // Summary
        let summary = format!(
            "  Notifications: {}  Dark mode: {}  Auto-save: {}",
            if state.notifications.is_checked() {
                "on"
            } else {
                "off"
            },
            if state.dark_mode.is_checked() {
                "on"
            } else {
                "off"
            },
            if state.auto_save.is_checked() {
                "on"
            } else {
                "off"
            },
        );
        let summary_widget = ratatui::widgets::Paragraph::new(summary).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Settings Summary"),
        );
        frame.render_widget(summary_widget, chunks[3]);

        let status = format!(
            " Focus: {} | Tab/Shift+Tab: navigate, Space/Enter: toggle, q: quit",
            state.focus_index
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[4],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
                KeyCode::Tab => return Some(Msg::FocusNext),
                KeyCode::BackTab => return Some(Msg::FocusPrev),
                _ => {}
            }
        }
        // Route event to focused checkbox
        match state.focus_index {
            0 => state
                .notifications
                .handle_event(event)
                .map(Msg::Notifications),
            1 => state.dark_mode.handle_event(event).map(Msg::DarkMode),
            _ => state.auto_save.handle_event(event).map(Msg::AutoSave),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<CheckboxApp, _>::virtual_terminal(55, 16)?;

    println!("=== Checkbox Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state (notifications checked):");
    println!("{}\n", vt.display());

    // Toggle notifications off
    vt.dispatch(Msg::Notifications(CheckboxMessage::Toggle));
    vt.tick()?;
    println!("After toggling notifications off:");
    println!("{}\n", vt.display());

    // Move to dark mode and toggle on
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::DarkMode(CheckboxMessage::Toggle));
    vt.tick()?;
    println!("After enabling dark mode:");
    println!("{}\n", vt.display());

    // Move to auto-save and toggle on
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::AutoSave(CheckboxMessage::Toggle));
    vt.tick()?;
    println!("After enabling auto-save:");
    println!("{}\n", vt.display());

    Ok(())
}
