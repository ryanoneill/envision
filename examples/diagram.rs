//! Diagram component example showing a service topology.
//!
//! Demonstrates the Diagram component with nodes, edges, clusters,
//! edge styles, and interactive navigation.
//!
//! Run with: cargo run --example diagram

use envision::component::diagram::{
    Diagram, DiagramCluster, DiagramEdge, DiagramMessage, DiagramNode, DiagramOutput, DiagramState,
    EdgeStyle, NodeStatus,
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
}

impl envision::app::App for App {
    type State = AppState;
    type Message = Msg;

    fn init() -> (AppState, Command<Msg>) {
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
                    .with_cluster("us-east"),
            )
            .with_node(
                DiagramNode::new("api-2", "API v2.1")
                    .with_status(NodeStatus::Degraded)
                    .with_cluster("eu-west"),
            )
            .with_node(
                DiagramNode::new("db", "PostgreSQL")
                    .with_status(NodeStatus::Healthy)
                    .with_cluster("us-east"),
            )
            .with_node(DiagramNode::new("cache", "Redis").with_status(NodeStatus::Healthy))
            .with_node(
                DiagramNode::new("queue", "RabbitMQ")
                    .with_status(NodeStatus::Down)
                    .with_cluster("eu-west"),
            )
            .with_edge(DiagramEdge::new("lb", "api-1").with_label("HTTP"))
            .with_edge(DiagramEdge::new("lb", "api-2").with_label("HTTP"))
            .with_edge(DiagramEdge::new("api-1", "db").with_label("SQL"))
            .with_edge(DiagramEdge::new("api-1", "cache").with_style(EdgeStyle::Dashed))
            .with_edge(DiagramEdge::new("api-2", "queue").with_style(EdgeStyle::Dotted))
            .with_edge(DiagramEdge::new("api-2", "cache"))
            .with_show_edge_labels(true)
            .with_title("Service Topology");

        (AppState { diagram }, Command::none())
    }

    fn update(state: &mut AppState, msg: Msg) -> Command<Msg> {
        let Msg::Diagram(diagram_msg) = msg;
        if let Some(DiagramOutput::NodeSelected(_id)) =
            Diagram::update(&mut state.diagram, diagram_msg)
        {
            // Could update a status bar or detail panel here
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
        let ctx = EventContext::new().focused(true);
        Diagram::handle_event(&state.diagram, event, &ctx).map(Msg::Diagram)
    }
}

fn main() {
    // Virtual terminal demo (no real terminal needed)
    let mut vt = Runtime::<App, _>::virtual_builder(100, 30).build().unwrap();
    vt.send(Event::char('j')); // Select first node
    vt.tick().unwrap();
    println!("{}", vt.display());
}
