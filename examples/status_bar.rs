//! StatusBar example -- application status display with dynamic content.
//!
//! Demonstrates the StatusBar component with left/center/right sections,
//! styled items, counters, and elapsed time indicators.
//!
//! Run with: cargo run --example status_bar --features display-components

use envision::prelude::*;

/// Application marker type.
struct StatusBarApp;

/// Application state.
#[derive(Clone)]
struct State {
    status: StatusBarState,
    mode: String,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    StatusBar(StatusBarMessage),
    SwitchToInsert,
    SwitchToNormal,
    Quit,
}

impl App for StatusBarApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut status = StatusBarState::new();

        // Left: mode indicator
        status.push_left(StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info));

        // Center: filename
        status.push_center(StatusBarItem::new("main.rs"));

        // Right: position and encoding
        status.push_right(StatusBarItem::counter().with_label("Ln"));
        status.push_right(StatusBarItem::new("UTF-8").with_style(StatusBarStyle::Muted));

        let state = State {
            status,
            mode: "NORMAL".to_string(),
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::StatusBar(m) => {
                StatusBar::update(&mut state.status, m);
            }
            Msg::SwitchToInsert => {
                state.mode = "INSERT".to_string();
                StatusBar::update(
                    &mut state.status,
                    StatusBarMessage::SetLeftItems(vec![
                        StatusBarItem::new("INSERT").with_style(StatusBarStyle::Success),
                    ]),
                );
            }
            Msg::SwitchToNormal => {
                state.mode = "NORMAL".to_string();
                StatusBar::update(
                    &mut state.status,
                    StatusBarMessage::SetLeftItems(vec![
                        StatusBarItem::new("NORMAL").with_style(StatusBarStyle::Info),
                    ]),
                );
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
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

        // Main content area
        let content = format!(
            "Current mode: {}\n\nPress 'i' for INSERT mode, Esc for NORMAL.\nPress 'q' to quit.",
            state.mode
        );
        let widget = ratatui::widgets::Paragraph::new(content).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Editor"),
        );
        frame.render_widget(widget, chunks[0]);

        // Status bar
        StatusBar::view(
            &state.status,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );

        // Help line
        frame.render_widget(
            ratatui::widgets::Paragraph::new(" i: insert mode, Esc: normal, +/-: counter, q: quit")
                .style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') => Some(Msg::Quit),
                Key::Char('i') => Some(Msg::SwitchToInsert),
                Key::Esc => Some(Msg::SwitchToNormal),
                Key::Char('+') => Some(Msg::StatusBar(StatusBarMessage::IncrementCounter {
                    section: Section::Right,
                    index: 0,
                })),
                Key::Char('-') => Some(Msg::StatusBar(StatusBarMessage::DecrementCounter {
                    section: Section::Right,
                    index: 0,
                })),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<StatusBarApp, _>::virtual_terminal(60, 10)?;

    println!("=== StatusBar Example ===\n");

    // Initial render: NORMAL mode
    vt.tick()?;
    println!("Initial state (NORMAL mode):");
    println!("{}\n", vt.display());

    // Switch to INSERT mode
    vt.dispatch(Msg::SwitchToInsert);
    vt.tick()?;
    println!("After switching to INSERT mode:");
    println!("{}\n", vt.display());

    // Increment line counter several times
    for _ in 0..42 {
        vt.dispatch(Msg::StatusBar(StatusBarMessage::IncrementCounter {
            section: Section::Right,
            index: 0,
        }));
    }
    vt.tick()?;
    println!("After incrementing line counter to 42:");
    println!("{}\n", vt.display());

    // Switch back to NORMAL
    vt.dispatch(Msg::SwitchToNormal);
    vt.tick()?;
    println!("Back to NORMAL mode:");
    println!("{}\n", vt.display());

    Ok(())
}
