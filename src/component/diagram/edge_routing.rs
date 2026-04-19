//! Edge routing utilities for the Diagram component.
//!
//! Provides functions for computing corner characters at path bends,
//! self-loop paths, and edge nudging to separate overlapping parallel edges.

use super::layout::{EdgePath, NodePosition, PathSegment};
use super::types::Orientation;

/// Computes edge paths with proper routing between node positions.
///
/// Generates S-shaped orthogonal routes (horizontal-vertical-horizontal
/// for left-to-right, vertical-horizontal-vertical for top-to-bottom)
/// with midpoint routing to avoid direct overlap.
pub(crate) fn compute_routed_edges(
    edge_pairs: &[(usize, usize)],
    node_ids: &[String],
    positions: &[NodePosition],
    orientation: &Orientation,
) -> Vec<EdgePath> {
    let mut paths = Vec::with_capacity(edge_pairs.len());

    for &(from_idx, to_idx) in edge_pairs {
        let Some(from_pos) = positions.get(from_idx) else {
            continue;
        };
        let Some(to_pos) = positions.get(to_idx) else {
            continue;
        };
        let from_id = node_ids.get(from_idx).cloned().unwrap_or_default();
        let to_id = node_ids.get(to_idx).cloned().unwrap_or_default();

        let segments = if from_idx == to_idx {
            compute_self_loop(from_pos, orientation)
        } else {
            compute_orthogonal_route(from_pos, to_pos, orientation)
        };

        paths.push(EdgePath::new(from_id, to_id, segments));
    }

    // Nudge overlapping parallel edges apart
    nudge_parallel_edges(&mut paths);

    paths
}

/// Computes an orthogonal S-shaped route between two nodes.
fn compute_orthogonal_route(
    from: &NodePosition,
    to: &NodePosition,
    orientation: &Orientation,
) -> Vec<PathSegment> {
    match orientation {
        Orientation::LeftToRight => {
            let start_x = from.x() + from.width();
            let start_y = from.center_y();
            let end_x = to.x();
            let end_y = to.center_y();

            if (start_y - end_y).abs() < 0.5 {
                // Same row — straight horizontal line
                vec![
                    PathSegment::MoveTo(start_x, start_y),
                    PathSegment::LineTo(end_x, end_y),
                ]
            } else {
                // S-shaped route: horizontal, vertical, horizontal
                let mid_x = (start_x + end_x) / 2.0;
                vec![
                    PathSegment::MoveTo(start_x, start_y),
                    PathSegment::LineTo(mid_x, start_y),
                    PathSegment::LineTo(mid_x, end_y),
                    PathSegment::LineTo(end_x, end_y),
                ]
            }
        }
        Orientation::TopToBottom => {
            let start_x = from.center_x();
            let start_y = from.y() + from.height();
            let end_x = to.center_x();
            let end_y = to.y();

            if (start_x - end_x).abs() < 0.5 {
                // Same column — straight vertical line
                vec![
                    PathSegment::MoveTo(start_x, start_y),
                    PathSegment::LineTo(end_x, end_y),
                ]
            } else {
                // S-shaped route: vertical, horizontal, vertical
                let mid_y = (start_y + end_y) / 2.0;
                vec![
                    PathSegment::MoveTo(start_x, start_y),
                    PathSegment::LineTo(start_x, mid_y),
                    PathSegment::LineTo(end_x, mid_y),
                    PathSegment::LineTo(end_x, end_y),
                ]
            }
        }
    }
}

/// Computes a self-loop path that exits and re-enters the same node.
///
/// For left-to-right: exits right, goes up, goes left, comes back down.
/// For top-to-bottom: exits bottom, goes right, goes up, comes back left.
fn compute_self_loop(node: &NodePosition, orientation: &Orientation) -> Vec<PathSegment> {
    let loop_offset = 2.0;

    match orientation {
        Orientation::LeftToRight => {
            let exit_x = node.x() + node.width();
            let exit_y = node.center_y() - 0.5;
            let peak_x = exit_x + loop_offset;
            let peak_y = node.y() - loop_offset;

            vec![
                PathSegment::MoveTo(exit_x, exit_y),
                PathSegment::LineTo(peak_x, exit_y),
                PathSegment::LineTo(peak_x, peak_y),
                PathSegment::LineTo(exit_x - node.width() / 2.0, peak_y),
                PathSegment::LineTo(exit_x - node.width() / 2.0, node.y()),
            ]
        }
        Orientation::TopToBottom => {
            let exit_x = node.center_x() + 0.5;
            let exit_y = node.y() + node.height();
            let peak_y = exit_y + loop_offset;
            let peak_x = node.x() + node.width() + loop_offset;

            vec![
                PathSegment::MoveTo(exit_x, exit_y),
                PathSegment::LineTo(exit_x, peak_y),
                PathSegment::LineTo(peak_x, peak_y),
                PathSegment::LineTo(peak_x, node.center_y()),
                PathSegment::LineTo(node.x() + node.width(), node.center_y()),
            ]
        }
    }
}

/// Nudges parallel edges (same from/to pair or reverse) apart so they
/// don't overlap visually. Offsets the midpoint by ±0.5 units.
fn nudge_parallel_edges(paths: &mut [EdgePath]) {
    // Group edges by sorted (from, to) pair to find parallels
    let len = paths.len();
    for i in 0..len {
        for j in (i + 1)..len {
            let same_pair = (paths[i].from_id() == paths[j].from_id()
                && paths[i].to_id() == paths[j].to_id())
                || (paths[i].from_id() == paths[j].to_id()
                    && paths[i].to_id() == paths[j].from_id());

            if same_pair {
                // Offset the midpoints in opposite directions
                offset_midpoints(&mut paths[i], 0.5);
                offset_midpoints(&mut paths[j], -0.5);
            }
        }
    }
}

/// Offsets the middle segments of a path by the given amount
/// perpendicular to the segment direction.
fn offset_midpoints(path: &mut EdgePath, offset: f64) {
    let segments = path.segments_mut();
    let len = segments.len();
    if len < 3 {
        return;
    }

    // Offset the middle waypoints (not start or end)
    for seg in segments[1..len - 1].iter_mut() {
        match seg {
            PathSegment::LineTo(_, y) | PathSegment::MoveTo(_, y) => {
                *y += offset;
            }
        }
    }
}

/// Returns the appropriate corner character for a bend in the path.
///
/// Given the direction entering and leaving a corner point, returns
/// the correct box-drawing corner character.
pub(crate) fn corner_char(
    from_x: f64,
    from_y: f64,
    corner_x: f64,
    corner_y: f64,
    to_x: f64,
    to_y: f64,
) -> &'static str {
    let entering_horizontal = (from_y - corner_y).abs() < 0.5;
    let leaving_horizontal = (corner_y - to_y).abs() < 0.5;

    if entering_horizontal && !leaving_horizontal {
        // Entering horizontally, leaving vertically
        let from_left = from_x < corner_x;
        let going_down = to_y > corner_y;

        match (from_left, going_down) {
            (true, true) => "\u{2510}",   // ┐ (from left, going down)
            (true, false) => "\u{2518}",  // ┘ (from left, going up)
            (false, true) => "\u{250c}",  // ┌ (from right, going down)
            (false, false) => "\u{2514}", // └ (from right, going up)
        }
    } else if !entering_horizontal && leaving_horizontal {
        // Entering vertically, leaving horizontally
        let from_above = from_y < corner_y;
        let going_right = to_x > corner_x;

        match (from_above, going_right) {
            (true, true) => "\u{2514}",   // └ (from above, going right)
            (true, false) => "\u{2518}",  // ┘ (from above, going left)
            (false, true) => "\u{250c}",  // ┌ (from below, going right)
            (false, false) => "\u{2510}", // ┐ (from below, going left)
        }
    } else {
        // Shouldn't happen with orthogonal routing, but fallback
        "\u{253c}" // ┼
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(id: &str, x: f64, y: f64) -> NodePosition {
        NodePosition::new(id.to_string(), x, y, 12.0, 3.0)
    }

    #[test]
    fn test_straight_horizontal_route() {
        let positions = vec![pos("a", 0.0, 5.0), pos("b", 20.0, 5.0)];
        let ids: Vec<String> = vec!["a".into(), "b".into()];
        let paths = compute_routed_edges(&[(0, 1)], &ids, &positions, &Orientation::LeftToRight);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].from_id(), "a");
        assert_eq!(paths[0].to_id(), "b");
        // Same row — should be 2 segments (MoveTo + LineTo)
        assert_eq!(paths[0].segments().len(), 2);
    }

    #[test]
    fn test_s_shaped_route_ltr() {
        let positions = vec![pos("a", 0.0, 0.0), pos("b", 30.0, 10.0)];
        let ids: Vec<String> = vec!["a".into(), "b".into()];
        let paths = compute_routed_edges(&[(0, 1)], &ids, &positions, &Orientation::LeftToRight);

        assert_eq!(paths.len(), 1);
        // Different rows — should be 4 segments (S-shape)
        assert_eq!(paths[0].segments().len(), 4);
    }

    #[test]
    fn test_s_shaped_route_ttb() {
        let positions = vec![pos("a", 5.0, 0.0), pos("b", 20.0, 15.0)];
        let ids: Vec<String> = vec!["a".into(), "b".into()];
        let paths = compute_routed_edges(&[(0, 1)], &ids, &positions, &Orientation::TopToBottom);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].segments().len(), 4);
    }

    #[test]
    fn test_self_loop() {
        let positions = vec![pos("a", 10.0, 10.0)];
        let ids: Vec<String> = vec!["a".into()];
        let paths = compute_routed_edges(&[(0, 0)], &ids, &positions, &Orientation::LeftToRight);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].from_id(), "a");
        assert_eq!(paths[0].to_id(), "a");
        assert!(paths[0].segments().len() >= 4); // Self-loops have multiple bends
    }

    #[test]
    fn test_parallel_edges_nudged() {
        let positions = vec![pos("a", 0.0, 5.0), pos("b", 30.0, 5.0)];
        let ids: Vec<String> = vec!["a".into(), "b".into()];
        let paths = compute_routed_edges(
            &[(0, 1), (1, 0)],
            &ids,
            &positions,
            &Orientation::LeftToRight,
        );

        assert_eq!(paths.len(), 2);
        // The paths should be different (nudged apart)
        assert_ne!(paths[0].segments(), paths[1].segments());
    }

    #[test]
    fn test_corner_chars() {
        // From left, going down → ┐
        assert_eq!(corner_char(0.0, 5.0, 10.0, 5.0, 10.0, 15.0), "\u{2510}");
        // From left, going up → ┘
        assert_eq!(corner_char(0.0, 5.0, 10.0, 5.0, 10.0, 0.0), "\u{2518}");
        // From right, going down → ┌
        assert_eq!(corner_char(20.0, 5.0, 10.0, 5.0, 10.0, 15.0), "\u{250c}");
        // From right, going up → └
        assert_eq!(corner_char(20.0, 5.0, 10.0, 5.0, 10.0, 0.0), "\u{2514}");
    }

    #[test]
    fn test_empty_edges() {
        let paths = compute_routed_edges(&[], &[], &[], &Orientation::LeftToRight);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_straight_vertical_route_ttb() {
        let positions = vec![pos("a", 5.0, 0.0), pos("b", 5.0, 15.0)];
        let ids: Vec<String> = vec!["a".into(), "b".into()];
        let paths = compute_routed_edges(&[(0, 1)], &ids, &positions, &Orientation::TopToBottom);

        assert_eq!(paths.len(), 1);
        // Same column — should be 2 segments (straight line)
        assert_eq!(paths[0].segments().len(), 2);
    }
}
