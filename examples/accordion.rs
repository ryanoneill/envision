//! Accordion example -- collapsible panels with keyboard navigation.
//!
//! Demonstrates the Accordion component with multiple panels that can
//! be expanded or collapsed, and keyboard-driven focus movement.
//!
//! Run with: cargo run --example accordion --features navigation-components

use envision::prelude::*;

/// Application marker type.
struct AccordionApp;

/// Application state wrapping a single Accordion.
#[derive(Clone)]
struct State {
    accordion: AccordionState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Accordion(AccordionMessage),
    Quit,
}

impl App for AccordionApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let panels = vec![
            AccordionPanel::new(
                "Getting Started",
                "Welcome to the application. Use the arrow keys to navigate\n\
                 between panels and press Enter to expand or collapse them.",
            )
            .expanded(),
            AccordionPanel::new(
                "Configuration",
                "You can customize your experience by editing the config\n\
                 file located at ~/.config/myapp/settings.toml.",
            ),
            AccordionPanel::new(
                "Keyboard Shortcuts",
                "Up/Down: Navigate panels\n\
                 Enter/Space: Toggle panel\n\
                 q/Esc: Quit application",
            ),
            AccordionPanel::new(
                "FAQ",
                "Q: How do I reset settings?\n\
                 A: Delete the config file and restart.\n\n\
                 Q: Where are logs stored?\n\
                 A: Check /var/log/myapp/ for log files.",
            ),
        ];

        let accordion = AccordionState::new(panels);

        (State { accordion }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Accordion(m) => {
                Accordion::update(&mut state.accordion, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        Accordion::view(
            &state.accordion,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let panel_idx = state
            .accordion
            .selected_index()
            .map(|i| i.to_string())
            .unwrap_or_else(|| "None".into());
        let expanded_count = state
            .accordion
            .panels()
            .iter()
            .filter(|p| p.is_expanded())
            .count();
        let status = format!(
            " Panel: {} | Expanded: {} | Up/Down: navigate, Enter: toggle, q: quit",
            panel_idx, expanded_count
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
        Accordion::handle_event(&state.accordion, event, &EventContext::new().focused(true))
            .map(Msg::Accordion)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<AccordionApp, _>::virtual_builder(60, 20).build()?;

    println!("=== Accordion Example ===\n");

    // Initial render: first panel expanded
    vt.tick()?;
    println!("Initial state (first panel expanded):");
    println!("{}\n", vt.display());

    // Navigate down to "Configuration"
    vt.dispatch(Msg::Accordion(AccordionMessage::Down));
    vt.tick()?;
    println!("After Down (focused on Configuration):");
    println!("{}\n", vt.display());

    // Expand "Configuration"
    vt.dispatch(Msg::Accordion(AccordionMessage::Toggle));
    vt.tick()?;
    println!("After Toggle (Configuration expanded):");
    println!("{}\n", vt.display());

    // Navigate down and expand "Keyboard Shortcuts"
    vt.dispatch(Msg::Accordion(AccordionMessage::Down));
    vt.dispatch(Msg::Accordion(AccordionMessage::Toggle));
    vt.tick()?;
    println!("After expanding Keyboard Shortcuts:");
    println!("{}\n", vt.display());

    Ok(())
}
