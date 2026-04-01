use super::*;
use crate::component::test_utils;

// =============================================================================
// Snapshot tests
// =============================================================================

#[test]
fn test_snapshot_empty() {
    let state: TreeState<()> = TreeState::new(vec![]);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_single_root() {
    let state = TreeState::new(vec![TreeNode::new("Documents", ())]);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_expanded_with_children() {
    let mut root = TreeNode::new_expanded("Projects", ());
    root.add_child(TreeNode::new("envision", ()));
    root.add_child(TreeNode::new("other", ()));
    let state = TreeState::new(vec![root]);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_collapsed_with_children() {
    let mut root = TreeNode::new("Projects", ());
    root.add_child(TreeNode::new("envision", ()));
    root.add_child(TreeNode::new("other", ()));
    let state = TreeState::new(vec![root]);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_focused_selected() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));
    root.add_child(TreeNode::new("Child 3", ()));
    let mut state = TreeState::new(vec![root]);
    state.set_focused(true);
    state.set_selected(Some(2));
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_deep_nesting() {
    let mut level1 = TreeNode::new_expanded("Level 1", ());
    let mut level2 = TreeNode::new_expanded("Level 2", ());
    let mut level3 = TreeNode::new_expanded("Level 3", ());
    level3.add_child(TreeNode::new("Leaf", ()));
    level2.add_child(level3);
    level1.add_child(level2);
    let state = TreeState::new(vec![level1]);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_disabled() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));
    let mut state = TreeState::new(vec![root]);
    state.set_disabled(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_mixed_expanded_collapsed() {
    let mut docs = TreeNode::new_expanded("Documents", ());
    docs.add_child(TreeNode::new("readme.md", ()));
    docs.add_child(TreeNode::new("guide.md", ()));
    let mut projects = TreeNode::new("Projects", ());
    projects.add_child(TreeNode::new("envision", ()));
    let downloads = TreeNode::new("Downloads", ());
    let state = TreeState::new(vec![docs, projects, downloads]);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
