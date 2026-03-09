//! StyledText example -- interactive rich text display with semantic formatting.
//!
//! Demonstrates the StyledText component for rendering structured content
//! including headings, paragraphs, bullet lists, code blocks, and horizontal
//! rules with keyboard-driven scrolling.
//!
//! Controls:
//!   Up/k        Scroll up one line
//!   Down/j      Scroll down one line
//!   Page Up     Scroll up one page
//!   Page Down   Scroll down one page
//!   Home        Jump to the top
//!   End         Jump to the bottom
//!   q/Esc       Quit
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
            .text(
                "Envision provides a component-based approach to building terminal \
                 user interfaces with first-class support for both interactive \
                 terminal use and programmatic control.",
            )
            .heading(2, "Features")
            .bullet_list(vec![
                vec![
                    styled_text::StyledInline::Bold("Component System".to_string()),
                    styled_text::StyledInline::Plain(
                        " - 25+ composable components".to_string(),
                    ),
                ],
                vec![
                    styled_text::StyledInline::Bold("Theme Support".to_string()),
                    styled_text::StyledInline::Plain(
                        " - Consistent styling across components".to_string(),
                    ),
                ],
                vec![
                    styled_text::StyledInline::Bold("Keyboard Navigation".to_string()),
                    styled_text::StyledInline::Plain(
                        " - Full keyboard-driven interaction".to_string(),
                    ),
                ],
                vec![
                    styled_text::StyledInline::Bold("Virtual Terminal".to_string()),
                    styled_text::StyledInline::Plain(
                        " - Headless testing without a real terminal".to_string(),
                    ),
                ],
                vec![
                    styled_text::StyledInline::Bold("Async Support".to_string()),
                    styled_text::StyledInline::Plain(
                        " - Built on tokio for async operations".to_string(),
                    ),
                ],
            ])
            .heading(2, "Getting Started")
            .text("Add envision to your Cargo.toml:")
            .code_block(
                Some("toml"),
                "[dependencies]\nenvision = { version = \"0.6\", features = [\"full\"] }",
            )
            .text("Then create your application:")
            .code_block(
                Some("rust"),
                "use envision::prelude::*;\n\n\
                 struct MyApp;\n\n\
                 impl App for MyApp {\n    \
                     type State = MyState;\n    \
                     type Message = MyMsg;\n    \
                     // ...\n\
                 }",
            )
            .horizontal_rule()
            .heading(2, "Architecture")
            .text(
                "Envision follows The Elm Architecture (TEA), where each component \
                 has three core functions: init, update, and view. State is immutable \
                 from the view's perspective, and all mutations happen through messages \
                 processed by the update function.",
            )
            .text(
                "Components implement the Component trait, which provides a consistent \
                 interface for state management, event handling, and rendering. Compound \
                 components compose simpler components to build richer interfaces.",
            )
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
            " Scroll: {} | Up/Down: scroll | PgUp/PgDn: page | Home/End: jump | q: quit",
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

#[tokio::main]
async fn main() -> envision::Result<()> {
    let _final_state = TerminalRuntime::<StyledTextApp>::new_terminal()?
        .run_terminal()
        .await?;
    Ok(())
}
