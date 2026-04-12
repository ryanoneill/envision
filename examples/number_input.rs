//! Number input example -- numeric value entry with increment/decrement.
//!
//! Demonstrates the NumberInput component with integer and float inputs,
//! including min/max bounds, custom step sizes, and text edit mode.
//!
//! Run with: cargo run --example number_input --features input-components

use envision::prelude::*;

/// Application marker type.
struct NumberInputApp;

/// Application state with multiple number inputs.
#[derive(Clone)]
struct State {
    quantity: NumberInputState,
    price: NumberInputState,
    temperature: NumberInputState,
    focus_index: usize,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Quantity(NumberInputMessage),
    Price(NumberInputMessage),
    Temperature(NumberInputMessage),
    FocusNext,
    FocusPrev,
    Quit,
}

const INPUT_COUNT: usize = 3;

impl App for NumberInputApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let quantity = NumberInputState::integer(1)
            .with_min(0.0)
            .with_max(100.0)
            .with_label("Quantity");

        let price = NumberInputState::new(9.99)
            .with_min(0.0)
            .with_step(0.01)
            .with_precision(2)
            .with_label("Price");

        let temperature = NumberInputState::new(22.0)
            .with_range(-40.0, 50.0)
            .with_step(0.5)
            .with_precision(1)
            .with_label("Temp");

        let state = State {
            quantity,
            price,
            temperature,
            focus_index: 0,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Quantity(m) => {
                NumberInput::update(&mut state.quantity, m);
            }
            Msg::Price(m) => {
                NumberInput::update(&mut state.price, m);
            }
            Msg::Temperature(m) => {
                NumberInput::update(&mut state.temperature, m);
            }
            Msg::FocusNext => {
                state.focus_index = (state.focus_index + 1) % INPUT_COUNT;
            }
            Msg::FocusPrev => {
                state.focus_index = (state.focus_index + INPUT_COUNT - 1) % INPUT_COUNT;
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

        NumberInput::view(
            &state.quantity,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );
        NumberInput::view(
            &state.price,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
        NumberInput::view(
            &state.temperature,
            &mut RenderContext::new(frame, chunks[2], &theme),
        );

        let status = format!(
            " Qty: {} | Price: {} | Temp: {} | Tab: navigate, Enter: edit, q: quit",
            state.quantity.format_value(),
            state.price.format_value(),
            state.temperature.format_value(),
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[4],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        // Check if any input is in edit mode
        let any_editing = state.quantity.is_editing()
            || state.price.is_editing()
            || state.temperature.is_editing();

        if let Some(key) = event.as_key() {
            if !any_editing {
                match key.code {
                    Key::Char('q') | Key::Esc => return Some(Msg::Quit),
                    Key::Tab if key.modifiers.shift() => return Some(Msg::FocusPrev),

                    Key::Tab => return Some(Msg::FocusNext),
                    _ => {}
                }
            }
        }

        // Route event to focused input
        match state.focus_index {
            0 => NumberInput::handle_event(
                &state.quantity,
                event,
                &EventContext::new().focused(true),
            )
            .map(Msg::Quantity),
            1 => NumberInput::handle_event(&state.price, event, &EventContext::new().focused(true))
                .map(Msg::Price),
            _ => NumberInput::handle_event(
                &state.temperature,
                event,
                &EventContext::new().focused(true),
            )
            .map(Msg::Temperature),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<NumberInputApp, _>::virtual_terminal(60, 16)?;

    println!("=== Number Input Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial state:");
    println!("{}\n", vt.display());

    // Increment quantity
    vt.dispatch(Msg::Quantity(NumberInputMessage::Increment));
    vt.dispatch(Msg::Quantity(NumberInputMessage::Increment));
    vt.tick()?;
    println!("After incrementing quantity twice:");
    println!("{}\n", vt.display());

    // Switch to price and increment
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::Price(NumberInputMessage::Increment));
    vt.tick()?;
    println!("After incrementing price:");
    println!("{}\n", vt.display());

    // Switch to temperature and set value
    vt.dispatch(Msg::FocusNext);
    vt.dispatch(Msg::Temperature(NumberInputMessage::SetValue(37.5)));
    vt.tick()?;
    println!("After setting temperature to 37.5:");
    println!("{}\n", vt.display());

    Ok(())
}
