use super::*;

// ========== TreeNode PartialEq Tests ==========

#[test]
fn test_node_equal() {
    let node1 = TreeNode::new("Label", 42);
    let node2 = TreeNode::new("Label", 42);
    assert_eq!(node1, node2);
}

#[test]
fn test_node_not_equal_label() {
    let node1 = TreeNode::new("Label A", 42);
    let node2 = TreeNode::new("Label B", 42);
    assert_ne!(node1, node2);
}

#[test]
fn test_node_not_equal_data() {
    let node1 = TreeNode::new("Label", 1);
    let node2 = TreeNode::new("Label", 2);
    assert_ne!(node1, node2);
}

#[test]
fn test_node_not_equal_expanded() {
    let node1 = TreeNode::new("Label", ());
    let node2 = TreeNode::new_expanded("Label", ());
    assert_ne!(node1, node2);
}

#[test]
fn test_node_not_equal_children() {
    let mut node1 = TreeNode::new("Parent", ());
    node1.add_child(TreeNode::new("Child A", ()));

    let mut node2 = TreeNode::new("Parent", ());
    node2.add_child(TreeNode::new("Child B", ()));

    assert_ne!(node1, node2);
}

#[test]
fn test_node_equal_with_children() {
    let mut node1 = TreeNode::new("Parent", ());
    node1.add_child(TreeNode::new("Child", ()));

    let mut node2 = TreeNode::new("Parent", ());
    node2.add_child(TreeNode::new("Child", ()));

    assert_eq!(node1, node2);
}

#[test]
fn test_node_not_equal_different_child_count() {
    let mut node1 = TreeNode::new("Parent", ());
    node1.add_child(TreeNode::new("Child 1", ()));

    let mut node2 = TreeNode::new("Parent", ());
    node2.add_child(TreeNode::new("Child 1", ()));
    node2.add_child(TreeNode::new("Child 2", ()));

    assert_ne!(node1, node2);
}

#[test]
fn test_node_equal_deep_nesting() {
    let mut child1 = TreeNode::new("Child", ());
    child1.add_child(TreeNode::new("Grandchild", ()));

    let mut parent1 = TreeNode::new("Parent", ());
    parent1.add_child(child1);

    let mut child2 = TreeNode::new("Child", ());
    child2.add_child(TreeNode::new("Grandchild", ()));

    let mut parent2 = TreeNode::new("Parent", ());
    parent2.add_child(child2);

    assert_eq!(parent1, parent2);
}

// ========== TreeState PartialEq Tests ==========

#[test]
fn test_state_equal() {
    let state1 = TreeState::new(vec![TreeNode::new("Root", 1)]);
    let state2 = TreeState::new(vec![TreeNode::new("Root", 1)]);
    assert_eq!(state1, state2);
}

#[test]
fn test_state_not_equal_roots() {
    let state1 = TreeState::new(vec![TreeNode::new("Root A", 1)]);
    let state2 = TreeState::new(vec![TreeNode::new("Root B", 1)]);
    assert_ne!(state1, state2);
}

#[test]
fn test_state_not_equal_selected_index() {
    let mut root1 = TreeNode::new_expanded("Root", ());
    root1.add_child(TreeNode::new("Child", ()));
    let mut root2 = TreeNode::new_expanded("Root", ());
    root2.add_child(TreeNode::new("Child", ()));

    let state1 = TreeState::new(vec![root1]).with_selected(0);
    let state2 = TreeState::new(vec![root2]).with_selected(1);
    assert_ne!(state1, state2);
}

#[test]
fn test_state_not_equal_filter_text() {
    let mut state1 = TreeState::new(vec![TreeNode::new("Root", ())]);
    let state2 = TreeState::new(vec![TreeNode::new("Root", ())]);

    state1.set_filter_text("abc");
    assert_ne!(state1, state2);
}

#[test]
fn test_state_equal_empty() {
    let state1: TreeState<()> = TreeState::new(Vec::new());
    let state2: TreeState<()> = TreeState::new(Vec::new());
    assert_eq!(state1, state2);
}

#[test]
fn test_state_equal_complex() {
    let mut root1 = TreeNode::new_expanded("Root", 1);
    root1.add_child(TreeNode::new("Child", 2));

    let mut root2 = TreeNode::new_expanded("Root", 1);
    root2.add_child(TreeNode::new("Child", 2));

    let state1 = TreeState::new(vec![root1]);
    let state2 = TreeState::new(vec![root2]);

    assert_eq!(state1, state2);
}
