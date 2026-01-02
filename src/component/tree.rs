//! A hierarchical tree view component.
//!
//! `Tree` displays data in a hierarchical structure with expandable/collapsible
//! nodes. It supports keyboard navigation and single selection.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Tree, TreeMessage, TreeState, TreeNode, Component, Focusable};
//!
//! // Create a tree with nodes
//! let mut root = TreeNode::new("Root", "root-data");
//! root.add_child(TreeNode::new("Child 1", "child1"));
//! root.add_child(TreeNode::new("Child 2", "child2"));
//!
//! let mut state = TreeState::new(vec![root]);
//! Tree::focus(&mut state);
//!
//! // Navigate and expand
//! Tree::update(&mut state, TreeMessage::Expand);
//! Tree::update(&mut state, TreeMessage::SelectNext);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, Focusable};

/// A node in the tree hierarchy.
#[derive(Clone, Debug)]
pub struct TreeNode<T> {
    /// Display label for the node.
    label: String,
    /// Custom data associated with this node.
    data: T,
    /// Child nodes.
    children: Vec<TreeNode<T>>,
    /// Whether this node is expanded (children visible).
    expanded: bool,
}

impl<T: Clone> TreeNode<T> {
    /// Creates a new tree node with a label and data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let node = TreeNode::new("Documents", "/home/user/docs");
    /// assert_eq!(node.label(), "Documents");
    /// assert_eq!(node.data(), &"/home/user/docs");
    /// ```
    pub fn new(label: impl Into<String>, data: T) -> Self {
        Self {
            label: label.into(),
            data,
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Creates a new node that starts expanded.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let node: TreeNode<()> = TreeNode::new_expanded("Root", ());
    /// assert!(node.is_expanded());
    /// ```
    pub fn new_expanded(label: impl Into<String>, data: T) -> Self {
        Self {
            label: label.into(),
            data,
            children: Vec::new(),
            expanded: true,
        }
    }

    /// Returns the node's label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the node's label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Returns a reference to the node's data.
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the node's data.
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Returns the children of this node.
    pub fn children(&self) -> &[TreeNode<T>] {
        &self.children
    }

    /// Returns a mutable reference to the children.
    pub fn children_mut(&mut self) -> &mut Vec<TreeNode<T>> {
        &mut self.children
    }

    /// Adds a child node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let mut parent = TreeNode::new("Parent", ());
    /// parent.add_child(TreeNode::new("Child", ()));
    /// assert_eq!(parent.children().len(), 1);
    /// ```
    pub fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }

    /// Returns true if this node has children.
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns true if this node is expanded.
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Sets the expanded state.
    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }

    /// Expands this node.
    pub fn expand(&mut self) {
        self.expanded = true;
    }

    /// Collapses this node.
    pub fn collapse(&mut self) {
        self.expanded = false;
    }

    /// Toggles the expanded state.
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }
}

/// A flattened view of a tree node for rendering.
#[derive(Clone, Debug)]
struct FlatNode {
    /// Index path to this node in the tree (e.g., [0, 2, 1] = roots[0].children[2].children[1]).
    path: Vec<usize>,
    /// The depth/indentation level.
    depth: usize,
    /// The display label.
    label: String,
    /// Whether this node has children.
    has_children: bool,
    /// Whether this node is expanded.
    is_expanded: bool,
}

/// Messages that can be sent to a Tree component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeMessage {
    /// Move selection to the next visible node.
    SelectNext,
    /// Move selection to the previous visible node.
    SelectPrevious,
    /// Expand the currently selected node.
    Expand,
    /// Collapse the currently selected node.
    Collapse,
    /// Toggle expand/collapse of the currently selected node.
    Toggle,
    /// Select the current node (emit Selected output).
    Select,
    /// Expand all nodes.
    ExpandAll,
    /// Collapse all nodes.
    CollapseAll,
}

/// Output messages from a Tree component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeOutput {
    /// A node was selected (activated). Contains the path to the node.
    Selected(Vec<usize>),
    /// A node was expanded. Contains the path to the node.
    Expanded(Vec<usize>),
    /// A node was collapsed. Contains the path to the node.
    Collapsed(Vec<usize>),
}

/// State for a Tree component.
#[derive(Clone, Debug)]
pub struct TreeState<T> {
    /// The root nodes of the tree.
    roots: Vec<TreeNode<T>>,
    /// Index of the currently selected node in the flattened view.
    selected_index: usize,
    /// Whether the tree has focus.
    focused: bool,
}

impl<T: Clone> Default for TreeState<T> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T: Clone> TreeState<T> {
    /// Creates a new tree state with the given root nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreeState, TreeNode};
    ///
    /// let roots = vec![
    ///     TreeNode::new("Root 1", 1),
    ///     TreeNode::new("Root 2", 2),
    /// ];
    /// let state = TreeState::new(roots);
    /// assert_eq!(state.roots().len(), 2);
    /// ```
    pub fn new(roots: Vec<TreeNode<T>>) -> Self {
        Self {
            roots,
            selected_index: 0,
            focused: false,
        }
    }

    /// Returns the root nodes.
    pub fn roots(&self) -> &[TreeNode<T>] {
        &self.roots
    }

    /// Returns a mutable reference to the root nodes.
    pub fn roots_mut(&mut self) -> &mut Vec<TreeNode<T>> {
        &mut self.roots
    }

    /// Sets the root nodes.
    pub fn set_roots(&mut self, roots: Vec<TreeNode<T>>) {
        self.roots = roots;
        self.selected_index = 0;
    }

    /// Returns the currently selected index in the flattened view.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns true if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    /// Flattens the tree into a list of visible nodes.
    fn flatten(&self) -> Vec<FlatNode> {
        let mut result = Vec::new();
        for (i, root) in self.roots.iter().enumerate() {
            Self::flatten_node(root, vec![i], 0, &mut result);
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

    /// Gets a node by its path.
    fn get_node(&self, path: &[usize]) -> Option<&TreeNode<T>> {
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
    fn get_node_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode<T>> {
        if path.is_empty() {
            return None;
        }

        let mut current = self.roots.get_mut(path[0])?;
        for &idx in &path[1..] {
            current = current.children.get_mut(idx)?;
        }
        Some(current)
    }

    /// Returns the path of the currently selected node.
    pub fn selected_path(&self) -> Option<Vec<usize>> {
        let flat = self.flatten();
        flat.get(self.selected_index).map(|n| n.path.clone())
    }

    /// Returns a reference to the currently selected node.
    pub fn selected_node(&self) -> Option<&TreeNode<T>> {
        let path = self.selected_path()?;
        self.get_node(&path)
    }

    /// Expands all nodes in the tree.
    pub fn expand_all(&mut self) {
        for root in &mut self.roots {
            Self::expand_all_recursive(root);
        }
    }

    /// Recursively expands a node and all its descendants.
    fn expand_all_recursive(node: &mut TreeNode<T>) {
        if node.has_children() {
            node.expand();
            for child in &mut node.children {
                Self::expand_all_recursive(child);
            }
        }
    }

    /// Collapses all nodes in the tree.
    pub fn collapse_all(&mut self) {
        for root in &mut self.roots {
            Self::collapse_all_recursive(root);
        }
        // Reset selection to ensure it's still valid
        self.selected_index = 0;
    }

    /// Recursively collapses a node and all its descendants.
    fn collapse_all_recursive(node: &mut TreeNode<T>) {
        node.collapse();
        for child in &mut node.children {
            Self::collapse_all_recursive(child);
        }
    }

    /// Returns the number of visible nodes.
    pub fn visible_count(&self) -> usize {
        self.flatten().len()
    }
}

/// A hierarchical tree view component.
///
/// Displays data in a tree structure with expandable/collapsible nodes.
/// Supports keyboard navigation and single selection.
///
/// # Visual Format
///
/// ```text
/// ▶ Documents
/// ▼ Projects
///   ├─ envision
///   │  ├─ src
///   │  └─ tests
///   └─ other
/// ```
///
/// # Keyboard Navigation
///
/// - `Up/Down` - Move selection
/// - `Right` - Expand node
/// - `Left` - Collapse node
/// - `Enter` - Select/activate node
///
/// # Example
///
/// ```rust
/// use envision::component::{Tree, TreeMessage, TreeOutput, TreeState, TreeNode, Component};
///
/// // Build tree structure
/// let mut docs = TreeNode::new("Documents", "docs");
/// docs.add_child(TreeNode::new("readme.md", "readme"));
/// docs.add_child(TreeNode::new("guide.md", "guide"));
///
/// let mut projects = TreeNode::new_expanded("Projects", "projects");
/// projects.add_child(TreeNode::new("envision", "envision"));
///
/// let mut state = TreeState::new(vec![docs, projects]);
///
/// // Navigate
/// Tree::update(&mut state, TreeMessage::SelectNext);
/// Tree::update(&mut state, TreeMessage::Expand);
/// ```
pub struct Tree<T>(std::marker::PhantomData<T>);

impl<T: Clone + 'static> Tree<T> {
    /// Renders the tree to a list of styled lines.
    fn render_lines(state: &TreeState<T>, width: u16) -> Vec<Line<'static>> {
        let flat = state.flatten();
        let mut lines = Vec::new();

        for (idx, node) in flat.iter().enumerate() {
            let is_selected = idx == state.selected_index;

            // Build the prefix with tree lines
            let indent = "  ".repeat(node.depth);

            // Expand/collapse indicator
            let indicator = if node.has_children {
                if node.is_expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let text = format!("{}{}{}", indent, indicator, node.label);

            // Pad to full width for selection highlight
            let padded = format!("{:<width$}", text, width = width as usize);

            let style = if is_selected && state.focused {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            lines.push(Line::from(Span::styled(padded, style)));
        }

        lines
    }
}

impl<T: Clone + 'static> Component for Tree<T> {
    type State = TreeState<T>;
    type Message = TreeMessage;
    type Output = TreeOutput;

    fn init() -> Self::State {
        TreeState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        let flat = state.flatten();
        if flat.is_empty() {
            return None;
        }

        match msg {
            TreeMessage::SelectNext => {
                if state.selected_index < flat.len() - 1 {
                    state.selected_index += 1;
                }
                None
            }
            TreeMessage::SelectPrevious => {
                if state.selected_index > 0 {
                    state.selected_index -= 1;
                }
                None
            }
            TreeMessage::Expand => {
                if let Some(node_info) = flat.get(state.selected_index) {
                    if node_info.has_children && !node_info.is_expanded {
                        let path = node_info.path.clone();
                        if let Some(node) = state.get_node_mut(&path) {
                            node.expand();
                            return Some(TreeOutput::Expanded(path));
                        }
                    }
                }
                None
            }
            TreeMessage::Collapse => {
                if let Some(node_info) = flat.get(state.selected_index) {
                    if node_info.has_children && node_info.is_expanded {
                        let path = node_info.path.clone();
                        if let Some(node) = state.get_node_mut(&path) {
                            node.collapse();
                            // Adjust selected index if needed
                            let new_flat = state.flatten();
                            if state.selected_index >= new_flat.len() {
                                state.selected_index = new_flat.len().saturating_sub(1);
                            }
                            return Some(TreeOutput::Collapsed(path));
                        }
                    }
                }
                None
            }
            TreeMessage::Toggle => {
                if let Some(node_info) = flat.get(state.selected_index) {
                    if node_info.has_children {
                        let path = node_info.path.clone();
                        let was_expanded = node_info.is_expanded;
                        if let Some(node) = state.get_node_mut(&path) {
                            node.toggle();
                            if was_expanded {
                                // Adjust selected index if needed after collapse
                                let new_flat = state.flatten();
                                if state.selected_index >= new_flat.len() {
                                    state.selected_index = new_flat.len().saturating_sub(1);
                                }
                                return Some(TreeOutput::Collapsed(path));
                            } else {
                                return Some(TreeOutput::Expanded(path));
                            }
                        }
                    }
                }
                None
            }
            TreeMessage::Select => flat
                .get(state.selected_index)
                .map(|node_info| TreeOutput::Selected(node_info.path.clone())),
            TreeMessage::ExpandAll => {
                state.expand_all();
                None
            }
            TreeMessage::CollapseAll => {
                state.collapse_all();
                None
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        let lines = Self::render_lines(state, area.width);
        let text = Text::from(lines);
        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, area);
    }
}

impl<T: Clone + 'static> Focusable for Tree<T> {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(state.selected_index(), 0);
        assert!(!state.is_empty());
    }

    #[test]
    fn test_state_default() {
        let state: TreeState<()> = TreeState::default();
        assert!(state.is_empty());
    }

    #[test]
    fn test_state_set_roots() {
        let mut state: TreeState<i32> = TreeState::default();
        state.set_roots(vec![TreeNode::new("Root", 1)]);

        assert_eq!(state.roots().len(), 1);
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

        state.selected_index = 1;
        assert_eq!(state.selected_path(), Some(vec![0, 0]));
    }

    #[test]
    fn test_state_selected_node() {
        let mut root = TreeNode::new_expanded("Root", "root_data");
        root.add_child(TreeNode::new("Child", "child_data"));

        let mut state = TreeState::new(vec![root]);

        let selected = state.selected_node();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().data(), &"root_data");

        state.selected_index = 1;
        let selected = state.selected_node();
        assert_eq!(selected.unwrap().data(), &"child_data");
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
        assert_eq!(state.selected_index(), 0);
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
        state.selected_index = 1;

        let cloned = state.clone();
        assert_eq!(cloned.visible_count(), 2);
        assert_eq!(cloned.selected_index(), 1);
    }

    // Tree component tests

    #[test]
    fn test_init() {
        let state: TreeState<()> = Tree::init();
        assert!(state.is_empty());
    }

    #[test]
    fn test_select_next() {
        let mut root = TreeNode::new_expanded("Root", ());
        root.add_child(TreeNode::new("Child", ()));

        let mut state = TreeState::new(vec![root]);
        assert_eq!(state.selected_index(), 0);

        Tree::update(&mut state, TreeMessage::SelectNext);
        assert_eq!(state.selected_index(), 1);
    }

    #[test]
    fn test_select_next_at_end() {
        let state_roots = vec![TreeNode::new("Root", ())];
        let mut state = TreeState::new(state_roots);

        Tree::<()>::update(&mut state, TreeMessage::SelectNext);
        assert_eq!(state.selected_index(), 0); // Stays at 0
    }

    #[test]
    fn test_select_previous() {
        let mut root = TreeNode::new_expanded("Root", ());
        root.add_child(TreeNode::new("Child", ()));

        let mut state = TreeState::new(vec![root]);
        state.selected_index = 1;

        Tree::update(&mut state, TreeMessage::SelectPrevious);
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn test_select_previous_at_start() {
        let state_roots = vec![TreeNode::new("Root", ())];
        let mut state = TreeState::new(state_roots);

        Tree::<()>::update(&mut state, TreeMessage::SelectPrevious);
        assert_eq!(state.selected_index(), 0); // Stays at 0
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
        state.selected_index = 1; // Select child

        Tree::update(&mut state, TreeMessage::SelectPrevious); // Go to root
        Tree::update(&mut state, TreeMessage::Collapse);

        // Selection should still be valid
        assert!(state.selected_index() < state.visible_count());
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
        state.selected_index = 1;

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
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let state: TreeState<()> = TreeState::new(Vec::new());

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Tree::view(&state, frame, frame.area());
            })
            .unwrap();

        // Should not panic
    }

    #[test]
    fn test_view_single_node() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let root = TreeNode::new("Root", ());
        let state = TreeState::new(vec![root]);

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Tree::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Root"));
    }

    #[test]
    fn test_view_with_children() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut root = TreeNode::new_expanded("Parent", ());
        root.add_child(TreeNode::new("Child", ()));

        let state = TreeState::new(vec![root]);

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Tree::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Parent"));
        assert!(output.contains("Child"));
    }

    #[test]
    fn test_view_collapsed_indicator() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut root = TreeNode::new("Root", ()); // Collapsed
        root.add_child(TreeNode::new("Child", ()));

        let state = TreeState::new(vec![root]);

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Tree::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("▶")); // Collapsed indicator
    }

    #[test]
    fn test_view_expanded_indicator() {
        use crate::backend::CaptureBackend;
        use ratatui::Terminal;

        let mut root = TreeNode::new_expanded("Root", ());
        root.add_child(TreeNode::new("Child", ()));

        let state = TreeState::new(vec![root]);

        let backend = CaptureBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                Tree::view(&state, frame, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("▼")); // Expanded indicator
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
}
