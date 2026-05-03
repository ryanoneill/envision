use super::*;
use crate::component::context::RenderContext;
use crate::component::test_utils;

#[test]
fn test_snapshot_empty() {
    let state = DiagramState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            Diagram::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_node() {
    let state = DiagramState::new()
        .with_node(DiagramNode::new("api", "API Gateway").with_status(NodeStatus::Healthy))
        .with_title("Service Graph");
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            Diagram::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_linear_graph() {
    let state = DiagramState::new()
        .with_node(DiagramNode::new("api", "API").with_status(NodeStatus::Healthy))
        .with_node(DiagramNode::new("db", "Database").with_status(NodeStatus::Degraded))
        .with_edge(DiagramEdge::new("api", "db"))
        .with_title("Linear");
    let (mut terminal, theme) = test_utils::setup_render(80, 12);
    terminal
        .draw(|frame| {
            Diagram::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_diamond() {
    let state = DiagramState::new()
        .with_node(DiagramNode::new("a", "Source").with_status(NodeStatus::Healthy))
        .with_node(DiagramNode::new("b", "Left").with_status(NodeStatus::Healthy))
        .with_node(DiagramNode::new("c", "Right").with_status(NodeStatus::Degraded))
        .with_node(DiagramNode::new("d", "Sink").with_status(NodeStatus::Down))
        .with_edge(DiagramEdge::new("a", "b"))
        .with_edge(DiagramEdge::new("a", "c"))
        .with_edge(DiagramEdge::new("b", "d"))
        .with_edge(DiagramEdge::new("c", "d"))
        .with_title("Diamond");
    let (mut terminal, theme) = test_utils::setup_render(80, 16);
    terminal
        .draw(|frame| {
            Diagram::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_selection() {
    let mut state = DiagramState::new()
        .with_node(DiagramNode::new("api", "API").with_status(NodeStatus::Healthy))
        .with_node(DiagramNode::new("db", "DB").with_status(NodeStatus::Healthy))
        .with_edge(DiagramEdge::new("api", "db"));
    state.select_next(); // Select first node

    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            let mut ctx = RenderContext::new(frame, frame.area(), &theme);
            ctx.focused = true;
            Diagram::view(&state, &mut ctx);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_statuses() {
    let state = DiagramState::new()
        .with_node(DiagramNode::new("a", "Healthy").with_status(NodeStatus::Healthy))
        .with_node(DiagramNode::new("b", "Degraded").with_status(NodeStatus::Degraded))
        .with_node(DiagramNode::new("c", "Down").with_status(NodeStatus::Down))
        .with_node(DiagramNode::new("d", "Unknown").with_status(NodeStatus::Unknown))
        .with_edge(DiagramEdge::new("a", "b"))
        .with_edge(DiagramEdge::new("b", "c"))
        .with_edge(DiagramEdge::new("c", "d"));
    let (mut terminal, theme) = test_utils::setup_render(80, 12);
    terminal
        .draw(|frame| {
            Diagram::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_top_to_bottom() {
    let state = DiagramState::new()
        .with_node(DiagramNode::new("a", "Top").with_status(NodeStatus::Healthy))
        .with_node(DiagramNode::new("b", "Bottom").with_status(NodeStatus::Healthy))
        .with_edge(DiagramEdge::new("a", "b"))
        .with_orientation(Orientation::TopToBottom)
        .with_title("Vertical");
    let (mut terminal, theme) = test_utils::setup_render(40, 16);
    terminal
        .draw(|frame| {
            Diagram::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_edge_styles() {
    let state = DiagramState::new()
        .with_node(DiagramNode::new("a", "A").with_status(NodeStatus::Healthy))
        .with_node(DiagramNode::new("b", "B").with_status(NodeStatus::Healthy))
        .with_node(DiagramNode::new("c", "C").with_status(NodeStatus::Healthy))
        .with_edge(DiagramEdge::new("a", "b").with_style(EdgeStyle::Dashed))
        .with_edge(DiagramEdge::new("a", "c").with_style(EdgeStyle::Dotted))
        .with_show_edge_labels(true);
    let (mut terminal, theme) = test_utils::setup_render(80, 12);
    terminal
        .draw(|frame| {
            Diagram::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_rounded_nodes() {
    let state = DiagramState::new()
        .with_node(
            DiagramNode::new("api", "API Gateway")
                .with_status(NodeStatus::Healthy)
                .with_shape(NodeShape::RoundedRectangle),
        )
        .with_node(
            DiagramNode::new("db", "Database")
                .with_status(NodeStatus::Healthy)
                .with_shape(NodeShape::RoundedRectangle),
        )
        .with_edge(DiagramEdge::new("api", "db"));
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            Diagram::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn view_chrome_owned_no_outer_border() {
    let state = DiagramState::new()
        .with_node(DiagramNode::new("api", "API").with_status(NodeStatus::Healthy))
        .with_title("Service Graph");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Diagram::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).chrome_owned(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
