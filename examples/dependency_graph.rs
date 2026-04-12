//! DependencyGraph example -- microservice architecture visualization.
//!
//! Demonstrates the DependencyGraph component displaying service
//! relationships with status coloring and navigation.
//!
//! Run with: cargo run --example dependency_graph

use envision::prelude::*;

/// Application marker type.
struct DependencyGraphApp;

/// Application state wrapping a single DependencyGraph.
#[derive(Clone)]
struct State {
    graph: DependencyGraphState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Graph(DependencyGraphMessage),
    Quit,
}

fn build_graph() -> DependencyGraphState {
    DependencyGraphState::new()
        .with_node(
            GraphNode::new("gateway", "API Gateway")
                .with_status(NodeStatus::Healthy)
                .with_color(Color::Green)
                .with_metadata("version", "2.1.0")
                .with_metadata("port", "8080"),
        )
        .with_node(
            GraphNode::new("auth", "Auth Service")
                .with_status(NodeStatus::Healthy)
                .with_color(Color::Cyan)
                .with_metadata("version", "1.5.2"),
        )
        .with_node(
            GraphNode::new("users", "User Service")
                .with_status(NodeStatus::Degraded)
                .with_color(Color::Yellow)
                .with_metadata("version", "3.0.1"),
        )
        .with_node(
            GraphNode::new("orders", "Order Service")
                .with_status(NodeStatus::Healthy)
                .with_color(Color::Blue)
                .with_metadata("version", "1.8.0"),
        )
        .with_node(
            GraphNode::new("db", "PostgreSQL")
                .with_status(NodeStatus::Healthy)
                .with_color(Color::Magenta),
        )
        .with_node(
            GraphNode::new("cache", "Redis Cache")
                .with_status(NodeStatus::Down)
                .with_color(Color::Red),
        )
        .with_edge(GraphEdge::new("gateway", "auth").with_label("gRPC"))
        .with_edge(GraphEdge::new("gateway", "users").with_label("HTTP"))
        .with_edge(GraphEdge::new("gateway", "orders").with_label("HTTP"))
        .with_edge(GraphEdge::new("auth", "db").with_label("SQL"))
        .with_edge(GraphEdge::new("users", "db").with_label("SQL"))
        .with_edge(GraphEdge::new("users", "cache").with_label("TCP"))
        .with_edge(GraphEdge::new("orders", "db").with_label("SQL"))
        .with_title("Microservice Architecture")
}

impl App for DependencyGraphApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let graph = build_graph();

        (State { graph }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Graph(m) => {
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

        DependencyGraph::view(
            &state.graph,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let selected_info = state
            .graph
            .selected_node()
            .map(|node| {
                let meta = node
                    .metadata
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ");
                if meta.is_empty() {
                    format!("{} ({:?})", node.label, node.status)
                } else {
                    format!("{} ({:?}) [{}]", node.label, node.status, meta)
                }
            })
            .unwrap_or_else(|| "None".into());

        let status = format!(
            " Selected: {} | Nodes: {} | Edges: {}",
            selected_info,
            state.graph.nodes().len(),
            state.graph.edges().len(),
        );
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
        DependencyGraph::handle_event(&state.graph, event, &EventContext::new().focused(true))
            .map(Msg::Graph)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<DependencyGraphApp, _>::virtual_terminal(100, 24)?;

    println!("=== DependencyGraph Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial dependency graph:");
    println!("{}\n", vt.display());

    // Select first node
    vt.dispatch(Msg::Graph(DependencyGraphMessage::SelectNext));
    vt.tick()?;
    println!("After selecting first node:");
    println!("{}\n", vt.display());

    // Navigate to next node
    vt.dispatch(Msg::Graph(DependencyGraphMessage::SelectNext));
    vt.tick()?;
    println!("After selecting next node:");
    println!("{}\n", vt.display());

    // Follow connection
    vt.dispatch(Msg::Graph(DependencyGraphMessage::SelectConnected));
    vt.tick()?;
    println!("After following connection:");
    println!("{}\n", vt.display());

    // Update a node status
    vt.dispatch(Msg::Graph(DependencyGraphMessage::UpdateNodeStatus {
        id: "cache".to_string(),
        status: NodeStatus::Healthy,
    }));
    vt.tick()?;
    println!("After cache recovery:");
    println!("{}\n", vt.display());

    Ok(())
}
