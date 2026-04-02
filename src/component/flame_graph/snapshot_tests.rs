use super::*;
use crate::component::test_utils;
use ratatui::style::Color;

#[test]
fn test_snapshot_empty() {
    let state = FlameGraphState::new();
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            FlameGraph::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_frame() {
    let state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    let (mut terminal, theme) = test_utils::setup_render(60, 8);
    terminal
        .draw(|frame| {
            FlameGraph::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_simple_tree() {
    let root = FlameNode::new("main()", 500)
        .with_color(Color::Red)
        .with_child(
            FlameNode::new("compute()", 300)
                .with_color(Color::Yellow)
                .with_child(FlameNode::new("sort()", 150).with_color(Color::Green))
                .with_child(FlameNode::new("hash()", 100).with_color(Color::Blue)),
        )
        .with_child(FlameNode::new("io()", 100).with_color(Color::Magenta));
    let state = FlameGraphState::with_root(root).with_title("Flame Graph");
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            FlameGraph::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_with_selection() {
    let root = FlameNode::new("main()", 500)
        .with_color(Color::Red)
        .with_child(FlameNode::new("compute()", 300).with_color(Color::Yellow))
        .with_child(FlameNode::new("io()", 100).with_color(Color::Magenta));
    let mut state = FlameGraphState::with_root(root).with_title("Flame Graph");
    state.set_focused(true);
    // Select compute() at depth 1
    state.select_down();
    let (mut terminal, theme) = test_utils::setup_render(60, 8);
    terminal
        .draw(|frame| {
            FlameGraph::view(
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
fn test_snapshot_zoomed() {
    let root = FlameNode::new("main()", 500)
        .with_color(Color::Red)
        .with_child(
            FlameNode::new("compute()", 300)
                .with_color(Color::Yellow)
                .with_child(FlameNode::new("sort()", 150).with_color(Color::Green))
                .with_child(FlameNode::new("hash()", 100).with_color(Color::Blue)),
        )
        .with_child(FlameNode::new("io()", 100).with_color(Color::Magenta));
    let mut state = FlameGraphState::with_root(root).with_title("Flame Graph");
    state.set_focused(true);
    state.select_down(); // select compute()
    state.zoom_in(); // zoom into compute()
    let (mut terminal, theme) = test_utils::setup_render(60, 8);
    terminal
        .draw(|frame| {
            FlameGraph::view(
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
fn test_snapshot_disabled() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let state = FlameGraphState::with_root(root)
        .with_title("Flame Graph")
        .with_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(60, 8);
    terminal
        .draw(|frame| {
            FlameGraph::view(
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
fn test_snapshot_with_search() {
    let root = FlameNode::new("main()", 500)
        .with_color(Color::Red)
        .with_child(
            FlameNode::new("compute()", 300)
                .with_color(Color::Yellow)
                .with_child(FlameNode::new("sort()", 150).with_color(Color::Green))
                .with_child(FlameNode::new("hash()", 100).with_color(Color::Blue)),
        )
        .with_child(FlameNode::new("io()", 100).with_color(Color::Magenta));
    let mut state = FlameGraphState::with_root(root).with_title("Flame Graph");
    state.set_search("sort".to_string());
    let (mut terminal, theme) = test_utils::setup_render(60, 10);
    terminal
        .draw(|frame| {
            FlameGraph::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_deep_nesting() {
    let root = FlameNode::new("main()", 1000)
        .with_color(Color::Red)
        .with_child(
            FlameNode::new("a()", 800)
                .with_color(Color::Yellow)
                .with_child(
                    FlameNode::new("b()", 600)
                        .with_color(Color::Green)
                        .with_child(
                            FlameNode::new("c()", 400)
                                .with_color(Color::Blue)
                                .with_child(FlameNode::new("d()", 200).with_color(Color::Magenta)),
                        ),
                ),
        );
    let state = FlameGraphState::with_root(root).with_title("Deep Profile");
    let (mut terminal, theme) = test_utils::setup_render(60, 12);
    terminal
        .draw(|frame| {
            FlameGraph::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
