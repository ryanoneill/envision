//! Internal indexed graph representation for efficient traversal.
//!
//! Converts the user-facing `Vec<DiagramNode>` + `Vec<DiagramEdge>` into
//! an adjacency-list structure with O(1) node lookup by ID and O(degree)
//! neighbor traversal.

use std::collections::HashMap;

use super::types::{DiagramEdge, DiagramNode};

/// Index-based graph for layout and navigation algorithms.
///
/// String node IDs are mapped to `usize` indices. The graph is rebuilt
/// from scratch whenever the node or edge data changes — this is fast
/// (O(V + E)) and avoids incremental consistency bugs.
#[derive(Clone, Debug)]
#[allow(dead_code)] // Fields/methods used across phases 4-8 (navigation, search)
pub(crate) struct IndexedGraph {
    node_to_index: HashMap<String, usize>,
    index_to_node: Vec<String>,
    adjacency_out: Vec<Vec<usize>>,
    adjacency_in: Vec<Vec<usize>>,
    edge_pairs: Vec<(usize, usize)>,
}

#[allow(dead_code)] // Methods used in phases 4-8 (navigation, search)
impl IndexedGraph {
    /// Builds the indexed graph from nodes and edges.
    ///
    /// Edges referencing unknown node IDs are silently skipped.
    pub(crate) fn build(nodes: &[DiagramNode], edges: &[DiagramEdge]) -> Self {
        let mut node_to_index = HashMap::with_capacity(nodes.len());
        let mut index_to_node = Vec::with_capacity(nodes.len());

        for (i, node) in nodes.iter().enumerate() {
            node_to_index.insert(node.id().to_string(), i);
            index_to_node.push(node.id().to_string());
        }

        let n = nodes.len();
        let mut adjacency_out = vec![Vec::new(); n];
        let mut adjacency_in = vec![Vec::new(); n];
        let mut edge_pairs = Vec::with_capacity(edges.len());

        for edge in edges {
            if let (Some(&from_idx), Some(&to_idx)) =
                (node_to_index.get(edge.from()), node_to_index.get(edge.to()))
            {
                adjacency_out[from_idx].push(to_idx);
                adjacency_in[to_idx].push(from_idx);
                edge_pairs.push((from_idx, to_idx));
            }
        }

        Self {
            node_to_index,
            index_to_node,
            adjacency_out,
            adjacency_in,
            edge_pairs,
        }
    }

    /// Returns the number of nodes.
    pub(crate) fn node_count(&self) -> usize {
        self.index_to_node.len()
    }

    /// Returns the index for a node ID, if it exists.
    pub(crate) fn index_of(&self, id: &str) -> Option<usize> {
        self.node_to_index.get(id).copied()
    }

    /// Returns the node ID for an index.
    pub(crate) fn id_of(&self, index: usize) -> Option<&str> {
        self.index_to_node.get(index).map(|s| s.as_str())
    }

    /// Returns indices of nodes reachable via outgoing edges from `index`.
    pub(crate) fn successors(&self, index: usize) -> &[usize] {
        self.adjacency_out.get(index).map_or(&[], |v| v.as_slice())
    }

    /// Returns indices of nodes with edges pointing to `index`.
    pub(crate) fn predecessors(&self, index: usize) -> &[usize] {
        self.adjacency_in.get(index).map_or(&[], |v| v.as_slice())
    }

    /// Returns all (from, to) edge pairs as index tuples.
    pub(crate) fn edge_pairs(&self) -> &[(usize, usize)] {
        &self.edge_pairs
    }

    /// Returns root nodes (no incoming edges).
    pub(crate) fn roots(&self) -> Vec<usize> {
        (0..self.node_count())
            .filter(|&i| self.adjacency_in[i].is_empty())
            .collect()
    }

    /// Returns true if there is an edge from `from` to `to`.
    pub(crate) fn has_edge(&self, from: usize, to: usize) -> bool {
        self.adjacency_out
            .get(from)
            .is_some_and(|out| out.contains(&to))
    }

    /// Returns all neighbors (both directions) of a node.
    pub(crate) fn neighbors(&self, index: usize) -> Vec<usize> {
        let mut result = Vec::new();
        if let Some(out) = self.adjacency_out.get(index) {
            result.extend(out);
        }
        if let Some(inc) = self.adjacency_in.get(index) {
            for &pred in inc {
                if !result.contains(&pred) {
                    result.push(pred);
                }
            }
        }
        result
    }
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

    #[test]
    fn test_empty_graph() {
        let g = IndexedGraph::build(&[], &[]);
        assert_eq!(g.node_count(), 0);
        assert!(g.roots().is_empty());
        assert!(g.edge_pairs().is_empty());
    }

    #[test]
    fn test_single_node() {
        let nodes = vec![make_node("a")];
        let g = IndexedGraph::build(&nodes, &[]);
        assert_eq!(g.node_count(), 1);
        assert_eq!(g.index_of("a"), Some(0));
        assert_eq!(g.id_of(0), Some("a"));
        assert_eq!(g.roots(), vec![0]);
        assert!(g.successors(0).is_empty());
        assert!(g.predecessors(0).is_empty());
    }

    #[test]
    fn test_linear_graph() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "c")];
        let g = IndexedGraph::build(&nodes, &edges);

        assert_eq!(g.node_count(), 3);
        assert_eq!(g.roots(), vec![0]); // only "a"
        assert_eq!(g.successors(0), &[1]);
        assert_eq!(g.successors(1), &[2]);
        assert!(g.successors(2).is_empty());
        assert_eq!(g.predecessors(2), &[1]);
        assert!(g.has_edge(0, 1));
        assert!(!g.has_edge(1, 0));
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
        let g = IndexedGraph::build(&nodes, &edges);

        assert_eq!(g.roots(), vec![0]);
        assert_eq!(g.successors(0), &[1, 2]);
        assert_eq!(g.predecessors(3), &[1, 2]); // d has two predecessors
    }

    #[test]
    fn test_unknown_edge_skipped() {
        let nodes = vec![make_node("a")];
        let edges = vec![make_edge("a", "nonexistent")];
        let g = IndexedGraph::build(&nodes, &edges);

        assert!(g.edge_pairs().is_empty());
        assert!(g.successors(0).is_empty());
    }

    #[test]
    fn test_cycle() {
        let nodes = vec![make_node("a"), make_node("b")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "a")];
        let g = IndexedGraph::build(&nodes, &edges);

        assert!(g.roots().is_empty()); // no roots in a cycle
        assert_eq!(g.successors(0), &[1]);
        assert_eq!(g.successors(1), &[0]);
        assert!(g.has_edge(0, 1));
        assert!(g.has_edge(1, 0));
    }

    #[test]
    fn test_neighbors() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "b"), make_edge("c", "a")];
        let g = IndexedGraph::build(&nodes, &edges);

        let mut neighbors = g.neighbors(0);
        neighbors.sort();
        assert_eq!(neighbors, vec![1, 2]); // b (outgoing) and c (incoming)
    }

    #[test]
    fn test_self_loop() {
        let nodes = vec![make_node("a")];
        let edges = vec![make_edge("a", "a")];
        let g = IndexedGraph::build(&nodes, &edges);

        assert_eq!(g.successors(0), &[0]);
        assert_eq!(g.predecessors(0), &[0]);
        assert!(g.has_edge(0, 0));
    }

    #[test]
    fn test_multiple_roots() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let edges = vec![make_edge("a", "c"), make_edge("b", "c")];
        let g = IndexedGraph::build(&nodes, &edges);

        let mut roots = g.roots();
        roots.sort();
        assert_eq!(roots, vec![0, 1]);
    }
}
