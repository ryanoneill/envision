//! TitleCard example — stylish display-only title component.
//!
//! Demonstrates the TitleCard component with various configurations:
//! plain title, title with subtitle, decorated with prefix/suffix,
//! and dynamic title updates via messages.
//!
//! Run with: cargo run --example title_card

use envision::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Application marker type.
struct TitleCardApp;

/// Application state holding multiple title card configurations.
#[derive(Clone)]
struct State {
    plain: TitleCardState,
    decorated: TitleCardState,
    styled: TitleCardState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    UpdatePlainTitle(String),
    AddSubtitle(String),
    Quit,
}

impl App for TitleCardApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let plain = TitleCardState::new("Envision");

        let decorated = TitleCardState::new("Dashboard")
            .with_subtitle("Real-time Monitoring")
            .with_prefix(">> ")
            .with_suffix(" <<");

        let styled = TitleCardState::new("Status Panel")
            .with_subtitle("All Systems Operational")
            .with_title_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .with_subtitle_style(Style::default().fg(Color::Yellow));

        let state = State {
            plain,
            decorated,
            styled,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::UpdatePlainTitle(title) => {
                state.plain.set_title(title);
            }
            Msg::AddSubtitle(subtitle) => {
                state.plain.set_subtitle(Some(subtitle));
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

        let theme = Theme::default();

        TitleCard::view(
            &state.plain,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );
        TitleCard::view(
            &state.decorated,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
        TitleCard::view(
            &state.styled,
            &mut RenderContext::new(frame, chunks[2], &theme),
        );

        let footer = Paragraph::new(" TitleCard configurations | Esc to quit")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(footer, chunks[3]);
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TitleCardApp, _>::virtual_terminal(60, 20)?;

    println!("=== TitleCard Example ===\n");

    // Initial render showing all three configurations
    vt.tick()?;
    println!("Three TitleCard configurations:");
    println!("{}\n", vt.display());

    // Update the plain title dynamically
    vt.dispatch(Msg::UpdatePlainTitle("Envision Framework".to_string()));
    vt.dispatch(Msg::AddSubtitle("v0.1.0".to_string()));
    vt.tick()?;
    println!("After updating plain title with subtitle:");
    println!("{}\n", vt.display());

    // Verify state
    println!("Plain title: {}", vt.state().plain.title());
    println!(
        "Plain subtitle: {}",
        vt.state().plain.subtitle().unwrap_or("(none)")
    );
    println!("Decorated title: {}", vt.state().decorated.title());
    println!(
        "Decorated prefix: {}",
        vt.state().decorated.prefix().unwrap_or("(none)")
    );

    Ok(())
}
