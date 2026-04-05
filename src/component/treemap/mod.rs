//! A treemap component for hierarchical proportional data visualization.
//!
//! [`Treemap`] displays hierarchical data as nested rectangles where each
//! rectangle's area is proportional to its value. Useful for disk usage,
//! memory allocation by module, request volume by service/endpoint, and
//! similar visualizations. State is stored in [`TreemapState`], updated via
//! [`TreemapMessage`], and produces [`TreemapOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Treemap, TreemapState, TreemapNode,
//! };
//! use ratatui::style::Color;
//!
//! let root = TreemapNode::new("root", 0.0)
//!     .with_child(TreemapNode::new("src", 60.0).with_color(Color::Blue))
//!     .with_child(TreemapNode::new("docs", 30.0).with_color(Color::Green))
//!     .with_child(TreemapNode::new("tests", 10.0).with_color(Color::Yellow));
//!
//! let state = TreemapState::new().with_root(root);
//! assert!(state.root().is_some());
//! assert_eq!(state.root().unwrap().children.len(), 3);
//! ```

use std::marker::PhantomData;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Layout algorithm module.
pub mod layout;
mod render;

pub use layout::{squarified_layout, LayoutRect};

/// A node in the treemap hierarchy.
///
/// Each node has a label, a value (which determines its proportional area),
/// a color, and optional children. Leaf nodes use their own value; parent
/// nodes compute their total value as the sum of children's total values.
///
/// # Example
///
/// ```rust
/// use envision::component::TreemapNode;
/// use ratatui::style::Color;
///
/// let leaf = TreemapNode::new("file.rs", 42.0).with_color(Color::Cyan);
/// assert!(leaf.is_leaf());
/// assert_eq!(leaf.total_value(), 42.0);
///
/// let parent = TreemapNode::new("src", 0.0)
///     .with_child(TreemapNode::new("main.rs", 30.0))
///     .with_child(TreemapNode::new("lib.rs", 20.0));
/// assert!(!parent.is_leaf());
/// assert_eq!(parent.total_value(), 50.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TreemapNode {
    /// Display label.
    pub label: String,
    /// Value (determines area proportion for leaf nodes).
    pub value: f64,
    /// Color for this node's rectangle.
    pub color: Color,
    /// Child nodes (if any).
    pub children: Vec<TreemapNode>,
}

impl TreemapNode {
    /// Creates a new leaf node with the given label and value.
    ///
    /// The default color is `Color::Gray`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapNode;
    /// use ratatui::style::Color;
    ///
    /// let node = TreemapNode::new("data", 100.0);
    /// assert_eq!(node.label, "data");
    /// assert_eq!(node.value, 100.0);
    /// assert_eq!(node.color, Color::Gray);
    /// assert!(node.children.is_empty());
    /// ```
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            color: Color::Gray,
            children: Vec::new(),
        }
    }

    /// Sets the color for this node (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapNode;
    /// use ratatui::style::Color;
    ///
    /// let node = TreemapNode::new("data", 100.0).with_color(Color::Red);
    /// assert_eq!(node.color, Color::Red);
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the color for this node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapNode;
    /// use ratatui::style::Color;
    ///
    /// let mut node = TreemapNode::new("data", 100.0);
    /// node.set_color(Color::Red);
    /// assert_eq!(node.color, Color::Red);
    /// ```
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Adds a single child node (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapNode;
    ///
    /// let parent = TreemapNode::new("root", 0.0)
    ///     .with_child(TreemapNode::new("a", 10.0))
    ///     .with_child(TreemapNode::new("b", 20.0));
    /// assert_eq!(parent.children.len(), 2);
    /// ```
    pub fn with_child(mut self, child: TreemapNode) -> Self {
        self.children.push(child);
        self
    }

    /// Sets multiple children at once (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapNode;
    ///
    /// let children = vec![
    ///     TreemapNode::new("a", 10.0),
    ///     TreemapNode::new("b", 20.0),
    /// ];
    /// let parent = TreemapNode::new("root", 0.0).with_children(children);
    /// assert_eq!(parent.children.len(), 2);
    /// ```
    pub fn with_children(mut self, children: Vec<TreemapNode>) -> Self {
        self.children = children;
        self
    }

    /// Returns the total value of this node.
    ///
    /// For leaf nodes, this is the node's own value. For parent nodes,
    /// this is the sum of all children's total values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapNode;
    ///
    /// let leaf = TreemapNode::new("x", 42.0);
    /// assert_eq!(leaf.total_value(), 42.0);
    ///
    /// let parent = TreemapNode::new("p", 0.0)
    ///     .with_child(TreemapNode::new("a", 10.0))
    ///     .with_child(TreemapNode::new("b", 20.0));
    /// assert_eq!(parent.total_value(), 30.0);
    /// ```
    pub fn total_value(&self) -> f64 {
        if self.children.is_empty() {
            self.value
        } else {
            self.children.iter().map(|c| c.total_value()).sum()
        }
    }

    /// Returns true if this node has no children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapNode;
    ///
    /// let leaf = TreemapNode::new("leaf", 10.0);
    /// assert!(leaf.is_leaf());
    ///
    /// let parent = TreemapNode::new("parent", 0.0)
    ///     .with_child(TreemapNode::new("child", 5.0));
    /// assert!(!parent.is_leaf());
    /// ```
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

/// Messages that can be sent to a Treemap.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, Treemap, TreemapState, TreemapNode, TreemapMessage,
/// };
/// use ratatui::style::Color;
///
/// let root = TreemapNode::new("root", 0.0)
///     .with_child(TreemapNode::new("a", 30.0).with_color(Color::Red))
///     .with_child(TreemapNode::new("b", 20.0).with_color(Color::Blue));
///
/// let mut state = TreemapState::new().with_root(root);
/// state.set_focused(true);
/// state.update(TreemapMessage::SelectNext);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum TreemapMessage {
    /// Set the root node.
    SetRoot(TreemapNode),
    /// Clear the treemap.
    Clear,
    /// Zoom into the selected node.
    ZoomIn,
    /// Zoom out to the parent.
    ZoomOut,
    /// Reset zoom to the root.
    ResetZoom,
    /// Select the next sibling.
    SelectNext,
    /// Select the previous sibling.
    SelectPrev,
    /// Select the first child of the current selection.
    SelectChild,
    /// Select the parent of the current selection.
    SelectParent,
}

/// Output messages from a Treemap.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, Treemap, TreemapState, TreemapNode, TreemapOutput,
/// };
/// use ratatui::style::Color;
///
/// let root = TreemapNode::new("root", 0.0)
///     .with_child(
///         TreemapNode::new("a", 0.0)
///             .with_color(Color::Red)
///             .with_child(TreemapNode::new("x", 15.0))
///             .with_child(TreemapNode::new("y", 15.0)),
///     );
/// let mut state = TreemapState::new().with_root(root);
/// state.set_focused(true);
///
/// let output = state.dispatch_event(&envision::input::Event::key(
///     envision::input::KeyCode::Enter,
/// ));
/// assert_eq!(
///     output,
///     Some(TreemapOutput::ZoomedIn("a".to_string()))
/// );
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum TreemapOutput {
    /// A node was confirmed/selected.
    NodeSelected {
        /// Label of the selected node.
        label: String,
        /// Value of the selected node.
        value: f64,
    },
    /// Zoomed into a node.
    ZoomedIn(String),
    /// Zoomed out.
    ZoomedOut,
}

/// State for a Treemap component.
///
/// Contains the root node, selection path, zoom path, and display options.
///
/// # Example
///
/// ```rust
/// use envision::component::{TreemapState, TreemapNode};
/// use ratatui::style::Color;
///
/// let root = TreemapNode::new("root", 0.0)
///     .with_child(TreemapNode::new("src", 60.0).with_color(Color::Blue))
///     .with_child(TreemapNode::new("docs", 30.0).with_color(Color::Green));
///
/// let state = TreemapState::new()
///     .with_root(root)
///     .with_title("Disk Usage");
/// assert_eq!(state.title(), Some("Disk Usage"));
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TreemapState {
    /// The root node of the tree.
    root: Option<TreemapNode>,
    /// Path to the currently selected node (indices at each depth).
    selected_path: Vec<usize>,
    /// Path to the zoomed-in node (empty = show root).
    zoom_path: Vec<usize>,
    /// Optional title for the treemap.
    title: Option<String>,
    /// Whether to show labels in rectangles.
    show_labels: bool,
    /// Whether to show values in rectangles.
    show_values: bool,
}

impl TreemapState {
    /// Creates a new empty treemap state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapState;
    ///
    /// let state = TreemapState::new();
    /// assert!(state.root().is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            show_labels: true,
            ..Default::default()
        }
    }

    /// Sets the root node (builder pattern).
    ///
    /// Resets the selection and zoom paths.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreemapState, TreemapNode};
    ///
    /// let root = TreemapNode::new("root", 0.0)
    ///     .with_child(TreemapNode::new("a", 10.0));
    /// let state = TreemapState::new().with_root(root);
    /// assert!(state.root().is_some());
    /// ```
    pub fn with_root(mut self, root: TreemapNode) -> Self {
        self.root = Some(root);
        self.selected_path = vec![0];
        self.zoom_path = Vec::new();
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapState;
    ///
    /// let state = TreemapState::new().with_title("Memory Usage");
    /// assert_eq!(state.title(), Some("Memory Usage"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show labels (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapState;
    ///
    /// let state = TreemapState::new().with_show_labels(false);
    /// assert!(!state.show_labels());
    /// ```
    pub fn with_show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Sets whether to show values (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapState;
    ///
    /// let state = TreemapState::new().with_show_values(true);
    /// assert!(state.show_values());
    /// ```
    pub fn with_show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    // ---- Accessors ----

    /// Returns a reference to the root node, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreemapState, TreemapNode};
    ///
    /// let state = TreemapState::new().with_root(TreemapNode::new("r", 10.0));
    /// assert_eq!(state.root().unwrap().label, "r");
    /// ```
    pub fn root(&self) -> Option<&TreemapNode> {
        self.root.as_ref()
    }

    /// Sets the root node, resetting selection and zoom.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreemapState, TreemapNode};
    ///
    /// let mut state = TreemapState::new();
    /// state.set_root(TreemapNode::new("root", 0.0)
    ///     .with_child(TreemapNode::new("a", 10.0)));
    /// assert!(state.root().is_some());
    /// ```
    pub fn set_root(&mut self, root: TreemapNode) {
        self.root = Some(root);
        self.selected_path = vec![0];
        self.zoom_path = Vec::new();
    }

    /// Clears the treemap.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreemapState, TreemapNode};
    ///
    /// let mut state = TreemapState::new().with_root(TreemapNode::new("r", 10.0));
    /// state.clear();
    /// assert!(state.root().is_none());
    /// ```
    pub fn clear(&mut self) {
        self.root = None;
        self.selected_path.clear();
        self.zoom_path.clear();
    }

    /// Returns the node at the current zoom path.
    ///
    /// If the zoom path is empty, returns the root node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreemapState, TreemapNode};
    ///
    /// let root = TreemapNode::new("root", 0.0)
    ///     .with_child(TreemapNode::new("a", 10.0));
    /// let state = TreemapState::new().with_root(root);
    /// let view = state.current_view_node();
    /// assert_eq!(view.unwrap().label, "root");
    /// ```
    pub fn current_view_node(&self) -> Option<&TreemapNode> {
        let root = self.root.as_ref()?;
        navigate_to_node(root, &self.zoom_path)
    }

    /// Returns the currently selected node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreemapState, TreemapNode};
    /// use ratatui::style::Color;
    ///
    /// let root = TreemapNode::new("root", 0.0)
    ///     .with_child(TreemapNode::new("a", 30.0).with_color(Color::Red))
    ///     .with_child(TreemapNode::new("b", 20.0).with_color(Color::Blue));
    /// let state = TreemapState::new().with_root(root);
    /// let selected = state.selected_node();
    /// assert_eq!(selected.unwrap().label, "a");
    /// ```
    pub fn selected_node(&self) -> Option<&TreemapNode> {
        let view_node = self.current_view_node()?;
        if self.selected_path.is_empty() {
            return None;
        }
        let idx = self.selected_path[0];
        view_node.children.get(idx)
    }

    /// Returns the title, if set.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TreemapState;
    ///
    /// let mut state = TreemapState::default();
    /// state.set_title("Disk Usage");
    /// assert_eq!(state.title(), Some("Disk Usage"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns whether labels are shown.
    pub fn show_labels(&self) -> bool {
        self.show_labels
    }

    /// Sets whether labels are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreemapState, TreemapNode};
    ///
    /// let mut state = TreemapState::new();
    /// state.set_show_labels(false);
    /// assert!(!state.show_labels());
    /// ```
    pub fn set_show_labels(&mut self, show: bool) {
        self.show_labels = show;
    }

    /// Returns whether values are shown.
    pub fn show_values(&self) -> bool {
        self.show_values
    }

    /// Sets whether values are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TreemapState, TreemapNode};
    ///
    /// let mut state = TreemapState::new();
    /// state.set_show_values(true);
    /// assert!(state.show_values());
    /// ```
    pub fn set_show_values(&mut self, show: bool) {
        self.show_values = show;
    }

    /// Returns the selected child index at the current zoom level.
    pub(crate) fn selected_child_index(&self) -> usize {
        self.selected_path.first().copied().unwrap_or(0)
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: TreemapMessage) -> Option<TreemapOutput> {
        Treemap::update(self, msg)
    }

    /// Returns the number of children of the current view node.
    fn current_child_count(&self) -> usize {
        self.current_view_node()
            .map(|n| n.children.len())
            .unwrap_or(0)
    }
}

/// Navigate to a node at the given path from the root.
fn navigate_to_node<'a>(root: &'a TreemapNode, path: &[usize]) -> Option<&'a TreemapNode> {
    let mut current = root;
    for &idx in path {
        current = current.children.get(idx)?;
    }
    Some(current)
}

/// A treemap component for hierarchical proportional data visualization.
///
/// Renders hierarchical data as nested colored rectangles where area is
/// proportional to value. Supports keyboard navigation for selection and
/// zoom in/out.
///
/// # Key Bindings
///
/// - `Left` / `h` -- Select previous sibling
/// - `Right` / `l` -- Select next sibling
/// - `Down` / `j` -- Select first child
/// - `Up` / `k` -- Select parent
/// - `Enter` -- Zoom into selected node
/// - `Escape` / `Backspace` -- Zoom out to parent
/// - `Home` -- Reset zoom to root
pub struct Treemap(PhantomData<()>);

impl Component for Treemap {
    type State = TreemapState;
    type Message = TreemapMessage;
    type Output = TreemapOutput;

    fn init() -> Self::State {
        TreemapState::new()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => Some(TreemapMessage::SelectPrev),
            KeyCode::Right | KeyCode::Char('l') => Some(TreemapMessage::SelectNext),
            KeyCode::Down | KeyCode::Char('j') => Some(TreemapMessage::SelectChild),
            KeyCode::Up | KeyCode::Char('k') => Some(TreemapMessage::SelectParent),
            KeyCode::Enter => Some(TreemapMessage::ZoomIn),
            KeyCode::Esc | KeyCode::Backspace => Some(TreemapMessage::ZoomOut),
            KeyCode::Home => Some(TreemapMessage::ResetZoom),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            TreemapMessage::SetRoot(root) => {
                state.set_root(root);
                None
            }
            TreemapMessage::Clear => {
                state.clear();
                None
            }
            TreemapMessage::ZoomIn => {
                let child_count = state.current_child_count();
                if child_count == 0 {
                    return None;
                }

                let selected_idx = state.selected_child_index();
                if selected_idx >= child_count {
                    return None;
                }

                // Check if the selected node has children to zoom into.
                let has_children = state
                    .current_view_node()
                    .and_then(|n| n.children.get(selected_idx))
                    .map(|n| !n.children.is_empty())
                    .unwrap_or(false);

                let label = state
                    .current_view_node()
                    .and_then(|n| n.children.get(selected_idx))
                    .map(|n| n.label.clone())
                    .unwrap_or_default();

                if has_children {
                    state.zoom_path.push(selected_idx);
                    state.selected_path = vec![0];
                    Some(TreemapOutput::ZoomedIn(label))
                } else {
                    // Leaf node -- emit NodeSelected.
                    let value = state
                        .current_view_node()
                        .and_then(|n| n.children.get(selected_idx))
                        .map(|n| n.total_value())
                        .unwrap_or(0.0);
                    Some(TreemapOutput::NodeSelected { label, value })
                }
            }
            TreemapMessage::ZoomOut => {
                if state.zoom_path.is_empty() {
                    return None;
                }
                let popped = state.zoom_path.pop().unwrap_or(0);
                state.selected_path = vec![popped];
                Some(TreemapOutput::ZoomedOut)
            }
            TreemapMessage::ResetZoom => {
                if state.zoom_path.is_empty() {
                    return None;
                }
                state.zoom_path.clear();
                state.selected_path = vec![0];
                Some(TreemapOutput::ZoomedOut)
            }
            TreemapMessage::SelectNext => {
                let child_count = state.current_child_count();
                if child_count == 0 {
                    return None;
                }
                let current = state.selected_child_index();
                if current + 1 < child_count {
                    state.selected_path = vec![current + 1];
                }
                None
            }
            TreemapMessage::SelectPrev => {
                let child_count = state.current_child_count();
                if child_count == 0 {
                    return None;
                }
                let current = state.selected_child_index();
                if current > 0 {
                    state.selected_path = vec![current - 1];
                }
                None
            }
            TreemapMessage::SelectChild => {
                // Navigate into the children of the selected node.
                let child_count = state.current_child_count();
                if child_count == 0 {
                    return None;
                }
                let selected_idx = state.selected_child_index();
                let has_children = state
                    .current_view_node()
                    .and_then(|n| n.children.get(selected_idx))
                    .map(|n| !n.children.is_empty())
                    .unwrap_or(false);

                if has_children {
                    // Zoom in and select first child.
                    let label = state
                        .current_view_node()
                        .and_then(|n| n.children.get(selected_idx))
                        .map(|n| n.label.clone())
                        .unwrap_or_default();
                    state.zoom_path.push(selected_idx);
                    state.selected_path = vec![0];
                    Some(TreemapOutput::ZoomedIn(label))
                } else {
                    None
                }
            }
            TreemapMessage::SelectParent => {
                if state.zoom_path.is_empty() {
                    return None;
                }
                let popped = state.zoom_path.pop().unwrap_or(0);
                state.selected_path = vec![popped];
                Some(TreemapOutput::ZoomedOut)
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, ctx: &ViewContext) {
        if area.height < 3 || area.width < 3 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::container("treemap")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let border_style = if ctx.disabled {
            theme.disabled_style()
        } else if ctx.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(ref title) = state.title {
            block = block.title(title.as_str());
        }

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 || state.root.is_none() {
            return;
        }

        render::render_treemap(state, frame, inner, theme, ctx.focused, ctx.disabled);
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
