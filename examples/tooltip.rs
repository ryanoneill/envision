//! Tooltip example -- contextual information overlay.
//!
//! Demonstrates the Tooltip component for displaying positioned
//! overlays with auto-hide support and configurable positioning.
//!
//! Run with: cargo run --example tooltip --features overlay-components

use envision::prelude::*;

/// Application marker type.
struct TooltipApp;

/// Application state.
#[derive(Clone)]
struct State {
    tooltip: TooltipState,
    action_count: usize,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Tooltip(TooltipMessage),
    ShowHelp,
    ShowWarning,
    Quit,
}

impl App for TooltipApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let tooltip = TooltipState::new("Press 'h' for help, 'w' for a warning tooltip")
            .with_title("Tip")
            .with_position(TooltipPosition::Below);

        let state = State {
            tooltip,
            action_count: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Tooltip(m) => {
                Tooltip::update(&mut state.tooltip, m);
            }
            Msg::ShowHelp => {
                state.tooltip = TooltipState::new(
                    "Keyboard shortcuts:\nh: Help\nw: Warning\nt: Toggle\nq: Quit",
                )
                .with_title("Help")
                .with_position(TooltipPosition::Below)
                .with_duration(5000);
                Tooltip::show(&mut state.tooltip);
                state.action_count += 1;
            }
            Msg::ShowWarning => {
                state.tooltip = TooltipState::new("Unsaved changes will be lost!")
                    .with_title("Warning")
                    .with_position(TooltipPosition::Below)
                    .with_fg_color(Color::Yellow)
                    .with_border_color(Color::Yellow)
                    .with_duration(3000);
                Tooltip::show(&mut state.tooltip);
                state.action_count += 1;
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(5),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        // Main content area
        let content = format!(
            "Tooltip Demo Application\n\
             Actions taken: {}\n\
             Tooltip visible: {}",
            state.action_count,
            state.tooltip.is_visible()
        );
        let widget = ratatui::widgets::Paragraph::new(content).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Application"),
        );
        frame.render_widget(widget, chunks[0]);

        // Render tooltip below the main content
        if state.tooltip.is_visible() {
            Tooltip::view_at(&state.tooltip, frame, chunks[0], area);
        }

        // Help line
        frame.render_widget(
            ratatui::widgets::Paragraph::new(
                " h: help tooltip, w: warning tooltip, t: toggle, q: quit",
            )
            .style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
                KeyCode::Char('h') => Some(Msg::ShowHelp),
                KeyCode::Char('w') => Some(Msg::ShowWarning),
                KeyCode::Char('t') => Some(Msg::Tooltip(TooltipMessage::Toggle)),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TooltipApp, _>::virtual_terminal(55, 16)?;

    println!("=== Tooltip Example ===\n");

    // Initial render: no tooltip
    vt.tick()?;
    println!("Initial state (no tooltip):");
    println!("{}\n", vt.display());

    // Show help tooltip
    vt.dispatch(Msg::ShowHelp);
    vt.tick()?;
    println!("After showing help tooltip:");
    println!("{}\n", vt.display());

    // Hide it
    vt.dispatch(Msg::Tooltip(TooltipMessage::Hide));
    vt.tick()?;
    println!("After hiding tooltip:");
    println!("{}\n", vt.display());

    // Show warning tooltip
    vt.dispatch(Msg::ShowWarning);
    vt.tick()?;
    println!("After showing warning tooltip:");
    println!("{}\n", vt.display());

    Ok(())
}
