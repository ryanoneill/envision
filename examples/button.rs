//! Button example -- clickable buttons with focus cycling.
//!
//! Demonstrates the Button component with multiple buttons,
//! manual focus cycling, and press/release behavior.
//!
//! Run with: cargo run --example button --features input-components

use envision::prelude::*;

/// Application marker type.
struct ButtonApp;

/// Application state with multiple buttons.
#[derive(Clone)]
struct State {
    save: ButtonState,
    cancel: ButtonState,
    submit: ButtonState,
    focus_index: usize,
    last_pressed: Option<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Save(ButtonMessage),
    Cancel(ButtonMessage),
    Submit(ButtonMessage),
    FocusNext,
    FocusPrev,
    Quit,
}

const BUTTON_COUNT: usize = 3;

impl State {
    fn focused_button_mut(&mut self) -> &mut ButtonState {
        match self.focus_index {
            0 => &mut self.save,
            1 => &mut self.cancel,
            _ => &mut self.submit,
        }
    }

    fn set_focus(&mut self, index: usize) {
        self.focused_button_mut().set_focused(false);
        self.focus_index = index;
        self.focused_button_mut().set_focused(true);
    }
}

impl App for ButtonApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut save = ButtonState::new("Save");
        save.set_focused(true);

        let cancel = ButtonState::new("Cancel");
        let submit = ButtonState::new("Submit").with_disabled(true);

        let state = State {
            save,
            cancel,
            submit,
            focus_index: 0,
            last_pressed: None,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Save(m) => {
                if let Some(ButtonOutput::Pressed) = Button::update(&mut state.save, m) {
                    state.last_pressed = Some("Save".to_string());
                }
            }
            Msg::Cancel(m) => {
                if let Some(ButtonOutput::Pressed) = Button::update(&mut state.cancel, m) {
                    state.last_pressed = Some("Cancel".to_string());
                }
            }
            Msg::Submit(m) => {
                if let Some(ButtonOutput::Pressed) = Button::update(&mut state.submit, m) {
                    state.last_pressed = Some("Submit".to_string());
                }
            }
            Msg::FocusNext => {
                let next = (state.focus_index + 1) % BUTTON_COUNT;
                state.set_focus(next);
            }
            Msg::FocusPrev => {
                let prev = (state.focus_index + BUTTON_COUNT - 1) % BUTTON_COUNT;
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
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        Button::view(&state.save, frame, chunks[0], &theme);
        Button::view(&state.cancel, frame, chunks[1], &theme);
        Button::view(&state.submit, frame, chunks[2], &theme);

        // Last pressed info
        let info = match &state.last_pressed {
            Some(name) => format!("  Last pressed: {}", name),
            None => "  No button pressed yet".to_string(),
        };
        let info_widget = ratatui::widgets::Paragraph::new(info).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Status"),
        );
        frame.render_widget(info_widget, chunks[3]);

        let status = format!(
            " Focus: {} | Tab/Shift+Tab: navigate, Enter/Space: press, q: quit",
            state.focus_index
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[4],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
                KeyCode::Tab => return Some(Msg::FocusNext),
                KeyCode::BackTab => return Some(Msg::FocusPrev),
                _ => {}
            }
        }
        // Route event to focused button
        match state.focus_index {
            0 => state.save.handle_event(event).map(Msg::Save),
            1 => state.cancel.handle_event(event).map(Msg::Cancel),
            _ => state.submit.handle_event(event).map(Msg::Submit),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ButtonApp, _>::virtual_terminal(50, 16)?;

    println!("=== Button Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state (Save focused, Submit disabled):");
    println!("{}\n", vt.display());

    // Press Save
    vt.dispatch(Msg::Save(ButtonMessage::Press));
    vt.tick()?;
    println!("After pressing Save:");
    println!("{}\n", vt.display());

    // Move to Cancel and press
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::Cancel(ButtonMessage::Press));
    vt.tick()?;
    println!("After pressing Cancel:");
    println!("{}\n", vt.display());

    // Move to Submit (disabled) and try to press
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::Submit(ButtonMessage::Press));
    vt.tick()?;
    println!("After attempting to press disabled Submit:");
    println!("{}\n", vt.display());

    Ok(())
}
