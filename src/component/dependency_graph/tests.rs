use super::*;
use crate::component::Component;
use crate::input::{Event, KeyCode};
use ratatui::style::Color;

// =============================================================================
// GraphNode Construction
// =============================================================================

#[test]
fn test_graph_node_new() {
    let node = GraphNode::new("api", "API Gateway");
    assert_eq!(node.id, "api");
    assert_eq!(node.label, "API Gateway");
    assert_eq!(node.status, NodeStatus::Healthy);
    assert_eq!(node.color, None);
    assert!(node.metadata.is_empty());
}

#[test]
fn test_graph_node_with_status() {
    let node = GraphNode::new("db", "Database").with_status(NodeStatus::Down);
    assert_eq!(node.status, NodeStatus::Down);
}

#[test]
fn test_graph_node_with_color() {
    let node = GraphNode::new("svc", "Service").with_color(Color::Cyan);
    assert_eq!(node.color, Some(Color::Cyan));
}

#[test]
fn test_graph_node_with_metadata() {
    let node = GraphNode::new("svc", "Service")
        .with_metadata("version", "1.0")
        .with_metadata("port", "8080");
    assert_eq!(node.metadata.len(), 2);
    assert_eq!(node.metadata[0], ("version".to_string(), "1.0".to_string()));
    assert_eq!(node.metadata[1], ("port".to_string(), "8080".to_string()));
}

#[test]
fn test_node_status_default() {
    let status = NodeStatus::default();
    assert_eq!(status, NodeStatus::Healthy);
}

#[test]
fn test_node_status_variants() {
    assert_eq!(NodeStatus::Healthy, NodeStatus::Healthy);
    assert_ne!(NodeStatus::Healthy, NodeStatus::Degraded);
    assert_ne!(NodeStatus::Down, NodeStatus::Unknown);
}

// =============================================================================
// GraphEdge Construction
// =============================================================================

#[test]
fn test_graph_edge_new() {
    let edge = GraphEdge::new("a", "b");
    assert_eq!(edge.from, "a");
    assert_eq!(edge.to, "b");
    assert_eq!(edge.label, None);
    assert_eq!(edge.color, None);
}

#[test]
fn test_graph_edge_with_label() {
    let edge = GraphEdge::new("a", "b").with_label("HTTP");
    assert_eq!(edge.label, Some("HTTP".to_string()));
}

#[test]
fn test_graph_edge_with_color() {
    let edge = GraphEdge::new("a", "b").with_color(Color::Yellow);
    assert_eq!(edge.color, Some(Color::Yellow));
}

#[test]
fn test_graph_edge_full_builder() {
    let edge = GraphEdge::new("api", "db")
        .with_label("gRPC")
        .with_color(Color::Blue);
    assert_eq!(edge.from, "api");
    assert_eq!(edge.to, "db");
    assert_eq!(edge.label, Some("gRPC".to_string()));
    assert_eq!(edge.color, Some(Color::Blue));
}

// =============================================================================
// GraphOrientation
// =============================================================================

#[test]
fn test_orientation_default() {
    let orientation = GraphOrientation::default();
    assert_eq!(orientation, GraphOrientation::LeftToRight);
}

// =============================================================================
// DependencyGraphState Construction
// =============================================================================

#[test]
fn test_state_new() {
    let state = DependencyGraphState::new();
    assert!(state.nodes().is_empty());
    assert!(state.edges().is_empty());
    assert_eq!(state.selected(), None);
    assert_eq!(state.title(), None);
    assert_eq!(state.orientation(), &GraphOrientation::LeftToRight);
    assert!(!state.show_edge_labels());
}

#[test]
fn test_state_default() {
    let state = DependencyGraphState::default();
    assert!(state.nodes().is_empty());
}

#[test]
fn test_state_with_node() {
    let state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    assert_eq!(state.nodes().len(), 2);
    assert_eq!(state.nodes()[0].id, "a");
    assert_eq!(state.nodes()[1].id, "b");
}

#[test]
fn test_state_with_edge() {
    let state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"))
        .with_edge(GraphEdge::new("a", "b"));
    assert_eq!(state.edges().len(), 1);
    assert_eq!(state.edges()[0].from, "a");
    assert_eq!(state.edges()[0].to, "b");
}

#[test]
fn test_state_with_title() {
    let state = DependencyGraphState::new().with_title("My Graph");
    assert_eq!(state.title(), Some("My Graph"));
}

#[test]
fn test_state_with_orientation() {
    let state = DependencyGraphState::new().with_orientation(GraphOrientation::TopToBottom);
    assert_eq!(state.orientation(), &GraphOrientation::TopToBottom);
}

#[test]
fn test_state_with_show_edge_labels() {
    let state = DependencyGraphState::new().with_show_edge_labels(true);
    assert!(state.show_edge_labels());
}

// =============================================================================
// Mutation: Add/Clear
// =============================================================================

#[test]
fn test_add_node() {
    let mut state = DependencyGraphState::new();
    state.add_node(GraphNode::new("a", "A"));
    assert_eq!(state.nodes().len(), 1);
    state.add_node(GraphNode::new("b", "B"));
    assert_eq!(state.nodes().len(), 2);
}

#[test]
fn test_add_edge() {
    let mut state = DependencyGraphState::new();
    state.add_node(GraphNode::new("a", "A"));
    state.add_node(GraphNode::new("b", "B"));
    state.add_edge(GraphEdge::new("a", "b"));
    assert_eq!(state.edges().len(), 1);
}

#[test]
fn test_clear() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"))
        .with_edge(GraphEdge::new("a", "b"));
    state.select_next();
    assert!(state.selected().is_some());

    state.clear();
    assert!(state.nodes().is_empty());
    assert!(state.edges().is_empty());
    assert_eq!(state.selected(), None);
}

// =============================================================================
// Status Updates
// =============================================================================

#[test]
fn test_update_node_status() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A").with_status(NodeStatus::Healthy));
    let old = state.update_node_status("a", NodeStatus::Down);
    assert_eq!(old, Some(NodeStatus::Healthy));
    assert_eq!(state.nodes()[0].status, NodeStatus::Down);
}

#[test]
fn test_update_node_status_not_found() {
    let mut state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    let old = state.update_node_status("nonexistent", NodeStatus::Down);
    assert_eq!(old, None);
}

#[test]
fn test_update_node_status_same_status() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A").with_status(NodeStatus::Healthy));
    let old = state.update_node_status("a", NodeStatus::Healthy);
    assert_eq!(old, Some(NodeStatus::Healthy));
}

// =============================================================================
// Navigation: SelectNext / SelectPrev
// =============================================================================

#[test]
fn test_select_next_empty() {
    let mut state = DependencyGraphState::new();
    assert!(!state.select_next());
    assert_eq!(state.selected(), None);
}

#[test]
fn test_select_next_first() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    assert!(state.select_next());
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_select_next_cycles() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    state.select_next(); // 0
    state.select_next(); // 1
    state.select_next(); // wraps to 0
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_select_prev_empty() {
    let mut state = DependencyGraphState::new();
    assert!(!state.select_prev());
    assert_eq!(state.selected(), None);
}

#[test]
fn test_select_prev_from_none() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    assert!(state.select_prev());
    assert_eq!(state.selected(), Some(1)); // wraps to last
}

#[test]
fn test_select_prev_wraps() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    state.select_next(); // 0
    state.select_prev(); // wraps to 1
    assert_eq!(state.selected(), Some(1));
}

#[test]
fn test_select_prev_decrements() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"))
        .with_node(GraphNode::new("c", "C"));
    state.select_next(); // 0
    state.select_next(); // 1
    state.select_next(); // 2
    state.select_prev(); // 1
    assert_eq!(state.selected(), Some(1));
}

// =============================================================================
// Navigation: SelectConnected
// =============================================================================

#[test]
fn test_select_connected() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"))
        .with_edge(GraphEdge::new("a", "b"));
    state.select_next(); // select a
    assert!(state.select_connected());
    assert_eq!(state.selected_node().unwrap().id, "b");
}

#[test]
fn test_select_connected_no_edges() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    state.select_next(); // select a
    assert!(!state.select_connected());
    assert_eq!(state.selected_node().unwrap().id, "a");
}

#[test]
fn test_select_connected_no_selection() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"))
        .with_edge(GraphEdge::new("a", "b"));
    assert!(!state.select_connected());
}

#[test]
fn test_select_connected_follows_first_edge() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"))
        .with_node(GraphNode::new("c", "C"))
        .with_edge(GraphEdge::new("a", "b"))
        .with_edge(GraphEdge::new("a", "c"));
    state.select_next(); // select a
    assert!(state.select_connected());
    assert_eq!(state.selected_node().unwrap().id, "b"); // follows first edge
}

// =============================================================================
// Selected Node
// =============================================================================

#[test]
fn test_selected_node_none() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert!(state.selected_node().is_none());
}

#[test]
fn test_selected_node_some() {
    let mut state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    state.select_next();
    assert_eq!(state.selected_node().unwrap().id, "a");
}

// =============================================================================
// Handle Event
// =============================================================================

#[test]
fn test_handle_event_not_focused() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    let msg =
        DependencyGraph::handle_event(&state, &Event::key(KeyCode::Down), &ViewContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    let msg =
        DependencyGraph::handle_event(&state, &Event::key(KeyCode::Down), &ViewContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_down() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &ViewContext::new().focused(true)
        ),
        Some(DependencyGraphMessage::SelectNext)
    );
}

#[test]
fn test_handle_event_j() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(&state, &Event::char('j'), &ViewContext::new().focused(true)),
        Some(DependencyGraphMessage::SelectNext)
    );
}

#[test]
fn test_handle_event_tab() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(
            &state,
            &Event::key(KeyCode::Tab),
            &ViewContext::new().focused(true)
        ),
        Some(DependencyGraphMessage::SelectNext)
    );
}

#[test]
fn test_handle_event_up() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &ViewContext::new().focused(true)
        ),
        Some(DependencyGraphMessage::SelectPrev)
    );
}

#[test]
fn test_handle_event_k() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(&state, &Event::char('k'), &ViewContext::new().focused(true)),
        Some(DependencyGraphMessage::SelectPrev)
    );
}

#[test]
fn test_handle_event_backtab() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(
            &state,
            &Event::key(KeyCode::BackTab),
            &ViewContext::new().focused(true)
        ),
        Some(DependencyGraphMessage::SelectPrev)
    );
}

#[test]
fn test_handle_event_enter() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(
            &state,
            &Event::key(KeyCode::Enter),
            &ViewContext::new().focused(true)
        ),
        Some(DependencyGraphMessage::SelectConnected)
    );
}

#[test]
fn test_handle_event_l() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(&state, &Event::char('l'), &ViewContext::new().focused(true)),
        Some(DependencyGraphMessage::SelectConnected)
    );
}

#[test]
fn test_handle_event_right() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(
            &state,
            &Event::key(KeyCode::Right),
            &ViewContext::new().focused(true)
        ),
        Some(DependencyGraphMessage::SelectConnected)
    );
}

#[test]
fn test_handle_event_unknown_key() {
    let state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert_eq!(
        DependencyGraph::handle_event(&state, &Event::char('x'), &ViewContext::new().focused(true)),
        None
    );
}

// =============================================================================
// Component Update Messages
// =============================================================================

#[test]
fn test_update_set_nodes() {
    let mut state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    state.select_next();
    assert!(state.selected().is_some());

    let output = DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::SetNodes(vec![GraphNode::new("x", "X"), GraphNode::new("y", "Y")]),
    );
    assert_eq!(output, None);
    assert_eq!(state.nodes().len(), 2);
    assert_eq!(state.selected(), None); // reset
}

#[test]
fn test_update_set_edges() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    let output = DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::SetEdges(vec![GraphEdge::new("a", "b")]),
    );
    assert_eq!(output, None);
    assert_eq!(state.edges().len(), 1);
}

#[test]
fn test_update_add_node() {
    let mut state = DependencyGraphState::new();
    let output = DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::AddNode(GraphNode::new("a", "A")),
    );
    assert_eq!(output, None);
    assert_eq!(state.nodes().len(), 1);
}

#[test]
fn test_update_add_edge() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    let output = DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::AddEdge(GraphEdge::new("a", "b")),
    );
    assert_eq!(output, None);
    assert_eq!(state.edges().len(), 1);
}

#[test]
fn test_update_status_change() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A").with_status(NodeStatus::Healthy));
    let output = DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::UpdateNodeStatus {
            id: "a".to_string(),
            status: NodeStatus::Down,
        },
    );
    assert_eq!(
        output,
        Some(DependencyGraphOutput::StatusChanged {
            id: "a".to_string(),
            old: NodeStatus::Healthy,
            new_status: NodeStatus::Down,
        })
    );
}

#[test]
fn test_update_status_no_change() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A").with_status(NodeStatus::Healthy));
    let output = DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::UpdateNodeStatus {
            id: "a".to_string(),
            status: NodeStatus::Healthy,
        },
    );
    assert_eq!(output, None);
}

#[test]
fn test_update_status_not_found() {
    let mut state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    let output = DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::UpdateNodeStatus {
            id: "nonexistent".to_string(),
            status: NodeStatus::Down,
        },
    );
    assert_eq!(output, None);
}

#[test]
fn test_update_clear() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_edge(GraphEdge::new("a", "b"));
    let output = DependencyGraph::update(&mut state, DependencyGraphMessage::Clear);
    assert_eq!(output, None);
    assert!(state.nodes().is_empty());
    assert!(state.edges().is_empty());
}

#[test]
fn test_update_select_next_output() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));

    let output = DependencyGraph::update(&mut state, DependencyGraphMessage::SelectNext);
    assert_eq!(
        output,
        Some(DependencyGraphOutput::NodeSelected("a".to_string()))
    );
}

#[test]
fn test_update_select_prev_output() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));

    let output = DependencyGraph::update(&mut state, DependencyGraphMessage::SelectPrev);
    assert_eq!(
        output,
        Some(DependencyGraphOutput::NodeSelected("b".to_string()))
    );
}

#[test]
fn test_update_select_connected_output() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"))
        .with_edge(GraphEdge::new("a", "b"));
    state.select_next(); // select a

    let output = DependencyGraph::update(&mut state, DependencyGraphMessage::SelectConnected);
    assert_eq!(
        output,
        Some(DependencyGraphOutput::NodeSelected("b".to_string()))
    );
}

#[test]
fn test_update_disabled_allows_data_changes() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A").with_status(NodeStatus::Healthy));

    // Data messages should still work when disabled
    let output = DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::UpdateNodeStatus {
            id: "a".to_string(),
            status: NodeStatus::Down,
        },
    );
    assert!(output.is_some());
}

// =============================================================================
// Dispatch Event
// =============================================================================

#[test]
fn test_dispatch_event() {
    let mut state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));

    let output = DependencyGraph::dispatch_event(
        &mut state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true),
    );
    assert_eq!(
        output,
        Some(DependencyGraphOutput::NodeSelected("a".to_string()))
    );
}

// =============================================================================
// Instance Methods
// =============================================================================

#[test]
fn test_instance_update() {
    let mut state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    let output = state.update(DependencyGraphMessage::SelectNext);
    assert!(output.is_some());
}

// =============================================================================
// Component Trait
// =============================================================================

#[test]
fn test_init() {
    let state = DependencyGraph::init();
    assert!(state.nodes().is_empty());
    assert!(state.edges().is_empty());
}

// =============================================================================
// Focusable / Disableable
// =============================================================================

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_single_node_no_edges() {
    let mut state = DependencyGraphState::new().with_node(GraphNode::new("a", "A"));
    assert!(state.select_next());
    assert_eq!(state.selected(), Some(0));
    assert!(!state.select_connected()); // no edges
}

#[test]
fn test_cycle_does_not_infinite_loop() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"))
        .with_edge(GraphEdge::new("a", "b"))
        .with_edge(GraphEdge::new("b", "a"));

    // Navigate through cycle
    state.select_next(); // select a
    assert!(state.select_connected()); // follow to b
    assert_eq!(state.selected_node().unwrap().id, "b");
    assert!(state.select_connected()); // follow back to a
    assert_eq!(state.selected_node().unwrap().id, "a");
}

#[test]
fn test_disconnected_nodes() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    // No edges between them
    state.select_next(); // select a
    assert!(!state.select_connected()); // no outgoing edges
}

#[test]
fn test_set_nodes_replaces_all() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"));
    state.select_next();

    DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::SetNodes(vec![GraphNode::new("x", "X")]),
    );
    assert_eq!(state.nodes().len(), 1);
    assert_eq!(state.nodes()[0].id, "x");
    assert_eq!(state.selected(), None);
}

#[test]
fn test_set_edges_replaces_all() {
    let mut state = DependencyGraphState::new()
        .with_node(GraphNode::new("a", "A"))
        .with_node(GraphNode::new("b", "B"))
        .with_edge(GraphEdge::new("a", "b"));
    assert_eq!(state.edges().len(), 1);

    DependencyGraph::update(
        &mut state,
        DependencyGraphMessage::SetEdges(vec![GraphEdge::new("b", "a"), GraphEdge::new("a", "b")]),
    );
    assert_eq!(state.edges().len(), 2);
}
