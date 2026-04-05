//! Calendar example -- a navigable month calendar.
//!
//! Demonstrates the Calendar component with date selection, event markers,
//! and month navigation.
//!
//! Run with: cargo run --example calendar --features display-components

use envision::prelude::*;

/// Application marker type.
struct CalendarApp;

/// Application state wrapping a Calendar.
#[derive(Clone)]
struct State {
    calendar: CalendarState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Calendar(CalendarMessage),
    Quit,
}

impl App for CalendarApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let calendar = CalendarState::new(2026, 3)
            .with_selected_day(1)
            .with_title("My Calendar")
            .with_event(2026, 3, 10, Color::Green)
            .with_event(2026, 3, 15, Color::Red)
            .with_event(2026, 3, 24, Color::Cyan);

        (State { calendar }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Calendar(m) => {
                Calendar::update(&mut state.calendar, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        Calendar::view(
            &state.calendar,
            frame,
            chunks[0],
            &theme,
            &ViewContext::default(),
        );

        let selected_info = match state.calendar.selected_day() {
            Some(day) => format!(
                "Selected: {} {}, {}",
                state.calendar.month_name(),
                day,
                state.calendar.year()
            ),
            None => "No date selected".to_string(),
        };
        let status = format!(
            " {} | Arrows: navigate, PgUp/PgDn: month, Enter: select, q: quit",
            selected_info
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                return Some(Msg::Quit);
            }
        }
        Calendar::handle_event(&state.calendar, event, &ViewContext::new().focused(true))
            .map(Msg::Calendar)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<CalendarApp, _>::virtual_terminal(40, 14)?;

    println!("=== Calendar Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state (March 2026):");
    println!("{}\n", vt.display());

    // Select a day
    vt.dispatch(Msg::Calendar(CalendarMessage::SelectDay(15)));
    vt.tick()?;
    println!("After selecting day 15:");
    println!("{}\n", vt.display());

    // Navigate to next month
    vt.dispatch(Msg::Calendar(CalendarMessage::NextMonth));
    vt.tick()?;
    println!("After navigating to April:");
    println!("{}\n", vt.display());

    // Navigate back
    vt.dispatch(Msg::Calendar(CalendarMessage::PrevMonth));
    vt.tick()?;
    println!("Back to March:");
    println!("{}\n", vt.display());

    Ok(())
}
