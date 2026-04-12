//! ScrollableText example — scrollable read-only text display.
//!
//! Demonstrates the ScrollableText component with keyboard-driven
//! scrolling through long content. Exercises scroll-up, scroll-down,
//! page navigation, and home/end jumps.
//!
//! Run with: cargo run --example scrollable_text

use envision::prelude::*;
use ratatui::widgets::Paragraph;

/// Application marker type.
struct ScrollableTextApp;

/// Application state wrapping a single ScrollableText.
#[derive(Clone)]
struct State {
    text: ScrollableTextState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Text(ScrollableTextMessage),
    Quit,
}

impl App for ScrollableTextApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let content = (1..=50)
            .map(|i| {
                format!(
                    "Line {}: Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
                    i
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let text = ScrollableTextState::new()
            .with_content(&content)
            .with_title("Scrollable Content");

        (State { text }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Text(m) => {
                state.text.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        let theme = Theme::default();
        ScrollableText::view(
            &state.text,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let status = Paragraph::new(format!(
            " Scroll offset: {} | Up/Down scroll | PgUp/PgDn page | Home/End | Esc quit",
            state.text.scroll_offset()
        ))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status, chunks[1]);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.key, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }

        ScrollableText::handle_event(&state.text, event, &EventContext::new().focused(true))
            .map(Msg::Text)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ScrollableTextApp, _>::virtual_terminal(70, 20)?;

    println!("=== ScrollableText Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial view (showing first lines):");
    println!("{}\n", vt.display());

    // Scroll down a few lines
    vt.dispatch(Msg::Text(ScrollableTextMessage::ScrollDown));
    vt.dispatch(Msg::Text(ScrollableTextMessage::ScrollDown));
    vt.dispatch(Msg::Text(ScrollableTextMessage::ScrollDown));
    vt.tick()?;
    println!("After scrolling down 3 lines:");
    println!("{}\n", vt.display());

    // Page down
    vt.dispatch(Msg::Text(ScrollableTextMessage::PageDown(10)));
    vt.tick()?;
    println!("After page down (10 lines):");
    println!("{}\n", vt.display());

    // Jump to end
    vt.dispatch(Msg::Text(ScrollableTextMessage::End));
    vt.tick()?;
    println!("At the end:");
    println!("{}\n", vt.display());

    // Jump back to top
    vt.dispatch(Msg::Text(ScrollableTextMessage::Home));
    vt.tick()?;
    println!("Back at the top:");
    println!("{}\n", vt.display());

    println!("Final scroll offset: {}", vt.state().text.scroll_offset());

    Ok(())
}
