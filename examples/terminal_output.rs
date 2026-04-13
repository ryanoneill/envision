//! TerminalOutput example — terminal output display with ANSI colors.
//!
//! Demonstrates the TerminalOutput component showing build-like output
//! with ANSI color codes, auto-scroll, line numbers, and status bar.
//!
//! Run with: cargo run --example terminal_output

use envision::prelude::*;
use ratatui::widgets::Paragraph;

/// Application marker type.
struct TerminalOutputApp;

/// Application state wrapping a single TerminalOutput.
#[derive(Clone)]
struct State {
    output: TerminalOutputState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Output(TerminalOutputMessage),
    Quit,
}

impl App for TerminalOutputApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let mut output = TerminalOutputState::new()
            .with_title("Build Output")
            .with_auto_scroll(true)
            .with_running(true);

        // Simulate some build output with ANSI colors
        output.push_line("\x1b[1;34m==>\x1b[0m Building envision v0.7.0");
        output.push_line("\x1b[32m   Compiling\x1b[0m envision v0.7.0 (path/to/envision)");
        output.push_line("\x1b[32m   Compiling\x1b[0m ratatui v0.29.0");
        output.push_line("\x1b[32m   Compiling\x1b[0m crossterm v0.28.0");
        output.push_line("\x1b[33mwarning\x1b[0m: unused import `std::io::Write`");
        output.push_line("  --> src/main.rs:3:5");
        output.push_line("   |");
        output.push_line("3  | use std::io::Write;");
        output.push_line("   |     ^^^^^^^^^^^^^^");
        output.push_line("   |");
        output.push_line("\x1b[31merror[E0308]\x1b[0m: mismatched types");
        output.push_line("  --> src/lib.rs:42:12");
        output.push_line("   |");
        output.push_line("42 |     return \"hello\";");
        output.push_line("   |            \x1b[31m^^^^^^^\x1b[0m expected `i32`, found `&str`");
        output.push_line("   |");
        output.push_line("\x1b[1;31merror\x1b[0m: could not compile `envision`");
        output.push_line("");
        output.push_line("\x1b[32m    Finished\x1b[0m `dev` profile in 4.2s");
        output.push_line("\x1b[1;34m==>\x1b[0m 1 warning, 1 error");

        output.set_running(false);
        output.set_exit_code(Some(1));

        (State { output }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Output(m) => {
                state.output.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        let theme = Theme::default();
        TerminalOutput::view(
            &state.output,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let status = Paragraph::new(
            " Lines: {} | a: auto-scroll | n: line nums | Up/Down/PgUp/PgDn | Esc quit"
                .replace("{}", &state.output.line_count().to_string()),
        )
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status, chunks[1]);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }

        TerminalOutput::handle_event(&state.output, event, &EventContext::new().focused(true))
            .map(Msg::Output)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<TerminalOutputApp, _>::virtual_builder(70, 25).build()?;

    println!("=== TerminalOutput Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Build output with ANSI colors:");
    println!("{}\n", vt.display());

    // Toggle line numbers
    vt.dispatch(Msg::Output(TerminalOutputMessage::ToggleLineNumbers));
    vt.tick()?;
    println!("With line numbers:");
    println!("{}\n", vt.display());

    // Scroll to top
    vt.dispatch(Msg::Output(TerminalOutputMessage::Home));
    vt.tick()?;
    println!("Scrolled to top:");
    println!("{}\n", vt.display());

    println!("Final line count: {}", vt.state().output.line_count());

    Ok(())
}
