use super::*;

#[test]
fn test_state_new() {
    let roots = vec![TreeNode::new("Root", ())];
    let state = TreeState::new(roots);

    assert_eq!(state.roots().len(), 1);
    assert_eq!(state.selected_index(), Some(0));
    assert!(!state.is_empty());
}

#[test]
fn test_state_new_empty() {
    let state: TreeState<()> = TreeState::new(Vec::new());
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_state_default() {
    let state: TreeState<()> = TreeState::default();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_state_set_roots() {
    let mut state: TreeState<i32> = TreeState::default();
    state.set_roots(vec![TreeNode::new("Root", 1)]);

    assert_eq!(state.roots().len(), 1);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_state_set_roots_to_empty() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    state.set_roots(Vec::new());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_state_roots_mut() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    state.roots_mut()[0].set_label("Modified");

    assert_eq!(state.roots()[0].label(), "Modified");
}

#[test]
fn test_state_flatten_single() {
    let state = TreeState::new(vec![TreeNode::new("Root", ())]);
    let flat = state.flatten();

    assert_eq!(flat.len(), 1);
    assert_eq!(flat[0].label, "Root");
    assert_eq!(flat[0].depth, 0);
}

#[test]
fn test_state_flatten_with_children() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));

    let state = TreeState::new(vec![root]);
    let flat = state.flatten();

    assert_eq!(flat.len(), 3);
    assert_eq!(flat[0].label, "Root");
    assert_eq!(flat[0].depth, 0);
    assert_eq!(flat[1].label, "Child 1");
    assert_eq!(flat[1].depth, 1);
    assert_eq!(flat[2].label, "Child 2");
    assert_eq!(flat[2].depth, 1);
}

#[test]
fn test_state_flatten_collapsed() {
    let mut root = TreeNode::new("Root", ()); // Not expanded
    root.add_child(TreeNode::new("Child", ()));

    let state = TreeState::new(vec![root]);
    let flat = state.flatten();

    // Children not visible when collapsed
    assert_eq!(flat.len(), 1);
}

#[test]
fn test_state_selected_path() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.selected_path(), Some(vec![0]));

    state.selected_index = Some(1);
    assert_eq!(state.selected_path(), Some(vec![0, 0]));
}

#[test]
fn test_state_selected_path_empty() {
    let state: TreeState<()> = TreeState::new(Vec::new());
    assert_eq!(state.selected_path(), None);
}

#[test]
fn test_state_selected_node() {
    let mut root = TreeNode::new_expanded("Root", "root_data");
    root.add_child(TreeNode::new("Child", "child_data"));

    let mut state = TreeState::new(vec![root]);

    let selected = state.selected_node();
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().data(), &"root_data");

    state.selected_index = Some(1);
    let selected = state.selected_node();
    assert_eq!(selected.unwrap().data(), &"child_data");
}

#[test]
fn test_state_selected_node_empty() {
    let state: TreeState<()> = TreeState::new(Vec::new());
    assert!(state.selected_node().is_none());
}

#[test]
fn test_state_expand_all() {
    let mut root = TreeNode::new("Root", ());
    let mut child = TreeNode::new("Child", ());
    child.add_child(TreeNode::new("Grandchild", ()));
    root.add_child(child);

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.visible_count(), 1);

    state.expand_all();
    assert_eq!(state.visible_count(), 3);
}

#[test]
fn test_state_collapse_all() {
    let mut root = TreeNode::new_expanded("Root", ());
    let mut child = TreeNode::new_expanded("Child", ());
    child.add_child(TreeNode::new("Grandchild", ()));
    root.add_child(child);

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.visible_count(), 3);

    state.collapse_all();
    assert_eq!(state.visible_count(), 1);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_state_collapse_all_empty() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());
    state.collapse_all();
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_state_visible_count() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));

    let state = TreeState::new(vec![root]);
    assert_eq!(state.visible_count(), 3);
}
