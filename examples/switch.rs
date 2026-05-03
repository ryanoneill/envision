//! Switch example -- on/off toggle switches with focus management.
//!
//! Demonstrates the Switch component with multiple switches,
//! manual focus cycling, and toggle behavior.
//!
//! Run with: cargo run --example switch --features input-components

use envision::prelude::*;

/// Application marker type.
struct SwitchApp;

/// Application state with multiple switches.
#[derive(Clone)]
struct State {
    wifi: SwitchState,
    bluetooth: SwitchState,
    dark_mode: SwitchState,
    notifications: SwitchState,
    focus_index: usize,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Wifi(SwitchMessage),
    Bluetooth(SwitchMessage),
    DarkMode(SwitchMessage),
    Notifications(SwitchMessage),
    FocusNext,
    FocusPrev,
    Quit,
}

const SWITCH_COUNT: usize = 4;

impl State {
    fn set_focus(&mut self, index: usize) {
        self.focus_index = index;
    }
}

impl App for SwitchApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let wifi = SwitchState::new().with_label("Wi-Fi").with_on(true);

        let bluetooth = SwitchState::new().with_label("Bluetooth");
        let dark_mode = SwitchState::new()
            .with_label("Dark Mode")
            .with_on_label("DARK")
            .with_off_label("LIGHT");
        let notifications = SwitchState::new().with_label("Notifications");

        let state = State {
            wifi,
            bluetooth,
            dark_mode,
            notifications,
            focus_index: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Wifi(m) => {
                Switch::update(&mut state.wifi, m);
            }
            Msg::Bluetooth(m) => {
                Switch::update(&mut state.bluetooth, m);
            }
            Msg::DarkMode(m) => {
                Switch::update(&mut state.dark_mode, m);
            }
            Msg::Notifications(m) => {
                Switch::update(&mut state.notifications, m);
            }
            Msg::FocusNext => {
                let next = (state.focus_index + 1) % SWITCH_COUNT;
                state.set_focus(next);
            }
            Msg::FocusPrev => {
                let prev = (state.focus_index + SWITCH_COUNT - 1) % SWITCH_COUNT;
                state.set_focus(prev);
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
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        // Title
        let title = ratatui::widgets::Paragraph::new("  Switch Settings")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::BOTTOM)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(title, chunks[0]);

        // Switches
        Switch::view(
            &state.wifi,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
        Switch::view(
            &state.bluetooth,
            &mut RenderContext::new(frame, chunks[2], &theme),
        );
        Switch::view(
            &state.dark_mode,
            &mut RenderContext::new(frame, chunks[3], &theme),
        );
        Switch::view(
            &state.notifications,
            &mut RenderContext::new(frame, chunks[4], &theme),
        );

        // Summary
        let summary = format!(
            "  Wi-Fi: {}  Bluetooth: {}  Dark Mode: {}  Notifications: {} (disabled)",
            if state.wifi.is_on() { "on" } else { "off" },
            if state.bluetooth.is_on() { "on" } else { "off" },
            if state.dark_mode.is_on() {
                "dark"
            } else {
                "light"
            },
            if state.notifications.is_on() {
                "on"
            } else {
                "off"
            },
        );
        let summary_widget = ratatui::widgets::Paragraph::new(summary).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Summary"),
        );
        frame.render_widget(summary_widget, chunks[5]);

        let status = format!(
            " Focus: {} | Tab/Shift+Tab: navigate, Space/Enter: toggle, q: quit",
            state.focus_index
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[6],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => return Some(Msg::Quit),
                Key::Tab if key.modifiers.shift() => return Some(Msg::FocusPrev),

                Key::Tab => return Some(Msg::FocusNext),
                _ => {}
            }
        }
        // Route event to focused switch
        match state.focus_index {
            0 => Switch::handle_event(&state.wifi, event, &EventContext::new().focused(true))
                .map(Msg::Wifi),
            1 => Switch::handle_event(&state.bluetooth, event, &EventContext::new().focused(true))
                .map(Msg::Bluetooth),
            2 => Switch::handle_event(&state.dark_mode, event, &EventContext::new().focused(true))
                .map(Msg::DarkMode),
            _ => Switch::handle_event(
                &state.notifications,
                event,
                &EventContext::new().focused(true),
            )
            .map(Msg::Notifications),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<SwitchApp, _>::virtual_builder(60, 14).build()?;

    println!("=== Switch Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state (Wi-Fi on, others off):");
    println!("{}\n", vt.display());

    // Toggle Wi-Fi off
    vt.dispatch(Msg::Wifi(SwitchMessage::Toggle));
    vt.tick()?;
    println!("After toggling Wi-Fi off:");
    println!("{}\n", vt.display());

    // Move to Bluetooth and toggle on
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::Bluetooth(SwitchMessage::Toggle));
    vt.tick()?;
    println!("After enabling Bluetooth:");
    println!("{}\n", vt.display());

    // Move to Dark Mode and toggle on
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::DarkMode(SwitchMessage::Toggle));
    vt.tick()?;
    println!("After enabling Dark Mode:");
    println!("{}\n", vt.display());

    // Try to toggle disabled Notifications (no effect)
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::Notifications(SwitchMessage::Toggle));
    vt.tick()?;
    println!("After attempting to toggle disabled Notifications:");
    println!("{}\n", vt.display());

    Ok(())
}
