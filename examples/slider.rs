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

const SLIDER_COUNT: usize = 4;

impl App for SliderApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let volume = SliderState::new(0.0, 100.0)
            .with_value(75.0)
            .with_label("Volume");

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
                state.focus_index = (state.focus_index + 1) % SLIDER_COUNT;
            }
            Msg::FocusPrev => {
                state.focus_index = (state.focus_index + SLIDER_COUNT - 1) % SLIDER_COUNT;
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

        Slider::view(
            &state.volume,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );
        Slider::view(
            &state.brightness,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
        Slider::view(
            &state.temperature,
            &mut RenderContext::new(frame, chunks[2], &theme),
        );
        Slider::view(
            &state.vertical,
            &mut RenderContext::new(frame, chunks[3], &theme),
        );

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
                Key::Char('q') | Key::Esc => return Some(Msg::Quit),
                Key::Tab if key.modifiers.shift() => return Some(Msg::FocusPrev),

                Key::Tab => return Some(Msg::FocusNext),
                _ => {}
            }
        }
        // Route event to focused slider
        match state.focus_index {
            0 => Slider::handle_event(&state.volume, event, &EventContext::new().focused(true))
                .map(Msg::Volume),
            1 => Slider::handle_event(&state.brightness, event, &EventContext::new().focused(true))
                .map(Msg::Brightness),
            2 => Slider::handle_event(
                &state.temperature,
                event,
                &EventContext::new().focused(true),
            )
            .map(Msg::Temperature),
            _ => Slider::handle_event(&state.vertical, event, &EventContext::new().focused(true))
                .map(Msg::Vertical),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<SliderApp, _>::virtual_builder(60, 24).build()?;

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
