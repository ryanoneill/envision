//! PaneLayout example -- interactive N-pane layout with proportional sizing.
//!
//! Demonstrates the PaneLayout compound component for managing multiple panes
//! with configurable proportions, focus cycling, and resize operations. Shows
//! a three-pane IDE-style layout with a sidebar, editor, and terminal.
//!
//! Controls:
//!   Tab         Cycle focus to the next pane
//!   Shift+Tab   Cycle focus to the previous pane
//!   Ctrl+Right  Grow the focused pane
//!   Ctrl+Left   Shrink the focused pane
//!   r           Reset proportions to defaults
//!   q/Esc       Quit
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

        let layout = PaneLayoutState::new(pane_layout::PaneDirection::Horizontal, panes)
            .with_resize_step(0.05);

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
        PaneLayout::view(
            &state.layout,
            frame,
            chunks[0],
            &theme,
            &ViewContext::default(),
        );

        // Render content in each pane
        let rects = state.layout.layout(chunks[0]);
        let pane_contents = [
            "src/\n  main.rs\n  lib.rs\n  utils.rs\n  config.rs",
            "fn main() {\n    let config = Config::load();\n    println!(\n      \"Hello, {}!\",\n      config.name\n    );\n}",
            "$ cargo build\n  Compiling envision v0.6.0\n  Finished dev [unoptimized + debuginfo]\n$ _",
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
            " Focus: {} | Panes: {} | Tab: cycle | Ctrl+Arrows: resize | r: reset | q: quit",
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
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
                KeyCode::Char('r') => {
                    return Some(Msg::Layout(PaneLayoutMessage::ResetProportions));
                }
                _ => {}
            }
        }
        PaneLayout::handle_event(&state.layout, event, &ViewContext::new().focused(true))
            .map(Msg::Layout)
    }
}

#[tokio::main]
async fn main() -> envision::Result<()> {
    let _final_state = TerminalRuntime::<PaneLayoutApp>::new_terminal()?
        .run_terminal()
        .await?;
    Ok(())
}
