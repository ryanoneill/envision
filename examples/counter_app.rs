//! Counter App example demonstrating The Elm Architecture (TEA) pattern.
//!
//! This example shows the core TEA concepts:
//! - State: The application's data model
//! - Message: Events that can modify state
//! - Update: Pure function that updates state based on messages
//! - View: Renders the UI based on current state
//! - Command: Side effects returned from update
//!
//! Run with: cargo run --example counter_app

use envision::prelude::*;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// The application marker type
struct CounterApp;

/// Application state - the single source of truth
#[derive(Default, Clone)]
struct State {
    count: i32,
    history: Vec<String>,
}

/// Messages that can modify state
#[derive(Clone, Debug)]
enum Msg {
    Increment,
    Decrement,
    Reset,
    Quit,
}

impl App for CounterApp {
    type State = State;
    type Message = Msg;

    /// Initialize the application state
    fn init() -> (State, Command<Msg>) {
        let mut state = State::default();
        state.history.push("App initialized".to_string());
        (state, Command::none())
    }

    /// Update state based on messages (pure function)
    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => {
                state.count += 1;
                state.history.push(format!("Incremented to {}", state.count));
            }
            Msg::Decrement => {
                state.count -= 1;
                state.history.push(format!("Decremented to {}", state.count));
            }
            Msg::Reset => {
                state.count = 0;
                state.history.push("Reset to 0".to_string());
            }
            Msg::Quit => {
                return Command::quit();
            }
        }

        // Keep only last 5 history entries
        if state.history.len() > 5 {
            state.history.remove(0);
        }

        Command::none()
    }

    /// Render the UI based on state (pure function)
    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();

        // Create main layout
        let chunks = Layout::vertical([
            Constraint::Length(3), // Title
            Constraint::Length(5), // Counter display
            Constraint::Length(3), // Controls
            Constraint::Min(0),    // History
        ])
        .split(area);

        // Title
        let title = Paragraph::new("Counter App - TEA Architecture Demo")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Counter display
        let counter_style = if state.count > 0 {
            Style::default().fg(Color::Green)
        } else if state.count < 0 {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let counter = Paragraph::new(format!("{}", state.count))
            .style(counter_style.add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Count")
                    .title_alignment(Alignment::Center),
            );
        frame.render_widget(counter, chunks[1]);

        // Controls
        let controls = Paragraph::new(Line::from(vec![
            Span::styled("[+] ", Style::default().fg(Color::Green)),
            Span::raw("Increment  "),
            Span::styled("[-] ", Style::default().fg(Color::Red)),
            Span::raw("Decrement  "),
            Span::styled("[R] ", Style::default().fg(Color::Yellow)),
            Span::raw("Reset  "),
            Span::styled("[Q] ", Style::default().fg(Color::Magenta)),
            Span::raw("Quit"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
        frame.render_widget(controls, chunks[2]);

        // History
        let history_lines: Vec<Line> = state
            .history
            .iter()
            .map(|s| Line::from(format!("  {}", s)))
            .collect();
        let history = Paragraph::new(history_lines)
            .block(Block::default().borders(Borders::ALL).title("History"));
        frame.render_widget(history, chunks[3]);
    }

    /// Handle terminal events
    fn handle_event(_state: &State, event: &SimulatedEvent) -> Option<Msg> {
        use crossterm::event::KeyCode;

        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('+') | KeyCode::Up => Some(Msg::Increment),
                KeyCode::Char('-') | KeyCode::Down => Some(Msg::Decrement),
                KeyCode::Char('r') | KeyCode::Char('R') => Some(Msg::Reset),
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a headless runtime for demonstration
    let mut runtime = Runtime::<CounterApp, _>::headless(60, 20)?;

    // Simulate some user interactions
    println!("=== Counter App Demo ===\n");

    // Initial render
    runtime.render()?;
    println!("Initial state:");
    println!("{}\n", runtime.backend());

    // Simulate incrementing
    runtime.dispatch(Msg::Increment);
    runtime.dispatch(Msg::Increment);
    runtime.dispatch(Msg::Increment);
    runtime.render()?;
    println!("After 3 increments:");
    println!("{}\n", runtime.backend());

    // Simulate decrementing
    runtime.dispatch(Msg::Decrement);
    runtime.render()?;
    println!("After 1 decrement:");
    println!("{}\n", runtime.backend());

    // Reset
    runtime.dispatch(Msg::Reset);
    runtime.render()?;
    println!("After reset:");
    println!("{}\n", runtime.backend());

    // Show final state
    println!("Final count: {}", runtime.state().count);
    println!("History entries: {}", runtime.state().history.len());

    Ok(())
}
