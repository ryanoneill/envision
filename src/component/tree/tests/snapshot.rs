use super::*;

// ========== Multi-root View Tests ==========

#[test]
fn test_view_multiple_roots_collapsed() {
    let mut root1 = TreeNode::new("Documents", ());
    root1.add_child(TreeNode::new("readme.md", ()));
    let mut root2 = TreeNode::new("Projects", ());
    root2.add_child(TreeNode::new("envision", ()));
    let root3 = TreeNode::new("Downloads", ());

    let state = TreeState::new(vec![root1, root2, root3]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_multiple_roots_expanded() {
    let mut root1 = TreeNode::new_expanded("Documents", ());
    root1.add_child(TreeNode::new("readme.md", ()));
    root1.add_child(TreeNode::new("guide.md", ()));
    let mut root2 = TreeNode::new_expanded("Projects", ());
    root2.add_child(TreeNode::new("envision", ()));
    let root3 = TreeNode::new("Downloads", ());

    let state = TreeState::new(vec![root1, root2, root3]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_multiple_roots_focused() {
    let mut root1 = TreeNode::new_expanded("Documents", ());
    root1.add_child(TreeNode::new("readme.md", ()));
    let mut root2 = TreeNode::new_expanded("Projects", ());
    root2.add_child(TreeNode::new("envision", ()));
    let root3 = TreeNode::new("Downloads", ());

    let mut state = TreeState::new(vec![root1, root2, root3]);
    state.set_focused(true);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(
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
fn test_view_selection_on_child() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));
    root.add_child(TreeNode::new("Child 3", ()));

    let mut state = TreeState::new(vec![root]);
    state.set_focused(true);
    state.selected_index = Some(2); // Select "Child 2"

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(
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

// ========== Deep Nesting View Tests ==========

#[test]
fn test_view_deep_nesting() {
    let mut level1 = TreeNode::new_expanded("Level 1", ());
    let mut level2 = TreeNode::new_expanded("Level 2", ());
    let mut level3 = TreeNode::new_expanded("Level 3", ());
    level3.add_child(TreeNode::new("Leaf", ()));
    level2.add_child(level3);
    level1.add_child(level2);

    let state = TreeState::new(vec![level1]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_deep_nesting_selection_at_leaf() {
    let mut level1 = TreeNode::new_expanded("Level 1", ());
    let mut level2 = TreeNode::new_expanded("Level 2", ());
    let mut level3 = TreeNode::new_expanded("Level 3", ());
    level3.add_child(TreeNode::new("Leaf", ()));
    level2.add_child(level3);
    level1.add_child(level2);

    let mut state = TreeState::new(vec![level1]);
    state.set_focused(true);
    state.selected_index = Some(3); // Select "Leaf"

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(
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

// ========== Mixed Expand/Collapse View Tests ==========

#[test]
fn test_view_mixed_expanded_collapsed() {
    let mut docs = TreeNode::new_expanded("Documents", ());
    docs.add_child(TreeNode::new("readme.md", ()));

    let mut projects = TreeNode::new("Projects", ()); // collapsed
    projects.add_child(TreeNode::new("envision", ()));

    let state = TreeState::new(vec![docs, projects]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========== Unicode View Tests ==========

#[test]
fn test_view_unicode_labels() {
    let mut folder = TreeNode::new_expanded("文件夹", ());
    folder.add_child(TreeNode::new("文档.txt", ()));
    folder.add_child(TreeNode::new("图片.png", ()));

    let state = TreeState::new(vec![folder, TreeNode::new("설정", ())]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========== Disabled View with Complex Tree ==========

#[test]
fn test_view_disabled_with_children() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));

    let mut state = TreeState::new(vec![root]);
    state.set_disabled(true);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(
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

// ========== Focused vs Unfocused with Expanded Tree ==========

#[test]
fn test_view_focused_expanded_tree() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));

    let mut state = TreeState::new(vec![root]);
    state.set_focused(true);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(
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
fn test_view_unfocused_expanded_tree() {
    let mut root = TreeNode::new_expanded("Root", ());
    root.add_child(TreeNode::new("Child 1", ()));
    root.add_child(TreeNode::new("Child 2", ()));

    let mut state = TreeState::new(vec![root]);
    state.set_focused(false);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========== Filtered View Snapshots ==========

#[test]
fn test_view_filtered() {
    let mut docs = TreeNode::new_expanded("Documents", ());
    docs.add_child(TreeNode::new("readme.md", ()));
    docs.add_child(TreeNode::new("guide.md", ()));

    let mut projects = TreeNode::new_expanded("Projects", ());
    projects.add_child(TreeNode::new("envision", ()));

    let downloads = TreeNode::new("Downloads", ());

    let mut state = TreeState::new(vec![docs, projects, downloads]);
    state.set_filter_text("readme");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_filtered_focused() {
    let mut docs = TreeNode::new_expanded("Documents", ());
    docs.add_child(TreeNode::new("readme.md", ()));
    docs.add_child(TreeNode::new("guide.md", ()));

    let mut state = TreeState::new(vec![docs]);
    state.set_focused(true);
    state.set_filter_text("readme");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(
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
fn test_view_filtered_no_matches() {
    let root = TreeNode::new("Root", ());
    let mut state = TreeState::new(vec![root]);
    state.set_filter_text("xyz_nonexistent");

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========== Sibling Tree View ==========

#[test]
fn test_view_many_siblings() {
    let mut root = TreeNode::new_expanded("Root", ());
    for i in 1..=5 {
        root.add_child(TreeNode::new(format!("Item {}", i), ()));
    }

    let state = TreeState::new(vec![root]);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ========== Focused selection on last item ==========

#[test]
fn test_view_selection_on_last_root() {
    let root1 = TreeNode::new("Root 1", ());
    let root2 = TreeNode::new("Root 2", ());
    let root3 = TreeNode::new("Root 3", ());

    let mut state = TreeState::new(vec![root1, root2, root3]);
    state.set_focused(true);
    state.selected_index = Some(2); // Select "Root 3"

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Tree::view(
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
