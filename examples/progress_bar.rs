//! ProgressBar example -- progress indicator with labels.
//!
//! Demonstrates the ProgressBar component with progress updates,
//! labels, and completion detection.
//!
//! Run with: cargo run --example progress_bar --features display-components

use envision::prelude::*;

/// Application marker type.
struct ProgressBarApp;

/// Application state with multiple progress bars.
#[derive(Clone)]
struct State {
    download: ProgressBarState,
    install: ProgressBarState,
    verify: ProgressBarState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    AdvanceDownload,
    AdvanceInstall,
    CompleteVerify,
    Quit,
}

impl App for ProgressBarApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let state = State {
            download: ProgressBarState::with_label("Downloading..."),
            install: ProgressBarState::with_label("Installing..."),
            verify: ProgressBarState::with_label("Verifying..."),
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::AdvanceDownload => {
                ProgressBar::update(&mut state.download, ProgressBarMessage::Increment(0.35));
            }
            Msg::AdvanceInstall => {
                ProgressBar::update(&mut state.install, ProgressBarMessage::SetProgress(0.6));
            }
            Msg::CompleteVerify => {
                ProgressBar::update(&mut state.verify, ProgressBarMessage::Complete);
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
        ])
        .split(area);

        ProgressBar::view(
            &state.download,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );
        ProgressBar::view(
            &state.install,
            &mut RenderContext::new(frame, chunks[1], &theme),
        );
        ProgressBar::view(
            &state.verify,
            &mut RenderContext::new(frame, chunks[2], &theme),
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<ProgressBarApp, _>::virtual_terminal(50, 10)?;

    println!("=== ProgressBar Example ===\n");

    vt.tick()?;
    println!("Initial state (all at 0%):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::AdvanceDownload);
    vt.tick()?;
    println!("After advancing download to 35%:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::AdvanceDownload);
    vt.dispatch(Msg::AdvanceInstall);
    vt.tick()?;
    println!("After download 70%, install 60%:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::CompleteVerify);
    vt.tick()?;
    println!("After completing verification:");
    println!("{}\n", vt.display());

    Ok(())
}
