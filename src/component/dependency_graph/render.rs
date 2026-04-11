//! Rendering functions for the DependencyGraph component.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::layout::{self, LayoutEdge, LayoutNode};
use super::{DependencyGraphState, NodeStatus};
use crate::theme::Theme;

/// Status indicator characters for each node status.
fn status_indicator(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Healthy => "\u{25cf}",  // ●
        NodeStatus::Degraded => "\u{25b2}", // ▲
        NodeStatus::Down => "\u{2716}",     // ✖
        NodeStatus::Unknown => "?",
    }
}

/// Returns the default color for a node status.
fn status_color(status: &NodeStatus) -> Color {
    match status {
        NodeStatus::Healthy => Color::Green,
        NodeStatus::Degraded => Color::Yellow,
        NodeStatus::Down => Color::Red,
        NodeStatus::Unknown => Color::DarkGray,
    }
}

/// Status label text.
fn status_label(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Healthy => "Healthy",
        NodeStatus::Degraded => "Degraded",
        NodeStatus::Down => "Down",
        NodeStatus::Unknown => "Unknown",
    }
}

/// Renders the full dependency graph including border, nodes, and edges.
pub(super) fn render_dependency_graph(
    state: &DependencyGraphState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    disabled: bool,
) {
    let border_style = if disabled {
        theme.disabled_style()
    } else if focused {
        theme.focused_border_style()
    } else {
        theme.normal_style()
    };

    let mut block = Block::default().borders(Borders::ALL).style(border_style);
    if let Some(title) = &state.title {
        block = block.title(format!(" {} ", title));
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    crate::annotation::with_registry(|reg| {
        reg.register(
            area,
            crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
                "DependencyGraph".to_string(),
            ))
            .with_id("dependency_graph")
            .with_focus(focused)
            .with_disabled(disabled),
        );
    });

    if state.nodes.is_empty() {
        render_empty_state(state, frame, inner, theme, disabled);
        return;
    }

    // Compute layout
    let (layout_nodes, layout_edges) =
        layout::compute_layout(&state.nodes, &state.edges, inner, &state.orientation);

    // Render edges first (behind nodes)
    for (edge_idx, layout_edge) in layout_edges.iter().enumerate() {
        let edge_color = state
            .edges
            .get(edge_idx)
            .and_then(|e| e.color)
            .unwrap_or_else(|| {
                if disabled {
                    Color::DarkGray
                } else {
                    theme.normal_style().fg.unwrap_or(Color::White)
                }
            });
        let edge_label = if state.show_edge_labels {
            state.edges.get(edge_idx).and_then(|e| e.label.as_deref())
        } else {
            None
        };
        render_edge(
            frame,
            layout_edge,
            edge_color,
            edge_label,
            inner,
            disabled,
            theme,
        );
    }

    // Render nodes
    for layout_node in &layout_nodes {
        let graph_node = state.nodes.iter().find(|n| n.id == layout_node.id);
        let is_selected = state
            .selected
            .and_then(|idx| state.nodes.get(idx))
            .map(|n| n.id == layout_node.id)
            .unwrap_or(false);

        if let Some(node) = graph_node {
            render_node(
                frame,
                node,
                layout_node,
                NodeRenderState {
                    is_selected,
                    focused,
                    disabled,
                },
                state,
                inner,
                theme,
            );
        }
    }
}

/// Renders an empty state message centered in the area.
fn render_empty_state(
    _state: &DependencyGraphState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    disabled: bool,
) {
    let style = if disabled {
        theme.disabled_style()
    } else {
        theme.normal_style()
    };
    let msg = "No nodes";
    let centered_x = area.x + area.width.saturating_sub(msg.len() as u16) / 2;
    let centered_y = area.y + area.height / 2;
    let msg_area = Rect::new(centered_x, centered_y, msg.len() as u16, 1);
    frame.render_widget(Paragraph::new(msg).style(style), msg_area);
}

/// Selection and focus state for rendering a graph node.
struct NodeRenderState {
    is_selected: bool,
    focused: bool,
    disabled: bool,
}

/// Renders a single graph node as a bordered box with label and status.
fn render_node(
    frame: &mut Frame,
    node: &super::GraphNode,
    layout_node: &LayoutNode,
    render_state: NodeRenderState,
    _state: &DependencyGraphState,
    clip: Rect,
    theme: &Theme,
) {
    let is_selected = render_state.is_selected;
    let focused = render_state.focused;
    let disabled = render_state.disabled;
    // Clip node to inner area
    let node_area = clip_rect(
        Rect::new(
            layout_node.x,
            layout_node.y,
            layout_node.width,
            layout_node.height,
        ),
        clip,
    );
    if node_area.width < 3 || node_area.height < 3 {
        return;
    }

    let node_color = node.color.unwrap_or_else(|| status_color(&node.status));

    let border_style = if disabled {
        theme.disabled_style()
    } else if is_selected && focused {
        Style::default().fg(node_color).add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default().fg(node_color)
    } else {
        Style::default().fg(theme.normal_style().fg.unwrap_or(Color::White))
    };

    // Draw border with label in title
    let label_display = if node.label.len() > node_area.width.saturating_sub(4) as usize {
        let max = node_area.width.saturating_sub(4) as usize;
        if max > 0 {
            format!(" {} ", &node.label[..max])
        } else {
            String::new()
        }
    } else {
        format!(" {} ", node.label)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(label_display)
        .style(border_style);
    let block_inner = block.inner(node_area);
    frame.render_widget(block, node_area);

    if block_inner.width == 0 || block_inner.height == 0 {
        return;
    }

    // Render status line inside the node
    let indicator = status_indicator(&node.status);
    let label = status_label(&node.status);
    let content = format!("{} {}", indicator, label);

    let content_style = if disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(node_color)
    };

    // Truncate content to fit
    let chars: Vec<char> = content.chars().collect();
    let truncated: String = chars.into_iter().take(block_inner.width as usize).collect();
    let content_area = Rect::new(block_inner.x, block_inner.y, block_inner.width, 1);
    frame.render_widget(Paragraph::new(truncated).style(content_style), content_area);
}

/// Renders an edge between two nodes using ASCII line characters.
fn render_edge(
    frame: &mut Frame,
    edge: &LayoutEdge,
    color: Color,
    label: Option<&str>,
    clip: Rect,
    disabled: bool,
    theme: &Theme,
) {
    let style = if disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(color)
    };

    // Simple edge rendering: draw horizontal and vertical segments
    // For left-to-right: horizontal line from source, then vertical connector, then to target
    // For top-to-bottom: vertical line from source, then horizontal connector, then to target

    if edge.from_x == edge.to_x {
        // Vertical edge
        let x = edge.from_x;
        let min_y = edge.from_y.min(edge.to_y);
        let max_y = edge.from_y.max(edge.to_y);
        for y in min_y..=max_y {
            if point_in_rect(x, y, clip) {
                let ch = if y == edge.to_y {
                    if edge.to_y > edge.from_y {
                        "\u{25bc}" // ▼
                    } else {
                        "\u{25b2}" // ▲
                    }
                } else {
                    "\u{2502}" // │
                };
                let area = Rect::new(x, y, 1, 1);
                frame.render_widget(Paragraph::new(ch).style(style), area);
            }
        }
    } else if edge.from_y == edge.to_y {
        // Horizontal edge
        let y = edge.from_y;
        let min_x = edge.from_x.min(edge.to_x);
        let max_x = edge.from_x.max(edge.to_x);
        for x in min_x..=max_x {
            if point_in_rect(x, y, clip) {
                let ch = if x == edge.to_x {
                    if edge.to_x > edge.from_x {
                        "\u{25b6}" // ▶
                    } else {
                        "\u{25c0}" // ◀
                    }
                } else {
                    "\u{2500}" // ─
                };
                let area = Rect::new(x, y, 1, 1);
                frame.render_widget(Paragraph::new(ch).style(style), area);
            }
        }
    } else {
        // L-shaped edge: go horizontal first, then vertical
        let mid_x = edge.to_x;
        let min_x = edge.from_x.min(mid_x);
        let max_x = edge.from_x.max(mid_x);

        // Horizontal segment
        for x in min_x..=max_x {
            if point_in_rect(x, edge.from_y, clip) {
                let ch = if x == mid_x {
                    // Corner
                    if edge.to_y > edge.from_y {
                        if mid_x > edge.from_x {
                            "\u{2510}" // ┐
                        } else {
                            "\u{250c}" // ┌
                        }
                    } else if mid_x > edge.from_x {
                        "\u{2518}" // ┘
                    } else {
                        "\u{2514}" // └
                    }
                } else {
                    "\u{2500}" // ─
                };
                let area = Rect::new(x, edge.from_y, 1, 1);
                frame.render_widget(Paragraph::new(ch).style(style), area);
            }
        }

        // Vertical segment
        let min_y = edge.from_y.min(edge.to_y);
        let max_y = edge.from_y.max(edge.to_y);
        for y in (min_y + 1)..=max_y {
            if point_in_rect(mid_x, y, clip) {
                let ch = if y == edge.to_y {
                    if edge.to_y > edge.from_y {
                        "\u{25bc}" // ▼
                    } else {
                        "\u{25b2}" // ▲
                    }
                } else {
                    "\u{2502}" // │
                };
                let area = Rect::new(mid_x, y, 1, 1);
                frame.render_widget(Paragraph::new(ch).style(style), area);
            }
        }
    }

    // Render edge label at midpoint if present
    if let Some(label_text) = label {
        let mid_x = (edge.from_x + edge.to_x) / 2;
        let mid_y = (edge.from_y + edge.to_y) / 2;
        let label_len = label_text.len() as u16;
        if point_in_rect(mid_x, mid_y, clip) && mid_x + label_len <= clip.x + clip.width {
            let area = Rect::new(mid_x, mid_y, label_len, 1);
            frame.render_widget(Paragraph::new(label_text).style(style), area);
        }
    }
}

/// Clips a rectangle to fit within a bounding area.
fn clip_rect(rect: Rect, clip: Rect) -> Rect {
    let x = rect.x.max(clip.x);
    let y = rect.y.max(clip.y);
    let right = (rect.x + rect.width).min(clip.x + clip.width);
    let bottom = (rect.y + rect.height).min(clip.y + clip.height);
    let width = right.saturating_sub(x);
    let height = bottom.saturating_sub(y);
    Rect::new(x, y, width, height)
}

/// Returns true if a point is within the given rectangle.
fn point_in_rect(x: u16, y: u16, rect: Rect) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_indicator() {
        assert_eq!(status_indicator(&NodeStatus::Healthy), "\u{25cf}");
        assert_eq!(status_indicator(&NodeStatus::Degraded), "\u{25b2}");
        assert_eq!(status_indicator(&NodeStatus::Down), "\u{2716}");
        assert_eq!(status_indicator(&NodeStatus::Unknown), "?");
    }

    #[test]
    fn test_status_color() {
        assert_eq!(status_color(&NodeStatus::Healthy), Color::Green);
        assert_eq!(status_color(&NodeStatus::Degraded), Color::Yellow);
        assert_eq!(status_color(&NodeStatus::Down), Color::Red);
        assert_eq!(status_color(&NodeStatus::Unknown), Color::DarkGray);
    }

    #[test]
    fn test_status_label() {
        assert_eq!(status_label(&NodeStatus::Healthy), "Healthy");
        assert_eq!(status_label(&NodeStatus::Degraded), "Degraded");
        assert_eq!(status_label(&NodeStatus::Down), "Down");
        assert_eq!(status_label(&NodeStatus::Unknown), "Unknown");
    }

    #[test]
    fn test_clip_rect_no_clip_needed() {
        let rect = Rect::new(5, 5, 10, 10);
        let clip = Rect::new(0, 0, 80, 24);
        let result = clip_rect(rect, clip);
        assert_eq!(result, rect);
    }

    #[test]
    fn test_clip_rect_partial_clip() {
        let rect = Rect::new(75, 20, 10, 10);
        let clip = Rect::new(0, 0, 80, 24);
        let result = clip_rect(rect, clip);
        assert_eq!(result, Rect::new(75, 20, 5, 4));
    }

    #[test]
    fn test_clip_rect_fully_outside() {
        let rect = Rect::new(100, 100, 10, 10);
        let clip = Rect::new(0, 0, 80, 24);
        let result = clip_rect(rect, clip);
        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    #[test]
    fn test_point_in_rect() {
        let rect = Rect::new(10, 10, 20, 10);
        assert!(point_in_rect(10, 10, rect));
        assert!(point_in_rect(29, 19, rect));
        assert!(!point_in_rect(30, 10, rect));
        assert!(!point_in_rect(10, 20, rect));
        assert!(!point_in_rect(9, 10, rect));
    }
}
