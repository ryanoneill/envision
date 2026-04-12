//! A dependency graph component for visualizing service/component relationships.
//!
//! [`DependencyGraph`] renders a directed graph of nodes and edges,
//! supporting status coloring, selection, and hierarchical layout. Useful
//! for service mesh topology, architecture diagrams, and dependency
//! visualization.
//!
//! State is stored in [`DependencyGraphState`], updated via
//! [`DependencyGraphMessage`], and produces [`DependencyGraphOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, DependencyGraph, DependencyGraphState,
//!     DependencyGraphMessage, GraphNode, GraphEdge, NodeStatus,
//! };
//!
//! let mut state = DependencyGraphState::new()
//!     .with_node(GraphNode::new("api", "API Gateway").with_status(NodeStatus::Healthy))
//!     .with_node(GraphNode::new("db", "Database").with_status(NodeStatus::Degraded))
//!     .with_edge(GraphEdge::new("api", "db"));
//!
//! // Select and navigate
//! DependencyGraph::update(&mut state, DependencyGraphMessage::SelectNext);
//! assert_eq!(state.selected(), Some(0));
//! ```

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, KeyCode};

pub mod layout;
mod render;
mod types;

pub use layout::{LayoutEdge, LayoutNode};
pub use types::{GraphEdge, GraphNode, GraphOrientation, NodeStatus};

// =============================================================================
// Messages
// =============================================================================

/// Messages that can be sent to a DependencyGraph component.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, DependencyGraph, DependencyGraphState,
///     DependencyGraphMessage, GraphNode,
/// };
///
/// let mut state = DependencyGraphState::new()
///     .with_node(GraphNode::new("a", "Node A"))
///     .with_node(GraphNode::new("b", "Node B"));
/// DependencyGraph::update(&mut state, DependencyGraphMessage::SelectNext);
/// assert_eq!(state.selected(), Some(0));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum DependencyGraphMessage {
    /// Replace all nodes.
    SetNodes(Vec<GraphNode>),
    /// Replace all edges.
    SetEdges(Vec<GraphEdge>),
    /// Add a single node.
    AddNode(GraphNode),
    /// Add a single edge.
    AddEdge(GraphEdge),
    /// Update the status of a node by id.
    UpdateNodeStatus {
        /// The node id to update.
        id: String,
        /// The new status.
        status: NodeStatus,
    },
    /// Clear all nodes and edges.
    Clear,
    /// Select the next node.
    SelectNext,
    /// Select the previous node.
    SelectPrev,
    /// Select a connected node (follow an outgoing edge).
    SelectConnected,
}

/// Output messages from a DependencyGraph component.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, DependencyGraph, DependencyGraphState,
///     DependencyGraphMessage, DependencyGraphOutput, GraphNode,
/// };
///
/// let mut state = DependencyGraphState::new()
///     .with_node(GraphNode::new("a", "Node A"));
///
/// let output = DependencyGraph::update(&mut state, DependencyGraphMessage::SelectNext);
/// assert!(matches!(output, Some(DependencyGraphOutput::NodeSelected(_))));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum DependencyGraphOutput {
    /// A node was selected.
    NodeSelected(String),
    /// A node's status was changed.
    StatusChanged {
        /// The node id.
        id: String,
        /// The previous status.
        old: NodeStatus,
        /// The new status.
        new_status: NodeStatus,
    },
}

// =============================================================================
// State
// =============================================================================

/// State for a DependencyGraph component.
///
/// Contains the graph nodes, edges, selection state, and display options.
///
/// # Example
///
/// ```rust
/// use envision::component::{DependencyGraphState, GraphNode, GraphEdge};
///
/// let state = DependencyGraphState::new()
///     .with_node(GraphNode::new("api", "API"))
///     .with_node(GraphNode::new("db", "Database"))
///     .with_edge(GraphEdge::new("api", "db"))
///     .with_title("Service Topology");
/// assert_eq!(state.nodes().len(), 2);
/// assert_eq!(state.edges().len(), 1);
/// assert_eq!(state.title(), Some("Service Topology"));
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DependencyGraphState {
    /// All nodes in the graph.
    pub(crate) nodes: Vec<GraphNode>,
    /// All edges in the graph.
    pub(crate) edges: Vec<GraphEdge>,
    /// Index of the currently selected node.
    pub(crate) selected: Option<usize>,
    /// Optional title.
    pub(crate) title: Option<String>,
    /// Layout orientation.
    pub(crate) orientation: GraphOrientation,
    /// Whether to show labels on edges.
    pub(crate) show_edge_labels: bool,
}

impl DependencyGraphState {
    /// Creates a new empty dependency graph state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let state = DependencyGraphState::new();
    /// assert!(state.nodes().is_empty());
    /// assert!(state.edges().is_empty());
    /// assert_eq!(state.selected(), None);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode};
    ///
    /// let state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "Node A"));
    /// assert_eq!(state.nodes().len(), 1);
    /// ```
    pub fn with_node(mut self, node: GraphNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Adds an edge (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode, GraphEdge};
    ///
    /// let state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "A"))
    ///     .with_node(GraphNode::new("b", "B"))
    ///     .with_edge(GraphEdge::new("a", "b"));
    /// assert_eq!(state.edges().len(), 1);
    /// ```
    pub fn with_edge(mut self, edge: GraphEdge) -> Self {
        self.edges.push(edge);
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let state = DependencyGraphState::new().with_title("Topology");
    /// assert_eq!(state.title(), Some("Topology"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the orientation (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphOrientation};
    ///
    /// let state = DependencyGraphState::new()
    ///     .with_orientation(GraphOrientation::TopToBottom);
    /// assert_eq!(state.orientation(), &GraphOrientation::TopToBottom);
    /// ```
    pub fn with_orientation(mut self, orientation: GraphOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets whether to show edge labels (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let state = DependencyGraphState::new().with_show_edge_labels(true);
    /// assert!(state.show_edge_labels());
    /// ```
    pub fn with_show_edge_labels(mut self, show: bool) -> Self {
        self.show_edge_labels = show;
        self
    }

    // ---- Accessors ----

    /// Returns all nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let state = DependencyGraphState::new();
    /// assert!(state.nodes().is_empty());
    /// ```
    pub fn nodes(&self) -> &[GraphNode] {
        &self.nodes
    }

    /// Returns a mutable reference to all nodes.
    ///
    /// This is safe because nodes are simple data containers.
    /// Selection is index-based and edges reference nodes by id,
    /// not by position.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode, NodeStatus};
    ///
    /// let mut state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("api", "API Gateway"));
    /// state.nodes_mut()[0].status = NodeStatus::Degraded;
    /// assert_eq!(state.nodes()[0].status, NodeStatus::Degraded);
    /// ```
    pub fn nodes_mut(&mut self) -> &mut Vec<GraphNode> {
        &mut self.nodes
    }

    /// Returns all edges.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let state = DependencyGraphState::new();
    /// assert!(state.edges().is_empty());
    /// ```
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    /// Returns a mutable reference to all edges.
    ///
    /// This is safe because edges are simple data containers with
    /// no derived indices or filter state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode, GraphEdge};
    ///
    /// let mut state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "A"))
    ///     .with_node(GraphNode::new("b", "B"))
    ///     .with_edge(GraphEdge::new("a", "b").with_label("calls"));
    /// state.edges_mut()[0] = GraphEdge::new("a", "b").with_label("depends on");
    /// assert_eq!(state.edges()[0].label.as_deref(), Some("depends on"));
    /// ```
    pub fn edges_mut(&mut self) -> &mut Vec<GraphEdge> {
        &mut self.edges
    }

    /// Returns the selected node index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let state = DependencyGraphState::new();
    /// assert_eq!(state.selected(), None);
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Returns the currently selected node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode};
    ///
    /// let mut state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "A"));
    /// assert!(state.selected_node().is_none());
    /// state.select_next();
    /// assert_eq!(state.selected_node().unwrap().id, "a");
    /// ```
    pub fn selected_node(&self) -> Option<&GraphNode> {
        self.selected.and_then(|idx| self.nodes.get(idx))
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let state = DependencyGraphState::new();
    /// assert_eq!(state.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let mut state = DependencyGraphState::new();
    /// state.set_title("Service Topology");
    /// assert_eq!(state.title(), Some("Service Topology"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns the orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphOrientation};
    ///
    /// let state = DependencyGraphState::new();
    /// assert_eq!(state.orientation(), &GraphOrientation::LeftToRight);
    /// ```
    pub fn orientation(&self) -> &GraphOrientation {
        &self.orientation
    }

    /// Returns whether edge labels are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let state = DependencyGraphState::new();
    /// assert!(!state.show_edge_labels());
    /// ```
    pub fn show_edge_labels(&self) -> bool {
        self.show_edge_labels
    }

    /// Sets the graph orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphOrientation};
    ///
    /// let mut state = DependencyGraphState::new();
    /// state.set_orientation(GraphOrientation::TopToBottom);
    /// assert_eq!(state.orientation(), &GraphOrientation::TopToBottom);
    /// ```
    pub fn set_orientation(&mut self, orientation: GraphOrientation) {
        self.orientation = orientation;
    }

    /// Sets whether edge labels are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DependencyGraphState;
    ///
    /// let mut state = DependencyGraphState::new();
    /// state.set_show_edge_labels(true);
    /// assert!(state.show_edge_labels());
    /// ```
    pub fn set_show_edge_labels(&mut self, show: bool) {
        self.show_edge_labels = show;
    }

    // ---- Mutation ----

    /// Adds a node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode};
    ///
    /// let mut state = DependencyGraphState::new();
    /// state.add_node(GraphNode::new("a", "A"));
    /// assert_eq!(state.nodes().len(), 1);
    /// ```
    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.push(node);
    }

    /// Adds an edge.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode, GraphEdge};
    ///
    /// let mut state = DependencyGraphState::new();
    /// state.add_node(GraphNode::new("a", "A"));
    /// state.add_node(GraphNode::new("b", "B"));
    /// state.add_edge(GraphEdge::new("a", "b"));
    /// assert_eq!(state.edges().len(), 1);
    /// ```
    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    /// Clears all nodes and edges.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode};
    ///
    /// let mut state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "A"));
    /// state.clear();
    /// assert!(state.nodes().is_empty());
    /// assert_eq!(state.selected(), None);
    /// ```
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.selected = None;
    }

    /// Selects the next node.
    ///
    /// Returns true if the selection changed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode};
    ///
    /// let mut state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "A"))
    ///     .with_node(GraphNode::new("b", "B"));
    /// assert!(state.select_next());
    /// assert_eq!(state.selected(), Some(0));
    /// assert!(state.select_next());
    /// assert_eq!(state.selected(), Some(1));
    /// ```
    pub fn select_next(&mut self) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        match self.selected {
            None => {
                self.selected = Some(0);
                true
            }
            Some(idx) => {
                if idx + 1 < self.nodes.len() {
                    self.selected = Some(idx + 1);
                    true
                } else {
                    // Wrap to first
                    self.selected = Some(0);
                    true
                }
            }
        }
    }

    /// Selects the previous node.
    ///
    /// Returns true if the selection changed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode};
    ///
    /// let mut state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "A"))
    ///     .with_node(GraphNode::new("b", "B"));
    /// state.select_next(); // select a
    /// state.select_next(); // select b
    /// assert!(state.select_prev());
    /// assert_eq!(state.selected(), Some(0));
    /// ```
    pub fn select_prev(&mut self) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        match self.selected {
            None => {
                self.selected = Some(self.nodes.len() - 1);
                true
            }
            Some(idx) => {
                if idx > 0 {
                    self.selected = Some(idx - 1);
                    true
                } else {
                    // Wrap to last
                    self.selected = Some(self.nodes.len() - 1);
                    true
                }
            }
        }
    }

    /// Selects a connected node by following an outgoing edge from the
    /// currently selected node.
    ///
    /// Returns true if the selection changed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode, GraphEdge};
    ///
    /// let mut state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "A"))
    ///     .with_node(GraphNode::new("b", "B"))
    ///     .with_edge(GraphEdge::new("a", "b"));
    /// state.select_next(); // select a
    /// assert!(state.select_connected());
    /// assert_eq!(state.selected_node().unwrap().id, "b");
    /// ```
    pub fn select_connected(&mut self) -> bool {
        let selected_idx = match self.selected {
            Some(idx) => idx,
            None => return false,
        };
        let selected_id = match self.nodes.get(selected_idx) {
            Some(node) => &node.id,
            None => return false,
        };

        // Find the first outgoing edge from the selected node
        let target_id = self
            .edges
            .iter()
            .find(|e| e.from == *selected_id)
            .map(|e| e.to.clone());

        if let Some(target) = target_id {
            if let Some(target_idx) = self.nodes.iter().position(|n| n.id == target) {
                self.selected = Some(target_idx);
                return true;
            }
        }
        false
    }

    /// Updates the status of a node by id.
    ///
    /// Returns the old status if the node was found.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, GraphNode, NodeStatus};
    ///
    /// let mut state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "A").with_status(NodeStatus::Healthy));
    /// let old = state.update_node_status("a", NodeStatus::Down);
    /// assert_eq!(old, Some(NodeStatus::Healthy));
    /// assert_eq!(state.nodes()[0].status, NodeStatus::Down);
    /// ```
    pub fn update_node_status(&mut self, id: &str, status: NodeStatus) -> Option<NodeStatus> {
        self.nodes
            .iter_mut()
            .find(|n| n.id == id)
            .map(|node| std::mem::replace(&mut node.status, status))
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DependencyGraphState, DependencyGraphMessage, GraphNode};
    ///
    /// let mut state = DependencyGraphState::new()
    ///     .with_node(GraphNode::new("a", "A"));
    /// let output = state.update(DependencyGraphMessage::SelectNext);
    /// assert!(output.is_some());
    /// ```
    pub fn update(&mut self, msg: DependencyGraphMessage) -> Option<DependencyGraphOutput> {
        DependencyGraph::update(self, msg)
    }
}

// =============================================================================
// Component
// =============================================================================

/// A dependency graph component for visualizing service/component relationships.
///
/// Renders a directed graph of nodes connected by edges, with status coloring
/// and selection support. Nodes are positioned using a layered hierarchical
/// layout algorithm.
///
/// # Visual Format
///
/// ```text
/// +- Service Topology ---------------------------------+
/// | +----- API ------+     +--- Database --+           |
/// | | * Healthy      |---->| ! Degraded    |           |
/// | +----------------+     +--------------+            |
/// |        |                                           |
/// |        v                                           |
/// | +----- Cache -----+                                |
/// | | * Healthy       |                                |
/// | +-----------------+                                |
/// +----------------------------------------------------+
/// ```
///
/// # Keyboard Navigation
///
/// - `Down/j/Tab`: Select next node
/// - `Up/k/Shift+Tab`: Select previous node
/// - `Enter/l/Right`: Follow edge to connected node
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, DependencyGraph, DependencyGraphState,
///     DependencyGraphMessage, GraphNode, GraphEdge, NodeStatus,
/// };
///
/// let mut state = DependencyGraphState::new()
///     .with_node(GraphNode::new("api", "API Gateway").with_status(NodeStatus::Healthy))
///     .with_node(GraphNode::new("db", "Database").with_status(NodeStatus::Degraded))
///     .with_edge(GraphEdge::new("api", "db"))
///     .with_title("Service Topology");
///
/// DependencyGraph::update(&mut state, DependencyGraphMessage::SelectNext);
/// assert_eq!(state.selected_node().unwrap().id, "api");
/// ```
pub struct DependencyGraph;

impl Component for DependencyGraph {
    type State = DependencyGraphState;
    type Message = DependencyGraphMessage;
    type Output = DependencyGraphOutput;

    fn init() -> Self::State {
        DependencyGraphState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            DependencyGraphMessage::SetNodes(nodes) => {
                state.nodes = nodes;
                state.selected = None;
                None
            }
            DependencyGraphMessage::SetEdges(edges) => {
                state.edges = edges;
                None
            }
            DependencyGraphMessage::AddNode(node) => {
                state.add_node(node);
                None
            }
            DependencyGraphMessage::AddEdge(edge) => {
                state.add_edge(edge);
                None
            }
            DependencyGraphMessage::UpdateNodeStatus { id, status } => {
                if let Some(old) = state.update_node_status(&id, status.clone()) {
                    if old != status {
                        Some(DependencyGraphOutput::StatusChanged {
                            id,
                            old,
                            new_status: status,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            DependencyGraphMessage::Clear => {
                state.clear();
                None
            }
            DependencyGraphMessage::SelectNext => {
                if state.select_next() {
                    make_selected_output(state)
                } else {
                    None
                }
            }
            DependencyGraphMessage::SelectPrev => {
                if state.select_prev() {
                    make_selected_output(state)
                } else {
                    None
                }
            }
            DependencyGraphMessage::SelectConnected => {
                if state.select_connected() {
                    make_selected_output(state)
                } else {
                    None
                }
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
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                    Some(DependencyGraphMessage::SelectNext)
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                    Some(DependencyGraphMessage::SelectPrev)
                }
                KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                    Some(DependencyGraphMessage::SelectConnected)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        render::render_dependency_graph(
            state,
            ctx.frame,
            ctx.area,
            ctx.theme,
            ctx.focused,
            ctx.disabled,
        );
    }
}

/// Creates a `NodeSelected` output from the current state.
fn make_selected_output(state: &DependencyGraphState) -> Option<DependencyGraphOutput> {
    state
        .selected_node()
        .map(|node| DependencyGraphOutput::NodeSelected(node.id.clone()))
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
