//! A graph visualization component for rendering diagrams.
//!
//! The Diagram component renders directed graphs with automatic layout,
//! edge routing, clusters, and interactive navigation. It supports two
//! layout algorithms — hierarchical (Sugiyama) and force-directed
//! (Fruchterman-Reingold) — and renders using Unicode box-drawing
//! characters or Braille patterns for higher fidelity.
//!
//! # Examples
//!
//! ```
//! use envision::diagram::{DiagramState, DiagramNode, DiagramEdge, NodeStatus};
//!
//! let state = DiagramState::new()
//!     .with_node(DiagramNode::new("api", "API Gateway").with_status(NodeStatus::Healthy))
//!     .with_node(DiagramNode::new("db", "Database").with_status(NodeStatus::Degraded))
//!     .with_edge(DiagramEdge::new("api", "db"))
//!     .with_title("Service Mesh");
//!
//! assert_eq!(state.nodes().len(), 2);
//! assert_eq!(state.edges().len(), 1);
//! assert_eq!(state.title(), Some("Service Mesh"));
//! ```

use std::collections::HashSet;

use crate::component::Component;
use crate::component::context::{EventContext, RenderContext};
use crate::input::Event;

// TODO(Phase 3): Remove dead_code allow when render.rs uses graph/layout
#[allow(dead_code)]
mod graph;
pub mod layout;
pub mod types;
mod viewport;

pub use layout::{EdgePath, LayoutResult, NodePosition, PathSegment};
pub use types::{
    DiagramCluster, DiagramEdge, DiagramNode, EdgeStyle, LayoutMode, NodeShape, NodeStatus,
    Orientation, RenderMode,
};
pub use viewport::{BoundingBox, Viewport2D};

// TODO(Phase 3): Remove dead_code allow when render.rs uses layout
#[allow(unused_imports)]
use graph::IndexedGraph;
#[allow(unused_imports)]
use layout::{LayoutAlgorithm, LayoutHints, SugiyamaLayout};

/// State for the Diagram component.
///
/// Holds the graph data (nodes, edges, clusters), layout configuration,
/// viewport state, and interaction state. The layout is cached and only
/// recomputed when the graph data changes.
///
/// # Examples
///
/// ```
/// use envision::diagram::{DiagramState, DiagramNode, DiagramEdge};
///
/// let state = DiagramState::new()
///     .with_node(DiagramNode::new("a", "Node A"))
///     .with_node(DiagramNode::new("b", "Node B"))
///     .with_edge(DiagramEdge::new("a", "b"));
///
/// assert_eq!(state.nodes().len(), 2);
/// assert_eq!(state.edges().len(), 1);
/// assert_eq!(state.selected(), None);
/// ```
#[derive(Clone, Debug, Default)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DiagramState {
    // Data
    pub(crate) nodes: Vec<DiagramNode>,
    pub(crate) edges: Vec<DiagramEdge>,
    pub(crate) clusters: Vec<DiagramCluster>,

    // Selection
    pub(crate) selected: Option<usize>,
    pub(crate) selection_history: Vec<usize>,

    // Layout config
    pub(crate) layout_mode: LayoutMode,
    pub(crate) orientation: Orientation,
    #[cfg_attr(feature = "serialization", serde(skip))]
    #[allow(dead_code)] // Used in Phase 3 rendering
    pub(crate) cached_layout: Option<LayoutResult>,
    pub(crate) layout_dirty: bool,

    // Viewport
    pub(crate) viewport: Viewport2D,

    // Display options
    pub(crate) title: Option<String>,
    pub(crate) show_edge_labels: bool,
    pub(crate) render_mode: RenderMode,
    pub(crate) show_minimap: bool,
    pub(crate) expanded_nodes: HashSet<String>,
    pub(crate) collapsed_clusters: HashSet<String>,
}

// Manual PartialEq (skip cached_layout since it's derived from data)
impl PartialEq for DiagramState {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
            && self.edges == other.edges
            && self.clusters == other.clusters
            && self.selected == other.selected
            && self.layout_mode == other.layout_mode
            && self.orientation == other.orientation
            && self.viewport == other.viewport
            && self.title == other.title
            && self.show_edge_labels == other.show_edge_labels
            && self.render_mode == other.render_mode
            && self.show_minimap == other.show_minimap
    }
}

impl DiagramState {
    /// Creates a new empty diagram state.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new();
    /// assert!(state.nodes().is_empty());
    /// assert!(state.edges().is_empty());
    /// assert!(state.clusters().is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    // -- Builders --

    /// Adds a node to the diagram.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode};
    ///
    /// let state = DiagramState::new()
    ///     .with_node(DiagramNode::new("api", "API"));
    /// assert_eq!(state.nodes().len(), 1);
    /// ```
    pub fn with_node(mut self, node: DiagramNode) -> Self {
        self.nodes.push(node);
        self.layout_dirty = true;
        self
    }

    /// Adds an edge to the diagram.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode, DiagramEdge};
    ///
    /// let state = DiagramState::new()
    ///     .with_node(DiagramNode::new("a", "A"))
    ///     .with_node(DiagramNode::new("b", "B"))
    ///     .with_edge(DiagramEdge::new("a", "b"));
    /// assert_eq!(state.edges().len(), 1);
    /// ```
    pub fn with_edge(mut self, edge: DiagramEdge) -> Self {
        self.edges.push(edge);
        self.layout_dirty = true;
        self
    }

    /// Adds a cluster to the diagram.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramCluster};
    ///
    /// let state = DiagramState::new()
    ///     .with_cluster(DiagramCluster::new("prod", "Production"));
    /// assert_eq!(state.clusters().len(), 1);
    /// ```
    pub fn with_cluster(mut self, cluster: DiagramCluster) -> Self {
        self.clusters.push(cluster);
        self.layout_dirty = true;
        self
    }

    /// Sets the diagram title.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new().with_title("My Diagram");
    /// assert_eq!(state.title(), Some("My Diagram"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the layout algorithm.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, LayoutMode};
    ///
    /// let state = DiagramState::new().with_layout_mode(LayoutMode::ForceDirected);
    /// assert_eq!(state.layout_mode(), &LayoutMode::ForceDirected);
    /// ```
    pub fn with_layout_mode(mut self, mode: LayoutMode) -> Self {
        self.layout_mode = mode;
        self.layout_dirty = true;
        self
    }

    /// Sets the layout orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, Orientation};
    ///
    /// let state = DiagramState::new().with_orientation(Orientation::TopToBottom);
    /// assert_eq!(state.orientation(), &Orientation::TopToBottom);
    /// ```
    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self.layout_dirty = true;
        self
    }

    /// Sets the rendering mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, RenderMode};
    ///
    /// let state = DiagramState::new().with_render_mode(RenderMode::Braille);
    /// assert_eq!(state.render_mode(), &RenderMode::Braille);
    /// ```
    pub fn with_render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    /// Enables or disables edge labels.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new().with_show_edge_labels(true);
    /// assert!(state.show_edge_labels());
    /// ```
    pub fn with_show_edge_labels(mut self, show: bool) -> Self {
        self.show_edge_labels = show;
        self
    }

    /// Enables or disables the minimap.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new().with_show_minimap(true);
    /// assert!(state.show_minimap());
    /// ```
    pub fn with_show_minimap(mut self, show: bool) -> Self {
        self.show_minimap = show;
        self
    }

    // -- Getters --

    /// Returns the nodes in the diagram.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new();
    /// assert!(state.nodes().is_empty());
    /// ```
    pub fn nodes(&self) -> &[DiagramNode] {
        &self.nodes
    }

    /// Returns mutable access to the nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode};
    ///
    /// let mut state = DiagramState::new()
    ///     .with_node(DiagramNode::new("a", "A"));
    /// state.nodes_mut().push(DiagramNode::new("b", "B"));
    /// assert_eq!(state.nodes().len(), 2);
    /// ```
    pub fn nodes_mut(&mut self) -> &mut Vec<DiagramNode> {
        self.layout_dirty = true;
        &mut self.nodes
    }

    /// Returns the edges in the diagram.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new();
    /// assert!(state.edges().is_empty());
    /// ```
    pub fn edges(&self) -> &[DiagramEdge] {
        &self.edges
    }

    /// Returns mutable access to the edges.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramEdge};
    ///
    /// let mut state = DiagramState::new();
    /// state.edges_mut().push(DiagramEdge::new("a", "b"));
    /// assert_eq!(state.edges().len(), 1);
    /// ```
    pub fn edges_mut(&mut self) -> &mut Vec<DiagramEdge> {
        self.layout_dirty = true;
        &mut self.edges
    }

    /// Returns the clusters in the diagram.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new();
    /// assert!(state.clusters().is_empty());
    /// ```
    pub fn clusters(&self) -> &[DiagramCluster] {
        &self.clusters
    }

    /// Returns the selected node index, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new();
    /// assert_eq!(state.selected(), None);
    /// ```
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Returns the selected node, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode};
    ///
    /// let state = DiagramState::new()
    ///     .with_node(DiagramNode::new("api", "API"));
    /// assert_eq!(state.selected_node(), None);
    /// ```
    pub fn selected_node(&self) -> Option<&DiagramNode> {
        self.selected.and_then(|i| self.nodes.get(i))
    }

    /// Returns the diagram title, if set.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new();
    /// assert_eq!(state.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the current layout mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, LayoutMode};
    ///
    /// let state = DiagramState::new();
    /// assert_eq!(state.layout_mode(), &LayoutMode::Hierarchical);
    /// ```
    pub fn layout_mode(&self) -> &LayoutMode {
        &self.layout_mode
    }

    /// Returns the current layout orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, Orientation};
    ///
    /// let state = DiagramState::new();
    /// assert_eq!(state.orientation(), &Orientation::LeftToRight);
    /// ```
    pub fn orientation(&self) -> &Orientation {
        &self.orientation
    }

    /// Returns the rendering mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, RenderMode};
    ///
    /// let state = DiagramState::new();
    /// assert_eq!(state.render_mode(), &RenderMode::BoxDrawing);
    /// ```
    pub fn render_mode(&self) -> &RenderMode {
        &self.render_mode
    }

    /// Returns whether edge labels are shown.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new();
    /// assert!(!state.show_edge_labels());
    /// ```
    pub fn show_edge_labels(&self) -> bool {
        self.show_edge_labels
    }

    /// Returns whether the minimap is shown.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let state = DiagramState::new();
    /// assert!(!state.show_minimap());
    /// ```
    pub fn show_minimap(&self) -> bool {
        self.show_minimap
    }

    // -- Setters --

    /// Sets the diagram title.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::DiagramState;
    ///
    /// let mut state = DiagramState::new();
    /// state.set_title("Updated");
    /// assert_eq!(state.title(), Some("Updated"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Sets the layout mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, LayoutMode};
    ///
    /// let mut state = DiagramState::new();
    /// state.set_layout_mode(LayoutMode::ForceDirected);
    /// assert_eq!(state.layout_mode(), &LayoutMode::ForceDirected);
    /// ```
    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout_mode = mode;
        self.layout_dirty = true;
    }

    /// Sets the layout orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, Orientation};
    ///
    /// let mut state = DiagramState::new();
    /// state.set_orientation(Orientation::TopToBottom);
    /// assert_eq!(state.orientation(), &Orientation::TopToBottom);
    /// ```
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
        self.layout_dirty = true;
    }

    // -- Mutations --

    /// Adds a node to the diagram.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode};
    ///
    /// let mut state = DiagramState::new();
    /// state.add_node(DiagramNode::new("a", "A"));
    /// assert_eq!(state.nodes().len(), 1);
    /// ```
    pub fn add_node(&mut self, node: DiagramNode) {
        self.nodes.push(node);
        self.layout_dirty = true;
    }

    /// Adds an edge to the diagram.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramEdge};
    ///
    /// let mut state = DiagramState::new();
    /// state.add_edge(DiagramEdge::new("a", "b"));
    /// assert_eq!(state.edges().len(), 1);
    /// ```
    pub fn add_edge(&mut self, edge: DiagramEdge) {
        self.edges.push(edge);
        self.layout_dirty = true;
    }

    /// Removes a node by ID, returning it if found.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode};
    ///
    /// let mut state = DiagramState::new()
    ///     .with_node(DiagramNode::new("a", "A"));
    /// let removed = state.remove_node("a");
    /// assert!(removed.is_some());
    /// assert!(state.nodes().is_empty());
    /// ```
    pub fn remove_node(&mut self, id: &str) -> Option<DiagramNode> {
        let pos = self.nodes.iter().position(|n| n.id() == id)?;
        // Adjust selected index
        match self.selected {
            Some(sel) if sel == pos => self.selected = None,
            Some(sel) if sel > pos => self.selected = Some(sel - 1),
            _ => {}
        }
        self.layout_dirty = true;
        Some(self.nodes.remove(pos))
    }

    /// Removes an edge by (from, to) pair, returning it if found.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramEdge};
    ///
    /// let mut state = DiagramState::new()
    ///     .with_edge(DiagramEdge::new("a", "b"));
    /// let removed = state.remove_edge("a", "b");
    /// assert!(removed.is_some());
    /// assert!(state.edges().is_empty());
    /// ```
    pub fn remove_edge(&mut self, from: &str, to: &str) -> Option<DiagramEdge> {
        let pos = self
            .edges
            .iter()
            .position(|e| e.from() == from && e.to() == to)?;
        self.layout_dirty = true;
        Some(self.edges.remove(pos))
    }

    /// Updates a node's status, returning the old status if found.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode, NodeStatus};
    ///
    /// let mut state = DiagramState::new()
    ///     .with_node(DiagramNode::new("db", "Database"));
    /// let old = state.update_node_status("db", NodeStatus::Down);
    /// assert_eq!(old, Some(NodeStatus::Healthy));
    /// assert_eq!(state.nodes()[0].status(), &NodeStatus::Down);
    /// ```
    pub fn update_node_status(&mut self, id: &str, status: NodeStatus) -> Option<NodeStatus> {
        let node = self.nodes.iter_mut().find(|n| n.id() == id)?;
        let old = node.status().clone();
        node.set_status(status);
        // Status changes do NOT dirty the layout
        Some(old)
    }

    /// Clears all nodes, edges, clusters, and selection.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode};
    ///
    /// let mut state = DiagramState::new()
    ///     .with_node(DiagramNode::new("a", "A"));
    /// state.clear();
    /// assert!(state.nodes().is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.clusters.clear();
        self.selected = None;
        self.selection_history.clear();
        self.layout_dirty = true;
    }

    // -- Navigation --

    /// Selects the next node (by index order).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode};
    ///
    /// let mut state = DiagramState::new()
    ///     .with_node(DiagramNode::new("a", "A"))
    ///     .with_node(DiagramNode::new("b", "B"));
    /// assert!(state.select_next());
    /// assert_eq!(state.selected(), Some(0));
    /// assert!(state.select_next());
    /// assert_eq!(state.selected(), Some(1));
    /// ```
    pub fn select_next(&mut self) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        self.selected = Some(match self.selected {
            None => 0,
            Some(i) => (i + 1) % self.nodes.len(),
        });
        true
    }

    /// Selects the previous node (by index order).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::diagram::{DiagramState, DiagramNode};
    ///
    /// let mut state = DiagramState::new()
    ///     .with_node(DiagramNode::new("a", "A"))
    ///     .with_node(DiagramNode::new("b", "B"));
    /// state.select_next(); // select "a"
    /// state.select_prev(); // wrap to "b"
    /// assert_eq!(state.selected(), Some(1));
    /// ```
    pub fn select_prev(&mut self) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        self.selected = Some(match self.selected {
            None => self.nodes.len() - 1,
            Some(0) => self.nodes.len() - 1,
            Some(i) => i - 1,
        });
        true
    }

    /// Computes the layout if dirty, returning a reference to the cached result.
    #[allow(dead_code)] // Used in Phase 3 rendering
    pub(crate) fn ensure_layout(&mut self) -> &LayoutResult {
        if self.layout_dirty || self.cached_layout.is_none() {
            let graph = IndexedGraph::build(&self.nodes, &self.edges);
            let hints = LayoutHints {
                orientation: self.orientation.clone(),
                previous_layout: self.cached_layout.as_ref(),
                ..LayoutHints::default()
            };

            let result = match self.layout_mode {
                LayoutMode::Hierarchical => SugiyamaLayout::default().compute(
                    &graph,
                    &self.nodes,
                    &self.edges,
                    &self.clusters,
                    &hints,
                ),
                LayoutMode::ForceDirected => {
                    // TODO: Implement force-directed layout (Phase 7)
                    SugiyamaLayout::default().compute(
                        &graph,
                        &self.nodes,
                        &self.edges,
                        &self.clusters,
                        &hints,
                    )
                }
            };

            self.viewport
                .set_content_bounds(result.bounding_box.clone());
            self.cached_layout = Some(result);
            self.layout_dirty = false;
        }
        self.cached_layout.as_ref().expect("layout just computed")
    }
}

// ---------------------------------------------------------------------------
// Message and Output
// ---------------------------------------------------------------------------

/// Messages that can be sent to the Diagram component.
///
/// # Examples
///
/// ```
/// use envision::diagram::{DiagramMessage, DiagramNode};
///
/// let msg = DiagramMessage::AddNode(DiagramNode::new("x", "X"));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum DiagramMessage {
    // Data mutations
    /// Replace all nodes.
    SetNodes(Vec<DiagramNode>),
    /// Replace all edges.
    SetEdges(Vec<DiagramEdge>),
    /// Add a single node.
    AddNode(DiagramNode),
    /// Add a single edge.
    AddEdge(DiagramEdge),
    /// Remove a node by ID.
    RemoveNode(String),
    /// Remove an edge by (from, to).
    RemoveEdge(String, String),
    /// Add a cluster.
    AddCluster(DiagramCluster),
    /// Remove a cluster by ID.
    RemoveCluster(String),
    /// Update a node's status.
    UpdateNodeStatus {
        /// Node ID.
        id: String,
        /// New status.
        status: NodeStatus,
    },
    /// Clear all data.
    Clear,

    // Navigation
    /// Select next node (insertion order).
    SelectNext,
    /// Select previous node (insertion order).
    SelectPrev,
    /// Select node by ID.
    SelectNode(String),

    // Viewport
    /// Zoom in.
    ZoomIn,
    /// Zoom out.
    ZoomOut,
    /// Fit entire graph in viewport.
    FitToView,

    // Display
    /// Toggle minimap visibility.
    ToggleMinimap,
    /// Set layout mode.
    SetLayoutMode(LayoutMode),
    /// Set layout orientation.
    SetOrientation(Orientation),
    /// Set render mode.
    SetRenderMode(RenderMode),
}

/// Outputs emitted by the Diagram component.
///
/// # Examples
///
/// ```
/// use envision::diagram::DiagramOutput;
///
/// let output = DiagramOutput::NodeSelected("api".to_string());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagramOutput {
    /// A node was selected.
    NodeSelected(String),
    /// Selection was cleared.
    NodeDeselected,
    /// A node's status changed.
    StatusChanged {
        /// Node ID.
        id: String,
        /// Previous status.
        old: NodeStatus,
        /// New status.
        new_status: NodeStatus,
    },
}

// ---------------------------------------------------------------------------
// Component Implementation
// ---------------------------------------------------------------------------

/// The Diagram component.
///
/// Renders a directed graph with automatic layout, edge routing,
/// and interactive navigation.
pub struct Diagram;

impl Component for Diagram {
    type State = DiagramState;
    type Message = DiagramMessage;
    type Output = DiagramOutput;

    fn init() -> Self::State {
        DiagramState::new()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        use crate::input::Key;

        match event {
            Event::Key(key) => match key.code {
                Key::Down | Key::Char('j') => Some(DiagramMessage::SelectNext),
                Key::Up | Key::Char('k') => Some(DiagramMessage::SelectPrev),
                Key::Tab if key.modifiers.shift() => Some(DiagramMessage::SelectPrev),
                Key::Tab => Some(DiagramMessage::SelectNext),
                Key::Char('+') | Key::Char('=') => Some(DiagramMessage::ZoomIn),
                Key::Char('-') => Some(DiagramMessage::ZoomOut),
                Key::Char('0') => Some(DiagramMessage::FitToView),
                Key::Char('m') => Some(DiagramMessage::ToggleMinimap),
                _ => None,
            },
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            DiagramMessage::SetNodes(nodes) => {
                state.nodes = nodes;
                state.selected = None;
                state.layout_dirty = true;
                None
            }
            DiagramMessage::SetEdges(edges) => {
                state.edges = edges;
                state.layout_dirty = true;
                None
            }
            DiagramMessage::AddNode(node) => {
                state.add_node(node);
                None
            }
            DiagramMessage::AddEdge(edge) => {
                state.add_edge(edge);
                None
            }
            DiagramMessage::RemoveNode(id) => {
                state.remove_node(&id);
                None
            }
            DiagramMessage::RemoveEdge(from, to) => {
                state.remove_edge(&from, &to);
                None
            }
            DiagramMessage::AddCluster(cluster) => {
                state.clusters.push(cluster);
                state.layout_dirty = true;
                None
            }
            DiagramMessage::RemoveCluster(id) => {
                state.clusters.retain(|c| c.id() != id);
                state.layout_dirty = true;
                None
            }
            DiagramMessage::UpdateNodeStatus { id, status } => {
                if let Some(old) = state.update_node_status(&id, status.clone()) {
                    if old != status {
                        return Some(DiagramOutput::StatusChanged {
                            id,
                            old,
                            new_status: status,
                        });
                    }
                }
                None
            }
            DiagramMessage::Clear => {
                state.clear();
                None
            }
            DiagramMessage::SelectNext => {
                if state.select_next() {
                    state
                        .selected_node()
                        .map(|n| DiagramOutput::NodeSelected(n.id().to_string()))
                } else {
                    None
                }
            }
            DiagramMessage::SelectPrev => {
                if state.select_prev() {
                    state
                        .selected_node()
                        .map(|n| DiagramOutput::NodeSelected(n.id().to_string()))
                } else {
                    None
                }
            }
            DiagramMessage::SelectNode(id) => {
                if let Some(idx) = state.nodes.iter().position(|n| n.id() == id) {
                    state.selected = Some(idx);
                    Some(DiagramOutput::NodeSelected(id))
                } else {
                    None
                }
            }
            DiagramMessage::ZoomIn => {
                state.viewport.zoom_in();
                None
            }
            DiagramMessage::ZoomOut => {
                state.viewport.zoom_out();
                None
            }
            DiagramMessage::FitToView => {
                state.viewport.fit_to_content();
                None
            }
            DiagramMessage::ToggleMinimap => {
                state.show_minimap = !state.show_minimap;
                None
            }
            DiagramMessage::SetLayoutMode(mode) => {
                state.set_layout_mode(mode);
                None
            }
            DiagramMessage::SetOrientation(orientation) => {
                state.set_orientation(orientation);
                None
            }
            DiagramMessage::SetRenderMode(mode) => {
                state.render_mode = mode;
                None
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        use ratatui::widgets::{Block, Borders, Paragraph};

        let block = Block::default().borders(Borders::ALL).title(
            state
                .title
                .as_deref()
                .map(|t| format!(" {} ", t))
                .unwrap_or_default(),
        );

        let inner = block.inner(ctx.area);
        ctx.frame.render_widget(block, ctx.area);

        if state.nodes.is_empty() {
            let msg = Paragraph::new("(empty diagram)");
            ctx.frame.render_widget(msg, inner);
            return;
        }

        // Render will be implemented in Phase 3
        let info = format!("{} nodes, {} edges", state.nodes.len(), state.edges.len());
        let msg = Paragraph::new(info);
        ctx.frame.render_widget(msg, inner);
    }
}
