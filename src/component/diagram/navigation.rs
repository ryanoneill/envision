//! Spatial navigation for the Diagram component.
//!
//! Provides directional node selection (arrow keys select the nearest
//! node in the pressed direction) and edge following with multi-target
//! support.

use super::graph::IndexedGraph;
use super::layout::NodePosition;

/// Cardinal direction for spatial navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Finds the nearest node in the given direction from the current node.
///
/// Uses a 45-degree cone heuristic: a node is "to the right" if
/// `dx > 0` and `|dx| > |dy|`. Among qualifying candidates, returns
/// the one with the smallest squared distance.
///
/// Returns `None` if no node exists in that direction.
pub(crate) fn find_nearest_in_direction(
    current_idx: usize,
    positions: &[NodePosition],
    direction: Direction,
) -> Option<usize> {
    let current = positions.get(current_idx)?;
    let cx = current.center_x();
    let cy = current.center_y();

    positions
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != current_idx)
        .filter(|(_, pos)| is_in_direction(cx, cy, pos.center_x(), pos.center_y(), direction))
        .min_by(|(_, a), (_, b)| {
            let dist_a = squared_distance(cx, cy, a.center_x(), a.center_y());
            let dist_b = squared_distance(cx, cy, b.center_x(), b.center_y());
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| idx)
}

/// Returns true if (tx, ty) is in the given direction from (cx, cy).
///
/// The direction test uses a 45-degree cone: for `Right`, the target
/// must have `dx > 0` and `|dx| >= |dy|`.
fn is_in_direction(cx: f64, cy: f64, tx: f64, ty: f64, direction: Direction) -> bool {
    let dx = tx - cx;
    let dy = ty - cy;

    match direction {
        Direction::Right => dx > 0.0 && dx.abs() >= dy.abs(),
        Direction::Left => dx < 0.0 && dx.abs() >= dy.abs(),
        Direction::Down => dy > 0.0 && dy.abs() >= dx.abs(),
        Direction::Up => dy < 0.0 && dy.abs() >= dx.abs(),
    }
}

fn squared_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    (x2 - x1).powi(2) + (y2 - y1).powi(2)
}

/// Finds the outgoing edge targets from a given node.
///
/// Returns a list of (target_index, target_id) pairs.
pub(crate) fn outgoing_targets(node_idx: usize, graph: &IndexedGraph) -> Vec<usize> {
    graph.successors(node_idx).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(id: &str, x: f64, y: f64) -> NodePosition {
        NodePosition::new(id.to_string(), x, y, 12.0, 3.0)
    }

    #[test]
    fn test_find_nearest_right() {
        let positions = vec![pos("a", 0.0, 5.0), pos("b", 20.0, 5.0), pos("c", 40.0, 5.0)];
        let result = find_nearest_in_direction(0, &positions, Direction::Right);
        assert_eq!(result, Some(1)); // b is nearest to the right of a
    }

    #[test]
    fn test_find_nearest_left() {
        let positions = vec![pos("a", 0.0, 5.0), pos("b", 20.0, 5.0), pos("c", 40.0, 5.0)];
        let result = find_nearest_in_direction(2, &positions, Direction::Left);
        assert_eq!(result, Some(1)); // b is nearest to the left of c
    }

    #[test]
    fn test_find_nearest_down() {
        let positions = vec![pos("a", 5.0, 0.0), pos("b", 5.0, 10.0), pos("c", 5.0, 20.0)];
        let result = find_nearest_in_direction(0, &positions, Direction::Down);
        assert_eq!(result, Some(1)); // b is below a
    }

    #[test]
    fn test_find_nearest_up() {
        let positions = vec![pos("a", 5.0, 0.0), pos("b", 5.0, 10.0), pos("c", 5.0, 20.0)];
        let result = find_nearest_in_direction(2, &positions, Direction::Up);
        assert_eq!(result, Some(1)); // b is above c
    }

    #[test]
    fn test_no_node_in_direction() {
        let positions = vec![pos("a", 0.0, 5.0), pos("b", 20.0, 5.0)];
        // Nothing to the left of a
        let result = find_nearest_in_direction(0, &positions, Direction::Left);
        assert_eq!(result, None);
    }

    #[test]
    fn test_diagonal_preference() {
        // Node c is diagonally placed — the 45° cone should pick the right direction
        let positions = vec![
            pos("a", 0.0, 0.0),
            pos("b", 30.0, 5.0), // mostly right, slightly down
            pos("c", 5.0, 30.0), // mostly down, slightly right
        ];
        // From a, looking right: b qualifies (dx=30 > dy=5), c does not (dy=30 > dx=5)
        let result = find_nearest_in_direction(0, &positions, Direction::Right);
        assert_eq!(result, Some(1));

        // From a, looking down: c qualifies (dy=30 > dx=5), b does not (dx=30 > dy=5)
        let result = find_nearest_in_direction(0, &positions, Direction::Down);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_single_node() {
        let positions = vec![pos("a", 0.0, 0.0)];
        let result = find_nearest_in_direction(0, &positions, Direction::Right);
        assert_eq!(result, None);
    }

    #[test]
    fn test_picks_closest() {
        let positions = vec![
            pos("a", 0.0, 5.0),
            pos("b", 20.0, 5.0), // closer
            pos("c", 50.0, 5.0), // farther
        ];
        let result = find_nearest_in_direction(0, &positions, Direction::Right);
        assert_eq!(result, Some(1)); // b is closer than c
    }

    #[test]
    fn test_outgoing_targets() {
        use super::super::types::{DiagramEdge, DiagramNode};

        let nodes = vec![
            DiagramNode::new("a", "A"),
            DiagramNode::new("b", "B"),
            DiagramNode::new("c", "C"),
        ];
        let edges = vec![DiagramEdge::new("a", "b"), DiagramEdge::new("a", "c")];
        let graph = IndexedGraph::build(&nodes, &edges);

        let targets = outgoing_targets(0, &graph);
        assert_eq!(targets, vec![1, 2]);
    }

    #[test]
    fn test_outgoing_targets_none() {
        use super::super::types::DiagramNode;

        let nodes = vec![DiagramNode::new("a", "A")];
        let graph = IndexedGraph::build(&nodes, &[]);

        let targets = outgoing_targets(0, &graph);
        assert!(targets.is_empty());
    }
}
