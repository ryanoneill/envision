//! SpanTree example — hierarchical trace span visualization.
//!
//! Demonstrates the SpanTree component displaying a trace with nested
//! service spans, timing bars, and keyboard navigation.
//!
//! Run with: cargo run --example span_tree

use envision::prelude::*;

/// Application marker type.
struct SpanTreeApp;

/// Application state wrapping a single SpanTree.
#[derive(Clone)]
struct State {
    tree: SpanTreeState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    SpanTree(SpanTreeMessage),
    Quit,
}

fn build_trace() -> Vec<SpanNode> {
    let root = SpanNode::new("req-1", "frontend/request", 0.0, 1000.0)
        .with_color(Color::Cyan)
        .with_status("200 OK")
        .with_child(
            SpanNode::new("api-1", "api/handler", 50.0, 850.0)
                .with_color(Color::Yellow)
                .with_child(
                    SpanNode::new("db-1", "db/query", 100.0, 350.0)
                        .with_color(Color::Green)
                        .with_status("OK"),
                )
                .with_child(
                    SpanNode::new("cache-1", "cache/lookup", 400.0, 600.0)
                        .with_color(Color::Blue)
                        .with_status("HIT"),
                )
                .with_child(
                    SpanNode::new("tmpl-1", "template/render", 650.0, 800.0)
                        .with_color(Color::Magenta),
                ),
        )
        .with_child(
            SpanNode::new("auth-1", "auth/validate", 10.0, 120.0)
                .with_color(Color::Red)
                .with_status("OK"),
        );

    vec![root]
}

impl App for SpanTreeApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let tree = SpanTreeState::new(build_trace())
            .with_title("HTTP Request Trace")
            .with_label_width(28);

        (State { tree }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::SpanTree(m) => {
                state.tree.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        SpanTree::view(
            &state.tree,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let selected = state
            .tree
            .selected_span()
            .map(|s| format!("{} ({:.0}ms)", s.label(), s.duration()))
            .unwrap_or_else(|| "None".into());
        let status = format!(" Selected: {}", selected);
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
        SpanTree::handle_event(&state.tree, event, &EventContext::new().focused(true))
            .map(Msg::SpanTree)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<SpanTreeApp, _>::virtual_terminal(70, 16)?;

    println!("=== SpanTree Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial trace (all expanded):");
    println!("{}\n", vt.display());

    // Navigate down to select api/handler
    vt.dispatch(Msg::SpanTree(SpanTreeMessage::SelectDown));
    vt.tick()?;
    println!("After Down (selected: api/handler):");
    println!("{}\n", vt.display());

    // Collapse api/handler
    vt.dispatch(Msg::SpanTree(SpanTreeMessage::Collapse));
    vt.tick()?;
    println!("After Collapse (api/handler collapsed):");
    println!("{}\n", vt.display());

    // Expand it again
    vt.dispatch(Msg::SpanTree(SpanTreeMessage::Expand));
    vt.tick()?;
    println!("After Expand (api/handler re-expanded):");
    println!("{}\n", vt.display());

    Ok(())
}
