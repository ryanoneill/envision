//! Slider example -- numeric range selection with keyboard control.
//!
//! Demonstrates the Slider component with multiple sliders using
//! different ranges, orientations, and step sizes.
//!
//! Run with: cargo run --example slider --features input-components

use envision::prelude::*;

/// Application marker type.
struct SliderApp;

/// Application state with multiple sliders.
#[derive(Clone)]
struct State {
    volume: SliderState,
    brightness: SliderState,
    temperature: SliderState,
    vertical: SliderState,
    focus_index: usize,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Volume(SliderMessage),
    Brightness(SliderMessage),
    Temperature(SliderMessage),
    Vertical(SliderMessage),
    FocusNext,
    FocusPrev,
    Quit,
}

impl State {
    fn focused_slider_mut(&mut self) -> &mut SliderState {
        match self.focus_index {
            0 => &mut self.volume,
            1 => &mut self.brightness,
            2 => &mut self.temperature,
            _ => &mut self.vertical,
        }
    }

    fn set_all_unfocused(&mut self) {
        self.volume.set_focused(false);
        self.brightness.set_focused(false);
        self.temperature.set_focused(false);
        self.vertical.set_focused(false);
    }
}

const SLIDER_COUNT: usize = 4;

impl App for SliderApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut volume = SliderState::new(0.0, 100.0)
            .with_value(75.0)
            .with_label("Volume");
        volume.set_focused(true);

        let brightness = SliderState::new(0.0, 100.0)
            .with_value(50.0)
            .with_step(5.0)
            .with_label("Brightness");

        let temperature = SliderState::new(-20.0, 40.0)
            .with_value(22.0)
            .with_step(0.5)
            .with_label("Temperature");

        let vertical = SliderState::new(0.0, 10.0)
            .with_value(5.0)
            .with_orientation(SliderOrientation::Vertical)
            .with_label("Level");

        let state = State {
            volume,
            brightness,
            temperature,
            vertical,
            focus_index: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Volume(m) => {
                Slider::update(&mut state.volume, m);
            }
            Msg::Brightness(m) => {
                Slider::update(&mut state.brightness, m);
            }
            Msg::Temperature(m) => {
                Slider::update(&mut state.temperature, m);
            }
            Msg::Vertical(m) => {
                Slider::update(&mut state.vertical, m);
            }
            Msg::FocusNext => {
                state.set_all_unfocused();
                state.focus_index = (state.focus_index + 1) % SLIDER_COUNT;
                state.focused_slider_mut().set_focused(true);
            }
            Msg::FocusPrev => {
                state.set_all_unfocused();
                state.focus_index = (state.focus_index + SLIDER_COUNT - 1) % SLIDER_COUNT;
                state.focused_slider_mut().set_focused(true);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(12),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        Slider::view(&state.volume, frame, chunks[0], &theme);
        Slider::view(&state.brightness, frame, chunks[1], &theme);
        Slider::view(&state.temperature, frame, chunks[2], &theme);
        Slider::view(&state.vertical, frame, chunks[3], &theme);

        let status = format!(
            " Vol: {} | Bright: {} | Temp: {} | Level: {} | Tab: navigate, q: quit",
            state.volume.value(),
            state.brightness.value(),
            state.temperature.value(),
            state.vertical.value(),
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[5],
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
        // Route event to focused slider
        match state.focus_index {
            0 => state.volume.handle_event(event).map(Msg::Volume),
            1 => state.brightness.handle_event(event).map(Msg::Brightness),
            2 => state.temperature.handle_event(event).map(Msg::Temperature),
            _ => state.vertical.handle_event(event).map(Msg::Vertical),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<SliderApp, _>::virtual_terminal(60, 24)?;

    println!("=== Slider Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state:");
    println!("{}\n", vt.display());

    // Increase volume
    vt.dispatch(Msg::Volume(SliderMessage::Increment));
    vt.dispatch(Msg::Volume(SliderMessage::Increment));
    vt.tick()?;
    println!("After incrementing volume twice:");
    println!("{}\n", vt.display());

    // Move to brightness and page increment
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::Brightness(SliderMessage::IncrementPage));
    vt.tick()?;
    println!("After page increment on brightness:");
    println!("{}\n", vt.display());

    // Move to temperature and set to max
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::Temperature(SliderMessage::SetMax));
    vt.tick()?;
    println!("After setting temperature to max:");
    println!("{}\n", vt.display());

    Ok(())
}
