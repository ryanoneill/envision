//! StyledText example -- rich text display with semantic formatting.
//!
//! Demonstrates the StyledText component for rendering headings,
//! paragraphs, bullet lists, code blocks, and horizontal rules.
//!
//! Run with: cargo run --example styled_text --features display-components

use envision::prelude::*;

/// Application marker type.
struct StyledTextApp;

/// Application state.
#[derive(Clone)]
struct State {
    text: StyledTextState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    StyledText(StyledTextMessage),
    Quit,
}

impl App for StyledTextApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let content = styled_text::StyledContent::new()
            .heading(1, "Welcome to Envision")
            .text("A modern TUI framework built with Rust, using The Elm Architecture.")
            .heading(2, "Features")
            .bullet_list(vec![
                vec![styled_text::StyledInline::Bold(
                    "Component System".to_string(),
                )],
                vec![styled_text::StyledInline::Bold("Theme Support".to_string())],
                vec![styled_text::StyledInline::Plain(
                    "Keyboard Navigation".to_string(),
                )],
                vec![styled_text::StyledInline::Plain(
                    "Virtual Terminal Testing".to_string(),
                )],
            ])
            .heading(2, "Getting Started")
            .text("Add envision to your Cargo.toml:")
            .code_block(Some("toml"), "[dependencies]\nenvision = \"0.1\"")
            .horizontal_rule()
            .text("Use Up/Down or j/k to scroll. Press q to quit.");

        let mut state = StyledTextState::new()
            .with_content(content)
            .with_title("Documentation");

        state.set_focused(true);

        (State { text: state }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::StyledText(m) => {
                StyledText::update(&mut state.text, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        StyledText::view(&state.text, frame, chunks[0], &theme);

        let status = format!(
            " Scroll: {} | Up/Down: scroll, q: quit",
            state.text.scroll_offset()
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
        state.text.handle_event(event).map(Msg::StyledText)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<StyledTextApp, _>::virtual_terminal(55, 18)?;

    println!("=== StyledText Example ===\n");

    // Initial render: top of content
    vt.tick()?;
    println!("Initial state (top of document):");
    println!("{}\n", vt.display());

    // Scroll down to see more content
    for _ in 0..5 {
        vt.dispatch(Msg::StyledText(StyledTextMessage::ScrollDown));
    }
    vt.tick()?;
    println!("After scrolling down 5 lines:");
    println!("{}\n", vt.display());

    // Scroll back to top
    vt.dispatch(Msg::StyledText(StyledTextMessage::Home));
    vt.tick()?;
    println!("After scrolling back to top:");
    println!("{}\n", vt.display());

    Ok(())
}
