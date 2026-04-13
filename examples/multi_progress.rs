//! MultiProgress example -- multiple concurrent progress indicators.
//!
//! Demonstrates the MultiProgress component for tracking multiple
//! tasks with individual progress bars, completion, and failure states.
//!
//! Run with: cargo run --example multi_progress --features display-components

use envision::prelude::*;

/// Application marker type.
struct MultiProgressApp;

/// Application state.
#[derive(Clone)]
struct State {
    progress: MultiProgressState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Progress(MultiProgressMessage),
    Quit,
}

impl App for MultiProgressApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut progress = MultiProgressState::new().with_title("Downloads");

        progress.add("file1", "linux-kernel.tar.gz");
        progress.add("file2", "rustup-installer.sh");
        progress.add("file3", "node-v20.tar.xz");
        progress.add("file4", "gcc-13.2.tar.bz2");

        (State { progress }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Progress(m) => {
                MultiProgress::update(&mut state.progress, m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        MultiProgress::view(
            &state.progress,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let completed = state.progress.completed_count();
        let total = state.progress.len();
        let status = format!(
            " Completed: {}/{} | Overall: {:.0}%",
            completed,
            total,
            state.progress.overall_progress() * 100.0
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
        MultiProgress::handle_event(&state.progress, event, &EventContext::new().focused(true))
            .map(Msg::Progress)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<MultiProgressApp, _>::virtual_builder(60, 10).build()?;

    println!("=== MultiProgress Example ===\n");

    // Initial render: all items pending
    vt.tick()?;
    println!("Initial state (all pending):");
    println!("{}\n", vt.display());

    // Update progress on first file
    vt.dispatch(Msg::Progress(MultiProgressMessage::SetProgress {
        id: "file1".to_string(),
        progress: 0.75,
    }));
    vt.dispatch(Msg::Progress(MultiProgressMessage::SetProgress {
        id: "file2".to_string(),
        progress: 0.3,
    }));
    vt.tick()?;
    println!("After updating progress:");
    println!("{}\n", vt.display());

    // Complete first file
    vt.dispatch(Msg::Progress(MultiProgressMessage::Complete(
        "file1".to_string(),
    )));
    vt.tick()?;
    println!("After completing first file:");
    println!("{}\n", vt.display());

    // Fail the fourth file
    vt.dispatch(Msg::Progress(MultiProgressMessage::Fail {
        id: "file4".to_string(),
        message: Some("Timeout".to_string()),
    }));
    vt.tick()?;
    println!("After fourth file failed:");
    println!("{}\n", vt.display());

    Ok(())
}
