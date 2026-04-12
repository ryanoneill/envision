use super::*;
use crate::component::test_utils;

#[test]
fn test_snapshot_empty() {
    let state = TreemapState::new();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_simple() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("src", 60.0).with_color(Color::Blue))
        .with_child(TreemapNode::new("docs", 30.0).with_color(Color::Green))
        .with_child(TreemapNode::new("tests", 10.0).with_color(Color::Yellow));
    let state = TreemapState::new().with_root(root);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("alpha", 60.0).with_color(Color::Red))
        .with_child(TreemapNode::new("beta", 40.0).with_color(Color::Blue));
    let state = TreemapState::new()
        .with_root(root)
        .with_title("Focused Treemap");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("a", 50.0).with_color(Color::Red))
        .with_child(TreemapNode::new("b", 50.0).with_color(Color::Blue));
    let state = TreemapState::new().with_root(root).with_title("Disabled");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).disabled(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_values() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("big", 70.0).with_color(Color::Cyan))
        .with_child(TreemapNode::new("med", 20.0).with_color(Color::Magenta))
        .with_child(TreemapNode::new("small", 10.0).with_color(Color::Yellow));
    let state = TreemapState::new()
        .with_root(root)
        .with_show_values(true)
        .with_title("Values");
    let (mut terminal, theme) = test_utils::setup_render(40, 12);
    terminal
        .draw(|frame| {
            Treemap::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_zoomed_in() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(
            TreemapNode::new("src", 0.0)
                .with_color(Color::Blue)
                .with_child(TreemapNode::new("main.rs", 30.0).with_color(Color::Cyan))
                .with_child(TreemapNode::new("lib.rs", 20.0).with_color(Color::LightBlue)),
        )
        .with_child(TreemapNode::new("docs", 10.0).with_color(Color::Green));
    let mut state = TreemapState::new().with_root(root).with_title("Zoomed");
    state.update(TreemapMessage::ZoomIn); // Zoom into "src".
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_selection_moved() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("first", 40.0).with_color(Color::Red))
        .with_child(TreemapNode::new("second", 30.0).with_color(Color::Green))
        .with_child(TreemapNode::new("third", 30.0).with_color(Color::Blue));
    let mut state = TreemapState::new().with_root(root);
    state.update(TreemapMessage::SelectNext); // Select "second".
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_child() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("only", 100.0).with_color(Color::Magenta));
    let state = TreemapState::new().with_root(root).with_title("Single");
    let (mut terminal, theme) = test_utils::setup_render(30, 8);
    terminal
        .draw(|frame| {
            Treemap::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
