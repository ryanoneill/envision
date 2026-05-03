//! Diagram component example — interactive service topology.
//!
//! Demonstrates the Diagram component as an interactive terminal
//! application with keyboard navigation, edge following, and search.
//!
//! Controls:
//!   h/j/k/l or arrows — spatial navigation (nearest node)
//!   Tab/Shift+Tab     — cycle through nodes
//!   Enter             — follow outgoing edge
//!   Backspace         — go back
//!   /                 — search nodes
//!   +/-               — zoom in/out
//!   H/J/K/L           — pan viewport
//!   0                 — fit to view
//!   m                 — toggle minimap
//!   q                 — quit
//!
//! Run with: cargo run --example diagram

use envision::component::diagram::{
    Diagram, DiagramCluster, DiagramEdge, DiagramMessage, DiagramNode, DiagramState, EdgeStyle,
    NodeStatus,
};
use envision::prelude::*;

struct App;

#[derive(Clone)]
struct AppState {
    diagram: DiagramState,
}

#[derive(Clone, Debug)]
enum Msg {
    Diagram(DiagramMessage),
    Quit,
}

impl envision::app::App for App {
    type State = AppState;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (AppState, Command<Msg>) {
        let diagram = DiagramState::new()
            .with_cluster(DiagramCluster::new("us-east", "US East"))
            .with_cluster(DiagramCluster::new("eu-west", "EU West"))
            .with_node(
                DiagramNode::new("lb", "Load Balancer")
                    .with_status(NodeStatus::Healthy)
                    .with_metadata("type", "ALB"),
            )
            .with_node(
                DiagramNode::new("api-1", "API v2.1")
                    .with_status(NodeStatus::Healthy)
                    .with_cluster("us-east")
                    .with_metadata("replicas", "3"),
            )
            .with_node(
                DiagramNode::new("api-2", "API v2.0")
                    .with_status(NodeStatus::Degraded)
                    .with_cluster("eu-west")
                    .with_metadata("replicas", "2")
                    .with_metadata("cpu", "89%"),
            )
            .with_node(
                DiagramNode::new("db", "PostgreSQL")
                    .with_status(NodeStatus::Healthy)
                    .with_cluster("us-east")
                    .with_metadata("version", "15.4"),
            )
            .with_node(
                DiagramNode::new("cache", "Redis")
                    .with_status(NodeStatus::Healthy)
                    .with_metadata("memory", "2.1GB"),
            )
            .with_node(
                DiagramNode::new("queue", "RabbitMQ")
                    .with_status(NodeStatus::Down)
                    .with_cluster("eu-west")
                    .with_metadata("pending", "45,231"),
            )
            .with_edge(DiagramEdge::new("lb", "api-1").with_label("HTTP"))
            .with_edge(DiagramEdge::new("lb", "api-2").with_label("HTTP"))
            .with_edge(DiagramEdge::new("api-1", "db").with_label("SQL"))
            .with_edge(DiagramEdge::new("api-1", "cache").with_style(EdgeStyle::Dashed))
            .with_edge(DiagramEdge::new("api-2", "queue").with_style(EdgeStyle::Dotted))
            .with_edge(DiagramEdge::new("api-2", "cache"))
            .with_show_edge_labels(true)
            .with_title("Service Topology — q to quit, / to search, arrows to navigate");

        (AppState { diagram }, Command::none())
    }

    fn update(state: &mut AppState, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Diagram(diagram_msg) => {
                Diagram::update(&mut state.diagram, diagram_msg);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &AppState, frame: &mut Frame) {
        let area = frame.area();
        let theme = Theme::default();
        let mut ctx = RenderContext::new(frame, area, &theme);
        ctx.focused = true;
        Diagram::view(&state.diagram, &mut ctx);
    }

    fn handle_event_with_state(state: &AppState, event: &Event) -> Option<Msg> {
        if let Event::Key(key) = event {
            if key.code == Key::Char('q') && !state.diagram.is_searching() {
                return Some(Msg::Quit);
            }
        }
        let ctx = EventContext::new().focused(true);
        Diagram::handle_event(&state.diagram, event, &ctx).map(Msg::Diagram)
    }
}

#[tokio::main]
async fn main() -> envision::error::Result<()> {
    Runtime::<App, _>::terminal_builder()?
        .build()?
        .run_terminal()
        .await?;
    Ok(())
}
