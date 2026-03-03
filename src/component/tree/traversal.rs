/// Tree traversal, flattening, and node lookup helpers for TreeState.
///
/// These are private implementation details extracted to keep
/// the main module under the 1000-line limit.
use super::{TreeNode, TreeState};

/// A flattened view of a tree node for rendering.
#[derive(Clone, Debug)]
pub(super) struct FlatNode {
    /// Index path to this node in the tree (e.g., [0, 2, 1] = roots[0].children[2].children[1]).
    pub(super) path: Vec<usize>,
    /// The depth/indentation level.
    pub(super) depth: usize,
    /// The display label.
    pub(super) label: String,
    /// Whether this node has children.
    pub(super) has_children: bool,
    /// Whether this node is expanded.
    pub(super) is_expanded: bool,
}

impl<T: Clone> TreeState<T> {
    /// Flattens the tree into a list of visible nodes.
    ///
    /// When a filter is active, only nodes whose label matches or whose
    /// descendants match are included. Ancestor nodes are auto-expanded
    /// to reveal matching descendants.
    pub(super) fn flatten(&self) -> Vec<FlatNode> {
        let mut result = Vec::new();
        if self.filter_text.is_empty() {
            for (i, root) in self.roots.iter().enumerate() {
                Self::flatten_node(root, vec![i], 0, &mut result);
            }
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            for (i, root) in self.roots.iter().enumerate() {
                Self::flatten_node_filtered(root, vec![i], 0, &filter_lower, &mut result);
            }
        }
        result
    }

    /// Recursively flattens a node and its visible children.
    fn flatten_node(
        node: &TreeNode<T>,
        path: Vec<usize>,
        depth: usize,
        result: &mut Vec<FlatNode>,
    ) {
        result.push(FlatNode {
            path: path.clone(),
            depth,
            label: node.label.clone(),
            has_children: node.has_children(),
            is_expanded: node.expanded,
        });

        if node.expanded {
            for (i, child) in node.children.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);
                Self::flatten_node(child, child_path, depth + 1, result);
            }
        }
    }

    /// Recursively flattens a node, filtering by label match.
    ///
    /// A node is included if its label matches the filter or any descendant
    /// matches. When a node has matching descendants, it is shown as expanded
    /// regardless of its actual expanded state.
    fn flatten_node_filtered(
        node: &TreeNode<T>,
        path: Vec<usize>,
        depth: usize,
        filter: &str,
        result: &mut Vec<FlatNode>,
    ) {
        let self_matches = node.label.to_lowercase().contains(filter);
        let has_matching_descendant = node
            .children
            .iter()
            .any(|child| Self::subtree_matches(child, filter));

        if !self_matches && !has_matching_descendant {
            return;
        }

        // When a node has matching descendants, force it expanded to reveal them.
        // When a node itself matches, use its actual expanded state for children.
        let show_expanded = if has_matching_descendant {
            true
        } else {
            node.expanded
        };

        result.push(FlatNode {
            path: path.clone(),
            depth,
            label: node.label.clone(),
            has_children: node.has_children(),
            is_expanded: show_expanded,
        });

        if show_expanded {
            for (i, child) in node.children.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);
                Self::flatten_node_filtered(child, child_path, depth + 1, filter, result);
            }
        }
    }

    /// Returns true if this node or any descendant matches the filter.
    fn subtree_matches(node: &TreeNode<T>, filter: &str) -> bool {
        if node.label.to_lowercase().contains(filter) {
            return true;
        }
        node.children
            .iter()
            .any(|child| Self::subtree_matches(child, filter))
    }

    /// Gets a node by its path.
    pub(super) fn get_node(&self, path: &[usize]) -> Option<&TreeNode<T>> {
        if path.is_empty() {
            return None;
        }

        let mut current = self.roots.get(path[0])?;
        for &idx in &path[1..] {
            current = current.children.get(idx)?;
        }
        Some(current)
    }

    /// Gets a mutable reference to a node by its path.
    pub(super) fn get_node_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode<T>> {
        if path.is_empty() {
            return None;
        }

        let mut current = self.roots.get_mut(path[0])?;
        for &idx in &path[1..] {
            current = current.children.get_mut(idx)?;
        }
        Some(current)
    }

    /// Recursively expands a node and all its descendants.
    pub(super) fn expand_all_recursive(node: &mut TreeNode<T>) {
        if node.has_children() {
            node.expand();
            for child in &mut node.children {
                Self::expand_all_recursive(child);
            }
        }
    }

    /// Recursively collapses a node and all its descendants.
    pub(super) fn collapse_all_recursive(node: &mut TreeNode<T>) {
        node.collapse();
        for child in &mut node.children {
            Self::collapse_all_recursive(child);
        }
    }

    /// Revalidates the selected index after a filter change.
    ///
    /// Tries to preserve the previously selected node by path. If that node
    /// is no longer visible, falls back to the first visible node.
    pub(super) fn revalidate_selection(&mut self, prev_path: Option<Vec<usize>>) {
        let flat = self.flatten();

        if flat.is_empty() {
            self.selected_index = None;
            return;
        }

        if let Some(path) = prev_path {
            if let Some(new_idx) = flat.iter().position(|n| n.path == path) {
                self.selected_index = Some(new_idx);
                return;
            }
        }

        self.selected_index = Some(0);
    }
}
