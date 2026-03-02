use super::*;

fn test_tree() -> Vec<TreeNode<&'static str>> {
    // Documents/
    //   readme.md
    //   guide.md
    // Projects/
    //   envision/
    //     src/
    //     tests/
    //   other/
    // Downloads/
    let mut documents = TreeNode::new_expanded("Documents", "documents");
    documents.add_child(TreeNode::new("readme.md", "readme"));
    documents.add_child(TreeNode::new("guide.md", "guide"));

    let mut envision = TreeNode::new("envision", "envision");
    envision.add_child(TreeNode::new("src", "src"));
    envision.add_child(TreeNode::new("tests", "tests"));

    let mut projects = TreeNode::new_expanded("Projects", "projects");
    projects.add_child(envision);
    projects.add_child(TreeNode::new("other", "other"));

    let downloads = TreeNode::new("Downloads", "downloads");

    vec![documents, projects, downloads]
}

#[test]
fn test_filter_text_default() {
    let state = TreeState::new(test_tree());
    assert_eq!(state.filter_text(), "");
}

#[test]
fn test_set_filter_text() {
    let mut state = TreeState::new(test_tree());
    state.set_filter_text("readme");
    assert_eq!(state.filter_text(), "readme");
}

#[test]
fn test_filter_case_insensitive() {
    let mut state = TreeState::new(test_tree());
    state.set_filter_text("README");
    // "readme.md" matches (case-insensitive)
    let flat = state.flatten();
    assert!(flat.iter().any(|n| n.label == "readme.md"));
}

#[test]
fn test_filter_shows_ancestors() {
    let mut state = TreeState::new(test_tree());
    // Filter for "readme" — should show Documents (ancestor) and readme.md
    state.set_filter_text("readme");
    let flat = state.flatten();

    // Should include Documents (ancestor) and readme.md (match)
    let labels: Vec<&str> = flat.iter().map(|n| n.label.as_str()).collect();
    assert!(labels.contains(&"Documents"), "Expected Documents ancestor, got: {:?}", labels);
    assert!(labels.contains(&"readme.md"), "Expected readme.md match, got: {:?}", labels);
    // Should NOT include guide.md (sibling that doesn't match)
    assert!(!labels.contains(&"guide.md"), "Should not include guide.md, got: {:?}", labels);
    // Should NOT include Projects or Downloads
    assert!(!labels.contains(&"Projects"), "Should not include Projects, got: {:?}", labels);
    assert!(!labels.contains(&"Downloads"), "Should not include Downloads, got: {:?}", labels);
}

#[test]
fn test_filter_auto_expands_ancestors() {
    // Create a tree where the matching node is under a collapsed parent
    let mut root = TreeNode::new("Root", "root"); // collapsed
    root.add_child(TreeNode::new("target", "target"));

    let mut state = TreeState::new(vec![root]);
    assert_eq!(state.visible_count(), 1); // Only root visible (collapsed)

    state.set_filter_text("target");
    // Root should be auto-expanded to show "target"
    assert_eq!(state.visible_count(), 2);
    let flat = state.flatten();
    assert_eq!(flat[0].label, "Root");
    assert!(flat[0].is_expanded);
    assert_eq!(flat[1].label, "target");
}

#[test]
fn test_filter_no_matches() {
    let mut state = TreeState::new(test_tree());
    state.set_filter_text("xyz_nonexistent");
    assert_eq!(state.visible_count(), 0);
    assert_eq!(state.selected_node(), None);
}

#[test]
fn test_filter_empty_string_shows_all() {
    let mut state = TreeState::new(test_tree());
    state.set_filter_text("readme");
    assert!(state.visible_count() < 6); // Filtered

    state.set_filter_text("");
    // All originally visible nodes should be back
    // Documents (expanded) has 2 children, Projects (expanded) has 2 children (envision collapsed, other)
    // Downloads is collapsed leaf
    // Total: Documents, readme.md, guide.md, Projects, envision, other, Downloads = 7
    assert_eq!(state.visible_count(), 7);
}

#[test]
fn test_clear_filter() {
    let mut state = TreeState::new(test_tree());
    state.set_filter_text("readme");
    assert!(state.visible_count() < 7);

    state.clear_filter();
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 7);
}

#[test]
fn test_filter_preserves_selection() {
    let mut state = TreeState::new(test_tree());
    // Navigate to readme.md (index 1 in expanded Documents)
    state.selected_index = Some(1);
    assert_eq!(state.selected_node().unwrap().label(), "readme.md");

    // Filter to "readme" — readme.md is still visible
    state.set_filter_text("readme");
    assert_eq!(state.selected_node().unwrap().label(), "readme.md");
}

#[test]
fn test_filter_resets_selection_when_node_hidden() {
    let mut state = TreeState::new(test_tree());
    // Select Downloads (last root, visible at the end)
    let flat = state.flatten();
    let downloads_idx = flat.iter().position(|n| n.label == "Downloads").unwrap();
    state.selected_index = Some(downloads_idx);
    assert_eq!(state.selected_node().unwrap().label(), "Downloads");

    // Filter to "readme" — Downloads is hidden
    state.set_filter_text("readme");
    // Selection should move to first visible
    assert!(state.selected_node().is_some());
    assert_eq!(state.selected_node().unwrap().label(), "Documents");
}

#[test]
fn test_filter_preserves_expand_state() {
    let mut state = TreeState::new(test_tree());
    // envision is collapsed
    assert!(!state.roots()[1].children()[0].is_expanded());

    // Filter forces envision's parent to auto-expand
    state.set_filter_text("src");
    let flat = state.flatten();
    // envision should be auto-expanded to show "src"
    assert!(flat.iter().any(|n| n.label == "src"));

    // Clear filter — envision should still be collapsed (actual state unchanged)
    state.clear_filter();
    assert!(!state.roots()[1].children()[0].is_expanded());
}

#[test]
fn test_filter_navigation() {
    let mut state = TreeState::new(test_tree());
    state.focused = true;
    state.set_filter_text("readme");
    // Should show: Documents, readme.md
    assert_eq!(state.visible_count(), 2);

    // Should be at first item
    assert_eq!(state.selected_node().unwrap().label(), "Documents");

    Tree::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_node().unwrap().label(), "readme.md");

    // At end, stay
    Tree::update(&mut state, TreeMessage::Down);
    assert_eq!(state.selected_node().unwrap().label(), "readme.md");
}

#[test]
fn test_filter_select_returns_correct_path() {
    let mut state = TreeState::new(test_tree());
    state.set_filter_text("readme");
    Tree::update(&mut state, TreeMessage::Down);

    let output = Tree::update(&mut state, TreeMessage::Select);
    // readme.md is at path [0, 0] (first root, first child)
    assert_eq!(output, Some(TreeOutput::Selected(vec![0, 0])));
}

#[test]
fn test_filter_deep_match() {
    let mut state = TreeState::new(test_tree());
    // "src" is nested: Projects > envision > src
    state.set_filter_text("src");
    let flat = state.flatten();

    let labels: Vec<&str> = flat.iter().map(|n| n.label.as_str()).collect();
    assert!(labels.contains(&"Projects"), "Expected Projects ancestor, got: {:?}", labels);
    assert!(labels.contains(&"envision"), "Expected envision ancestor, got: {:?}", labels);
    assert!(labels.contains(&"src"), "Expected src match, got: {:?}", labels);
    // "tests" is a sibling of "src" but doesn't match — should NOT be shown
    assert!(!labels.contains(&"tests"), "Should not include tests sibling, got: {:?}", labels);
}

#[test]
fn test_filter_message_set_filter() {
    let mut state = TreeState::new(test_tree());
    let output = Tree::update(&mut state, TreeMessage::SetFilter("readme".into()));
    assert_eq!(state.filter_text(), "readme");
    assert_eq!(output, Some(TreeOutput::FilterChanged("readme".into())));
}

#[test]
fn test_filter_message_clear_filter() {
    let mut state = TreeState::new(test_tree());
    state.set_filter_text("readme");

    let output = Tree::update(&mut state, TreeMessage::ClearFilter);
    assert_eq!(state.filter_text(), "");
    assert_eq!(output, Some(TreeOutput::FilterChanged(String::new())));
}

#[test]
fn test_set_roots_clears_filter() {
    let mut state = TreeState::new(test_tree());
    state.set_filter_text("readme");
    assert!(state.visible_count() < 7);

    state.set_roots(vec![TreeNode::new("New Root", "new")]);
    assert_eq!(state.filter_text(), "");
    assert_eq!(state.visible_count(), 1);
}

#[test]
fn test_filter_disabled_still_allows_filter_change() {
    let mut state = TreeState::new(test_tree());
    state.set_disabled(true);

    let output = Tree::update(&mut state, TreeMessage::SetFilter("readme".into()));
    assert_eq!(state.filter_text(), "readme");
    assert_eq!(output, Some(TreeOutput::FilterChanged("readme".into())));
}

#[test]
fn test_filter_disabled_blocks_navigation() {
    let mut state = TreeState::new(test_tree());
    state.set_disabled(true);
    state.set_filter_text("readme");

    let output = Tree::update(&mut state, TreeMessage::Down);
    assert_eq!(output, None);
}

#[test]
fn test_filter_with_expand_collapse() {
    let mut state = TreeState::new(test_tree());
    // Filter to show envision subtree
    state.set_filter_text("src");
    // Projects > envision > src should be visible
    assert!(state.visible_count() > 0);

    // Clear filter — envision should still be collapsed (wasn't actually expanded)
    state.clear_filter();
    assert!(!state.roots()[1].children()[0].is_expanded());
}

#[test]
fn test_filter_multiple_matches() {
    let mut state = TreeState::new(test_tree());
    // Both "readme.md" and "guide.md" contain ".md"
    state.set_filter_text(".md");
    let flat = state.flatten();

    let labels: Vec<&str> = flat.iter().map(|n| n.label.as_str()).collect();
    assert!(labels.contains(&"Documents"), "Expected Documents, got: {:?}", labels);
    assert!(labels.contains(&"readme.md"), "Expected readme.md, got: {:?}", labels);
    assert!(labels.contains(&"guide.md"), "Expected guide.md, got: {:?}", labels);
}

#[test]
fn test_filter_root_node_matches() {
    let mut state = TreeState::new(test_tree());
    // "Documents" itself matches the filter
    state.set_filter_text("Documents");
    let flat = state.flatten();

    let labels: Vec<&str> = flat.iter().map(|n| n.label.as_str()).collect();
    assert!(labels.contains(&"Documents"));
    // Children are only shown if they also match the filter.
    // "readme.md" and "guide.md" don't contain "Documents", so they're filtered out.
    assert!(!labels.contains(&"readme.md"));
    assert!(!labels.contains(&"guide.md"));
    assert_eq!(flat.len(), 1);
}

#[test]
fn test_filter_parent_match_shows_matching_children_only() {
    let mut state = TreeState::new(test_tree());
    // Filter "d" matches: Documents, Downloads, readme.md (contains 'd'), guide.md (contains 'd')
    state.set_filter_text("d");
    let flat = state.flatten();

    let labels: Vec<&str> = flat.iter().map(|n| n.label.as_str()).collect();
    assert!(labels.contains(&"Documents"), "Expected Documents, got: {:?}", labels);
    assert!(labels.contains(&"readme.md"), "Expected readme.md (contains 'd'), got: {:?}", labels);
    assert!(labels.contains(&"guide.md"), "Expected guide.md (contains 'd'), got: {:?}", labels);
    assert!(labels.contains(&"Downloads"), "Expected Downloads, got: {:?}", labels);
}

#[test]
fn test_filter_empty_tree() {
    let mut state: TreeState<()> = TreeState::new(Vec::new());
    state.set_filter_text("test");
    assert_eq!(state.visible_count(), 0);
    assert_eq!(state.selected_node(), None);
}
