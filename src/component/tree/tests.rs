use super::*;
use crate::input::{Event, KeyCode};

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

    Tree::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_select_next_at_end() {
    let state_roots = vec![TreeNode::new("Root", ())];
    let mut state = TreeState::new(state_roots);

    Tree::<()>::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_index(), Some(0)); // Stays at 0
}

#[test]
fn test_select_previous() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child", ()));

    let mut state = TreeState::new(vec![root]);
    state.selected_index = Some(1);

    Tree::update(&mut state, TreeMessage::Up);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_select_previous_at_start() {
    let state_roots = vec![TreeNode::new("Root", ())];
    let mut state = TreeState::new(state_roots);

    Tree::<()>::update(&mut state, TreeMessage::Up);
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

    Tree::update(&mut state, TreeMessage::Up); // Go to root
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
    let output = Tree::update(&mut state, TreeMessage::Down);
    assert_eq!(output, None);

    let output = Tree::update(&mut state, TreeMessage::Select);
    assert_eq!(output, None);
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
    Tree::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_node().unwrap().label(), "src");

    // Expand src
    Tree::update(&mut state, TreeMessage::Expand);
    assert_eq!(state.visible_count(), 6); // project, src, main.rs, lib.rs, tests, Cargo.toml

    // Navigate to main.rs and select
    Tree::update(&mut state, TreeMessage::Down);
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

#[test]
fn test_large_tree_navigation() {
    // 100 flat nodes (siblings)
    let nodes: Vec<TreeNode<()>> = (0..100)
        .map(|i| TreeNode::new(format!("Node {}", i), ()))
        .collect();
    let mut state = TreeState::new(nodes);

    assert_eq!(state.visible_count(), 100);
    assert_eq!(state.selected_index(), Some(0));

    // Navigate down to middle
    for _ in 0..50 {
        Tree::<()>::update(&mut state, TreeMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(50));
    assert_eq!(state.selected_node().unwrap().label(), "Node 50");

    // Up back to start
    for _ in 0..50 {
        Tree::<()>::update(&mut state, TreeMessage::Up);
    }
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.selected_node().unwrap().label(), "Node 0");

    // Navigate to last
    for _ in 0..99 {
        Tree::<()>::update(&mut state, TreeMessage::Down);
    }
    assert_eq!(state.selected_index(), Some(99));
    assert_eq!(state.selected_node().unwrap().label(), "Node 99");
}

#[test]
fn test_deep_tree_navigation() {
    // Build a tree 50 levels deep, starting with innermost leaf
    let mut node = TreeNode::new("Leaf", 49);
    for i in (0..49).rev() {
        let mut parent = TreeNode::new(format!("Level {}", i), i);
        parent.add_child(node);
        node = parent;
    }

    let mut state = TreeState::new(vec![node]);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.visible_count(), 1); // Only root visible initially

    // Expand all levels and navigate down
    for _ in 0..49 {
        Tree::<i32>::update(&mut state, TreeMessage::Expand);
        Tree::<i32>::update(&mut state, TreeMessage::Down);
    }

    // Should be at the leaf node
    assert_eq!(state.selected_node().unwrap().label(), "Leaf");
    assert_eq!(*state.selected_node().unwrap().data(), 49);
    assert_eq!(state.visible_count(), 50);
}

#[test]
fn test_unicode_node_labels() {
    let mut folder = TreeNode::new("文件夹", ());
    folder.add_child(TreeNode::new("文档.txt", ()));
    folder.add_child(TreeNode::new("图片.png", ()));

    let mut state = TreeState::new(vec![folder, TreeNode::new("설정", ())]);

    Tree::<()>::update(&mut state, TreeMessage::Down);
    // Should navigate through unicode-labeled nodes without issue
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.selected_node().unwrap().label(), "설정");
}

// ========== handle_event Tests ==========

fn make_tree_state() -> TreeState<&'static str> {
    let mut root = TreeNode::new_expanded("Root", "root");
    root.add_child(TreeNode::new("Child 1", "child1"));
    root.add_child(TreeNode::new("Child 2", "child2"));
    TreeState::new(vec![root])
}

#[test]
fn test_handle_event_up_when_focused() {
    let mut state = make_tree_state();
    state.set_focused(true);
    state.selected_index = Some(1);

    let event = Event::key(KeyCode::Up);
    let msg = Tree::<&str>::handle_event(&state, &event);
    assert_eq!(msg, Some(TreeMessage::Up));
}

#[test]
fn test_handle_event_down_when_focused() {
    let mut state = make_tree_state();
    state.set_focused(true);

    let event = Event::key(KeyCode::Down);
    let msg = Tree::<&str>::handle_event(&state, &event);
    assert_eq!(msg, Some(TreeMessage::Down));
}

#[test]
fn test_handle_event_expand_when_focused() {
    let mut state = make_tree_state();
    state.set_focused(true);

    let event = Event::key(KeyCode::Right);
    let msg = Tree::<&str>::handle_event(&state, &event);
    assert_eq!(msg, Some(TreeMessage::Expand));
}

#[test]
fn test_handle_event_collapse_when_focused() {
    let mut state = make_tree_state();
    state.set_focused(true);

    let event = Event::key(KeyCode::Left);
    let msg = Tree::<&str>::handle_event(&state, &event);
    assert_eq!(msg, Some(TreeMessage::Collapse));
}

#[test]
fn test_handle_event_toggle_when_focused() {
    let mut state = make_tree_state();
    state.set_focused(true);

    let event = Event::char(' ');
    let msg = Tree::<&str>::handle_event(&state, &event);
    assert_eq!(msg, Some(TreeMessage::Toggle));
}

#[test]
fn test_handle_event_select_when_focused() {
    let mut state = make_tree_state();
    state.set_focused(true);

    let event = Event::key(KeyCode::Enter);
    let msg = Tree::<&str>::handle_event(&state, &event);
    assert_eq!(msg, Some(TreeMessage::Select));
}

#[test]
fn test_handle_event_vim_keys() {
    let mut state = make_tree_state();
    state.set_focused(true);

    let msg_k = Tree::<&str>::handle_event(&state, &Event::char('k'));
    assert_eq!(msg_k, Some(TreeMessage::Up));

    let msg_j = Tree::<&str>::handle_event(&state, &Event::char('j'));
    assert_eq!(msg_j, Some(TreeMessage::Down));

    let msg_h = Tree::<&str>::handle_event(&state, &Event::char('h'));
    assert_eq!(msg_h, Some(TreeMessage::Collapse));

    let msg_l = Tree::<&str>::handle_event(&state, &Event::char('l'));
    assert_eq!(msg_l, Some(TreeMessage::Expand));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = make_tree_state();
    // focused is false by default

    let msg = Tree::<&str>::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(&state, &Event::char('j'));
    assert_eq!(msg, None);
}

// ========== dispatch_event Tests ==========

#[test]
fn test_dispatch_event() {
    let mut state = make_tree_state();
    state.set_focused(true);

    // Dispatch Down: should move selection from 0 to 1
    let output = Tree::<&str>::dispatch_event(&mut state, &Event::key(KeyCode::Down));
    assert_eq!(output, None); // Down returns None but updates state
    assert_eq!(state.selected_index(), Some(1));

    // Dispatch Enter: should select the current node
    let output = Tree::<&str>::dispatch_event(&mut state, &Event::key(KeyCode::Enter));
    assert_eq!(output, Some(TreeOutput::Selected(vec![0, 0])));
}

// ========== Instance Method Tests ==========

#[test]
fn test_instance_methods() {
    let mut state = make_tree_state();

    // is_focused / set_focused
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());

    // dispatch_event via instance method
    state.set_focused(true);
    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, None); // Down returns None but updates state
    assert_eq!(state.selected_index(), Some(1));

    // update via instance method
    let output = state.update(TreeMessage::Select);
    assert_eq!(output, Some(TreeOutput::Selected(vec![0, 0])));

    // handle_event via instance method
    let msg = state.handle_event(&Event::key(KeyCode::Up));
    assert_eq!(msg, Some(TreeMessage::Up));
}
