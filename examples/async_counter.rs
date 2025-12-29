//! Async Counter example demonstrating the async runtime with subscriptions.
//!
//! This example shows async features:
//! - AsyncRuntime for running TEA apps with tokio
//! - TickSubscription for periodic updates
//! - Command::perform_async for async side effects
//! - Subscriptions that produce messages over time
//!
//! Run with: cargo run --example async_counter

use std::time::Duration;

use envision::app::tick;
use envision::prelude::*;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// The application marker type
struct AsyncCounterApp;

/// Application state
#[derive(Default, Clone)]
struct State {
    count: i32,
    auto_increment: bool,
    ticks: u64,
    async_result: Option<String>,
}

/// Messages for the async counter
#[derive(Clone, Debug)]
#[allow(dead_code)]
enum Msg {
    /// Increment the counter
    Increment,
    /// Decrement the counter
    Decrement,
    /// Toggle auto-increment mode
    ToggleAuto,
    /// Tick from subscription (auto-increment when enabled)
    Tick,
    /// Result from async operation
    AsyncResult(String),
    /// Trigger an async operation
    FetchData,
    /// Quit the application
    Quit,
}

impl App for AsyncCounterApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => {
                state.count += 1;
            }
            Msg::Decrement => {
                state.count -= 1;
            }
            Msg::ToggleAuto => {
                state.auto_increment = !state.auto_increment;
            }
            Msg::Tick => {
                state.ticks += 1;
                if state.auto_increment {
                    state.count += 1;
                }
            }
            Msg::FetchData => {
                state.async_result = Some("Loading...".to_string());
                // Simulate an async API call
                return Command::perform_async(async {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    Some(Msg::AsyncResult("Data loaded successfully!".to_string()))
                });
            }
            Msg::AsyncResult(result) => {
                state.async_result = Some(result);
            }
            Msg::Quit => {
                return Command::quit();
            }
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::vertical([
            Constraint::Length(3), // Title
            Constraint::Length(5), // Counter
            Constraint::Length(3), // Tick counter
            Constraint::Length(3), // Auto-increment status
            Constraint::Length(3), // Async result
            Constraint::Length(3), // Controls
            Constraint::Min(0),    // Spacer
        ])
        .split(area);

        // Title
        let title = Paragraph::new("Async Counter - Subscription Demo")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Counter display
        let counter_style = if state.count >= 0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };

        let counter = Paragraph::new(format!("Count: {}", state.count))
            .style(counter_style.add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Counter"));
        frame.render_widget(counter, chunks[1]);

        // Tick counter
        let ticks = Paragraph::new(format!("Ticks: {}", state.ticks))
            .style(Style::default().fg(Color::Blue))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Subscription Ticks"),
            );
        frame.render_widget(ticks, chunks[2]);

        // Auto-increment status
        let auto_status = if state.auto_increment {
            Span::styled("ON", Style::default().fg(Color::Green))
        } else {
            Span::styled("OFF", Style::default().fg(Color::Red))
        };
        let auto_para =
            Paragraph::new(Line::from(vec![Span::raw("Auto-Increment: "), auto_status]))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
        frame.render_widget(auto_para, chunks[3]);

        // Async result
        let async_text = state
            .async_result
            .as_deref()
            .unwrap_or("Press [F] to fetch data");
        let async_para = Paragraph::new(async_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Async Result"));
        frame.render_widget(async_para, chunks[4]);

        // Controls
        let controls = Paragraph::new(Line::from(vec![
            Span::styled("[+/-] ", Style::default().fg(Color::Green)),
            Span::raw("Count  "),
            Span::styled("[A] ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle Auto  "),
            Span::styled("[F] ", Style::default().fg(Color::Yellow)),
            Span::raw("Fetch  "),
            Span::styled("[Q] ", Style::default().fg(Color::Magenta)),
            Span::raw("Quit"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Controls"));
        frame.render_widget(controls, chunks[5]);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an async runtime
    let mut runtime = AsyncRuntime::<AsyncCounterApp, _>::headless(60, 20)?;

    // Add tick subscription manually (since subscriptions aren't part of App trait)
    runtime.subscribe(tick(Duration::from_millis(500)).with_message(|| Msg::Tick));

    println!("=== Async Counter Demo ===\n");

    // Initial render
    runtime.render()?;
    println!("Initial state:");
    println!("{}\n", runtime.backend());

    // Simulate some messages
    runtime.dispatch(Msg::Increment);
    runtime.dispatch(Msg::Increment);
    runtime.render()?;
    println!("After 2 increments:");
    println!("{}\n", runtime.backend());

    // Enable auto-increment
    runtime.dispatch(Msg::ToggleAuto);
    runtime.render()?;
    println!("Auto-increment enabled:");
    println!("{}\n", runtime.backend());

    // Run a few ticks (this would normally happen in an event loop)
    println!("Running 3 ticks with subscriptions...\n");
    for i in 0..3 {
        runtime.run_ticks(1)?;
        // Give subscription time to fire
        tokio::time::sleep(Duration::from_millis(600)).await;
        runtime.process_pending();
        runtime.render()?;
        println!("After tick {}:", i + 1);
        println!(
            "Count: {}, Ticks: {}\n",
            runtime.state().count,
            runtime.state().ticks
        );
    }

    // Trigger async operation
    println!("Triggering async fetch...");
    runtime.dispatch(Msg::FetchData);
    runtime.render()?;
    println!("Before async completes:");
    println!("{}\n", runtime.backend());

    // Wait for async to complete
    tokio::time::sleep(Duration::from_millis(600)).await;
    runtime.process_pending();
    runtime.render()?;
    println!("After async completes:");
    println!("{}\n", runtime.backend());

    println!("Final state:");
    println!("  Count: {}", runtime.state().count);
    println!("  Ticks: {}", runtime.state().ticks);
    println!("  Auto: {}", runtime.state().auto_increment);
    println!("  Async Result: {:?}", runtime.state().async_result);

    Ok(())
}
