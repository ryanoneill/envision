use super::*;
use crate::component::test_utils;

#[test]
fn test_snapshot_empty() {
    let state = DependencyGraphState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            DependencyGraph::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_node() {
    let state = DependencyGraphState::new()
        .with_node(GraphNode::new("api", "API Gateway").with_status(NodeStatus::Healthy))
        .with_title("Service Graph");
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            DependencyGraph::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_simple_graph() {
    let state = DependencyGraphState::new()
        .with_node(GraphNode::new("api", "API").with_status(NodeStatus::Healthy))
        .with_node(GraphNode::new("db", "Database").with_status(NodeStatus::Degraded))
        .with_node(GraphNode::new("cache", "Cache").with_status(NodeStatus::Healthy))
        .with_edge(GraphEdge::new("api", "db"))
        .with_edge(GraphEdge::new("api", "cache"))
        .with_title("Service Topology");
    let (mut terminal, theme) = test_utils::setup_render(80, 16);
    terminal
        .draw(|frame| {
            DependencyGraph::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_selection() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("api", "API").with_status(NodeStatus::Healthy))
        .with_node(GraphNode::new("db", "Database").with_status(NodeStatus::Degraded))
        .with_edge(GraphEdge::new("api", "db"))
        .with_title("Service Topology");
    state.select_next(); // select api
    let (mut terminal, theme) = test_utils::setup_render(80, 12);
    terminal
        .draw(|frame| {
            DependencyGraph::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_statuses() {
    let state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "Frontend").with_status(NodeStatus::Healthy))
        .with_node(GraphNode::new("b", "Backend").with_status(NodeStatus::Degraded))
        .with_node(GraphNode::new("c", "Database").with_status(NodeStatus::Down))
        .with_node(GraphNode::new("d", "Cache").with_status(NodeStatus::Unknown))
        .with_edge(GraphEdge::new("a", "b"))
        .with_edge(GraphEdge::new("b", "c"))
        .with_edge(GraphEdge::new("b", "d"))
        .with_title("All Statuses");
    let (mut terminal, theme) = test_utils::setup_render(80, 16);
    terminal
        .draw(|frame| {
            DependencyGraph::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let state = DependencyGraphState::new()
        .with_node(GraphNode::new("api", "API").with_status(NodeStatus::Healthy))
        .with_node(GraphNode::new("db", "Database").with_status(NodeStatus::Down))
        .with_edge(GraphEdge::new("api", "db"))
        .with_title("Disabled Graph");
    let (mut terminal, theme) = test_utils::setup_render(80, 12);
    terminal
        .draw(|frame| {
            DependencyGraph::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_top_to_bottom() {
    let state = DependencyGraphState::new()
        .with_node(GraphNode::new("api", "API").with_status(NodeStatus::Healthy))
        .with_node(GraphNode::new("db", "DB").with_status(NodeStatus::Healthy))
        .with_edge(GraphEdge::new("api", "db"))
        .with_orientation(GraphOrientation::TopToBottom)
        .with_title("Top to Bottom");
    let (mut terminal, theme) = test_utils::setup_render(60, 16);
    terminal
        .draw(|frame| {
            DependencyGraph::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
