//! Select example -- dropdown selection with options.
//!
//! Demonstrates the Select component with a list of options,
//! keyboard navigation, and selection confirmation.
//!
//! Run with: cargo run --example select --features input-components

use envision::prelude::*;

/// Application marker type.
struct SelectApp;

/// Application state with a select dropdown.
#[derive(Clone)]
struct State {
    color: SelectState,
    size: SelectState,
    focus_index: usize,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Color(SelectMessage),
    Size(SelectMessage),
    FocusNext,
    FocusPrev,
    Quit,
}

impl State {
    fn set_focus(&mut self, index: usize) {
        self.focus_index = index;
    }
}

impl App for SelectApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let color = SelectState::new(vec!["Red", "Green", "Blue", "Yellow"])
            .with_placeholder("Choose a color...");

        let size = SelectState::new(vec!["Small", "Medium", "Large", "Extra Large"])
            .with_placeholder("Choose a size...");

        let state = State {
            color,
            size,
            focus_index: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Color(m) => {
                Select::update(&mut state.color, m);
            }
            Msg::Size(m) => {
                Select::update(&mut state.size, m);
            }
            Msg::FocusNext => {
                let next = (state.focus_index + 1) % 2;
                state.set_focus(next);
            }
            Msg::FocusPrev => {
                let prev = (state.focus_index + 1) % 2;
                state.set_focus(prev);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        Select::view(
            &state.color,
            frame,
            chunks[0],
            &theme,
            &ViewContext::new().focused(state.focus_index == 0),
        );
        Select::view(
            &state.size,
            frame,
            chunks[1],
            &theme,
            &ViewContext::new().focused(state.focus_index == 1),
        );

        // Summary
        let color_val = state.color.selected_value().unwrap_or("none");
        let size_val = state.size.selected_value().unwrap_or("none");
        let summary = format!("  Color: {}  Size: {}", color_val, size_val);
        let summary_widget = ratatui::widgets::Paragraph::new(summary).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Selection Summary"),
        );
        frame.render_widget(summary_widget, chunks[2]);

        let status = format!(
            " Focus: {} | Tab: switch, Enter/Space: open, Arrows: navigate, q: quit",
            state.focus_index
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[3],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            // Only allow quit/tab when dropdown is closed
            let any_open = state.color.is_open() || state.size.is_open();
            if !any_open {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
                    KeyCode::Tab => return Some(Msg::FocusNext),
                    KeyCode::BackTab => return Some(Msg::FocusPrev),
                    _ => {}
                }
            }
        }
        // Route event to focused select
        match state.focus_index {
            0 => Select::handle_event(&state.color, event, &ViewContext::new().focused(true))
                .map(Msg::Color),
            _ => Select::handle_event(&state.size, event, &ViewContext::new().focused(true))
                .map(Msg::Size),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<SelectApp, _>::virtual_terminal(50, 14)?;

    println!("=== Select Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state (placeholder shown):");
    println!("{}\n", vt.display());

    // Open color dropdown
    vt.dispatch(Msg::Color(SelectMessage::Open));
    vt.tick()?;
    println!("After opening color dropdown:");
    println!("{}\n", vt.display());

    // Navigate down and confirm
    vt.dispatch(Msg::Color(SelectMessage::Down));
    vt.dispatch(Msg::Color(SelectMessage::Down));
    vt.dispatch(Msg::Color(SelectMessage::Confirm));
    vt.tick()?;
    println!("After selecting 'Blue':");
    println!("{}\n", vt.display());

    // Switch to size and select
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::Size(SelectMessage::Open));
    vt.dispatch(Msg::Size(SelectMessage::Down));
    vt.dispatch(Msg::Size(SelectMessage::Confirm));
    vt.tick()?;
    println!("After selecting 'Medium' size:");
    println!("{}\n", vt.display());

    Ok(())
}
