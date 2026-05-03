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
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
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
        output.push_line("");
        output.push_line("\x1b[1m--- 24-bit RGB Color Demo ---\x1b[0m");
        output.push_line(
            "\x1b[38;2;255;100;0mOrange text\x1b[0m  \
             \x1b[38;2;0;200;100mTeal text\x1b[0m  \
             \x1b[38;2;180;80;255mPurple text\x1b[0m",
        );
        output.push_line(
            "\x1b[48;2;40;40;80m\x1b[38;2;200;200;255m Syntax-highlighted block \x1b[0m",
        );
        output.push_line(
            "\x1b[38;2;255;0;0mR\x1b[38;2;255;128;0mA\x1b[38;2;255;255;0mI\
             \x1b[38;2;0;255;0mN\x1b[38;2;0;128;255mB\x1b[38;2;128;0;255mO\
             \x1b[38;2;255;0;128mW\x1b[0m gradient",
        );

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

#[tokio::main]
async fn main() -> envision::error::Result<()> {
    Runtime::<TerminalOutputApp, _>::terminal_builder()?
        .build()?
        .run_terminal()
        .await?;
    Ok(())
}
