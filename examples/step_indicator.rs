//! StepIndicator example -- interactive pipeline/workflow visualization.
//!
//! Demonstrates the StepIndicator component for showing step-by-step progress
//! through a CI pipeline. Supports completing, failing, skipping, and resetting
//! steps with keyboard shortcuts, plus navigation between steps.
//!
//! Controls:
//!   Left/Right  Navigate between steps (keyboard focus)
//!   c           Complete the active step
//!   n           Activate the next pending step
//!   f           Fail the active step
//!   s           Skip the active step
//!   r           Reset all steps to pending
//!   q/Esc       Quit
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
    ResetPipeline,
    Quit,
}

fn build_steps() -> Vec<step_indicator::Step> {
    vec![
        step_indicator::Step::new("Checkout")
            .with_status(step_indicator::StepStatus::Completed)
            .with_description("Clone repository"),
        step_indicator::Step::new("Build")
            .with_status(step_indicator::StepStatus::Active)
            .with_description("Compile sources"),
        step_indicator::Step::new("Test").with_description("Run test suite"),
        step_indicator::Step::new("Lint").with_description("Check formatting"),
        step_indicator::Step::new("Deploy").with_description("Push to production"),
    ]
}

fn build_pipeline() -> StepIndicatorState {
    StepIndicatorState::new(build_steps())
        .with_title("CI Pipeline")
        .with_orientation(step_indicator::StepOrientation::Horizontal)
}

impl App for StepIndicatorApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let pipeline = build_pipeline();
        (State { pipeline }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Step(m) => {
                StepIndicator::update(&mut state.pipeline, m);
            }
            Msg::ResetPipeline => {
                state.pipeline = build_pipeline();
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

        StepIndicator::view(
            &state.pipeline,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        // Info panel
        let active = state
            .pipeline
            .active_step_index()
            .and_then(|i| state.pipeline.step(i).map(|s| s.label().to_string()))
            .unwrap_or_else(|| "None".into());
        let completed = state
            .pipeline
            .steps()
            .iter()
            .filter(|s| *s.status() == step_indicator::StepStatus::Completed)
            .count();
        let total = state.pipeline.steps().len();
        let info = format!(
            "  Active: {} | Progress: {}/{} | All done: {}",
            active,
            completed,
            total,
            state.pipeline.is_all_completed()
        );
        let info_widget = ratatui::widgets::Paragraph::new(info).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Pipeline Status"),
        );
        frame.render_widget(info_widget, chunks[1]);

        let status = " c: complete | n: next | f: fail | s: skip | r: reset | q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Some(Msg::Quit),
                KeyCode::Char('c') => return Some(Msg::Step(StepIndicatorMessage::CompleteActive)),
                KeyCode::Char('n') => return Some(Msg::Step(StepIndicatorMessage::ActivateNext)),
                KeyCode::Char('f') => return Some(Msg::Step(StepIndicatorMessage::FailActive)),
                KeyCode::Char('s') => {
                    if let Some(idx) = state.pipeline.active_step_index() {
                        return Some(Msg::Step(StepIndicatorMessage::Skip(idx)));
                    }
                }
                KeyCode::Char('r') => return Some(Msg::ResetPipeline),
                _ => {}
            }
        }
        StepIndicator::handle_event(&state.pipeline, event, &EventContext::new().focused(true))
            .map(Msg::Step)
    }
}

#[tokio::main]
async fn main() -> envision::Result<()> {
    let _final_state = TerminalRuntime::<StepIndicatorApp>::new_terminal()?
        .run_terminal()
        .await?;
    Ok(())
}
