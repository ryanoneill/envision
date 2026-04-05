use super::*;
use crate::component::test_utils;

#[test]
fn test_snapshot_empty() {
    let state = SpanTreeState::default();
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            SpanTree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_simple_tree() {
    let root = SpanNode::new("req", "frontend/request", 0.0, 1000.0)
        .with_color(Color::Cyan)
        .with_child(
            SpanNode::new("api", "api/handler", 50.0, 800.0)
                .with_color(Color::Yellow)
                .with_child(SpanNode::new("db", "db/query", 100.0, 400.0).with_color(Color::Green))
                .with_child(
                    SpanNode::new("cache", "cache/lookup", 450.0, 700.0).with_color(Color::Blue),
                ),
        )
        .with_child(SpanNode::new("auth", "auth/validate", 10.0, 200.0).with_color(Color::Magenta));
    let state = SpanTreeState::new(vec![root]).with_title("Trace");
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            SpanTree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_selection() {
    let root = SpanNode::new("req", "frontend/request", 0.0, 1000.0)
        .with_child(SpanNode::new("api", "api/handler", 50.0, 800.0))
        .with_child(SpanNode::new("auth", "auth/validate", 10.0, 200.0));
    let mut state = SpanTreeState::new(vec![root]).with_title("Trace");
    state.selected_index = Some(1); // select api/handler
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            SpanTree::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_collapsed() {
    let root = SpanNode::new("req", "frontend/request", 0.0, 1000.0)
        .with_child(SpanNode::new("api", "api/handler", 50.0, 800.0))
        .with_child(SpanNode::new("auth", "auth/validate", 10.0, 200.0));
    let mut state = SpanTreeState::new(vec![root]).with_title("Trace");
    state.collapse("req");
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            SpanTree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let root = SpanNode::new("req", "frontend/request", 0.0, 1000.0).with_child(SpanNode::new(
        "api",
        "api/handler",
        50.0,
        800.0,
    ));
    let state = SpanTreeState::new(vec![root]).with_title("Trace");
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            SpanTree::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_span() {
    let state = SpanTreeState::new(vec![SpanNode::new("r", "single-span", 0.0, 500.0)]);
    let (mut terminal, theme) = test_utils::setup_render(60, 8);
    terminal
        .draw(|frame| {
            SpanTree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_custom_label_width() {
    let root = SpanNode::new("req", "frontend/request", 0.0, 1000.0).with_child(SpanNode::new(
        "api",
        "api/handler",
        50.0,
        800.0,
    ));
    let state = SpanTreeState::new(vec![root])
        .with_title("Trace")
        .with_label_width(20);
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            SpanTree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
