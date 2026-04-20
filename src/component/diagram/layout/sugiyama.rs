//! Sugiyama hierarchical layout algorithm.
//!
//! Implements a four-phase approach for laying out directed graphs:
//! 1. Layer assignment via BFS from roots
//! 2. Crossing minimization via barycenter heuristic
//! 3. Coordinate assignment with minimum spacing
//! 4. Edge path computation

use std::collections::VecDeque;

use super::{LayoutAlgorithm, LayoutHints, LayoutResult, NodePosition};
use crate::component::diagram::edge_routing;
use crate::component::diagram::graph::IndexedGraph;
use crate::component::diagram::types::{DiagramCluster, DiagramEdge, DiagramNode, Orientation};
use crate::component::diagram::viewport::BoundingBox;

/// Sugiyama-style hierarchical layout.
pub(crate) struct SugiyamaLayout {
    /// Number of crossing minimization sweep iterations.
    max_iterations: usize,
}

impl Default for SugiyamaLayout {
    fn default() -> Self {
        Self { max_iterations: 4 }
    }
}

impl LayoutAlgorithm for SugiyamaLayout {
    fn compute(
        &self,
        graph: &IndexedGraph,
        nodes: &[DiagramNode],
        _edges: &[DiagramEdge],
        _clusters: &[DiagramCluster],
        hints: &LayoutHints<'_>,
    ) -> LayoutResult {
        if graph.node_count() == 0 {
            return LayoutResult::empty();
        }

        // Phase 1: Layer assignment
        let layers = self.assign_layers(graph);

        // Group nodes by layer
        let max_layer = layers.iter().copied().max().unwrap_or(0);
        let mut layer_groups: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
        for (node_idx, &layer) in layers.iter().enumerate() {
            layer_groups[layer].push(node_idx);
        }

        // Remove empty layers
        layer_groups.retain(|group| !group.is_empty());

        // Phase 2: Crossing minimization
        self.minimize_crossings(&mut layer_groups, graph);

        // Phase 3: Coordinate assignment
        let node_positions = self.assign_coordinates(&layer_groups, nodes, hints);

        // Phase 4: Edge routing
        let node_ids: Vec<String> = nodes.iter().map(|n| n.id().to_string()).collect();
        let edge_paths = edge_routing::compute_routed_edges(
            graph.edge_pairs(),
            &node_ids,
            &node_positions,
            &hints.orientation,
        );

        // Compute bounding box
        let bounding_box = compute_bounding_box(&node_positions);

        LayoutResult {
            node_positions,
            edge_paths,
            bounding_box,
        }
    }
}

impl SugiyamaLayout {
    /// Assigns each node to a layer using BFS from roots.
    ///
    /// Nodes with no incoming edges are placed at layer 0.
    /// Each node is assigned to max(predecessor_layers) + 1.
    /// Nodes in cycles or disconnected get max_layer + 1.
    fn assign_layers(&self, graph: &IndexedGraph) -> Vec<usize> {
        let n = graph.node_count();
        let mut layers = vec![0usize; n];
        let mut visited = vec![false; n];

        let roots = graph.roots();
        let mut queue = VecDeque::new();

        // Start BFS from all roots
        for &root in &roots {
            visited[root] = true;
            queue.push_back(root);
        }

        // If no roots (all cycles), start from node 0
        if roots.is_empty() && n > 0 {
            visited[0] = true;
            queue.push_back(0);
        }

        while let Some(node) = queue.pop_front() {
            for &succ in graph.successors(node) {
                let new_layer = layers[node] + 1;
                if new_layer > layers[succ] {
                    layers[succ] = new_layer;
                }
                if !visited[succ] {
                    visited[succ] = true;
                    queue.push_back(succ);
                }
            }
        }

        // Handle unvisited nodes (disconnected components)
        let max_layer = layers.iter().copied().max().unwrap_or(0);
        for i in 0..n {
            if !visited[i] {
                layers[i] = max_layer + 1;
            }
        }

        layers
    }

    /// Minimizes edge crossings using barycenter heuristic.
    ///
    /// Iterates through adjacent layer pairs, reordering each layer
    /// to minimize crossings with its neighbor. Alternates sweep
    /// direction (top-to-bottom, then bottom-to-top).
    fn minimize_crossings(&self, layer_groups: &mut [Vec<usize>], graph: &IndexedGraph) {
        if layer_groups.len() <= 1 {
            return;
        }

        for iteration in 0..self.max_iterations {
            let sweep_down = iteration % 2 == 0;

            if sweep_down {
                for i in 1..layer_groups.len() {
                    let (fixed, free) = layer_groups.split_at_mut(i);
                    let fixed_layer = &fixed[i - 1];
                    let free_layer = &mut free[0];
                    self.reorder_by_barycenter(free_layer, fixed_layer, graph, true);
                }
            } else {
                for i in (0..layer_groups.len() - 1).rev() {
                    let (free_part, fixed_part) = layer_groups.split_at_mut(i + 1);
                    let free_layer = &mut free_part[i];
                    let fixed_layer = &fixed_part[0];
                    self.reorder_by_barycenter(free_layer, fixed_layer, graph, false);
                }
            }
        }
    }

    /// Reorders `free_layer` to minimize crossings with `fixed_layer`.
    ///
    /// For each node in `free_layer`, computes its barycenter (average
    /// position of its connected nodes in `fixed_layer`) and sorts by it.
    fn reorder_by_barycenter(
        &self,
        free_layer: &mut [usize],
        fixed_layer: &[usize],
        graph: &IndexedGraph,
        use_predecessors: bool,
    ) {
        // Build position map for fixed layer
        let mut position: Vec<Option<usize>> = vec![None; graph.node_count()];
        for (pos, &node) in fixed_layer.iter().enumerate() {
            position[node] = Some(pos);
        }

        // Compute barycenters
        let mut barycenters: Vec<(usize, f64)> = free_layer
            .iter()
            .map(|&node| {
                let connected = if use_predecessors {
                    graph.predecessors(node)
                } else {
                    graph.successors(node)
                };

                let positions: Vec<f64> = connected
                    .iter()
                    .filter_map(|&adj| position[adj].map(|p| p as f64))
                    .collect();

                let bc = if positions.is_empty() {
                    f64::MAX // no connections, keep in place
                } else {
                    positions.iter().sum::<f64>() / positions.len() as f64
                };

                (node, bc)
            })
            .collect();

        // Sort by barycenter (stable sort preserves order for equal values)
        barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Apply new ordering
        for (i, (node, _)) in barycenters.into_iter().enumerate() {
            free_layer[i] = node;
        }
    }

    /// Assigns x/y coordinates to nodes based on their layer and position.
    fn assign_coordinates(
        &self,
        layer_groups: &[Vec<usize>],
        nodes: &[DiagramNode],
        hints: &LayoutHints<'_>,
    ) -> Vec<NodePosition> {
        let min_node_width = 12.0f64;
        let node_height = 3.0;
        let layer_spacing = hints.layer_spacing;
        let node_spacing = hints.node_spacing;

        // Compute max label width per layer for uniform column widths
        let max_width_per_layer: Vec<f64> = layer_groups
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|&idx| {
                        let label_len = nodes.get(idx).map_or(4, |n| n.label().len());
                        (label_len as f64 + 4.0).max(min_node_width)
                    })
                    .fold(min_node_width, f64::max)
            })
            .collect();

        let max_nodes_in_layer = layer_groups.iter().map(|g| g.len()).max().unwrap_or(1);

        let mut positions = vec![NodePosition::new(String::new(), 0.0, 0.0, 0.0, 0.0); nodes.len()];

        match hints.orientation {
            Orientation::LeftToRight => {
                let mut x = 0.0;
                for (layer_idx, group) in layer_groups.iter().enumerate() {
                    let col_width = max_width_per_layer[layer_idx];
                    let total_height =
                        max_nodes_in_layer as f64 * (node_height + node_spacing) - node_spacing;
                    let group_height =
                        group.len() as f64 * (node_height + node_spacing) - node_spacing;
                    let y_offset = (total_height - group_height) / 2.0;

                    for (pos_in_layer, &node_idx) in group.iter().enumerate() {
                        let y = y_offset + pos_in_layer as f64 * (node_height + node_spacing);
                        let id = nodes
                            .get(node_idx)
                            .map_or_else(String::new, |n| n.id().to_string());
                        positions[node_idx] = NodePosition::new(id, x, y, col_width, node_height);
                    }

                    x += col_width + layer_spacing;
                }
            }
            Orientation::TopToBottom => {
                let mut y = 0.0;
                for (layer_idx, group) in layer_groups.iter().enumerate() {
                    let row_width = max_width_per_layer[layer_idx];
                    let total_width =
                        max_nodes_in_layer as f64 * (row_width + node_spacing) - node_spacing;
                    let group_width =
                        group.len() as f64 * (row_width + node_spacing) - node_spacing;
                    let x_offset = (total_width - group_width) / 2.0;

                    for (pos_in_layer, &node_idx) in group.iter().enumerate() {
                        let x = x_offset + pos_in_layer as f64 * (row_width + node_spacing);
                        let id = nodes
                            .get(node_idx)
                            .map_or_else(String::new, |n| n.id().to_string());
                        positions[node_idx] = NodePosition::new(id, x, y, row_width, node_height);
                    }

                    y += node_height + layer_spacing;
                }
            }
        }

        positions
    }
}

fn compute_bounding_box(positions: &[NodePosition]) -> BoundingBox {
    if positions.is_empty() {
        return BoundingBox::default();
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for pos in positions {
        min_x = min_x.min(pos.x());
        min_y = min_y.min(pos.y());
        max_x = max_x.max(pos.x() + pos.width());
        max_y = max_y.max(pos.y() + pos.height());
    }

    BoundingBox::new(min_x, min_y, max_x, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: &str) -> DiagramNode {
        DiagramNode::new(id, id)
    }

    fn make_edge(from: &str, to: &str) -> DiagramEdge {
        DiagramEdge::new(from, to)
    }

    fn run_layout(nodes: &[DiagramNode], edges: &[DiagramEdge]) -> LayoutResult {
        let graph = IndexedGraph::build(nodes, edges);
        let layout = SugiyamaLayout::default();
        layout.compute(&graph, nodes, edges, &[], &LayoutHints::default())
    }

    #[test]
    fn test_empty_graph() {
        let result = run_layout(&[], &[]);
        assert!(result.node_positions.is_empty());
        assert!(result.edge_paths.is_empty());
    }

    #[test]
    fn test_single_node() {
        let nodes = vec![make_node("a")];
        let result = run_layout(&nodes, &[]);

        assert_eq!(result.node_positions.len(), 1);
        assert_eq!(result.node_positions[0].id(), "a");
        assert!(result.node_positions[0].width() >= 12.0);
    }

    #[test]
    fn test_linear_chain() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "c")];
        let result = run_layout(&nodes, &edges);

        assert_eq!(result.node_positions.len(), 3);
        assert_eq!(result.edge_paths.len(), 2);

        // In left-to-right layout, a should be left of b, b left of c
        let a = &result.node_positions[0];
        let b = &result.node_positions[1];
        let c = &result.node_positions[2];
        assert!(a.x() < b.x(), "a should be left of b");
        assert!(b.x() < c.x(), "b should be left of c");
    }

    #[test]
    fn test_diamond_graph() {
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
        let result = run_layout(&nodes, &edges);

        assert_eq!(result.node_positions.len(), 4);

        // a should be at layer 0, b/c at layer 1, d at layer 2
        let a = &result.node_positions[0];
        let d = &result.node_positions[3];
        assert!(a.x() < d.x(), "a should be left of d");
    }

    #[test]
    fn test_no_node_overlap() {
        let nodes = vec![
            make_node("a"),
            make_node("b"),
            make_node("c"),
            make_node("d"),
            make_node("e"),
        ];
        let edges = vec![
            make_edge("a", "b"),
            make_edge("a", "c"),
            make_edge("a", "d"),
            make_edge("a", "e"),
        ];
        let result = run_layout(&nodes, &edges);

        // Check no two nodes overlap
        for i in 0..result.node_positions.len() {
            for j in (i + 1)..result.node_positions.len() {
                let a = &result.node_positions[i];
                let b = &result.node_positions[j];
                let overlaps_x = a.x() < b.x() + b.width() && b.x() < a.x() + a.width();
                let overlaps_y = a.y() < b.y() + b.height() && b.y() < a.y() + a.height();
                assert!(
                    !(overlaps_x && overlaps_y),
                    "Nodes {} and {} overlap",
                    a.id(),
                    b.id()
                );
            }
        }
    }

    #[test]
    fn test_cycle_does_not_hang() {
        let nodes = vec![make_node("a"), make_node("b")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "a")];

        // This should complete without hanging
        let result = run_layout(&nodes, &edges);
        assert_eq!(result.node_positions.len(), 2);
    }

    #[test]
    fn test_crossing_minimization() {
        // Create a graph where insertion order would cause crossings
        // but barycenter should reduce them:
        //   a -> c
        //   b -> d
        // If c and d are in the wrong order, the edges cross.
        let nodes = vec![
            make_node("a"),
            make_node("b"),
            make_node("c"),
            make_node("d"),
        ];
        let edges = vec![make_edge("a", "c"), make_edge("b", "d")];
        let result = run_layout(&nodes, &edges);

        // After crossing minimization, a-c and b-d should not cross
        // meaning the vertical order of {c,d} should match {a,b}
        let a_y = result.node_positions[0].center_y();
        let b_y = result.node_positions[1].center_y();
        let c_y = result.node_positions[2].center_y();
        let d_y = result.node_positions[3].center_y();

        // If a is above b, c should be above d (and vice versa)
        if a_y < b_y {
            assert!(c_y <= d_y, "c should be above or at same level as d");
        } else {
            assert!(d_y <= c_y, "d should be above or at same level as c");
        }
    }

    #[test]
    fn test_top_to_bottom_orientation() {
        let nodes = vec![make_node("a"), make_node("b")];
        let edges = vec![make_edge("a", "b")];
        let graph = IndexedGraph::build(&nodes, &edges);
        let layout = SugiyamaLayout::default();
        let hints = LayoutHints {
            orientation: Orientation::TopToBottom,
            ..LayoutHints::default()
        };
        let result = layout.compute(&graph, &nodes, &edges, &[], &hints);

        let a = &result.node_positions[0];
        let b = &result.node_positions[1];
        assert!(a.y() < b.y(), "a should be above b in top-to-bottom");
    }

    #[test]
    fn test_bounding_box() {
        let nodes = vec![make_node("a"), make_node("b")];
        let edges = vec![make_edge("a", "b")];
        let result = run_layout(&nodes, &edges);

        let bbox = &result.bounding_box;
        assert!(bbox.width() > 0.0);
        assert!(bbox.height() > 0.0);

        // All nodes should be within bounding box
        for pos in &result.node_positions {
            assert!(pos.x() >= bbox.min_x);
            assert!(pos.y() >= bbox.min_y);
            assert!(pos.x() + pos.width() <= bbox.max_x);
            assert!(pos.y() + pos.height() <= bbox.max_y);
        }
    }

    #[test]
    fn test_edge_paths_generated() {
        let nodes = vec![make_node("a"), make_node("b")];
        let edges = vec![make_edge("a", "b")];
        let result = run_layout(&nodes, &edges);

        assert_eq!(result.edge_paths.len(), 1);
        let path = &result.edge_paths[0];
        assert_eq!(path.from_id(), "a");
        assert_eq!(path.to_id(), "b");
        assert!(!path.segments().is_empty());
    }

    #[test]
    fn test_disconnected_nodes() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        // No edges — all disconnected
        let result = run_layout(&nodes, &[]);

        assert_eq!(result.node_positions.len(), 3);
        // All should still be positioned without overlap
        for i in 0..result.node_positions.len() {
            for j in (i + 1)..result.node_positions.len() {
                let a = &result.node_positions[i];
                let b = &result.node_positions[j];
                let overlaps_x = a.x() < b.x() + b.width() && b.x() < a.x() + a.width();
                let overlaps_y = a.y() < b.y() + b.height() && b.y() < a.y() + a.height();
                assert!(!(overlaps_x && overlaps_y));
            }
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    /// Returns a node ID string like "n0", "n1", etc.
    fn node_id(index: usize) -> String {
        format!("n{index}")
    }

    /// Strategy that generates edges referencing valid node indices.
    fn edges_strategy(
        node_count: usize,
        max_edges: usize,
    ) -> impl Strategy<Value = Vec<DiagramEdge>> {
        let edge_count = std::cmp::min(max_edges, node_count * node_count);
        prop::collection::vec((0..node_count, 0..node_count), 0..=edge_count).prop_map(|pairs| {
            pairs
                .into_iter()
                .map(|(from, to)| DiagramEdge::new(node_id(from), node_id(to)))
                .collect()
        })
    }

    /// Strategy that generates a random graph: (nodes, edges).
    fn graph_strategy(
        max_nodes: usize,
        max_edges: usize,
    ) -> impl Strategy<Value = (Vec<DiagramNode>, Vec<DiagramEdge>)> {
        (1..=max_nodes).prop_flat_map(move |count| {
            let nodes = Just(
                (0..count)
                    .map(|i| {
                        let id = node_id(i);
                        DiagramNode::new(&id, &id)
                    })
                    .collect::<Vec<_>>(),
            );
            let edges = edges_strategy(count, max_edges);
            (nodes, edges)
        })
    }

    fn run_layout(nodes: &[DiagramNode], edges: &[DiagramEdge]) -> LayoutResult {
        let graph = IndexedGraph::build(nodes, edges);
        let layout = SugiyamaLayout::default();
        layout.compute(&graph, nodes, edges, &[], &LayoutHints::default())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        #[test]
        fn no_node_overlap(
            (nodes, edges) in graph_strategy(50, 100)
        ) {
            let result = run_layout(&nodes, &edges);

            for i in 0..result.node_positions.len() {
                for j in (i + 1)..result.node_positions.len() {
                    let a = &result.node_positions[i];
                    let b = &result.node_positions[j];
                    let overlaps_x =
                        a.x() < b.x() + b.width() && b.x() < a.x() + a.width();
                    let overlaps_y =
                        a.y() < b.y() + b.height() && b.y() < a.y() + a.height();
                    prop_assert!(
                        !(overlaps_x && overlaps_y),
                        "Nodes {} and {} overlap: ({}, {}, {}x{}) vs ({}, {}, {}x{})",
                        a.id(), b.id(),
                        a.x(), a.y(), a.width(), a.height(),
                        b.x(), b.y(), b.width(), b.height(),
                    );
                }
            }
        }

        #[test]
        fn all_nodes_positioned(
            (nodes, edges) in graph_strategy(50, 100)
        ) {
            let result = run_layout(&nodes, &edges);

            prop_assert_eq!(
                result.node_positions.len(),
                nodes.len(),
                "Expected {} positions, got {}",
                nodes.len(),
                result.node_positions.len(),
            );

            let positioned_ids: HashSet<&str> =
                result.node_positions.iter().map(|p| p.id()).collect();
            for node in &nodes {
                prop_assert!(
                    positioned_ids.contains(node.id()),
                    "Node {} not found in layout output",
                    node.id(),
                );
            }
        }

        #[test]
        fn bounding_box_contains_all_nodes(
            (nodes, edges) in graph_strategy(50, 100)
        ) {
            let result = run_layout(&nodes, &edges);
            let bbox = &result.bounding_box;

            for pos in &result.node_positions {
                prop_assert!(
                    pos.x() >= bbox.min_x,
                    "Node {} x={} is below bbox min_x={}",
                    pos.id(), pos.x(), bbox.min_x,
                );
                prop_assert!(
                    pos.y() >= bbox.min_y,
                    "Node {} y={} is below bbox min_y={}",
                    pos.id(), pos.y(), bbox.min_y,
                );
                prop_assert!(
                    pos.x() + pos.width() <= bbox.max_x,
                    "Node {} right edge {} exceeds bbox max_x={}",
                    pos.id(), pos.x() + pos.width(), bbox.max_x,
                );
                prop_assert!(
                    pos.y() + pos.height() <= bbox.max_y,
                    "Node {} bottom edge {} exceeds bbox max_y={}",
                    pos.id(), pos.y() + pos.height(), bbox.max_y,
                );
            }
        }

        #[test]
        fn no_panics_on_arbitrary_input(
            (nodes, edges) in graph_strategy(50, 100)
        ) {
            // Simply running layout without panicking is the assertion.
            let _result = run_layout(&nodes, &edges);
        }

        #[test]
        fn edge_paths_generated_for_all_valid_edges(
            (nodes, edges) in graph_strategy(50, 100)
        ) {
            let result = run_layout(&nodes, &edges);

            let node_ids: HashSet<&str> = nodes.iter().map(|n| n.id()).collect();

            // Collect valid edges: both endpoints exist in node set.
            // Deduplicate since the layout may deduplicate edge paths.
            let valid_edges: HashSet<(&str, &str)> = edges
                .iter()
                .filter(|e| node_ids.contains(e.from()) && node_ids.contains(e.to()))
                .map(|e| (e.from(), e.to()))
                .collect();

            let path_edges: HashSet<(&str, &str)> = result
                .edge_paths
                .iter()
                .map(|p| (p.from_id(), p.to_id()))
                .collect();

            for (from, to) in &valid_edges {
                prop_assert!(
                    path_edges.contains(&(*from, *to)),
                    "Missing edge path from {} to {}",
                    from, to,
                );
            }
        }

        #[test]
        fn cycles_do_not_hang(
            node_count in 2..20usize,
        ) {
            // Create a cycle: n0 -> n1 -> n2 -> ... -> n0
            let nodes: Vec<DiagramNode> = (0..node_count)
                .map(|i| {
                    let id = node_id(i);
                    DiagramNode::new(&id, &id)
                })
                .collect();
            let edges: Vec<DiagramEdge> = (0..node_count)
                .map(|i| DiagramEdge::new(node_id(i), node_id((i + 1) % node_count)))
                .collect();

            let result = run_layout(&nodes, &edges);
            prop_assert_eq!(result.node_positions.len(), node_count);
        }

        #[test]
        fn self_loops_handled(
            node_count in 1..20usize,
        ) {
            let nodes: Vec<DiagramNode> = (0..node_count)
                .map(|i| {
                    let id = node_id(i);
                    DiagramNode::new(&id, &id)
                })
                .collect();
            // Every node has a self-loop
            let edges: Vec<DiagramEdge> = (0..node_count)
                .map(|i| DiagramEdge::new(node_id(i), node_id(i)))
                .collect();

            let result = run_layout(&nodes, &edges);
            prop_assert_eq!(result.node_positions.len(), node_count);
        }

        #[test]
        fn disconnected_components_positioned(
            component_count in 2..6usize,
            nodes_per_component in 1..10usize,
        ) {
            // Create multiple disconnected linear chains
            let total = component_count * nodes_per_component;
            let nodes: Vec<DiagramNode> = (0..total)
                .map(|i| {
                    let id = node_id(i);
                    DiagramNode::new(&id, &id)
                })
                .collect();
            let edges: Vec<DiagramEdge> = (0..component_count)
                .flat_map(|c| {
                    let start = c * nodes_per_component;
                    (start..start + nodes_per_component - 1)
                        .map(move |i| DiagramEdge::new(node_id(i), node_id(i + 1)))
                })
                .collect();

            let result = run_layout(&nodes, &edges);
            prop_assert_eq!(result.node_positions.len(), total);
        }
    }
}
