use super::*;

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
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
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

// Annotation tests

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{with_annotations, WidgetType};
    let state = TreeState::new(vec![TreeNode::new("Root", ())]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::Tree);
    assert_eq!(regions.len(), 1);
}
