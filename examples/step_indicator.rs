//! StepIndicator example -- pipeline/workflow visualization.
//!
//! Demonstrates the StepIndicator component for showing step-by-step
//! progress through a build pipeline with completion, failure, and
//! skip states.
//!
//! Run with: cargo run --example step_indicator --features navigation-components

use envision::prelude::*;

/// Application marker type.
struct StepIndicatorApp;

/// Application state.
#[derive(Clone)]
struct State {
    pipeline: StepIndicatorState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Step(StepIndicatorMessage),
    Quit,
}

impl App for StepIndicatorApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let steps = vec![
            step_indicator::Step::new("Checkout")
                .with_status(step_indicator::StepStatus::Completed)
                .with_description("Clone repository"),
            step_indicator::Step::new("Build")
                .with_status(step_indicator::StepStatus::Active)
                .with_description("Compile sources"),
            step_indicator::Step::new("Test").with_description("Run test suite"),
            step_indicator::Step::new("Lint").with_description("Check formatting"),
            step_indicator::Step::new("Deploy").with_description("Push to production"),
        ];

        let mut pipeline = StepIndicatorState::new(steps)
            .with_title("CI Pipeline")
            .with_orientation(step_indicator::StepOrientation::Horizontal);

        pipeline.set_focused(true);

        (State { pipeline }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Step(m) => {
                StepIndicator::update(&mut state.pipeline, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        StepIndicator::view(&state.pipeline, frame, chunks[0], &theme);

        let active = state
            .pipeline
            .active_step_index()
            .map(|i| {
                state
                    .pipeline
                    .step(i)
                    .map(|s| s.label().to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_else(|| "None".into());
        let status = format!(
            " Active: {} | All done: {} | Left/Right: navigate, q: quit",
            active,
            state.pipeline.is_all_completed()
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
        state.pipeline.handle_event(event).map(Msg::Step)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<StepIndicatorApp, _>::virtual_terminal(70, 5)?;

    println!("=== StepIndicator Example ===\n");

    // Initial render: pipeline with Build active
    vt.tick()?;
    println!("Initial state (Build active):");
    println!("{}\n", vt.display());

    // Complete the Build step
    vt.dispatch(Msg::Step(StepIndicatorMessage::CompleteActive));
    vt.tick()?;
    println!("After completing Build:");
    println!("{}\n", vt.display());

    // Activate and complete Test
    vt.dispatch(Msg::Step(StepIndicatorMessage::ActivateNext));
    vt.dispatch(Msg::Step(StepIndicatorMessage::CompleteActive));
    vt.tick()?;
    println!("After completing Test:");
    println!("{}\n", vt.display());

    // Skip Lint and activate Deploy
    vt.dispatch(Msg::Step(StepIndicatorMessage::Skip(3)));
    vt.dispatch(Msg::Step(StepIndicatorMessage::ActivateNext));
    vt.tick()?;
    println!("After skipping Lint and activating Deploy:");
    println!("{}\n", vt.display());

    // Complete Deploy
    vt.dispatch(Msg::Step(StepIndicatorMessage::CompleteActive));
    vt.tick()?;
    println!("All steps complete:");
    println!("{}\n", vt.display());

    Ok(())
}
