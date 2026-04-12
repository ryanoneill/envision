//! FlameGraph example -- interactive flame graph visualization.
//!
//! Demonstrates the FlameGraph component displaying simulated profiling
//! data with zoom, navigation, and search.
//!
//! Run with: cargo run --example flame_graph

use envision::prelude::*;

/// Application marker type.
struct FlameGraphApp;

/// Application state wrapping a single FlameGraph.
#[derive(Clone)]
struct State {
    graph: FlameGraphState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    FlameGraph(FlameGraphMessage),
    Quit,
}

fn build_profile() -> FlameNode {
    FlameNode::new("main()", 1000)
        .with_color(Color::Red)
        .with_child(
            FlameNode::new("server::run()", 800)
                .with_color(Color::Yellow)
                .with_child(
                    FlameNode::new("handle_request()", 500)
                        .with_color(Color::Green)
                        .with_child(
                            FlameNode::new("db::query()", 200)
                                .with_color(Color::Cyan)
                                .with_child(
                                    FlameNode::new("sql::parse()", 80).with_color(Color::Blue),
                                )
                                .with_child(
                                    FlameNode::new("sql::execute()", 100).with_color(Color::Blue),
                                ),
                        )
                        .with_child(FlameNode::new("serialize()", 150).with_color(Color::Magenta)),
                )
                .with_child(
                    FlameNode::new("auth::validate()", 200)
                        .with_color(Color::Red)
                        .with_child(FlameNode::new("jwt::decode()", 120).with_color(Color::Yellow)),
                ),
        )
        .with_child(FlameNode::new("cleanup()", 100).with_color(Color::DarkGray))
}

impl App for FlameGraphApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let graph = FlameGraphState::with_root(build_profile()).with_title("CPU Profile");

        (State { graph }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::FlameGraph(m) => {
                state.graph.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        FlameGraph::view(
            &state.graph,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let selected = state
            .graph
            .selected_frame()
            .map(|f| {
                format!(
                    "{} ({} samples, self: {})",
                    f.label(),
                    f.total_value(),
                    f.self_value()
                )
            })
            .unwrap_or_else(|| "None".into());
        let zoom_info = if state.graph.zoom_stack().is_empty() {
            String::new()
        } else {
            format!(" | Zoomed: {}", state.graph.zoom_stack().join(" > "))
        };
        let status = format!(" Selected: {}{}", selected, zoom_info);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, KeyCode::Char('q')) {
                return Some(Msg::Quit);
            }
        }
        FlameGraph::handle_event(&state.graph, event, &EventContext::new().focused(true))
            .map(Msg::FlameGraph)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<FlameGraphApp, _>::virtual_terminal(70, 18)?;

    println!("=== FlameGraph Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial flame graph:");
    println!("{}\n", vt.display());

    // Navigate down to server::run()
    vt.dispatch(Msg::FlameGraph(FlameGraphMessage::SelectDown));
    vt.tick()?;
    println!("After Down (selected: server::run()):");
    println!("{}\n", vt.display());

    // Navigate down to handle_request()
    vt.dispatch(Msg::FlameGraph(FlameGraphMessage::SelectDown));
    vt.tick()?;
    println!("After Down (selected: handle_request()):");
    println!("{}\n", vt.display());

    // Zoom into handle_request()
    vt.dispatch(Msg::FlameGraph(FlameGraphMessage::ZoomIn));
    vt.tick()?;
    println!("After ZoomIn (zoomed into handle_request()):");
    println!("{}\n", vt.display());

    // Zoom back out
    vt.dispatch(Msg::FlameGraph(FlameGraphMessage::ZoomOut));
    vt.tick()?;
    println!("After ZoomOut (back to full view):");
    println!("{}\n", vt.display());

    // Search for "sql"
    vt.dispatch(Msg::FlameGraph(FlameGraphMessage::SetSearch(
        "sql".to_string(),
    )));
    vt.tick()?;
    println!("After search for 'sql':");
    println!("{}\n", vt.display());

    Ok(())
}
