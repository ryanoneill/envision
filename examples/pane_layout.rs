//! PaneLayout example -- N-pane layout with proportional sizing.
//!
//! Demonstrates the PaneLayout component for managing multiple panes
//! with configurable proportions, focus cycling, and resize operations.
//!
//! Run with: cargo run --example pane_layout --features compound-components

use envision::prelude::*;

/// Application marker type.
struct PaneLayoutApp;

/// Application state.
#[derive(Clone)]
struct State {
    layout: PaneLayoutState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Layout(PaneLayoutMessage),
    Quit,
}

impl App for PaneLayoutApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let panes = vec![
            pane_layout::PaneConfig::new("sidebar")
                .with_title("Files")
                .with_proportion(0.25)
                .with_min_size(10),
            pane_layout::PaneConfig::new("editor")
                .with_title("Editor")
                .with_proportion(0.50),
            pane_layout::PaneConfig::new("terminal")
                .with_title("Terminal")
                .with_proportion(0.25)
                .with_min_size(10),
        ];

        let mut layout = PaneLayoutState::new(pane_layout::PaneDirection::Horizontal, panes)
            .with_resize_step(0.05);

        layout.set_focused(true);

        (State { layout }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Layout(m) => {
                PaneLayout::update(&mut state.layout, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        // Render pane borders
        PaneLayout::view(&state.layout, frame, chunks[0], &theme);

        // Render content in each pane
        let rects = state.layout.layout(chunks[0]);
        let pane_contents = [
            "src/\n  main.rs\n  lib.rs\n  utils.rs",
            "fn main() {\n    println!(\n      \"Hello\"\n    );\n}",
            "$ cargo build\n  Compiling...\n  Finished",
        ];

        for (i, rect) in rects.iter().enumerate() {
            let inner = ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .inner(*rect);
            if let Some(content) = pane_contents.get(i) {
                frame.render_widget(ratatui::widgets::Paragraph::new(*content), inner);
            }
        }

        let focused_id = state.layout.focused_pane_id().unwrap_or("none").to_string();
        let status = format!(
            " Focus: {} | Panes: {} | Tab: cycle, Ctrl+Arrows: resize, q: quit",
            focused_id,
            state.layout.pane_count()
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
        state.layout.handle_event(event).map(Msg::Layout)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<PaneLayoutApp, _>::virtual_terminal(80, 14)?;

    println!("=== PaneLayout Example ===\n");

    // Initial render: three panes
    vt.tick()?;
    println!("Initial state (sidebar focused):");
    println!("{}\n", vt.display());

    // Cycle focus to editor pane
    vt.dispatch(Msg::Layout(PaneLayoutMessage::FocusNext));
    vt.tick()?;
    println!("After Tab (editor focused):");
    println!("{}\n", vt.display());

    // Cycle focus to terminal pane
    vt.dispatch(Msg::Layout(PaneLayoutMessage::FocusNext));
    vt.tick()?;
    println!("After Tab (terminal focused):");
    println!("{}\n", vt.display());

    // Grow the focused pane (terminal)
    vt.dispatch(Msg::Layout(PaneLayoutMessage::GrowFocused));
    vt.dispatch(Msg::Layout(PaneLayoutMessage::GrowFocused));
    vt.tick()?;
    println!("After growing terminal pane:");
    println!("{}\n", vt.display());

    // Reset proportions
    vt.dispatch(Msg::Layout(PaneLayoutMessage::ResetProportions));
    vt.tick()?;
    println!("After resetting proportions:");
    println!("{}\n", vt.display());

    Ok(())
}
