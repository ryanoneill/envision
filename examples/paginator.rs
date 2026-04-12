//! Paginator example -- page navigation indicators.
//!
//! Demonstrates all four Paginator display styles:
//! PageOfTotal, RangeOfTotal, Dots, and Compact.
//!
//! Run with: cargo run --example paginator --features display-components

use envision::prelude::*;

/// Application marker type.
struct PaginatorApp;

/// Application state with multiple paginator styles.
#[derive(Clone)]
struct State {
    page_of_total: PaginatorState,
    range_of_total: PaginatorState,
    dots: PaginatorState,
    compact: PaginatorState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Next,
    Prev,
    Quit,
}

impl App for PaginatorApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let state = State {
            page_of_total: PaginatorState::new(12)
                .with_style(PaginatorStyle::PageOfTotal)
                .with_current_page(2),
            range_of_total: PaginatorState::from_items(2847, 100)
                .with_style(PaginatorStyle::RangeOfTotal)
                .with_current_page(2),
            dots: PaginatorState::new(8)
                .with_style(PaginatorStyle::Dots)
                .with_current_page(3),
            compact: PaginatorState::new(12)
                .with_style(PaginatorStyle::Compact)
                .with_current_page(5),
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Next => {
                Paginator::update(&mut state.page_of_total, PaginatorMessage::NextPage);
                Paginator::update(&mut state.range_of_total, PaginatorMessage::NextPage);
                Paginator::update(&mut state.dots, PaginatorMessage::NextPage);
                Paginator::update(&mut state.compact, PaginatorMessage::NextPage);
            }
            Msg::Prev => {
                Paginator::update(&mut state.page_of_total, PaginatorMessage::PrevPage);
                Paginator::update(&mut state.range_of_total, PaginatorMessage::PrevPage);
                Paginator::update(&mut state.dots, PaginatorMessage::PrevPage);
                Paginator::update(&mut state.compact, PaginatorMessage::PrevPage);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(1), // PageOfTotal label
            Constraint::Length(1), // PageOfTotal
            Constraint::Length(1), // spacer
            Constraint::Length(1), // RangeOfTotal label
            Constraint::Length(1), // RangeOfTotal
            Constraint::Length(1), // spacer
            Constraint::Length(1), // Dots label
            Constraint::Length(1), // Dots
            Constraint::Length(1), // spacer
            Constraint::Length(1), // Compact label
            Constraint::Length(1), // Compact
            Constraint::Min(0),
        ])
        .split(area);

        frame.render_widget(
            ratatui::widgets::Paragraph::new("PageOfTotal:").style(theme.info_style()),
            chunks[0],
        );
        Paginator::view(
            &state.page_of_total,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );

        frame.render_widget(
            ratatui::widgets::Paragraph::new("RangeOfTotal:").style(theme.info_style()),
            chunks[3],
        );
        Paginator::view(
            &state.range_of_total,
            &mut RenderContext::new(frame, chunks[4], &theme),
        );

        frame.render_widget(
            ratatui::widgets::Paragraph::new("Dots:").style(theme.info_style()),
            chunks[6],
        );
        Paginator::view(
            &state.dots,
            &mut RenderContext::new(frame, chunks[7], &theme),
        );

        frame.render_widget(
            ratatui::widgets::Paragraph::new("Compact:").style(theme.info_style()),
            chunks[9],
        );
        Paginator::view(
            &state.compact,
            &mut RenderContext::new(frame, chunks[10], &theme),
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                Key::Right | Key::Char('l') => Some(Msg::Next),
                Key::Left | Key::Char('h') => Some(Msg::Prev),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<PaginatorApp, _>::virtual_terminal(50, 14)?;

    println!("=== Paginator Example ===\n");

    vt.tick()?;
    println!("Initial state:");
    println!("{}\n", vt.display());

    // Navigate forward
    vt.dispatch(Msg::Next);
    vt.tick()?;
    println!("After navigating forward:");
    println!("{}\n", vt.display());

    // Navigate backward twice
    vt.dispatch(Msg::Prev);
    vt.dispatch(Msg::Prev);
    vt.tick()?;
    println!("After navigating backward twice:");
    println!("{}\n", vt.display());

    Ok(())
}
