//! Layout algorithm for the Treemap component.
//!
//! Implements the squarified treemap algorithm (Bruls, Huizing, van Wijk)
//! which produces rectangles with aspect ratios close to 1 (squares),
//! making them easier to read than thin strips.

use ratatui::prelude::*;

use super::TreemapNode;

/// A positioned rectangle in the treemap layout.
///
/// Each `LayoutRect` represents a single node in the treemap with its
/// computed position, size, and associated metadata.
///
/// # Example
///
/// ```rust
/// use envision::component::treemap::LayoutRect;
/// use ratatui::style::Color;
///
/// let rect = LayoutRect {
///     x: 0,
///     y: 0,
///     width: 20,
///     height: 10,
///     label: "root".to_string(),
///     value: 100.0,
///     color: Color::Blue,
///     depth: 0,
///     node_index: vec![0],
/// };
/// assert_eq!(rect.label, "root");
/// assert_eq!(rect.depth, 0);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct LayoutRect {
    /// X position (column).
    pub x: u16,
    /// Y position (row).
    pub y: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
    /// Display label for this node.
    pub label: String,
    /// Value of this node.
    pub value: f64,
    /// Color for this node.
    pub color: Color,
    /// Depth in the tree (0 = root children).
    pub depth: usize,
    /// Index path from the root to this node.
    pub node_index: Vec<usize>,
}

/// Compute the squarified treemap layout for the given nodes within an area.
///
/// Returns a list of [`LayoutRect`] entries, one per visible leaf node.
/// The algorithm sorts children by value descending and uses the squarified
/// approach to produce near-square rectangles.
///
/// # Arguments
///
/// * `nodes` - The nodes to lay out (typically children of the current view).
/// * `area` - The rectangular area to fill.
/// * `depth` - The current depth level (for coloring/labeling).
/// * `index_prefix` - The index path prefix for node identification.
///
/// # Example
///
/// ```rust
/// use envision::component::treemap::{TreemapNode, squarified_layout};
/// use ratatui::prelude::*;
/// use ratatui::style::Color;
///
/// let nodes = vec![
///     TreemapNode::new("A", 60.0).with_color(Color::Red),
///     TreemapNode::new("B", 30.0).with_color(Color::Green),
///     TreemapNode::new("C", 10.0).with_color(Color::Blue),
/// ];
/// let area = Rect::new(0, 0, 20, 10);
/// let rects = squarified_layout(&nodes, area, 0, &[]);
/// assert_eq!(rects.len(), 3);
/// ```
pub fn squarified_layout(
    nodes: &[TreemapNode],
    area: Rect,
    depth: usize,
    index_prefix: &[usize],
) -> Vec<LayoutRect> {
    if nodes.is_empty() || area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    let total_value: f64 = nodes.iter().map(|n| n.total_value()).sum();
    if total_value <= 0.0 {
        return Vec::new();
    }

    // Build sorted indices by value descending.
    let mut sorted_indices: Vec<usize> = (0..nodes.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        nodes[b]
            .total_value()
            .partial_cmp(&nodes[a].total_value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut result = Vec::new();
    let mut remaining_area = area;
    let mut remaining_value = total_value;
    let mut idx = 0;

    while idx < sorted_indices.len() {
        if remaining_area.width == 0 || remaining_area.height == 0 {
            break;
        }

        let shorter_side = remaining_area.width.min(remaining_area.height) as f64;

        // Determine the row: keep adding items while the worst aspect ratio improves.
        let mut row = vec![sorted_indices[idx]];
        let mut row_value = nodes[sorted_indices[idx]].total_value();
        idx += 1;

        let mut current_worst = worst_aspect_ratio(
            &row,
            nodes,
            row_value,
            shorter_side,
            remaining_value,
            remaining_area,
        );

        while idx < sorted_indices.len() {
            let candidate = sorted_indices[idx];
            let candidate_value = row_value + nodes[candidate].total_value();
            let mut candidate_row = row.clone();
            candidate_row.push(candidate);

            let new_worst = worst_aspect_ratio(
                &candidate_row,
                nodes,
                candidate_value,
                shorter_side,
                remaining_value,
                remaining_area,
            );

            if new_worst <= current_worst {
                row = candidate_row;
                row_value = candidate_value;
                current_worst = new_worst;
                idx += 1;
            } else {
                break;
            }
        }

        // Lay out the row.
        let (row_rects, new_remaining) = layout_row(
            &row,
            nodes,
            row_value,
            remaining_value,
            remaining_area,
            depth,
            index_prefix,
        );

        result.extend(row_rects);
        remaining_value -= row_value;
        remaining_area = new_remaining;
    }

    result
}

/// Compute the worst aspect ratio for a row of nodes.
fn worst_aspect_ratio(
    row: &[usize],
    nodes: &[TreemapNode],
    row_value: f64,
    shorter_side: f64,
    total_remaining: f64,
    area: Rect,
) -> f64 {
    if row.is_empty() || total_remaining <= 0.0 || shorter_side <= 0.0 {
        return f64::MAX;
    }

    let total_area = area.width as f64 * area.height as f64;
    let row_area = total_area * (row_value / total_remaining);

    // The row occupies a strip along the shorter side.
    let row_length = row_area / shorter_side;
    if row_length <= 0.0 {
        return f64::MAX;
    }

    let mut worst = 0.0_f64;
    for &node_idx in row {
        let node_value = nodes[node_idx].total_value();
        if node_value <= 0.0 {
            continue;
        }
        let node_area = total_area * (node_value / total_remaining);
        let node_side = node_area / row_length;
        let aspect = if row_length > node_side {
            row_length / node_side
        } else {
            node_side / row_length
        };
        worst = worst.max(aspect);
    }

    worst
}

/// Lay out a single row of nodes within the remaining area.
///
/// Returns the laid out rectangles and the remaining area after the row.
fn layout_row(
    row: &[usize],
    nodes: &[TreemapNode],
    row_value: f64,
    total_remaining: f64,
    area: Rect,
    depth: usize,
    index_prefix: &[usize],
) -> (Vec<LayoutRect>, Rect) {
    if row.is_empty() || total_remaining <= 0.0 {
        return (Vec::new(), area);
    }

    let row_fraction = row_value / total_remaining;
    let horizontal = area.width >= area.height;

    let mut rects = Vec::new();

    if horizontal {
        // Row is a vertical strip on the left side.
        let row_width = (area.width as f64 * row_fraction).round().max(1.0) as u16;
        let row_width = row_width.min(area.width);

        let mut y_offset = area.y;
        let bottom = area.y + area.height;

        for (i, &node_idx) in row.iter().enumerate() {
            let node = &nodes[node_idx];
            let node_fraction = if row_value > 0.0 {
                node.total_value() / row_value
            } else {
                1.0 / row.len() as f64
            };

            let node_height = if i == row.len() - 1 {
                // Last node takes remaining space to avoid rounding gaps.
                bottom.saturating_sub(y_offset)
            } else {
                let h = (area.height as f64 * node_fraction).round().max(1.0) as u16;
                h.min(bottom.saturating_sub(y_offset))
            };

            if node_height == 0 {
                continue;
            }

            let mut path = index_prefix.to_vec();
            path.push(node_idx);

            rects.push(LayoutRect {
                x: area.x,
                y: y_offset,
                width: row_width,
                height: node_height,
                label: node.label.clone(),
                value: node.total_value(),
                color: node.color,
                depth,
                node_index: path,
            });

            y_offset += node_height;
        }

        let new_area = Rect::new(
            area.x + row_width,
            area.y,
            area.width.saturating_sub(row_width),
            area.height,
        );
        (rects, new_area)
    } else {
        // Row is a horizontal strip on the top.
        let row_height = (area.height as f64 * row_fraction).round().max(1.0) as u16;
        let row_height = row_height.min(area.height);

        let mut x_offset = area.x;
        let right = area.x + area.width;

        for (i, &node_idx) in row.iter().enumerate() {
            let node = &nodes[node_idx];
            let node_fraction = if row_value > 0.0 {
                node.total_value() / row_value
            } else {
                1.0 / row.len() as f64
            };

            let node_width = if i == row.len() - 1 {
                right.saturating_sub(x_offset)
            } else {
                let w = (area.width as f64 * node_fraction).round().max(1.0) as u16;
                w.min(right.saturating_sub(x_offset))
            };

            if node_width == 0 {
                continue;
            }

            let mut path = index_prefix.to_vec();
            path.push(node_idx);

            rects.push(LayoutRect {
                x: x_offset,
                y: area.y,
                width: node_width,
                height: row_height,
                label: node.label.clone(),
                value: node.total_value(),
                color: node.color,
                depth,
                node_index: path,
            });

            x_offset += node_width;
        }

        let new_area = Rect::new(
            area.x,
            area.y + row_height,
            area.width,
            area.height.saturating_sub(row_height),
        );
        (rects, new_area)
    }
}

/// Check whether two rectangles overlap.
///
/// Returns true if the two rectangles share any pixel.
#[cfg(test)]
pub(super) fn rects_overlap(a: &LayoutRect, b: &LayoutRect) -> bool {
    let a_right = a.x + a.width;
    let a_bottom = a.y + a.height;
    let b_right = b.x + b.width;
    let b_bottom = b.y + b.height;

    a.x < b_right && b.x < a_right && a.y < b_bottom && b.y < a_bottom
}
