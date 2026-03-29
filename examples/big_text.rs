//! BigText example -- large pixel text for dashboard hero numbers.
//!
//! Demonstrates the BigText component with various configurations:
//! clock display, KPI values, and colored metrics.
//!
//! Run with: cargo run --example big_text

use envision::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Application marker type.
struct BigTextApp;

/// Application state holding multiple big text configurations.
#[derive(Clone)]
struct State {
    clock: BigTextState,
    metric: BigTextState,
    percentage: BigTextState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    UpdateClock(String),
    UpdateMetric(String),
    Quit,
}

impl App for BigTextApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let clock = BigTextState::new("12:30")
            .with_color(Color::Cyan)
            .with_alignment(Alignment::Center);

        let metric = BigTextState::new("1234")
            .with_color(Color::Green)
            .with_alignment(Alignment::Center);

        let percentage = BigTextState::new("99.9%")
            .with_color(Color::Yellow)
            .with_alignment(Alignment::Center);

        let state = State {
            clock,
            metric,
            percentage,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::UpdateClock(time) => state.clock.set_text(time),
            Msg::UpdateMetric(value) => state.metric.set_text(value),
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

        let theme = Theme::default();

        let clock_label = Paragraph::new(" Clock").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(clock_label, chunks[0]);
        BigText::view(&state.clock, frame, chunks[1], &theme);

        let metric_label =
            Paragraph::new(" Active Users").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(metric_label, chunks[2]);
        BigText::view(&state.metric, frame, chunks[3], &theme);

        let pct_label = Paragraph::new(" Uptime").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(pct_label, chunks[4]);
        BigText::view(&state.percentage, frame, chunks[5], &theme);

        let footer = Paragraph::new(" BigText dashboard metrics | Esc to quit")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(footer, chunks[6]);
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<BigTextApp, _>::virtual_terminal(60, 24)?;

    println!("=== BigText Dashboard Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Dashboard with big text metrics:");
    println!("{}\n", vt.display());

    // Update the clock
    vt.dispatch(Msg::UpdateClock("13:45".to_string()));
    vt.tick()?;
    println!("After updating clock:");
    println!("{}\n", vt.display());

    // Update the metric
    vt.dispatch(Msg::UpdateMetric("5678".to_string()));
    vt.tick()?;
    println!("After updating metric:");
    println!("{}\n", vt.display());

    // Verify state
    println!("Clock: {}", vt.state().clock.text());
    println!("Metric: {}", vt.state().metric.text());
    println!("Percentage: {}", vt.state().percentage.text());

    Ok(())
}
