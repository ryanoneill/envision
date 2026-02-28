use super::*;

// TreeNode tests

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

// TreeState tests

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

#[test]
fn test_state_clone() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    state.selected_index = Some(1);

    let cloned = state.clone();
    assert_eq!(cloned.visible_count(), 2);
    assert_eq!(cloned.selected_index(), Some(1));
}

// Tree component tests

#[test]
fn test_init() {
    let state: TreeState<()> = Tree::init();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_select_next() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.selected_index(), Some(0));

    Tree::update(&mut state, TreeMessage::SelectNext);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_select_next_at_end() {
    let state_roots = vec![TreeNode::new("Root", ())];
    let mut state = TreeState::new(state_roots);

    Tree::<()>::update(&mut state, TreeMessage::SelectNext);
    assert_eq!(state.selected_index(), Some(0)); // Stays at 0
}

#[test]
fn test_select_previous() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    state.selected_index = Some(1);

    Tree::update(&mut state, TreeMessage::SelectPrevious);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_select_previous_at_start() {
    let state_roots = vec![TreeNode::new("Root", ())];
    let mut state = TreeState::new(state_roots);

    Tree::<()>::update(&mut state, TreeMessage::SelectPrevious);
    assert_eq!(state.selected_index(), Some(0)); // Stays at 0
}

#[test]
fn test_expand() {
    let mut root = TreeNode::new("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.visible_count(), 1);

    let output = Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(output, Some(TreeOutput::Expanded(vec![0])));
    assert_eq!(state.visible_count(), 2);
}

#[test]
fn test_expand_already_expanded() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);

    let output = Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(output, None); // Already expanded
}

#[test]
fn test_expand_no_children() {
    let root = TreeNode::new("Leaf", ());
    let mut state = TreeState::new(vec![root]);

    let output = Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(output, None); // No children to expand
}

#[test]
fn test_collapse() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.visible_count(), 2);

    let output = Tree::update(&mut state, TreeMessage::Collapse);
    assert_eq!(output, Some(TreeOutput::Collapsed(vec![0])));
    assert_eq!(state.visible_count(), 1);
}

#[test]
fn test_collapse_already_collapsed() {
    let mut root = TreeNode::new("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);

    let output = Tree::update(&mut state, TreeMessage::Collapse);
    assert_eq!(output, None); // Already collapsed
}

#[test]
fn test_collapse_adjusts_selection() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    state.selected_index = Some(1); // Select child

    Tree::update(&mut state, TreeMessage::SelectPrevious); // Go to root
    Tree::update(&mut state, TreeMessage::Collapse);

    // Selection should still be valid
    assert!(state.selected_index().unwrap() < state.visible_count());
}

#[test]
fn test_toggle_expand() {
    let mut root = TreeNode::new("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);

    let output = Tree::update(&mut state, TreeMessage::Toggle);
    assert_eq!(output, Some(TreeOutput::Expanded(vec![0])));
    assert!(state.roots()[0].is_expanded());
}

#[test]
fn test_toggle_collapse() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);

    let output = Tree::update(&mut state, TreeMessage::Toggle);
    assert_eq!(output, Some(TreeOutput::Collapsed(vec![0])));
    assert!(!state.roots()[0].is_expanded());
}

#[test]
fn test_toggle_no_children() {
    let root = TreeNode::new("Leaf", ());
    let mut state = TreeState::new(vec![root]);

    let output = Tree::update(&mut state, TreeMessage::Toggle);
    assert_eq!(output, None);
}

#[test]
fn test_select() {
    let root = TreeNode::new("Root", "data");
    let mut state = TreeState::new(vec![root]);

    let output = Tree::update(&mut state, TreeMessage::Select);
    assert_eq!(output, Some(TreeOutput::Selected(vec![0])));
}

#[test]
fn test_select_child() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    state.selected_index = Some(1);

    let output = Tree::update(&mut state, TreeMessage::Select);
    assert_eq!(output, Some(TreeOutput::Selected(vec![0, 0])));
}

#[test]
fn test_expand_all_message() {
    let mut root = TreeNode::new("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.visible_count(), 1);

    Tree::update(&mut state, TreeMessage::ExpandAll);
    assert_eq!(state.visible_count(), 2);
}

#[test]
fn test_collapse_all_message() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);

    Tree::update(&mut state, TreeMessage::CollapseAll);
    assert_eq!(state.visible_count(), 1);
}

#[test]
fn test_empty_tree() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());

    // Should not panic
    let output = Tree::update(&mut state, TreeMessage::SelectNext);
    assert_eq!(output, None);

    let output = Tree::update(&mut state, TreeMessage::Select);
    assert_eq!(output, None);
}

// Focusable tests

#[test]
fn test_focusable_is_focused() {
    let state: TreeState<()> = TreeState::new(Vec::new());
    assert!(!Tree::is_focused(&state));
}

#[test]
fn test_focusable_set_focused() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());

    Tree::set_focused(&mut state, true);
    assert!(Tree::is_focused(&state));

    Tree::set_focused(&mut state, false);
    assert!(!Tree::is_focused(&state));
}

#[test]
fn test_focusable_focus() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());

    Tree::focus(&mut state);
    assert!(Tree::is_focused(&state));
}

#[test]
fn test_focusable_blur() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());
    Tree::set_focused(&mut state, true);

    Tree::blur(&mut state);
    assert!(!Tree::is_focused(&state));
}

// View tests

#[test]
fn test_view_empty() {
    let state: TreeState<()> = TreeState::new(Vec::new());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_single_node() {
    let root = TreeNode::new("Root", ());
    let state = TreeState::new(vec![root]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_children() {
    let mut root = TreeNode::new_expanded("Parent", ());
    root.add_child(TreeNode::new("Child", ()));

    let state = TreeState::new(vec![root]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_collapsed_indicator() {
    let mut root = TreeNode::new("Root", ()); // Collapsed
    root.add_child(TreeNode::new("Child", ()));

    let state = TreeState::new(vec![root]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_expanded_indicator() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let state = TreeState::new(vec![root]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// Integration tests

#[test]
fn test_file_tree_workflow() {
    // Simulate a file browser
    let mut src = TreeNode::new("src", "/src");
    src.add_child(TreeNode::new("main.rs", "/src/main.rs"));
    src.add_child(TreeNode::new("lib.rs", "/src/lib.rs"));

    let mut tests = TreeNode::new("tests", "/tests");
    tests.add_child(TreeNode::new("test_main.rs", "/tests/test_main.rs"));

    let mut root = TreeNode::new_expanded("project", "/project");
    root.add_child(src);
    root.add_child(tests);
    root.add_child(TreeNode::new("Cargo.toml", "/project/Cargo.toml"));

    let mut state = TreeState::new(vec![root]);
    Tree::focus(&mut state);

    // Navigate to src
    Tree::update(&mut state, TreeMessage::SelectNext);
    assert_eq!(state.selected_node().unwrap().label(), "src");

    // Expand src
    Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(state.visible_count(), 6); // project, src, main.rs, lib.rs, tests, Cargo.toml

    // Navigate to main.rs and select
    Tree::update(&mut state, TreeMessage::SelectNext);
    let output = Tree::update(&mut state, TreeMessage::Select);
    assert_eq!(output, Some(TreeOutput::Selected(vec![0, 0, 0])));
    assert_eq!(state.selected_node().unwrap().data(), &"/src/main.rs");
}

#[test]
fn test_deep_nesting() {
    let mut level1 = TreeNode::new_expanded("Level 1", 1);
    let mut level2 = TreeNode::new_expanded("Level 2", 2);
    let mut level3 = TreeNode::new_expanded("Level 3", 3);
    level3.add_child(TreeNode::new("Level 4", 4));
    level2.add_child(level3);
    level1.add_child(level2);

    let state = TreeState::new(vec![level1]);
    assert_eq!(state.visible_count(), 4);

    // Check paths
    let flat = state.flatten();
    assert_eq!(flat[0].path, vec![0]);
    assert_eq!(flat[1].path, vec![0, 0]);
    assert_eq!(flat[2].path, vec![0, 0, 0]);
    assert_eq!(flat[3].path, vec![0, 0, 0, 0]);
}

#[test]
fn test_multiple_roots() {
    let root1 = TreeNode::new("Root 1", ());
    let root2 = TreeNode::new("Root 2", ());
    let root3 = TreeNode::new("Root 3", ());

    let state = TreeState::new(vec![root1, root2, root3]);
    assert_eq!(state.visible_count(), 3);

    let flat = state.flatten();
    assert_eq!(flat[0].path, vec![0]);
    assert_eq!(flat[1].path, vec![1]);
    assert_eq!(flat[2].path, vec![2]);
}

#[test]
fn test_view_focused_selection() {
    let root = TreeNode::new("Root", ());
    let mut state = TreeState::new(vec![root]);
    state.focused = true; // Set focused for different highlight style

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_unfocused_selection() {
    let root = TreeNode::new("Root", ());
    let mut state = TreeState::new(vec![root]);
    state.focused = false; // Unfocused state

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_collapse_with_child_selected() {
    // When collapsing a node while a child is selected, selection should adjust
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));

    let mut state = TreeState::new(vec![root]);
    // Select the last child
    state.selected_index = Some(2);

    // Navigate back to root and collapse
    state.selected_index = Some(0);
    let output = Tree::update(&mut state, TreeMessage::Collapse);

    assert_eq!(output, Some(TreeOutput::Collapsed(vec![0])));
    // Selection should still be valid
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_toggle_collapse_adjusts_selection() {
    // When toggling to collapse while a child is selected beyond the new range
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));
    root.add_child(TreeNode::new("Child 3", ()));

    let mut state = TreeState::new(vec![root]);
    // Select the last child (index 3)
    state.selected_index = Some(3);

    // Navigate to root and toggle (collapse)
    state.selected_index = Some(0);
    let output = Tree::update(&mut state, TreeMessage::Toggle);

    assert_eq!(output, Some(TreeOutput::Collapsed(vec![0])));
    // Selection should be clamped to valid range
    assert!(state.selected_index().unwrap() < state.visible_count());
}

#[test]
fn test_get_node_deep_path() {
    let mut root = TreeNode::new_expanded("Root", 0);
    let mut child = TreeNode::new_expanded("Child", 1);
    child.add_child(TreeNode::new("Grandchild", 2));
    root.add_child(child);

    let state = TreeState::new(vec![root]);

    // Select grandchild
    let mut temp_state = state.clone();
    temp_state.selected_index = Some(2);
    let selected = temp_state.selected_node();
    assert!(selected.is_some());
    assert_eq!(*selected.unwrap().data(), 2);
}

#[test]
fn test_tree_message_debug() {
    let msg = TreeMessage::SelectNext;
    let debug = format!("{:?}", msg);
    assert_eq!(debug, "SelectNext");
}

#[test]
fn test_tree_output_debug() {
    let out = TreeOutput::Selected(vec![0, 1, 2]);
    let debug = format!("{:?}", out);
    assert!(debug.contains("Selected"));
}

#[test]
fn test_tree_message_eq() {
    assert_eq!(TreeMessage::Expand, TreeMessage::Expand);
    assert_eq!(TreeMessage::Collapse, TreeMessage::Collapse);
    assert_eq!(TreeMessage::Toggle, TreeMessage::Toggle);
    assert_eq!(TreeMessage::Select, TreeMessage::Select);
    assert_eq!(TreeMessage::SelectNext, TreeMessage::SelectNext);
    assert_eq!(TreeMessage::SelectPrevious, TreeMessage::SelectPrevious);
    assert_eq!(TreeMessage::ExpandAll, TreeMessage::ExpandAll);
    assert_eq!(TreeMessage::CollapseAll, TreeMessage::CollapseAll);
}

#[test]
fn test_tree_output_eq() {
    let out1 = TreeOutput::Selected(vec![0]);
    let out2 = TreeOutput::Selected(vec![0]);
    assert_eq!(out1, out2);

    let out3 = TreeOutput::Expanded(vec![0, 1]);
    let out4 = TreeOutput::Expanded(vec![0, 1]);
    assert_eq!(out3, out4);

    let out5 = TreeOutput::Collapsed(vec![2]);
    let out6 = TreeOutput::Collapsed(vec![2]);
    assert_eq!(out5, out6);
}

#[test]
fn test_node_debug() {
    let node = TreeNode::new("Test", 42);
    let debug = format!("{:?}", node);
    assert!(debug.contains("TreeNode"));
}

#[test]
fn test_state_debug() {
    let state: TreeState<i32> = TreeState::default();
    let debug = format!("{:?}", state);
    assert!(debug.contains("TreeState"));
}

#[test]
fn test_view_leaf_node_no_indicator() {
    // A leaf node (no children) should show no expand/collapse indicator
    let leaf = TreeNode::new("Leaf", ());
    let state = TreeState::new(vec![leaf]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_expand_on_leaf_node() {
    let leaf = TreeNode::new("Leaf", ());
    let mut state = TreeState::new(vec![leaf]);

    // Expanding a leaf should do nothing
    let output = Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(output, None);
}

#[test]
fn test_collapse_on_leaf_node() {
    let leaf = TreeNode::new("Leaf", ());
    let mut state = TreeState::new(vec![leaf]);

    // Collapsing a leaf should do nothing
    let output = Tree::update(&mut state, TreeMessage::Collapse);
    assert_eq!(output, None);
}
