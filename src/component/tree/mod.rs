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
use crate::theme::Theme;

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
    fn render_lines(state: &TreeState<T>, width: u16, theme: &Theme) -> Vec<Line<'static>> {
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

            let style = if is_selected {
                theme.selected_highlight_style(state.focused)
            } else {
                theme.normal_style()
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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        let lines = Self::render_lines(state, area.width, theme);
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
mod tests;
