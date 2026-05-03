//! Rendering functions for the Diagram component.
//!
//! Draws nodes as bordered boxes with labels and status indicators,
//! and edges as box-drawing character paths. Uses batch buffer writes
//! for edges instead of per-cell widget rendering.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::edge_routing;
use super::layout::{EdgePath, LayoutResult, NodePosition, PathSegment};
use super::types::{DiagramEdge, DiagramNode, NodeShape, NodeStatus};
use super::viewport::Viewport2D;
use crate::component::RenderContext;
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

/// Renders the complete diagram: border, edges, nodes, and info bar.
///
/// `ctx.chrome_owned` signals that the parent has already drawn the outer
/// chrome for `ctx.area`. When true, the outer `Block` draw is suppressed
/// and content is rendered against `ctx.area` directly.
pub(super) fn render_diagram(
    state: &super::DiagramState,
    layout: &LayoutResult,
    ctx: &mut RenderContext<'_, '_>,
) {
    let inner = if ctx.chrome_owned {
        ctx.area
    } else {
        // Outer border
        let border_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else if ctx.focused {
            ctx.theme.focused_border_style()
        } else {
            ctx.theme.border_style()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(
                state
                    .title
                    .as_deref()
                    .map(|t| format!(" {} ", t))
                    .unwrap_or_default(),
            );

        let inner = block.inner(ctx.area);
        ctx.frame.render_widget(block, ctx.area);
        inner
    };

    if inner.width < 3 || inner.height < 2 {
        return;
    }

    if state.nodes.is_empty() {
        let msg = Paragraph::new("(empty diagram)").style(ctx.theme.normal_style());
        ctx.frame.render_widget(msg, inner);
        return;
    }

    let params = RenderParams {
        viewport: &state.viewport,
        clip: inner,
        disabled: ctx.disabled,
        theme: ctx.theme,
    };

    // Render edges first (behind nodes), then nodes on top
    render_edges(
        ctx.frame.buffer_mut(),
        &state.edges,
        layout.edge_paths(),
        &params,
        state.show_edge_labels,
    );

    render_nodes(
        ctx.frame,
        &state.nodes,
        layout.node_positions(),
        &params,
        state.selected,
        ctx.focused,
    );

    // Minimap in bottom-right corner
    if state.show_minimap && state.viewport.needs_scroll() && inner.width >= 20 && inner.height >= 8
    {
        render_minimap(
            ctx.frame.buffer_mut(),
            layout.node_positions(),
            &state.nodes,
            &state.viewport,
            inner,
        );
    }

    // Search bar at bottom
    if state.search.active {
        render_search_bar(ctx.frame, &state.search, inner, ctx.theme);
    }

    // Info bar at bottom showing selected node details
    if let Some(sel_idx) = state.selected {
        if let Some(node) = state.nodes.get(sel_idx) {
            render_info_bar(ctx.frame, node, inner, ctx.disabled, ctx.theme);
        }
    }
}

/// Context for rendering edges and nodes.
struct RenderParams<'a> {
    viewport: &'a Viewport2D,
    clip: Rect,
    disabled: bool,
    theme: &'a Theme,
}

/// Renders all edges using batch buffer writes.
///
/// Instead of creating a Paragraph widget per edge cell (the DependencyGraph
/// approach), we compute all edge characters and write them directly to the
/// buffer. This is dramatically faster for large graphs.
fn render_edges(
    buf: &mut Buffer,
    edges: &[DiagramEdge],
    edge_paths: &[EdgePath],
    params: &RenderParams<'_>,
    show_labels: bool,
) {
    for path in edge_paths {
        let edge = edges
            .iter()
            .find(|e| e.from() == path.from_id() && e.to() == path.to_id());

        let edge_color = edge.and_then(|e| e.color()).unwrap_or(Color::DarkGray);

        let style = if params.disabled {
            params.theme.disabled_style()
        } else {
            Style::default().fg(edge_color)
        };

        let is_dashed = edge.is_some_and(|e| *e.style() == super::types::EdgeStyle::Dashed);
        let is_dotted = edge.is_some_and(|e| *e.style() == super::types::EdgeStyle::Dotted);

        // Walk segments and draw line characters + corners
        let segments = path.segments();
        for i in 0..segments.len().saturating_sub(1) {
            let (x0, y0) = segment_coords(&segments[i]);
            let (x1, y1) = segment_coords(&segments[i + 1]);

            let ls = LineStyle {
                style,
                is_dashed,
                is_dotted,
                is_last: i + 1 == segments.len() - 1,
            };
            draw_line_segment(buf, params, (x0, y0, x1, y1), &ls);

            // Draw corner character at the bend point between two segments
            if i + 2 < segments.len() {
                let (x2, y2) = segment_coords(&segments[i + 2]);
                let corner = edge_routing::corner_char(x0, y0, x1, y1, x2, y2);
                let (sx, sy) = params.viewport.to_screen(x1, y1, params.clip);
                if sx >= params.clip.x as i32
                    && sx < params.clip.right() as i32
                    && sy >= params.clip.y as i32
                    && sy < params.clip.bottom() as i32
                {
                    let buf_area = Rect::new(0, 0, buf.area.width, buf.area.height);
                    set_cell(buf, sx as u16, sy as u16, corner, style, buf_area);
                }
            }
        }

        // Edge label at midpoint
        if show_labels {
            if let Some(label) = edge.and_then(|e| e.label()) {
                render_edge_label(buf, path, params, label, style);
            }
        }
    }
}

/// Extracts (x, y) from a path segment.
fn segment_coords(seg: &PathSegment) -> (f64, f64) {
    match *seg {
        PathSegment::MoveTo(x, y) | PathSegment::LineTo(x, y) => (x, y),
    }
}

/// Style parameters for a line segment.
struct LineStyle {
    style: Style,
    is_dashed: bool,
    is_dotted: bool,
    is_last: bool,
}

/// Draws a single line segment (horizontal or vertical) with box-drawing chars.
fn draw_line_segment(
    buf: &mut Buffer,
    params: &RenderParams<'_>,
    endpoints: (f64, f64, f64, f64),
    ls: &LineStyle,
) {
    let (x0, y0, x1, y1) = endpoints;
    let clip = params.clip;
    let viewport = params.viewport;
    let buf_area = Rect::new(0, 0, buf.area.width, buf.area.height);

    if (y0 - y1).abs() < 0.5 {
        // Horizontal segment
        let sy = viewport.to_screen(x0, y0, clip).1;
        if sy < clip.y as i32 || sy >= (clip.y + clip.height) as i32 {
            return;
        }
        let sy = sy as u16;

        let sx0 = viewport.to_screen(x0.min(x1), y0, clip).0;
        let sx1 = viewport.to_screen(x0.max(x1), y0, clip).0;

        let dir_right = x1 > x0;
        for sx in sx0.max(clip.x as i32)..=sx1.min((clip.right() - 1) as i32) {
            let sx = sx as u16;
            let ch = if ls.is_last && sx == (if dir_right { sx1 } else { sx0 }) as u16 {
                if dir_right { "\u{25b6}" } else { "\u{25c0}" } // ▶ or ◀
            } else if ls.is_dotted {
                "\u{00b7}" // ·
            } else if ls.is_dashed && (sx % 3 == 0) {
                " "
            } else {
                "\u{2500}" // ─
            };
            set_cell(buf, sx, sy, ch, ls.style, buf_area);
        }
    } else if (x0 - x1).abs() < 0.5 {
        // Vertical segment
        let sx = viewport.to_screen(x0, y0, clip).0;
        if sx < clip.x as i32 || sx >= (clip.x + clip.width) as i32 {
            return;
        }
        let sx = sx as u16;

        let sy0 = viewport.to_screen(x0, y0.min(y1), clip).1;
        let sy1 = viewport.to_screen(x0, y0.max(y1), clip).1;

        let dir_down = y1 > y0;
        for sy in sy0.max(clip.y as i32)..=sy1.min((clip.bottom() - 1) as i32) {
            let sy = sy as u16;
            let ch = if ls.is_last && sy == (if dir_down { sy1 } else { sy0 }) as u16 {
                if dir_down { "\u{25bc}" } else { "\u{25b2}" } // ▼ or ▲
            } else if ls.is_dotted {
                "\u{00b7}" // ·
            } else if ls.is_dashed && (sy % 3 == 0) {
                " "
            } else {
                "\u{2502}" // │
            };
            set_cell(buf, sx, sy, ch, ls.style, buf_area);
        }
    }
    // Diagonal segments are not rendered (orthogonal routing only)
}

/// Renders an edge label near the midpoint of the path.
fn render_edge_label(
    buf: &mut Buffer,
    path: &EdgePath,
    params: &RenderParams<'_>,
    label: &str,
    style: Style,
) {
    let segments = path.segments();
    if segments.len() < 2 {
        return;
    }

    // Find midpoint of the path
    let mid_idx = segments.len() / 2;
    let (mx, my) = segment_coords(&segments[mid_idx]);
    let (sx, sy) = params.viewport.to_screen(mx, my, params.clip);

    if sy < params.clip.y as i32 || sy >= params.clip.bottom() as i32 {
        return;
    }
    let sy = sy as u16;
    let sx = sx.max(params.clip.x as i32) as u16;

    let buf_area = Rect::new(0, 0, buf.area.width, buf.area.height);
    for (i, ch) in label.chars().enumerate() {
        let cx = sx + i as u16;
        if cx >= params.clip.right() {
            break;
        }
        let mut s = String::new();
        s.push(ch);
        set_cell(buf, cx, sy, &s, style, buf_area);
    }
}

/// Writes a single character to the buffer at (x, y) if within bounds.
fn set_cell(buf: &mut Buffer, x: u16, y: u16, ch: &str, style: Style, area: Rect) {
    if x >= area.x && x < area.right() && y >= area.y && y < area.bottom() {
        if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
            cell.set_symbol(ch);
            cell.set_style(style);
        }
    }
}

/// Renders all visible nodes.
fn render_nodes(
    frame: &mut ratatui::Frame,
    nodes: &[DiagramNode],
    positions: &[NodePosition],
    params: &RenderParams<'_>,
    selected: Option<usize>,
    focused: bool,
) {
    for (idx, (node, pos)) in nodes.iter().zip(positions.iter()).enumerate() {
        // Viewport culling
        if !params
            .viewport
            .is_visible(pos.x(), pos.y(), pos.width(), pos.height())
        {
            continue;
        }

        let (sx, sy) = params.viewport.to_screen(pos.x(), pos.y(), params.clip);
        let sw = (pos.width() * params.viewport.zoom()) as u16;
        let sh = (pos.height() * params.viewport.zoom()) as u16;

        // Clamp to visible area
        if sx >= params.clip.right() as i32 || sy >= params.clip.bottom() as i32 {
            continue;
        }
        let sx = (sx.max(params.clip.x as i32)) as u16;
        let sy = (sy.max(params.clip.y as i32)) as u16;
        let sw = sw.min(params.clip.right().saturating_sub(sx));
        let sh = sh.min(params.clip.bottom().saturating_sub(sy));

        if sw < 3 || sh < 2 {
            continue;
        }

        let node_area = Rect::new(sx, sy, sw, sh);
        let is_selected = selected == Some(idx);

        render_single_node(frame, node, node_area, is_selected, focused, params);
    }
}

/// Renders a single node as a bordered box with status indicator.
fn render_single_node(
    frame: &mut ratatui::Frame,
    node: &DiagramNode,
    area: Rect,
    is_selected: bool,
    focused: bool,
    params: &RenderParams<'_>,
) {
    let node_color = node.color().unwrap_or_else(|| status_color(node.status()));

    let border_style = if params.disabled {
        params.theme.disabled_style()
    } else if is_selected && focused {
        Style::default().fg(node_color).add_modifier(Modifier::BOLD)
    } else if is_selected {
        Style::default().fg(node_color)
    } else {
        Style::default().fg(params.theme.normal_style().fg.unwrap_or(Color::White))
    };

    // Truncate label to fit
    let max_label = area.width.saturating_sub(4) as usize;
    let label_display = if node.label().len() > max_label {
        if max_label > 0 {
            format!(" {} ", &node.label()[..max_label])
        } else {
            String::new()
        }
    } else {
        format!(" {} ", node.label())
    };

    let block = match node.shape() {
        NodeShape::RoundedRectangle => Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(label_display)
            .style(border_style),
        _ => Block::default()
            .borders(Borders::ALL)
            .title(label_display)
            .style(border_style),
    };

    let block_inner = block.inner(area);
    frame.render_widget(block, area);

    if block_inner.width == 0 || block_inner.height == 0 {
        return;
    }

    // Status line inside the node
    let indicator = status_indicator(node.status());
    let label = status_label(node.status());
    let content = format!("{indicator} {label}");

    let content_style = if params.disabled {
        params.theme.disabled_style()
    } else {
        Style::default().fg(node_color)
    };

    let chars: Vec<char> = content.chars().collect();
    let truncated: String = chars.into_iter().take(block_inner.width as usize).collect();
    let content_area = Rect::new(block_inner.x, block_inner.y, block_inner.width, 1);
    frame.render_widget(Paragraph::new(truncated).style(content_style), content_area);
}

/// Renders an info bar at the bottom of the diagram showing selected node details.
fn render_info_bar(
    frame: &mut ratatui::Frame,
    node: &DiagramNode,
    area: Rect,
    disabled: bool,
    theme: &Theme,
) {
    if area.height < 3 {
        return;
    }

    let bar_area = Rect::new(area.x, area.bottom() - 1, area.width, 1);

    let indicator = status_indicator(node.status());
    let node_color = node.color().unwrap_or_else(|| status_color(node.status()));

    let mut info = format!(
        " {} {} [{} {}]",
        node.id(),
        node.label(),
        indicator,
        status_label(node.status()),
    );

    // Append metadata if present
    for (key, value) in node.metadata() {
        info.push_str(&format!(" | {key}: {value}"));
    }

    // Truncate to fit
    let max_len = bar_area.width as usize;
    if info.len() > max_len {
        info.truncate(max_len.saturating_sub(1));
        info.push('\u{2026}'); // …
    }

    let style = if disabled {
        theme.disabled_style()
    } else {
        Style::default().fg(node_color)
    };

    frame.render_widget(Paragraph::new(info).style(style), bar_area);
}

/// Renders a minimap in the bottom-right corner showing the full graph
/// with a viewport indicator rectangle.
fn render_minimap(
    buf: &mut Buffer,
    positions: &[NodePosition],
    nodes: &[DiagramNode],
    viewport: &Viewport2D,
    area: Rect,
) {
    let map_w: u16 = 16;
    let map_h: u16 = 6;
    let map_x = area.right().saturating_sub(map_w + 1);
    let map_y = area.bottom().saturating_sub(map_h + 1);
    let map_area = Rect::new(map_x, map_y, map_w, map_h);

    if map_area.width < 4 || map_area.height < 3 {
        return;
    }

    let buf_area = buf.area;

    // Clear minimap area with dark background
    for y in map_area.y..map_area.bottom() {
        for x in map_area.x..map_area.right() {
            set_cell(buf, x, y, " ", Style::default().bg(Color::Black), buf_area);
        }
    }

    // Draw border
    let border_style = Style::default().fg(Color::DarkGray);
    for x in map_area.x..map_area.right() {
        set_cell(buf, x, map_area.y, "\u{2500}", border_style, buf_area);
        set_cell(
            buf,
            x,
            map_area.bottom() - 1,
            "\u{2500}",
            border_style,
            buf_area,
        );
    }
    for y in map_area.y..map_area.bottom() {
        set_cell(buf, map_area.x, y, "\u{2502}", border_style, buf_area);
        set_cell(
            buf,
            map_area.right() - 1,
            y,
            "\u{2502}",
            border_style,
            buf_area,
        );
    }
    set_cell(
        buf,
        map_area.x,
        map_area.y,
        "\u{250c}",
        border_style,
        buf_area,
    );
    set_cell(
        buf,
        map_area.right() - 1,
        map_area.y,
        "\u{2510}",
        border_style,
        buf_area,
    );
    set_cell(
        buf,
        map_area.x,
        map_area.bottom() - 1,
        "\u{2514}",
        border_style,
        buf_area,
    );
    set_cell(
        buf,
        map_area.right() - 1,
        map_area.bottom() - 1,
        "\u{2518}",
        border_style,
        buf_area,
    );

    // Compute scale to fit all nodes in the minimap inner area
    let inner = Rect::new(map_area.x + 1, map_area.y + 1, map_w - 2, map_h - 2);
    if inner.width == 0 || inner.height == 0 || positions.is_empty() {
        return;
    }

    let bbox = &viewport;
    let content_w = bbox.offset_x() + f64::from(area.width) / bbox.zoom();
    let content_h = bbox.offset_y() + f64::from(area.height) / bbox.zoom();

    // Find actual content bounds from positions
    let max_x = positions
        .iter()
        .map(|p| p.x() + p.width())
        .fold(0.0f64, f64::max);
    let max_y = positions
        .iter()
        .map(|p| p.y() + p.height())
        .fold(0.0f64, f64::max);
    let total_w = max_x.max(content_w);
    let total_h = max_y.max(content_h);

    if total_w <= 0.0 || total_h <= 0.0 {
        return;
    }

    let scale_x = f64::from(inner.width) / total_w;
    let scale_y = f64::from(inner.height) / total_h;

    // Draw nodes as dots
    for (i, pos) in positions.iter().enumerate() {
        let mx = inner.x + (pos.center_x() * scale_x) as u16;
        let my = inner.y + (pos.center_y() * scale_y) as u16;
        if mx >= inner.x && mx < inner.right() && my >= inner.y && my < inner.bottom() {
            let color = nodes.get(i).and_then(|n| n.color()).unwrap_or_else(|| {
                nodes
                    .get(i)
                    .map_or(Color::White, |n| status_color(n.status()))
            });
            set_cell(
                buf,
                mx,
                my,
                "\u{25cf}",
                Style::default().fg(color),
                buf_area,
            );
        }
    }

    // Draw viewport rectangle
    let vp_x = inner.x + (viewport.offset_x() * scale_x) as u16;
    let vp_y = inner.y + (viewport.offset_y() * scale_y) as u16;
    let vp_w = ((f64::from(area.width) / viewport.zoom()) * scale_x) as u16;
    let vp_h = ((f64::from(area.height) / viewport.zoom()) * scale_y) as u16;
    let vp_style = Style::default().fg(Color::Cyan);

    let vp_right = (vp_x + vp_w).min(inner.right() - 1);
    let vp_bottom = (vp_y + vp_h).min(inner.bottom() - 1);

    for x in vp_x..=vp_right {
        if x >= inner.x && x < inner.right() {
            if vp_y >= inner.y && vp_y < inner.bottom() {
                set_cell(buf, x, vp_y, "\u{2500}", vp_style, buf_area);
            }
            if vp_bottom >= inner.y && vp_bottom < inner.bottom() {
                set_cell(buf, x, vp_bottom, "\u{2500}", vp_style, buf_area);
            }
        }
    }
    for y in vp_y..=vp_bottom {
        if y >= inner.y && y < inner.bottom() {
            if vp_x >= inner.x && vp_x < inner.right() {
                set_cell(buf, vp_x, y, "\u{2502}", vp_style, buf_area);
            }
            if vp_right >= inner.x && vp_right < inner.right() {
                set_cell(buf, vp_right, y, "\u{2502}", vp_style, buf_area);
            }
        }
    }
}

/// Renders the search bar at the bottom of the diagram.
fn render_search_bar(
    frame: &mut ratatui::Frame,
    search: &super::search::SearchState,
    area: Rect,
    theme: &Theme,
) {
    if area.height < 2 {
        return;
    }

    let bar_area = Rect::new(area.x, area.bottom() - 1, area.width, 1);

    let match_info = if search.matches.is_empty() {
        if search.query.is_empty() {
            String::new()
        } else {
            " (no matches)".to_string()
        }
    } else {
        format!(" [{}/{}]", search.current_match + 1, search.matches.len())
    };

    let text = format!("/{}{}", search.query, match_info);
    let style = Style::default().fg(theme.focused_style().fg.unwrap_or(Color::Yellow));

    frame.render_widget(Paragraph::new(text).style(style), bar_area);
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
        assert_eq!(status_label(&NodeStatus::Down), "Down");
    }

    #[test]
    fn test_segment_coords() {
        assert_eq!(segment_coords(&PathSegment::MoveTo(1.0, 2.0)), (1.0, 2.0));
        assert_eq!(segment_coords(&PathSegment::LineTo(3.0, 4.0)), (3.0, 4.0));
    }

    #[test]
    fn test_set_cell_in_bounds() {
        let area = Rect::new(0, 0, 10, 5);
        let mut buf = Buffer::empty(area);
        set_cell(&mut buf, 3, 2, "X", Style::default(), area);
        assert_eq!(buf.cell(Position::new(3, 2)).unwrap().symbol(), "X");
    }

    #[test]
    fn test_set_cell_out_of_bounds() {
        let area = Rect::new(0, 0, 10, 5);
        let mut buf = Buffer::empty(area);
        // Should not panic
        set_cell(&mut buf, 15, 2, "X", Style::default(), area);
        set_cell(&mut buf, 3, 10, "X", Style::default(), area);
    }
}
