//! ScrollView example -- a generic scrollable container.
//!
//! Demonstrates the ScrollView component wrapping arbitrary content.
//! The parent application renders content into the area returned by
//! `content_area()`, while the ScrollView manages borders, scrollbar,
//! and scroll state.
//!
//! Run with: cargo run --example scroll_view --features display-components

use envision::prelude::*;
use ratatui::widgets::Paragraph;

/// Application marker type.
struct ScrollViewApp;

/// Application state wrapping a single ScrollView.
#[derive(Clone)]
struct State {
    scroll_view: ScrollViewState,
    lines: Vec<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    ScrollView(ScrollViewMessage),
    Quit,
}

impl App for ScrollViewApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let lines: Vec<String> = (1..=80)
            .map(|i| format!("Item {:>3}: Configuration setting for module {}", i, i))
            .collect();

        let scroll_view = ScrollViewState::new()
            .with_content_height(lines.len() as u16)
            .with_title("Configuration Settings");

        (State { scroll_view, lines }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::ScrollView(m) => {
                state.scroll_view.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        // Render the scroll view (border + scrollbar)
        ScrollView::view(
            &state.scroll_view,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        // Render content inside the content area
        let content_area = state.scroll_view.content_area(chunks[0]);
        if content_area.height > 0 && content_area.width > 0 {
            let offset = state.scroll_view.scroll_offset();
            let visible_lines: Vec<&str> = state
                .lines
                .iter()
                .skip(offset)
                .take(content_area.height as usize)
                .map(|s| s.as_str())
                .collect();
            let text = visible_lines.join("\n");
            let content = Paragraph::new(text).style(theme.normal_style());
            frame.render_widget(content, content_area);
        }

        let status = Paragraph::new(format!(
            " Offset: {} / {} | Up/Down scroll | PgUp/PgDn page | Home/End | q: quit",
            state.scroll_view.scroll_offset(),
            state.lines.len(),
        ))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status, chunks[1]);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }
        ScrollView::handle_event(
            &state.scroll_view,
            event,
            &EventContext::new().focused(true),
        )
        .map(Msg::ScrollView)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ScrollViewApp, _>::virtual_builder(70, 20).build()?;

    println!("=== ScrollView Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial view (showing first items):");
    println!("{}\n", vt.display());

    // Scroll down a few lines
    vt.dispatch(Msg::ScrollView(ScrollViewMessage::ScrollDown));
    vt.dispatch(Msg::ScrollView(ScrollViewMessage::ScrollDown));
    vt.dispatch(Msg::ScrollView(ScrollViewMessage::ScrollDown));
    vt.tick()?;
    println!("After scrolling down 3 lines:");
    println!("{}\n", vt.display());

    // Page down
    vt.dispatch(Msg::ScrollView(ScrollViewMessage::PageDown));
    vt.tick()?;
    println!("After page down:");
    println!("{}\n", vt.display());

    // Jump to end
    vt.dispatch(Msg::ScrollView(ScrollViewMessage::End));
    vt.tick()?;
    println!("At the end:");
    println!("{}\n", vt.display());

    // Jump back to top
    vt.dispatch(Msg::ScrollView(ScrollViewMessage::Home));
    vt.tick()?;
    println!("Back at the top:");
    println!("{}\n", vt.display());

    println!(
        "Final scroll offset: {}",
        vt.state().scroll_view.scroll_offset()
    );

    Ok(())
}
