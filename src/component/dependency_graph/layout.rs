//! Layout algorithm for positioning graph nodes and computing edge paths.
//!
//! Uses a layered/hierarchical approach:
//! 1. Find root nodes (no incoming edges)
//! 2. Assign layers via BFS from roots
//! 3. Position nodes evenly within each layer

use std::collections::{HashMap, HashSet, VecDeque};

use ratatui::prelude::Rect;

use super::{GraphEdge, GraphNode, GraphOrientation};

/// A positioned node in the computed layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutNode {
    /// The node id.
    pub id: String,
    /// X position of the top-left corner.
    pub x: u16,
    /// Y position of the top-left corner.
    pub y: u16,
    /// Width of the node box.
    pub width: u16,
    /// Height of the node box.
    pub height: u16,
}

/// A positioned edge in the computed layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutEdge {
    /// X position of the edge start point.
    pub from_x: u16,
    /// Y position of the edge start point.
    pub from_y: u16,
    /// X position of the edge end point.
    pub to_x: u16,
    /// Y position of the edge end point.
    pub to_y: u16,
}

/// Assigns layer indices to nodes via BFS from root nodes.
///
/// Nodes with no incoming edges are placed at layer 0. Each subsequent
/// layer contains nodes whose predecessors are all in earlier layers.
/// Nodes in cycles or disconnected from roots are assigned to a final layer.
fn assign_layers(nodes: &[GraphNode], edges: &[GraphEdge]) -> HashMap<String, usize> {
    let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    let mut incoming: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut outgoing: HashMap<&str, Vec<&str>> = HashMap::new();

    for id in &node_ids {
        incoming.entry(id).or_default();
        outgoing.entry(id).or_default();
    }

    for edge in edges {
        if node_ids.contains(edge.from.as_str()) && node_ids.contains(edge.to.as_str()) {
            incoming
                .entry(edge.to.as_str())
                .or_default()
                .insert(edge.from.as_str());
            outgoing
                .entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }
    }

    // Find roots: nodes with no incoming edges
    let roots: Vec<&str> = node_ids
        .iter()
        .filter(|id| incoming.get(*id).map_or(true, |s| s.is_empty()))
        .copied()
        .collect();

    let mut layers: HashMap<String, usize> = HashMap::new();
    let mut queue: VecDeque<&str> = VecDeque::new();

    for root in &roots {
        layers.insert(root.to_string(), 0);
        queue.push_back(root);
    }

    // BFS to assign layers
    while let Some(current) = queue.pop_front() {
        let current_layer = layers[current];
        if let Some(neighbors) = outgoing.get(current) {
            for &neighbor in neighbors {
                let new_layer = current_layer + 1;
                let entry = layers.entry(neighbor.to_string()).or_insert(0);
                if *entry < new_layer {
                    *entry = new_layer;
                    queue.push_back(neighbor);
                }
            }
        }
    }

    // Assign unvisited nodes (in cycles or disconnected) to max_layer + 1
    let max_layer = layers.values().copied().max().unwrap_or(0);
    for id in &node_ids {
        layers
            .entry(id.to_string())
            .or_insert(max_layer.saturating_add(1));
    }

    layers
}

/// Computes layout positions for all nodes and edges within the given area.
///
/// Returns `(layout_nodes, layout_edges)` where each node has a position and
/// size, and each edge has start and end coordinates.
pub fn compute_layout(
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    area: Rect,
    orientation: &GraphOrientation,
) -> (Vec<LayoutNode>, Vec<LayoutEdge>) {
    if nodes.is_empty() || area.width < 2 || area.height < 2 {
        return (Vec::new(), Vec::new());
    }

    let layers_map = assign_layers(nodes, edges);

    // Group nodes by layer, preserving insertion order within each layer
    let max_layer = layers_map.values().copied().max().unwrap_or(0);
    let mut layer_groups: Vec<Vec<&GraphNode>> = vec![Vec::new(); max_layer + 1];
    for node in nodes {
        if let Some(&layer) = layers_map.get(&node.id) {
            layer_groups[layer].push(node);
        }
    }

    // Remove empty layers
    layer_groups.retain(|g| !g.is_empty());

    let num_layers = layer_groups.len();
    if num_layers == 0 {
        return (Vec::new(), Vec::new());
    }

    let max_nodes_in_layer = layer_groups.iter().map(|g| g.len()).max().unwrap_or(1);

    // Compute node dimensions and positions based on orientation
    let node_positions = match orientation {
        GraphOrientation::LeftToRight => {
            compute_left_to_right(&layer_groups, num_layers, max_nodes_in_layer, area)
        }
        GraphOrientation::TopToBottom => {
            compute_top_to_bottom(&layer_groups, num_layers, max_nodes_in_layer, area)
        }
    };

    // Build position lookup for edge computation
    let position_map: HashMap<&str, &LayoutNode> =
        node_positions.iter().map(|n| (n.id.as_str(), n)).collect();

    // Compute edges
    let layout_edges = edges
        .iter()
        .filter_map(|edge| {
            let from_node = position_map.get(edge.from.as_str())?;
            let to_node = position_map.get(edge.to.as_str())?;
            Some(compute_edge_endpoints(from_node, to_node, orientation))
        })
        .collect();

    (node_positions, layout_edges)
}

/// Computes node positions for left-to-right orientation.
///
/// Layers are arranged as columns from left to right.
fn compute_left_to_right(
    layer_groups: &[Vec<&GraphNode>],
    num_layers: usize,
    max_nodes_in_layer: usize,
    area: Rect,
) -> Vec<LayoutNode> {
    // Node box: 3 lines tall (border + content + border), width varies
    let node_height: u16 = 3;

    // Divide horizontal space among layers
    let col_width = area.width / num_layers as u16;
    // Divide vertical space among nodes in each layer
    let row_height = if max_nodes_in_layer > 0 {
        area.height / max_nodes_in_layer as u16
    } else {
        area.height
    };

    let node_width = col_width.saturating_sub(2).max(6);

    let mut positions = Vec::new();

    for (layer_idx, layer) in layer_groups.iter().enumerate() {
        let col_x = area.x + layer_idx as u16 * col_width;
        let total_height = layer.len() as u16 * node_height + layer.len().saturating_sub(1) as u16;
        let start_y = area.y + area.height.saturating_sub(total_height) / 2;

        for (node_idx, node) in layer.iter().enumerate() {
            let node_y = if layer.len() == 1 {
                start_y
            } else {
                let slot_y = area.y + node_idx as u16 * row_height;
                slot_y + row_height.saturating_sub(node_height) / 2
            };
            let node_x = col_x + col_width.saturating_sub(node_width) / 2;

            positions.push(LayoutNode {
                id: node.id.clone(),
                x: node_x,
                y: node_y,
                width: node_width,
                height: node_height,
            });
        }
    }

    positions
}

/// Computes node positions for top-to-bottom orientation.
///
/// Layers are arranged as rows from top to bottom.
fn compute_top_to_bottom(
    layer_groups: &[Vec<&GraphNode>],
    num_layers: usize,
    max_nodes_in_layer: usize,
    area: Rect,
) -> Vec<LayoutNode> {
    let node_height: u16 = 3;

    // Divide vertical space among layers
    let row_height = area.height / num_layers as u16;
    // Divide horizontal space among nodes in each layer
    let col_width = if max_nodes_in_layer > 0 {
        area.width / max_nodes_in_layer as u16
    } else {
        area.width
    };

    let node_width = col_width.saturating_sub(2).max(6);

    let mut positions = Vec::new();

    for (layer_idx, layer) in layer_groups.iter().enumerate() {
        let row_y = area.y + layer_idx as u16 * row_height;
        let total_width = layer.len() as u16 * node_width + layer.len().saturating_sub(1) as u16;
        let start_x = area.x + area.width.saturating_sub(total_width) / 2;

        for (node_idx, node) in layer.iter().enumerate() {
            let node_x = if layer.len() == 1 {
                start_x
            } else {
                let slot_x = area.x + node_idx as u16 * col_width;
                slot_x + col_width.saturating_sub(node_width) / 2
            };
            let node_y = row_y + row_height.saturating_sub(node_height) / 2;

            positions.push(LayoutNode {
                id: node.id.clone(),
                x: node_x,
                y: node_y,
                width: node_width,
                height: node_height,
            });
        }
    }

    positions
}

/// Computes edge start and end points based on node positions and orientation.
fn compute_edge_endpoints(
    from: &LayoutNode,
    to: &LayoutNode,
    orientation: &GraphOrientation,
) -> LayoutEdge {
    match orientation {
        GraphOrientation::LeftToRight => {
            // Edge exits from right side of source, enters left side of target
            LayoutEdge {
                from_x: from.x + from.width,
                from_y: from.y + from.height / 2,
                to_x: to.x,
                to_y: to.y + to.height / 2,
            }
        }
        GraphOrientation::TopToBottom => {
            // Edge exits from bottom of source, enters top of target
            LayoutEdge {
                from_x: from.x + from.width / 2,
                from_y: from.y + from.height,
                to_x: to.x + to.width / 2,
                to_y: to.y,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str) -> GraphNode {
        GraphNode::new(id, id)
    }

    fn make_edge(from: &str, to: &str) -> GraphEdge {
        GraphEdge::new(from, to)
    }

    #[test]
    fn test_assign_layers_linear() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "c")];
        let layers = assign_layers(&nodes, &edges);
        assert_eq!(layers["a"], 0);
        assert_eq!(layers["b"], 1);
        assert_eq!(layers["c"], 2);
    }

    #[test]
    fn test_assign_layers_diamond() {
        let nodes = vec![
            make_node("a"),
            make_node("b"),
            make_node("c"),
            make_node("d"),
        ];
        let edges = vec![
            make_edge("a", "b"),
            make_edge("a", "c"),
            make_edge("b", "d"),
            make_edge("c", "d"),
        ];
        let layers = assign_layers(&nodes, &edges);
        assert_eq!(layers["a"], 0);
        assert_eq!(layers["b"], 1);
        assert_eq!(layers["c"], 1);
        assert_eq!(layers["d"], 2);
    }

    #[test]
    fn test_assign_layers_no_edges() {
        let nodes = vec![make_node("a"), make_node("b")];
        let edges: Vec<GraphEdge> = vec![];
        let layers = assign_layers(&nodes, &edges);
        assert_eq!(layers["a"], 0);
        assert_eq!(layers["b"], 0);
    }

    #[test]
    fn test_assign_layers_cycle() {
        // Cycles should not cause infinite loops
        let nodes = vec![make_node("a"), make_node("b")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "a")];
        let layers = assign_layers(&nodes, &edges);
        // Both nodes should be assigned layers without panic
        assert!(layers.contains_key("a"));
        assert!(layers.contains_key("b"));
    }

    #[test]
    fn test_compute_layout_empty() {
        let area = Rect::new(0, 0, 80, 24);
        let (nodes, edges) = compute_layout(&[], &[], area, &GraphOrientation::LeftToRight);
        assert!(nodes.is_empty());
        assert!(edges.is_empty());
    }

    #[test]
    fn test_compute_layout_single_node() {
        let nodes = vec![make_node("a")];
        let edges: Vec<GraphEdge> = vec![];
        let area = Rect::new(0, 0, 80, 24);
        let (layout_nodes, layout_edges) =
            compute_layout(&nodes, &edges, area, &GraphOrientation::LeftToRight);
        assert_eq!(layout_nodes.len(), 1);
        assert_eq!(layout_nodes[0].id, "a");
        assert!(layout_edges.is_empty());
    }

    #[test]
    fn test_compute_layout_nodes_no_overlap() {
        let nodes = vec![
            make_node("a"),
            make_node("b"),
            make_node("c"),
            make_node("d"),
        ];
        let edges = vec![
            make_edge("a", "b"),
            make_edge("a", "c"),
            make_edge("b", "d"),
            make_edge("c", "d"),
        ];
        let area = Rect::new(0, 0, 120, 40);
        let (layout_nodes, _) =
            compute_layout(&nodes, &edges, area, &GraphOrientation::LeftToRight);

        // Verify no overlap: for any two nodes, their bounding boxes should not intersect
        for i in 0..layout_nodes.len() {
            for j in (i + 1)..layout_nodes.len() {
                let a = &layout_nodes[i];
                let b = &layout_nodes[j];
                let overlaps_x = a.x < b.x + b.width && b.x < a.x + a.width;
                let overlaps_y = a.y < b.y + b.height && b.y < a.y + a.height;
                assert!(
                    !(overlaps_x && overlaps_y),
                    "Nodes {} and {} overlap: ({},{} {}x{}) vs ({},{} {}x{})",
                    a.id,
                    b.id,
                    a.x,
                    a.y,
                    a.width,
                    a.height,
                    b.x,
                    b.y,
                    b.width,
                    b.height,
                );
            }
        }
    }

    #[test]
    fn test_compute_layout_edges_connect_correct_nodes() {
        let nodes = vec![make_node("a"), make_node("b")];
        let edges = vec![make_edge("a", "b")];
        let area = Rect::new(0, 0, 80, 24);
        let (layout_nodes, layout_edges) =
            compute_layout(&nodes, &edges, area, &GraphOrientation::LeftToRight);

        assert_eq!(layout_edges.len(), 1);
        let edge = &layout_edges[0];

        // Edge should start from the right side of 'a' and end at the left side of 'b'
        let a = layout_nodes.iter().find(|n| n.id == "a").unwrap();
        let b = layout_nodes.iter().find(|n| n.id == "b").unwrap();
        assert_eq!(edge.from_x, a.x + a.width);
        assert_eq!(edge.to_x, b.x);
    }

    #[test]
    fn test_compute_layout_top_to_bottom() {
        let nodes = vec![make_node("a"), make_node("b")];
        let edges = vec![make_edge("a", "b")];
        let area = Rect::new(0, 0, 80, 24);
        let (layout_nodes, layout_edges) =
            compute_layout(&nodes, &edges, area, &GraphOrientation::TopToBottom);

        assert_eq!(layout_nodes.len(), 2);
        assert_eq!(layout_edges.len(), 1);

        let a = layout_nodes.iter().find(|n| n.id == "a").unwrap();
        let b = layout_nodes.iter().find(|n| n.id == "b").unwrap();

        // In top-to-bottom, 'a' should be above 'b'
        assert!(a.y < b.y);

        let edge = &layout_edges[0];
        assert_eq!(edge.from_y, a.y + a.height);
        assert_eq!(edge.to_y, b.y);
    }

    #[test]
    fn test_compute_layout_tiny_area() {
        let nodes = vec![make_node("a")];
        let area = Rect::new(0, 0, 1, 1);
        let (layout_nodes, _) = compute_layout(&nodes, &[], area, &GraphOrientation::LeftToRight);
        assert!(layout_nodes.is_empty());
    }

    #[test]
    fn test_compute_layout_cycle_does_not_hang() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![
            make_edge("a", "b"),
            make_edge("b", "c"),
            make_edge("c", "a"),
        ];
        let area = Rect::new(0, 0, 120, 40);
        // This should complete without hanging
        let (layout_nodes, layout_edges) =
            compute_layout(&nodes, &edges, area, &GraphOrientation::LeftToRight);
        assert_eq!(layout_nodes.len(), 3);
        assert_eq!(layout_edges.len(), 3);
    }

    #[test]
    fn test_layout_node_within_area() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "c")];
        let area = Rect::new(5, 5, 80, 24);
        let (layout_nodes, _) =
            compute_layout(&nodes, &edges, area, &GraphOrientation::LeftToRight);

        for node in &layout_nodes {
            assert!(
                node.x >= area.x,
                "Node {} x={} is before area x={}",
                node.id,
                node.x,
                area.x
            );
            assert!(
                node.y >= area.y,
                "Node {} y={} is before area y={}",
                node.id,
                node.y,
                area.y
            );
        }
    }
}
