use super::*;

#[test]
fn test_node_new() {
    let node = TreeNode::new("Label", "data");
    assert_eq!(node.label(), "Label");
    assert_eq!(node.data(), &"data");
    assert!(!node.is_expanded());
    assert!(!node.has_children());
}

#[test]
fn test_node_new_expanded() {
    let node: TreeNode<()> = TreeNode::new_expanded("Label", ());
    assert!(node.is_expanded());
}

#[test]
fn test_node_set_label() {
    let mut node = TreeNode::new("Old", ());
    node.set_label("New");
    assert_eq!(node.label(), "New");
}

#[test]
fn test_node_data_mut() {
    let mut node = TreeNode::new("Label", 42);
    *node.data_mut() = 100;
    assert_eq!(node.data(), &100);
}

#[test]
fn test_node_add_child() {
    let mut parent = TreeNode::new("Parent", ());
    parent.add_child(TreeNode::new("Child 1", ()));
    parent.add_child(TreeNode::new("Child 2", ()));

    assert!(parent.has_children());
    assert_eq!(parent.children().len(), 2);
}

#[test]
fn test_node_children_mut() {
    let mut parent = TreeNode::new("Parent", ());
    parent.add_child(TreeNode::new("Child", ()));

    parent.children_mut()[0].set_label("Modified");
    assert_eq!(parent.children()[0].label(), "Modified");
}

#[test]
fn test_node_expand_collapse() {
    let mut node = TreeNode::new("Node", ());
    assert!(!node.is_expanded());

    node.expand();
    assert!(node.is_expanded());

    node.collapse();
    assert!(!node.is_expanded());
}

#[test]
fn test_node_toggle() {
    let mut node = TreeNode::new("Node", ());
    assert!(!node.is_expanded());

    node.toggle();
    assert!(node.is_expanded());

    node.toggle();
    assert!(!node.is_expanded());
}

#[test]
fn test_node_set_expanded() {
    let mut node = TreeNode::new("Node", ());
    node.set_expanded(true);
    assert!(node.is_expanded());
    node.set_expanded(false);
    assert!(!node.is_expanded());
}

#[test]
fn test_node_clone() {
    let mut node = TreeNode::new("Parent", "data");
    node.add_child(TreeNode::new("Child", "child_data"));
    node.expand();

    let cloned = node.clone();
    assert_eq!(cloned.label(), "Parent");
    assert_eq!(cloned.data(), &"data");
    assert!(cloned.is_expanded());
    assert_eq!(cloned.children().len(), 1);
}
