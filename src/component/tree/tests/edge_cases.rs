use super::*;

// ========== Single Node Edge Cases ==========

#[test]
fn test_single_node_navigate_down_stays() {
    let mut state = TreeState::new(vec![TreeNode::new("Only", ())]);
    let output = Tree::update(&mut state, TreeMessage::Down);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_single_node_navigate_up_stays() {
    let mut state = TreeState::new(vec![TreeNode::new("Only", ())]);
    let output = Tree::update(&mut state, TreeMessage::Up);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_single_node_toggle_no_children() {
    let mut state = TreeState::new(vec![TreeNode::new("Leaf", ())]);
    let output = Tree::update(&mut state, TreeMessage::Toggle);
    assert_eq!(output, None);
}

#[test]
fn test_single_node_select() {
    let mut state = TreeState::new(vec![TreeNode::new("Only", "data")]);
    let output = Tree::update(&mut state, TreeMessage::Select);
    assert_eq!(output, Some(TreeOutput::Selected(vec![0])));
}

#[test]
fn test_single_node_expand_all() {
    let mut state = TreeState::new(vec![TreeNode::new("Leaf", ())]);
    Tree::update(&mut state, TreeMessage::ExpandAll);
    // Leaf node should not be expanded (no children)
    assert!(!state.roots()[0].is_expanded());
}

#[test]
fn test_single_node_collapse_all() {
    let mut state = TreeState::new(vec![TreeNode::new("Leaf", ())]);
    Tree::update(&mut state, TreeMessage::CollapseAll);
    assert_eq!(state.selected_index(), Some(0));
}

// ========== Empty Tree Edge Cases ==========

#[test]
fn test_empty_tree_expand() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());
    let output = Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(output, None);
}

#[test]
fn test_empty_tree_collapse() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());
    let output = Tree::update(&mut state, TreeMessage::Collapse);
    assert_eq!(output, None);
}

#[test]
fn test_empty_tree_toggle() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());
    let output = Tree::update(&mut state, TreeMessage::Toggle);
    assert_eq!(output, None);
}

#[test]
fn test_empty_tree_up() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());
    let output = Tree::update(&mut state, TreeMessage::Up);
    assert_eq!(output, None);
}

#[test]
fn test_empty_tree_expand_all() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());
    let output = Tree::update(&mut state, TreeMessage::ExpandAll);
    assert_eq!(output, None);
}

#[test]
fn test_empty_tree_collapse_all() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());
    let output = Tree::update(&mut state, TreeMessage::CollapseAll);
    assert_eq!(output, None);
}

#[test]
fn test_empty_tree_selected_path() {
    let state: TreeState<()> = TreeState::new(Vec::new());
    assert_eq!(state.selected_path(), None);
}

#[test]
fn test_empty_tree_selected_node() {
    let state: TreeState<()> = TreeState::new(Vec::new());
    assert_eq!(state.selected_node(), None);
}

#[test]
fn test_empty_tree_selected_item() {
    let state: TreeState<()> = TreeState::new(Vec::new());
    assert_eq!(state.selected_item(), None);
}

#[test]
fn test_empty_tree_visible_count() {
    let state: TreeState<()> = TreeState::new(Vec::new());
    assert_eq!(state.visible_count(), 0);
}

// ========== Boundary Selection Tests ==========

#[test]
fn test_with_selected_zero_on_collapsed_with_children() {
    let mut root = TreeNode::new("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let state = TreeState::new(vec![root]).with_selected(0);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_node().unwrap().label(), "Root");
}

#[test]
fn test_navigate_boundary_down_then_up() {
    let nodes = vec![TreeNode::new("A", ()), TreeNode::new("B", ())];
    let mut state = TreeState::new(nodes);

    // At start (0), go down to 1
    Tree::<()>::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_index(), Some(1));

    // At end (1), go down again - stay at 1
    Tree::<()>::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_index(), Some(1));

    // Go up to 0
    Tree::<()>::update(&mut state, TreeMessage::Up);
    assert_eq!(state.selected_index(), Some(0));

    // At start (0), go up again - stay at 0
    Tree::<()>::update(&mut state, TreeMessage::Up);
    assert_eq!(state.selected_index(), Some(0));
}

// ========== Collapse Adjusts Selection Boundary Tests ==========

#[test]
fn test_collapse_when_selected_child_is_beyond_new_range() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));
    root.add_child(TreeNode::new("Child 3", ()));

    let second_root = TreeNode::new("Other", ());

    let mut state = TreeState::new(vec![root, second_root]);
    // Visible: Root(0), Child 1(1), Child 2(2), Child 3(3), Other(4)
    assert_eq!(state.visible_count(), 5);

    // Select Root and collapse
    state.selected_index = Some(0);
    Tree::update(&mut state, TreeMessage::Collapse);

    // After collapse: Root(0), Other(1)
    assert_eq!(state.visible_count(), 2);
    assert!(state.selected_index().unwrap() < state.visible_count());
}

// ========== Expand/Collapse Sequences ==========

#[test]
fn test_expand_collapse_expand_cycle() {
    let mut root = TreeNode::new("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.visible_count(), 1);

    // Expand
    let output = Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(output, Some(TreeOutput::Expanded(vec![0])));
    assert_eq!(state.visible_count(), 2);

    // Collapse
    let output = Tree::update(&mut state, TreeMessage::Collapse);
    assert_eq!(output, Some(TreeOutput::Collapsed(vec![0])));
    assert_eq!(state.visible_count(), 1);

    // Expand again
    let output = Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(output, Some(TreeOutput::Expanded(vec![0])));
    assert_eq!(state.visible_count(), 2);
}

#[test]
fn test_toggle_twice_returns_to_original_state() {
    let mut root = TreeNode::new("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    let original_expanded = state.roots()[0].is_expanded();
    let original_visible = state.visible_count();

    Tree::update(&mut state, TreeMessage::Toggle);
    Tree::update(&mut state, TreeMessage::Toggle);

    assert_eq!(state.roots()[0].is_expanded(), original_expanded);
    assert_eq!(state.visible_count(), original_visible);
}

// ========== SetFilter/ClearFilter Message Edge Cases ==========

#[test]
fn test_set_filter_empty_string_via_message() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    state.set_filter_text("something");

    let output = Tree::update(&mut state, TreeMessage::SetFilter(String::new()));
    assert_eq!(state.filter_text(), "");
    assert_eq!(output, Some(TreeOutput::FilterChanged(String::new())));
}

#[test]
fn test_clear_filter_when_already_empty() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    assert_eq!(state.filter_text(), "");

    let output = Tree::update(&mut state, TreeMessage::ClearFilter);
    assert_eq!(output, Some(TreeOutput::FilterChanged(String::new())));
    assert_eq!(state.filter_text(), "");
}

#[test]
fn test_set_filter_allowed_when_disabled() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    state.set_disabled(true);

    let output = Tree::update(&mut state, TreeMessage::SetFilter("test".into()));
    assert_eq!(output, Some(TreeOutput::FilterChanged("test".into())));
    assert_eq!(state.filter_text(), "test");
}

#[test]
fn test_clear_filter_allowed_when_disabled() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    state.set_disabled(true);
    state.set_filter_text("test");

    let output = Tree::update(&mut state, TreeMessage::ClearFilter);
    assert_eq!(output, Some(TreeOutput::FilterChanged(String::new())));
    assert_eq!(state.filter_text(), "");
}

// ========== Focusable Trait Tests ==========

#[test]
fn test_focus_sets_focused() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    assert!(!Tree::<()>::is_focused(&state));

    Tree::focus(&mut state);
    assert!(Tree::<()>::is_focused(&state));
    assert!(state.is_focused());
}

#[test]
fn test_blur_unsets_focused() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    Tree::focus(&mut state);
    assert!(state.is_focused());

    Tree::blur(&mut state);
    assert!(!Tree::<()>::is_focused(&state));
    assert!(!state.is_focused());
}

#[test]
fn test_set_focused_via_trait() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    Tree::<()>::set_focused(&mut state, true);
    assert!(Tree::<()>::is_focused(&state));

    Tree::<()>::set_focused(&mut state, false);
    assert!(!Tree::<()>::is_focused(&state));
}

// ========== Node without Children Expand/Collapse Tests ==========

#[test]
fn test_expand_leaf_via_dispatch_event() {
    let mut state = TreeState::new(vec![TreeNode::new("Leaf", ())]);
    state.set_focused(true);

    let output = Tree::<()>::dispatch_event(&mut state, &Event::key(KeyCode::Right));
    assert_eq!(output, None);
}

#[test]
fn test_collapse_leaf_via_dispatch_event() {
    let mut state = TreeState::new(vec![TreeNode::new("Leaf", ())]);
    state.set_focused(true);

    let output = Tree::<()>::dispatch_event(&mut state, &Event::key(KeyCode::Left));
    assert_eq!(output, None);
}

// ========== Unhandled Key Tests ==========

#[test]
fn test_handle_event_unrecognized_key() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    state.set_focused(true);

    let msg = Tree::<()>::handle_event(&state, &Event::char('z'));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_tab_key() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    state.set_focused(true);

    let msg = Tree::<()>::handle_event(&state, &Event::key(KeyCode::Tab));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_escape_key() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    state.set_focused(true);

    let msg = Tree::<()>::handle_event(&state, &Event::key(KeyCode::Esc));
    assert_eq!(msg, None);
}

// ========== Multiple Roots Navigation ==========

#[test]
fn test_navigate_across_roots() {
    let root1 = TreeNode::new("Root 1", "r1");
    let root2 = TreeNode::new("Root 2", "r2");
    let root3 = TreeNode::new("Root 3", "r3");

    let mut state = TreeState::new(vec![root1, root2, root3]);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_node().unwrap().data(), &"r1");

    Tree::<&str>::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_node().unwrap().data(), &"r2");

    Tree::<&str>::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_node().unwrap().data(), &"r3");

    // At end
    Tree::<&str>::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_node().unwrap().data(), &"r3");
}

#[test]
fn test_select_across_expanded_roots() {
    let mut root1 = TreeNode::new_expanded("Root 1", "r1");
    root1.add_child(TreeNode::new("R1 Child", "r1c"));

    let mut root2 = TreeNode::new_expanded("Root 2", "r2");
    root2.add_child(TreeNode::new("R2 Child", "r2c"));

    let mut state = TreeState::new(vec![root1, root2]);
    // Visible: Root 1(0), R1 Child(1), Root 2(2), R2 Child(3)
    assert_eq!(state.visible_count(), 4);

    // Navigate to R2 Child
    Tree::<&str>::update(&mut state, TreeMessage::Down); // R1 Child
    Tree::<&str>::update(&mut state, TreeMessage::Down); // Root 2
    Tree::<&str>::update(&mut state, TreeMessage::Down); // R2 Child

    let output = Tree::<&str>::update(&mut state, TreeMessage::Select);
    assert_eq!(output, Some(TreeOutput::Selected(vec![1, 0])));
    assert_eq!(state.selected_node().unwrap().data(), &"r2c");
}

// ========== Expand All / Collapse All with Deep Trees ==========

#[test]
fn test_expand_all_deep_tree() {
    let mut level1 = TreeNode::new("L1", ());
    let mut level2 = TreeNode::new("L2", ());
    let mut level3 = TreeNode::new("L3", ());
    level3.add_child(TreeNode::new("L4", ()));
    level2.add_child(level3);
    level1.add_child(level2);

    let mut state = TreeState::new(vec![level1]);
    assert_eq!(state.visible_count(), 1); // Only L1

    state.expand_all();
    assert_eq!(state.visible_count(), 4); // L1, L2, L3, L4
    assert!(state.roots()[0].is_expanded());
    assert!(state.roots()[0].children()[0].is_expanded());
    assert!(state.roots()[0].children()[0].children()[0].is_expanded());
}

#[test]
fn test_collapse_all_deep_tree() {
    let mut level1 = TreeNode::new_expanded("L1", ());
    let mut level2 = TreeNode::new_expanded("L2", ());
    let mut level3 = TreeNode::new_expanded("L3", ());
    level3.add_child(TreeNode::new("L4", ()));
    level2.add_child(level3);
    level1.add_child(level2);

    let mut state = TreeState::new(vec![level1]);
    assert_eq!(state.visible_count(), 4);

    state.collapse_all();
    assert_eq!(state.visible_count(), 1);
    assert!(!state.roots()[0].is_expanded());
    assert!(!state.roots()[0].children()[0].is_expanded());
    assert!(!state.roots()[0].children()[0].children()[0].is_expanded());
    assert_eq!(state.selected_index(), Some(0)); // Reset to first
}

// ========== set_roots Edge Cases ==========

#[test]
fn test_set_roots_replaces_tree() {
    let mut state = TreeState::new(vec![TreeNode::new("Old", 1)]);
    assert_eq!(state.roots()[0].label(), "Old");

    state.set_roots(vec![TreeNode::new("New A", 2), TreeNode::new("New B", 3)]);
    assert_eq!(state.roots().len(), 2);
    assert_eq!(state.roots()[0].label(), "New A");
    assert_eq!(state.roots()[1].label(), "New B");
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_set_roots_clears_filter_text() {
    let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    state.set_filter_text("filter");
    assert_eq!(state.filter_text(), "filter");

    state.set_roots(vec![TreeNode::new("New Root", ())]);
    assert_eq!(state.filter_text(), "");
}

// ========== selected_item is alias of selected_node ==========

#[test]
fn test_selected_item_equals_selected_node() {
    let mut root = TreeNode::new_expanded("Root", 10);
    root.add_child(TreeNode::new("Child", 20));

    let mut state = TreeState::new(vec![root]);

    // At root
    assert_eq!(
        state.selected_item().map(|n| n.label()),
        state.selected_node().map(|n| n.label())
    );
    assert_eq!(
        state.selected_item().map(|n| n.data()),
        state.selected_node().map(|n| n.data())
    );

    // At child
    state.selected_index = Some(1);
    assert_eq!(
        state.selected_item().map(|n| n.label()),
        state.selected_node().map(|n| n.label())
    );
}

#[test]
fn test_selected_item_empty_tree() {
    let state: TreeState<()> = TreeState::new(Vec::new());
    assert_eq!(state.selected_item(), None);
    assert_eq!(state.selected_node(), None);
}

// ========== Default Trait ==========

#[test]
fn test_default_is_empty() {
    let state: TreeState<String> = TreeState::default();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.visible_count(), 0);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert_eq!(state.filter_text(), "");
}

// ========== Node with Empty Label ==========

#[test]
fn test_node_empty_label() {
    let node = TreeNode::new("", 42);
    assert_eq!(node.label(), "");
    assert_eq!(node.data(), &42);
}

#[test]
fn test_tree_with_empty_labels() {
    let mut state = TreeState::new(vec![TreeNode::new("", 1), TreeNode::new("", 2)]);
    assert_eq!(state.visible_count(), 2);

    Tree::<i32>::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_node().unwrap().data(), &2);
}

// ========== Navigation After Expand/Collapse ==========

#[test]
fn test_navigate_after_expand() {
    let mut root = TreeNode::new("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.visible_count(), 1);

    // Expand root
    Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(state.visible_count(), 3);

    // Navigate through newly visible children
    Tree::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_node().unwrap().label(), "Child 1");

    Tree::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_node().unwrap().label(), "Child 2");
}

#[test]
fn test_navigate_after_collapse_resets_if_needed() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));

    let second = TreeNode::new("Second", ());

    let mut state = TreeState::new(vec![root, second]);
    // Visible: Root(0), Child 1(1), Child 2(2), Second(3)

    // Navigate to Second
    state.selected_index = Some(3);
    assert_eq!(state.selected_node().unwrap().label(), "Second");

    // Go back to Root and collapse
    state.selected_index = Some(0);
    Tree::update(&mut state, TreeMessage::Collapse);
    // Visible: Root(0), Second(1)
    assert_eq!(state.visible_count(), 2);
    assert!(state.selected_index().unwrap() < state.visible_count());
}
