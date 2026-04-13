//! MarkdownRenderer example -- rendered markdown with scrolling and source toggle.
//!
//! Demonstrates the MarkdownRenderer component with keyboard-driven scrolling
//! through rendered markdown content. Press `s` to toggle between the rendered
//! and raw source views.
//!
//! Run with: cargo run --example markdown_renderer --features markdown

use envision::prelude::*;
use ratatui::widgets::Paragraph;

/// Application marker type.
struct MarkdownRendererApp;

/// Application state wrapping a single MarkdownRenderer.
#[derive(Clone)]
struct State {
    renderer: MarkdownRendererState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Renderer(MarkdownRendererMessage),
    Quit,
}

const SAMPLE_MARKDOWN: &str = "\
# Envision Markdown Renderer

A **full-featured** markdown renderer for the terminal.

## Features

Supports the following markdown elements:

- **Bold text** for emphasis
- *Italic text* for subtle emphasis
- ~~Strikethrough~~ for deleted content
- `inline code` for code references

### Code Blocks

```rust
fn main() {
    println!(\"Hello, world!\");
    let answer = 42;
}
```

### Lists

Bullet lists:

- Item one
- Item two
- Item three

Numbered lists:

1. First step
2. Second step
3. Third step

### Links

Visit [Envision on GitHub](https://github.com/ryanoneill/envision) for more info.

### Blockquotes

> This is a blockquote. It is rendered with a left border
> and italic styling.

---

*Press `s` to toggle between rendered and source views.*
";

impl App for MarkdownRendererApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let renderer = MarkdownRendererState::new()
            .with_source(SAMPLE_MARKDOWN)
            .with_title("Markdown Preview");

        (State { renderer }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Renderer(m) => {
                state.renderer.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        let theme = Theme::default();
        MarkdownRenderer::view(
            &state.renderer,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let mode = if state.renderer.show_source() {
            "source"
        } else {
            "rendered"
        };
        let status = Paragraph::new(format!(
            " [{}] Scroll: {} | Up/Down | PgUp/PgDn | Home/End | s=toggle source | Esc=quit",
            mode,
            state.renderer.scroll_offset()
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

        MarkdownRenderer::handle_event(&state.renderer, event, &EventContext::new().focused(true))
            .map(Msg::Renderer)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<MarkdownRendererApp, _>::virtual_builder(70, 30).build()?;

    println!("=== MarkdownRenderer Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial view (rendered markdown):");
    println!("{}\n", vt.display());

    // Scroll down a few lines
    vt.dispatch(Msg::Renderer(MarkdownRendererMessage::ScrollDown));
    vt.dispatch(Msg::Renderer(MarkdownRendererMessage::ScrollDown));
    vt.dispatch(Msg::Renderer(MarkdownRendererMessage::ScrollDown));
    vt.tick()?;
    println!("After scrolling down 3 lines:");
    println!("{}\n", vt.display());

    // Toggle to source view
    vt.dispatch(Msg::Renderer(MarkdownRendererMessage::ToggleSource));
    vt.tick()?;
    println!("Source view:");
    println!("{}\n", vt.display());

    // Toggle back to rendered
    vt.dispatch(Msg::Renderer(MarkdownRendererMessage::ToggleSource));
    vt.tick()?;
    println!("Back to rendered view:");
    println!("{}\n", vt.display());

    println!(
        "Final scroll offset: {}",
        vt.state().renderer.scroll_offset()
    );

    Ok(())
}
