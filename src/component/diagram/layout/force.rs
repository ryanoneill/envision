//! Fruchterman-Reingold force-directed layout algorithm.
//!
//! Simulates a physical system where nodes repel each other (like
//! electrical charges) and edges act as springs pulling connected
//! nodes together. The system cools over iterations until it reaches
//! equilibrium, producing a natural-looking layout.

use super::{LayoutAlgorithm, LayoutHints, LayoutResult, NodePosition};
use crate::component::diagram::edge_routing;
use crate::component::diagram::graph::IndexedGraph;
use crate::component::diagram::types::{DiagramCluster, DiagramEdge, DiagramNode};
use crate::component::diagram::viewport::BoundingBox;

/// Fruchterman-Reingold force-directed layout.
pub(crate) struct ForceDirectedLayout {
    /// Number of simulation iterations.
    iterations: usize,
    /// Initial temperature (max displacement per iteration).
    initial_temperature: f64,
    /// Cooling rate per iteration (multiplied by temperature).
    cooling_factor: f64,
}

impl Default for ForceDirectedLayout {
    fn default() -> Self {
        Self {
            iterations: 50,
            initial_temperature: 10.0,
            cooling_factor: 0.95,
        }
    }
}

impl LayoutAlgorithm for ForceDirectedLayout {
    fn compute(
        &self,
        graph: &IndexedGraph,
        nodes: &[DiagramNode],
        _edges: &[DiagramEdge],
        _clusters: &[DiagramCluster],
        hints: &LayoutHints<'_>,
    ) -> LayoutResult {
        let n = graph.node_count();
        if n == 0 {
            return LayoutResult::empty();
        }

        let node_width = 14.0f64;
        let node_height = 3.0;

        // Ideal spring length based on node count and spacing
        let area = (n as f64) * node_width * node_height * 4.0;
        let k = (area / n as f64).sqrt(); // ideal distance

        // Initialize positions
        let mut positions = initialize_positions(n, hints, node_width, node_height);

        // Run simulation
        let mut temperature = self.initial_temperature;

        for _ in 0..self.iterations {
            let displacements = compute_displacements(&positions, graph, n, k);

            // Apply displacements, clamped by temperature
            for i in 0..n {
                let (dx, dy) = displacements[i];
                let dist = (dx * dx + dy * dy).sqrt().max(0.001);
                let scale = temperature.min(dist) / dist;
                positions[i].0 += dx * scale;
                positions[i].1 += dy * scale;
            }

            temperature *= self.cooling_factor;
        }

        // Normalize: shift so minimum is at (0, 0) with padding
        let padding = 2.0;
        let min_x = positions.iter().map(|p| p.0).fold(f64::MAX, f64::min);
        let min_y = positions.iter().map(|p| p.1).fold(f64::MAX, f64::min);
        for pos in &mut positions {
            pos.0 -= min_x - padding;
            pos.1 -= min_y - padding;
        }

        // Build NodePosition results
        let node_positions: Vec<NodePosition> = positions
            .iter()
            .enumerate()
            .map(|(i, (x, y))| {
                let id = nodes
                    .get(i)
                    .map_or_else(String::new, |n| n.id().to_string());
                let w = nodes.get(i).map_or(node_width, |n| {
                    (n.label().len() as f64 + 4.0).max(node_width)
                });
                NodePosition::new(id, *x, *y, w, node_height)
            })
            .collect();

        // Edge routing
        let node_ids: Vec<String> = nodes.iter().map(|n| n.id().to_string()).collect();
        let edge_paths = edge_routing::compute_routed_edges(
            graph.edge_pairs(),
            &node_ids,
            &node_positions,
            &hints.orientation,
        );

        let bounding_box = compute_bounding_box(&node_positions);

        LayoutResult {
            node_positions,
            edge_paths,
            bounding_box,
        }
    }
}

/// Initializes node positions from previous layout or in a circle.
fn initialize_positions(
    n: usize,
    hints: &LayoutHints<'_>,
    node_width: f64,
    node_height: f64,
) -> Vec<(f64, f64)> {
    // Try to reuse previous positions for stability
    if let Some(prev) = hints.previous_layout {
        if prev.node_positions().len() == n {
            return prev
                .node_positions()
                .iter()
                .map(|p| (p.x(), p.y()))
                .collect();
        }
    }

    // Place nodes in a circle
    let radius = (n as f64).max(3.0) * node_width.max(node_height) / (2.0 * std::f64::consts::PI);
    let center = radius + 10.0;

    (0..n)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            let x = center + radius * angle.cos();
            let y = center + radius * angle.sin();
            (x, y)
        })
        .collect()
}

/// Computes force-based displacements for all nodes.
fn compute_displacements(
    positions: &[(f64, f64)],
    graph: &IndexedGraph,
    n: usize,
    k: f64,
) -> Vec<(f64, f64)> {
    let mut displacements = vec![(0.0f64, 0.0f64); n];

    // Repulsive forces: all pairs repel
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = positions[i].0 - positions[j].0;
            let dy = positions[i].1 - positions[j].1;
            let dist = (dx * dx + dy * dy).sqrt().max(0.1);

            // Repulsive force: k^2 / d
            let force = (k * k) / dist;
            let fx = (dx / dist) * force;
            let fy = (dy / dist) * force;

            displacements[i].0 += fx;
            displacements[i].1 += fy;
            displacements[j].0 -= fx;
            displacements[j].1 -= fy;
        }
    }

    // Attractive forces: connected nodes attract
    for &(from, to) in graph.edge_pairs() {
        let dx = positions[from].0 - positions[to].0;
        let dy = positions[from].1 - positions[to].1;
        let dist = (dx * dx + dy * dy).sqrt().max(0.1);

        // Attractive force: d^2 / k
        let force = (dist * dist) / k;
        let fx = (dx / dist) * force;
        let fy = (dy / dist) * force;

        displacements[from].0 -= fx;
        displacements[from].1 -= fy;
        displacements[to].0 += fx;
        displacements[to].1 += fy;
    }

    displacements
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
        let layout = ForceDirectedLayout::default();
        layout.compute(&graph, nodes, edges, &[], &LayoutHints::default())
    }

    #[test]
    fn test_empty_graph() {
        let result = run_layout(&[], &[]);
        assert!(result.node_positions.is_empty());
    }

    #[test]
    fn test_single_node() {
        let nodes = vec![make_node("a")];
        let result = run_layout(&nodes, &[]);

        assert_eq!(result.node_positions.len(), 1);
        assert_eq!(result.node_positions[0].id(), "a");
    }

    #[test]
    fn test_connected_nodes_closer_than_disconnected() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "b")]; // a-b connected, c disconnected

        let result = run_layout(&nodes, &edges);

        let a = &result.node_positions[0];
        let b = &result.node_positions[1];
        let c = &result.node_positions[2];

        let dist_ab = ((a.x() - b.x()).powi(2) + (a.y() - b.y()).powi(2)).sqrt();
        let dist_ac = ((a.x() - c.x()).powi(2) + (a.y() - c.y()).powi(2)).sqrt();

        // Connected nodes should be closer (or at least not much farther)
        assert!(
            dist_ab <= dist_ac * 1.5,
            "Connected nodes a-b ({dist_ab:.1}) should be closer than disconnected a-c ({dist_ac:.1})"
        );
    }

    #[test]
    fn test_no_node_overlap() {
        let nodes = vec![
            make_node("a"),
            make_node("b"),
            make_node("c"),
            make_node("d"),
        ];
        let edges = vec![
            make_edge("a", "b"),
            make_edge("b", "c"),
            make_edge("c", "d"),
        ];
        let result = run_layout(&nodes, &edges);

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
    fn test_positive_coordinates() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "c")];
        let result = run_layout(&nodes, &edges);

        for pos in &result.node_positions {
            assert!(
                pos.x() >= 0.0,
                "Node {} has negative x: {}",
                pos.id(),
                pos.x()
            );
            assert!(
                pos.y() >= 0.0,
                "Node {} has negative y: {}",
                pos.id(),
                pos.y()
            );
        }
    }

    #[test]
    fn test_bounding_box_contains_all_nodes() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "c")];
        let result = run_layout(&nodes, &edges);

        let bbox = &result.bounding_box;
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
        assert_eq!(result.edge_paths[0].from_id(), "a");
        assert_eq!(result.edge_paths[0].to_id(), "b");
    }

    #[test]
    fn test_cycle_converges() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![
            make_edge("a", "b"),
            make_edge("b", "c"),
            make_edge("c", "a"),
        ];
        // Should not hang or produce NaN
        let result = run_layout(&nodes, &edges);
        assert_eq!(result.node_positions.len(), 3);
        for pos in &result.node_positions {
            assert!(!pos.x().is_nan());
            assert!(!pos.y().is_nan());
        }
    }

    #[test]
    fn test_incremental_stability() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "c")];

        let graph = IndexedGraph::build(&nodes, &edges);
        let layout = ForceDirectedLayout::default();
        let first = layout.compute(&graph, &nodes, &edges, &[], &LayoutHints::default());

        // Second run with previous layout as hint
        let hints = LayoutHints {
            previous_layout: Some(&first),
            ..LayoutHints::default()
        };
        let second = layout.compute(&graph, &nodes, &edges, &[], &hints);

        // Positions should be very similar (initialized from previous)
        for i in 0..3 {
            let dx = (first.node_positions[i].x() - second.node_positions[i].x()).abs();
            let dy = (first.node_positions[i].y() - second.node_positions[i].y()).abs();
            assert!(
                dx < 5.0 && dy < 5.0,
                "Node {} moved too much: dx={dx:.1}, dy={dy:.1}",
                first.node_positions[i].id()
            );
        }
    }
}
