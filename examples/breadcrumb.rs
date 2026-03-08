//! Breadcrumb example -- hierarchical navigation path.
//!
//! Demonstrates the Breadcrumb component with keyboard-driven segment
//! navigation and selection for path-based navigation.
//!
//! Run with: cargo run --example breadcrumb --features navigation-components

use envision::prelude::*;

/// Application marker type.
struct BreadcrumbApp;

/// Application state.
#[derive(Clone)]
struct State {
    breadcrumb: BreadcrumbState,
    selections: Vec<String>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Breadcrumb(BreadcrumbMessage),
    Quit,
}

impl App for BreadcrumbApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let segments = vec![
            BreadcrumbSegment::new("Home").with_data("/"),
            BreadcrumbSegment::new("Documents").with_data("/documents"),
            BreadcrumbSegment::new("Projects").with_data("/documents/projects"),
            BreadcrumbSegment::new("Envision").with_data("/documents/projects/envision"),
        ];

        let mut breadcrumb = BreadcrumbState::new(segments);
        breadcrumb.set_focused(true);

        let state = State {
            breadcrumb,
            selections: Vec::new(),
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Breadcrumb(m) => {
                if let Some(output) = Breadcrumb::update(&mut state.breadcrumb, m) {
                    if let BreadcrumbOutput::Selected(idx) = output {
                        if let Some(segment) = state.breadcrumb.segments().get(idx) {
                            state.selections.push(format!(
                                "Navigated to: {} ({})",
                                segment.label(),
                                segment.data().unwrap_or("no path")
                            ));
                        }
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
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        Breadcrumb::view(&state.breadcrumb, frame, chunks[0], &theme);

        // Show selection history
        let log_lines: Vec<Line> = state
            .selections
            .iter()
            .map(|s| Line::from(format!("  {}", s)))
            .collect();
        let log = ratatui::widgets::Paragraph::new(log_lines).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Navigation History"),
        );
        frame.render_widget(log, chunks[1]);

        let focused_idx = state.breadcrumb.focused_index();
        let status = format!(
            " Focused: {} | Left/Right: navigate, Enter: select, q: quit",
            focused_idx
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                return Some(Msg::Quit);
            }
        }
        state.breadcrumb.handle_event(event).map(Msg::Breadcrumb)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<BreadcrumbApp, _>::virtual_terminal(65, 14)?;

    println!("=== Breadcrumb Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial breadcrumb path:");
    println!("{}\n", vt.display());

    // Navigate right to "Documents"
    vt.dispatch(Msg::Breadcrumb(BreadcrumbMessage::Right));
    vt.tick()?;
    println!("After Right (focused on Documents):");
    println!("{}\n", vt.display());

    // Select "Documents"
    vt.dispatch(Msg::Breadcrumb(BreadcrumbMessage::Select));
    vt.tick()?;
    println!("After Select (navigated to Documents):");
    println!("{}\n", vt.display());

    // Navigate to "Projects" and select
    vt.dispatch(Msg::Breadcrumb(BreadcrumbMessage::Right));
    vt.dispatch(Msg::Breadcrumb(BreadcrumbMessage::Select));
    vt.tick()?;
    println!("After selecting Projects:");
    println!("{}\n", vt.display());

    Ok(())
}
