//! Form example -- multi-field form with text, checkbox, and select inputs.
//!
//! Demonstrates the Form compound component with text input, checkbox,
//! and select fields, along with Tab navigation and form submission.
//!
//! Run with: cargo run --example form --features compound-components

use envision::prelude::*;

/// Application marker type.
struct FormApp;

/// Application state.
#[derive(Clone)]
struct State {
    form: FormState,
    submitted: bool,
    submission_summary: Option<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Form(FormMessage),
    Quit,
}

impl App for FormApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let fields = vec![
            FormField::text_with_placeholder("name", "Full Name", "Enter your name..."),
            FormField::text_with_placeholder("email", "Email", "user@example.com"),
            FormField::checkbox("newsletter", "Subscribe to newsletter"),
            FormField::select(
                "role",
                "Role",
                vec!["Developer", "Designer", "Manager", "Other"],
            ),
        ];

        let mut form = FormState::new(fields);
        form.set_focused(true);

        let state = State {
            form,
            submitted: false,
            submission_summary: None,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Form(m) => {
                if let Some(output) = Form::update(&mut state.form, m) {
                    if let FormOutput::Submitted(values) = output {
                        state.submitted = true;
                        let summary_parts: Vec<String> = values
                            .iter()
                            .map(|(id, val)| format!("  {}: {:?}", id, val))
                            .collect();
                        state.submission_summary = Some(summary_parts.join("\n"));
                    }
                }
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        if state.submitted {
            // Show submission summary
            let text = state
                .submission_summary
                .as_deref()
                .unwrap_or("(empty submission)");
            let content = format!("Form submitted successfully!\n\n{}", text);
            let widget = ratatui::widgets::Paragraph::new(content).block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Submission Result"),
            );
            frame.render_widget(widget, chunks[0]);
        } else {
            Form::view(&state.form, frame, chunks[0], &theme);
        }

        let status =
            " Tab/Shift+Tab: navigate fields, Space: toggle checkbox, Ctrl+S: submit, q: quit";
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.code == KeyCode::Esc {
                return Some(Msg::Quit);
            }
        }
        state.form.handle_event(event).map(Msg::Form)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<FormApp, _>::virtual_terminal(60, 20)?;

    println!("=== Form Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial form (empty fields):");
    println!("{}\n", vt.display());

    // Type a name
    for ch in "Jane Doe".chars() {
        vt.dispatch(Msg::Form(FormMessage::Input(ch)));
    }
    vt.tick()?;
    println!("After typing name:");
    println!("{}\n", vt.display());

    // Tab to email and type
    vt.dispatch(Msg::Form(FormMessage::FocusNext));
    for ch in "jane@example.com".chars() {
        vt.dispatch(Msg::Form(FormMessage::Input(ch)));
    }
    vt.tick()?;
    println!("After typing email:");
    println!("{}\n", vt.display());

    // Tab to newsletter checkbox and toggle
    vt.dispatch(Msg::Form(FormMessage::FocusNext));
    vt.dispatch(Msg::Form(FormMessage::Toggle));
    vt.tick()?;
    println!("After toggling newsletter checkbox:");
    println!("{}\n", vt.display());

    // Tab to role select and pick an option
    vt.dispatch(Msg::Form(FormMessage::FocusNext));
    vt.dispatch(Msg::Form(FormMessage::SelectDown));
    vt.tick()?;
    println!("After selecting a role:");
    println!("{}\n", vt.display());

    // Submit the form
    vt.dispatch(Msg::Form(FormMessage::Submit));
    vt.tick()?;
    println!("After form submission:");
    println!("{}\n", vt.display());

    Ok(())
}
