//! A hierarchical tree view component.
//!
//! [`Tree<T>`] displays data in a hierarchical structure with expandable/collapsible
//! nodes. It supports keyboard navigation and single selection. State is stored in
//! [`TreeState<T>`], updated via [`TreeMessage`], and produces [`TreeOutput`].
//! Tree data is provided via [`TreeNode<T>`].
//!
//!
//! See also [`Accordion`](super::Accordion) for a simpler collapsible panel list.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Tree, TreeMessage, TreeState, TreeNode, Component};
//!
//! // Create a tree with nodes
//! let mut root = TreeNode::new("Root", "root-data");
//! root.add_child(TreeNode::new("Child 1", "child1"));
//! root.add_child(TreeNode::new("Child 2", "child2"));
//!
//! let mut state = TreeState::new(vec![root]);
//!
//! // Navigate and expand
//! Tree::update(&mut state, TreeMessage::Expand);
//! Tree::update(&mut state, TreeMessage::Down);
//! ```

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::scroll::ScrollState;

mod render;
mod traversal;

/// A node in the tree hierarchy.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
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

impl<T: PartialEq> PartialEq for TreeNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
            && self.data == other.data
            && self.children == other.children
            && self.expanded == other.expanded
    }
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let mut node = TreeNode::new("Old", ());
    /// node.set_label("New");
    /// assert_eq!(node.label(), "New");
    /// ```
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }

    /// Returns a reference to the node's data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let node = TreeNode::new("Root", 42u32);
    /// assert_eq!(node.data(), &42u32);
    /// ```
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the node's data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let mut node = TreeNode::new("Root", 0u32);
    /// *node.data_mut() = 99;
    /// assert_eq!(node.data(), &99u32);
    /// ```
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Returns the children of this node.
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
    pub fn children(&self) -> &[TreeNode<T>] {
        &self.children
    }

    /// Returns a mutable reference to the children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let mut node = TreeNode::new("Root", ());
    /// node.children_mut().push(TreeNode::new("Child", ()));
    /// assert_eq!(node.children().len(), 1);
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let mut parent = TreeNode::new("Parent", ());
    /// assert!(!parent.has_children());
    /// parent.add_child(TreeNode::new("Child", ()));
    /// assert!(parent.has_children());
    /// ```
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns true if this node is expanded.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let node = TreeNode::new("Root", ());
    /// assert!(!node.is_expanded());
    /// ```
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Sets the expanded state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let mut node = TreeNode::new("Root", ());
    /// node.set_expanded(true);
    /// assert!(node.is_expanded());
    /// ```
    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }

    /// Expands this node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let mut node = TreeNode::new("Root", ());
    /// node.expand();
    /// assert!(node.is_expanded());
    /// ```
    pub fn expand(&mut self) {
        self.expanded = true;
    }

    /// Collapses this node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let mut node = TreeNode::new_expanded("Root", ());
    /// node.collapse();
    /// assert!(!node.is_expanded());
    /// ```
    pub fn collapse(&mut self) {
        self.expanded = false;
    }

    /// Toggles the expanded state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeNode;
    ///
    /// let mut node = TreeNode::new("Root", ());
    /// node.toggle();
    /// assert!(node.is_expanded());
    /// node.toggle();
    /// assert!(!node.is_expanded());
    /// ```
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }
}

/// Messages that can be sent to a Tree component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeMessage {
    /// Move selection down to the next visible node.
    Down,
    /// Move selection up to the previous visible node.
    Up,
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
    /// Set the filter text for searching nodes.
    SetFilter(String),
    /// Clear the filter text.
    ClearFilter,
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
    /// The filter text changed.
    FilterChanged(String),
}

/// State for a Tree component.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TreeState<T> {
    /// The root nodes of the tree.
    roots: Vec<TreeNode<T>>,
    /// Index of the currently selected node in the flattened view, or `None` if empty.
    selected_index: Option<usize>,
    /// Current filter text for searching nodes by label.
    filter_text: String,
    /// Scroll state for scrollbar rendering.
    #[cfg_attr(feature = "serialization", serde(skip))]
    scroll: ScrollState,
}

impl<T: Clone + PartialEq> PartialEq for TreeState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.roots == other.roots
            && self.selected_index == other.selected_index
            && self.filter_text == other.filter_text
    }
}

impl<T: Clone> Default for TreeState<T> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T: Clone> TreeState<T> {
    /// Creates a new tree state with the given root nodes.
    ///
    /// If roots are non-empty, the first node is selected.
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
        let selected_index = if roots.is_empty() { None } else { Some(0) };
        Self {
            roots,
            selected_index,
            filter_text: String::new(),
            scroll: ScrollState::default(),
        }
    }

    /// Sets the initially selected index in the flattened view (builder method).
    ///
    /// The index is clamped to the valid range of visible nodes.
    /// Has no effect on empty trees.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreeState, TreeNode};
    ///
    /// let mut root = TreeNode::new_expanded("Root", ());
    /// root.add_child(TreeNode::new("Child 1", ()));
    /// root.add_child(TreeNode::new("Child 2", ()));
    ///
    /// let state = TreeState::new(vec![root]).with_selected(2);
    /// assert_eq!(state.selected_index(), Some(2));
    /// ```
    pub fn with_selected(mut self, index: usize) -> Self {
        if self.roots.is_empty() {
            return self;
        }
        let visible = self.flatten().len();
        self.selected_index = Some(index.min(visible.saturating_sub(1)));
        self
    }

    /// Returns the root nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TreeState::new(vec![
    ///     TreeNode::new("Root 1", 1),
    ///     TreeNode::new("Root 2", 2),
    /// ]);
    /// assert_eq!(state.roots().len(), 2);
    /// assert_eq!(state.roots()[0].label(), "Root 1");
    /// ```
    pub fn roots(&self) -> &[TreeNode<T>] {
        &self.roots
    }

    /// Returns a mutable reference to the root nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreeState, TreeNode};
    ///
    /// let mut state = TreeState::new(vec![TreeNode::new("Root", ())]);
    /// state.roots_mut().push(TreeNode::new("Another Root", ()));
    /// assert_eq!(state.roots().len(), 2);
    /// ```
    pub fn roots_mut(&mut self) -> &mut Vec<TreeNode<T>> {
        &mut self.roots
    }

    /// Updates a root node at the given index via a closure.
    ///
    /// No-ops if the index is out of bounds. This is safe because it
    /// does not change the number of root nodes or their positions,
    /// so selection and filter state remain valid.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreeState, TreeNode};
    ///
    /// let mut state = TreeState::new(vec![
    ///     TreeNode::new("Root 1", 1),
    ///     TreeNode::new("Root 2", 2),
    /// ]);
    /// state.update_root(0, |root| root.set_label("Updated Root"));
    /// assert_eq!(state.roots()[0].label(), "Updated Root");
    /// ```
    pub fn update_root(&mut self, index: usize, f: impl FnOnce(&mut TreeNode<T>)) {
        if let Some(root) = self.roots.get_mut(index) {
            f(root);
        }
    }

    /// Sets the root nodes.
    ///
    /// Resets selection to the first node, or `None` if the new roots are empty.
    /// Clears any active filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut state = TreeState::new(vec![TreeNode::new("Old", 0)]);
    /// state.set_roots(vec![TreeNode::new("New 1", 1), TreeNode::new("New 2", 2)]);
    /// assert_eq!(state.roots().len(), 2);
    /// assert_eq!(state.selected_index(), Some(0));
    /// ```
    pub fn set_roots(&mut self, roots: Vec<TreeNode<T>>) {
        self.roots = roots;
        self.filter_text.clear();
        self.selected_index = if self.roots.is_empty() { None } else { Some(0) };
        self.scroll.set_content_length(self.flatten().len());
    }

    /// Returns the currently selected index in the flattened view.
    ///
    /// Returns `None` if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TreeState::new(vec![TreeNode::new("Root", ())]);
    /// assert_eq!(state.selected_index(), Some(0));
    ///
    /// let empty: TreeState<()> = TreeState::new(vec![]);
    /// assert_eq!(empty.selected_index(), None);
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Alias for [`selected_index()`](Self::selected_index).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreeState, TreeNode};
    ///
    /// let state = TreeState::new(vec![TreeNode::new("Root", ())]);
    /// assert_eq!(state.selected(), Some(0));
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected_index()
    }

    /// Sets the selected index in the flattened view.
    ///
    /// The index is clamped to the valid range of visible nodes.
    /// Has no effect on empty trees.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreeState, TreeNode};
    ///
    /// let mut root = TreeNode::new_expanded("Root", ());
    /// root.add_child(TreeNode::new("Child 1", ()));
    /// root.add_child(TreeNode::new("Child 2", ()));
    ///
    /// let mut state = TreeState::new(vec![root]);
    /// state.set_selected(Some(2));
    /// assert_eq!(state.selected_index(), Some(2));
    /// ```
    pub fn set_selected(&mut self, index: Option<usize>) {
        match index {
            Some(i) => {
                if self.roots.is_empty() {
                    return;
                }
                let visible = self.flatten().len();
                self.selected_index = Some(i.min(visible.saturating_sub(1)));
            }
            None => self.selected_index = None,
        }
    }

    /// Returns true if the tree is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeState;
    ///
    /// let empty: TreeState<()> = TreeState::new(vec![]);
    /// assert!(empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    /// Returns the path of the currently selected node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreeState, TreeNode};
    ///
    /// let state = TreeState::new(vec![TreeNode::new("Root", ())]);
    /// assert_eq!(state.selected_path(), Some(vec![0]));
    /// ```
    pub fn selected_path(&self) -> Option<Vec<usize>> {
        let flat = self.flatten();
        flat.get(self.selected_index?).map(|n| n.path.clone())
    }

    /// Returns a reference to the currently selected node.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TreeState::new(vec![TreeNode::new("Root", 42)]);
    /// let node = state.selected_node().unwrap();
    /// assert_eq!(node.label(), "Root");
    /// assert_eq!(node.data(), &42);
    /// ```
    pub fn selected_node(&self) -> Option<&TreeNode<T>> {
        let path = self.selected_path()?;
        self.get_node(&path)
    }

    /// Returns a reference to the currently selected node.
    ///
    /// This is an alias for [`selected_node()`](Self::selected_node) that provides a
    /// consistent accessor name across all selection-based components.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let state = TreeState::new(vec![TreeNode::new("Root", 1)]);
    /// assert_eq!(state.selected_item(), state.selected_node());
    /// ```
    pub fn selected_item(&self) -> Option<&TreeNode<T>> {
        self.selected_node()
    }

    /// Expands all nodes in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut root = TreeNode::new("Root", ());
    /// root.add_child(TreeNode::new("Child", ()));
    /// let mut state = TreeState::new(vec![root]);
    /// state.expand_all();
    /// assert!(state.roots()[0].is_expanded());
    /// ```
    pub fn expand_all(&mut self) {
        for root in &mut self.roots {
            Self::expand_all_recursive(root);
        }
        self.scroll.set_content_length(self.flatten().len());
    }

    /// Collapses all nodes in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut root = TreeNode::new_expanded("Root", ());
    /// root.add_child(TreeNode::new("Child", ()));
    /// let mut state = TreeState::new(vec![root]);
    /// state.collapse_all();
    /// assert!(!state.roots()[0].is_expanded());
    /// ```
    pub fn collapse_all(&mut self) {
        for root in &mut self.roots {
            Self::collapse_all_recursive(root);
        }
        // Reset selection to ensure it's still valid
        self.selected_index = if self.roots.is_empty() { None } else { Some(0) };
        self.scroll.set_content_length(self.flatten().len());
    }

    /// Returns the number of visible nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::prelude::*;
    ///
    /// let mut root = TreeNode::new_expanded("Root", ());
    /// root.add_child(TreeNode::new("Child 1", ()));
    /// root.add_child(TreeNode::new("Child 2", ()));
    /// let state = TreeState::new(vec![root]);
    /// // Root + 2 children visible (root is expanded)
    /// assert_eq!(state.visible_count(), 3);
    /// ```
    pub fn visible_count(&self) -> usize {
        self.flatten().len()
    }

    /// Returns the current filter text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreeState;
    ///
    /// let state: TreeState<()> = TreeState::new(vec![]);
    /// assert_eq!(state.filter_text(), "");
    /// ```
    pub fn filter_text(&self) -> &str {
        &self.filter_text
    }

    /// Sets the filter text for case-insensitive substring matching on node labels.
    ///
    /// When a filter is active, only nodes whose label matches or whose
    /// descendants match are shown. Ancestor nodes are auto-expanded to
    /// reveal matching descendants without modifying their actual expanded state.
    ///
    /// Selection is preserved if the selected node remains visible,
    /// otherwise it moves to the first visible node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreeState, TreeNode};
    ///
    /// let mut state = TreeState::new(vec![
    ///     TreeNode::new("Alpha", ()),
    ///     TreeNode::new("Beta", ()),
    /// ]);
    /// state.set_filter_text("alpha");
    /// assert_eq!(state.filter_text(), "alpha");
    /// assert_eq!(state.visible_count(), 1);
    /// ```
    pub fn set_filter_text(&mut self, text: &str) {
        let prev_path = self.selected_path();
        self.filter_text = text.to_string();
        self.revalidate_selection(prev_path);
        self.scroll.set_content_length(self.flatten().len());
    }

    /// Clears the filter, showing all nodes with their original expanded state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreeState, TreeNode};
    ///
    /// let mut state = TreeState::new(vec![
    ///     TreeNode::new("Alpha", ()),
    ///     TreeNode::new("Beta", ()),
    /// ]);
    /// state.set_filter_text("alpha");
    /// assert_eq!(state.visible_count(), 1);
    /// state.clear_filter();
    /// assert_eq!(state.filter_text(), "");
    /// assert_eq!(state.visible_count(), 2);
    /// ```
    pub fn clear_filter(&mut self) {
        let prev_path = self.selected_path();
        self.filter_text.clear();
        self.revalidate_selection(prev_path);
        self.scroll.set_content_length(self.flatten().len());
    }
}

impl<T: Clone + 'static> TreeState<T> {
    /// Updates the tree state with a message, returning any output.
    pub fn update(&mut self, msg: TreeMessage) -> Option<TreeOutput> {
        Tree::update(self, msg)
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
/// Tree::update(&mut state, TreeMessage::Down);
/// Tree::update(&mut state, TreeMessage::Expand);
/// ```
pub struct Tree<T>(std::marker::PhantomData<T>);

impl<T: Clone + 'static> Component for Tree<T> {
    type State = TreeState<T>;
    type Message = TreeMessage;
    type Output = TreeOutput;

    fn init() -> Self::State {
        TreeState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        // Filter messages are handled before the disabled check,
        // allowing filter changes even when the tree is disabled.
        match msg {
            TreeMessage::SetFilter(ref text) => {
                state.set_filter_text(text);
                return Some(TreeOutput::FilterChanged(text.clone()));
            }
            TreeMessage::ClearFilter => {
                state.clear_filter();
                return Some(TreeOutput::FilterChanged(String::new()));
            }
            _ => {}
        }

        let flat = state.flatten();
        if flat.is_empty() {
            return None;
        }

        let selected = state.selected_index?;

        match msg {
            TreeMessage::Down => {
                if selected < flat.len() - 1 {
                    state.selected_index = Some(selected + 1);
                }
                None
            }
            TreeMessage::Up => {
                if selected > 0 {
                    state.selected_index = Some(selected - 1);
                }
                None
            }
            TreeMessage::Expand => {
                if let Some(node_info) = flat.get(selected) {
                    if node_info.has_children && !node_info.is_expanded {
                        let path = node_info.path.clone();
                        if let Some(node) = state.get_node_mut(&path) {
                            node.expand();
                            state.scroll.set_content_length(state.flatten().len());
                            return Some(TreeOutput::Expanded(path));
                        }
                    }
                }
                None
            }
            TreeMessage::Collapse => {
                if let Some(node_info) = flat.get(selected) {
                    if node_info.has_children && node_info.is_expanded {
                        let path = node_info.path.clone();
                        if let Some(node) = state.get_node_mut(&path) {
                            node.collapse();
                            // Adjust selected index if needed
                            let new_flat = state.flatten();
                            if selected >= new_flat.len() {
                                state.selected_index = Some(new_flat.len().saturating_sub(1));
                            }
                            state.scroll.set_content_length(new_flat.len());
                            return Some(TreeOutput::Collapsed(path));
                        }
                    }
                }
                None
            }
            TreeMessage::Toggle => {
                if let Some(node_info) = flat.get(selected) {
                    if node_info.has_children {
                        let path = node_info.path.clone();
                        let was_expanded = node_info.is_expanded;
                        if let Some(node) = state.get_node_mut(&path) {
                            node.toggle();
                            if was_expanded {
                                // Adjust selected index if needed after collapse
                                let new_flat = state.flatten();
                                if selected >= new_flat.len() {
                                    state.selected_index = Some(new_flat.len().saturating_sub(1));
                                }
                                state.scroll.set_content_length(new_flat.len());
                                return Some(TreeOutput::Collapsed(path));
                            } else {
                                state.scroll.set_content_length(state.flatten().len());
                                return Some(TreeOutput::Expanded(path));
                            }
                        }
                    }
                }
                None
            }
            TreeMessage::Select => flat
                .get(selected)
                .map(|node_info| TreeOutput::Selected(node_info.path.clone())),
            TreeMessage::ExpandAll => {
                state.expand_all();
                // scroll content length already updated by expand_all()
                None
            }
            TreeMessage::CollapseAll => {
                state.collapse_all();
                // scroll content length already updated by collapse_all()
                None
            }
            TreeMessage::SetFilter(_) | TreeMessage::ClearFilter => {
                unreachable!("handled above")
            }
        }
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Up | Key::Char('k') => Some(TreeMessage::Up),
                Key::Down | Key::Char('j') => Some(TreeMessage::Down),
                Key::Left | Key::Char('h') => Some(TreeMessage::Collapse),
                Key::Right | Key::Char('l') => Some(TreeMessage::Expand),
                Key::Char(' ') => Some(TreeMessage::Toggle),
                Key::Enter => Some(TreeMessage::Select),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        render::view(state, ctx);
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
