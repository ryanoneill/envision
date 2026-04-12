//! SplitPanel example -- resizable two-pane layout.
//!
//! Demonstrates the SplitPanel component for creating a resizable
//! split view with focus toggling and ratio adjustment.
//!
//! Run with: cargo run --example split_panel --features compound-components

use envision::prelude::*;

/// Application marker type.
struct SplitPanelApp;

/// Application state.
#[derive(Clone)]
struct State {
    split: SplitPanelState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Split(SplitPanelMessage),
    Quit,
}

impl App for SplitPanelApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let split = SplitPanelState::new(SplitOrientation::Vertical)
            .with_ratio(0.4)
            .with_resize_step(0.1)
            .with_bounds(0.2, 0.8);

        (State { split }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Split(m) => {
                SplitPanel::update(&mut state.split, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        // Render the split panel borders
        let theme = Theme::default();
        SplitPanel::view(
            &state.split,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        // Render content inside each pane
        let (first_area, second_area) = state.split.layout(chunks[0]);

        // Add padding for borders
        let first_inner = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .inner(first_area);
        let second_inner = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .inner(second_area);

        let first_content = "File List\n\n\
             src/\n  main.rs\n  lib.rs\n\
             tests/\n  test.rs";
        frame.render_widget(ratatui::widgets::Paragraph::new(first_content), first_inner);

        let second_content = "Preview\n\n\
             fn main() {\n\
             \x20\x20\x20 println!(\"Hello\");\n\
             }";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(second_content),
            second_inner,
        );

        let pane = if state.split.is_first_pane_focused() {
            "1"
        } else {
            "2"
        };
        let status = format!(
            " Ratio: {:.0}% | Pane: {} | Tab: switch, Ctrl+Arrows: resize, q: quit",
            state.split.ratio() * 100.0,
            pane
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }
        SplitPanel::handle_event(&state.split, event, &EventContext::new().focused(true))
            .map(Msg::Split)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<SplitPanelApp, _>::virtual_terminal(60, 14)?;

    println!("=== SplitPanel Example ===\n");

    // Initial render: 40/60 split with first pane focused
    vt.tick()?;
    println!("Initial state (40/60 split, pane 1 focused):");
    println!("{}\n", vt.display());

    // Toggle focus to second pane
    vt.dispatch(Msg::Split(SplitPanelMessage::FocusOther));
    vt.tick()?;
    println!("After toggling focus to pane 2:");
    println!("{}\n", vt.display());

    // Grow first pane
    vt.dispatch(Msg::Split(SplitPanelMessage::GrowFirst));
    vt.tick()?;
    println!("After growing first pane (50/50):");
    println!("{}\n", vt.display());

    // Reset ratio
    vt.dispatch(Msg::Split(SplitPanelMessage::ResetRatio));
    vt.dispatch(Msg::Split(SplitPanelMessage::FocusFirst));
    vt.tick()?;
    println!("After resetting ratio and focusing pane 1:");
    println!("{}\n", vt.display());

    Ok(())
}
