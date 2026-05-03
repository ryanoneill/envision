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
//! use envision::component::diagram::{DiagramState, DiagramNode, DiagramEdge, NodeStatus};
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
//!
//! # Performance
//!
//! Layout is cached and only recomputed when graph data (nodes, edges, clusters)
//! changes — selection, zoom, and pan operations skip layout entirely.
//!
//! During rendering, viewport culling ensures only visible nodes and edges are
//! drawn. Edge paths use batch buffer writes rather than per-cell widget
//! allocation, reducing overhead for dense graphs.
//!
//! Benchmark results (release mode, single-threaded):
//!
//! | Nodes | Hierarchical layout + render |
//! |------:|-----------------------------:|
//! |    10 |                        ~55µs |
//! |    50 |                       ~110µs |
//! |   100 |                       ~250µs |
//!
//! For large graphs (500+ nodes), enable the minimap and use viewport pan/zoom
//! to keep frame times low.
//!
//! The force-directed layout runs O(V²) per iteration with 50 iterations by
//! default, making it suitable for graphs up to ~200 nodes. For larger graphs,
//! prefer `LayoutMode::Hierarchical`.

use std::collections::HashSet;

use crate::component::Component;
use crate::component::context::{EventContext, RenderContext};
use crate::input::Event;

mod edge_routing;
mod graph;
pub mod layout;
mod navigation;
mod render;
mod search;
mod state;
pub mod types;
mod viewport;

pub use layout::{EdgePath, LayoutResult, NodePosition, PathSegment};
pub use types::{
    DiagramCluster, DiagramEdge, DiagramNode, EdgeStyle, LayoutMode, NodeShape, NodeStatus,
    Orientation, RenderMode,
};
pub use viewport::{BoundingBox, Viewport2D};

/// State for the Diagram component.
///
/// Holds the graph data (nodes, edges, clusters), layout configuration,
/// viewport state, and interaction state. The layout is cached and only
/// recomputed when the graph data changes.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::{DiagramState, DiagramNode, DiagramEdge};
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

    // Edge follow mode
    #[cfg_attr(feature = "serialization", serde(skip))]
    pub(crate) follow_targets: Option<Vec<String>>,

    // Search
    #[cfg_attr(feature = "serialization", serde(skip))]
    pub(crate) search: search::SearchState,
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

// ---------------------------------------------------------------------------
// Message and Output
// ---------------------------------------------------------------------------

/// Messages that can be sent to the Diagram component.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::{DiagramMessage, DiagramNode};
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

    // Navigation — insertion order
    /// Select next node (insertion order).
    SelectNext,
    /// Select previous node (insertion order).
    SelectPrev,
    /// Select node by ID.
    SelectNode(String),

    // Navigation — spatial
    /// Select nearest node above.
    SelectUp,
    /// Select nearest node below.
    SelectDown,
    /// Select nearest node to the left.
    SelectLeft,
    /// Select nearest node to the right.
    SelectRight,

    // Edge following
    /// Follow outgoing edge from selected node.
    FollowEdge,
    /// Pick a target when multiple outgoing edges exist.
    FollowEdgeChoice(usize),
    /// Go back to previous node in selection history.
    GoBack,
    /// Cancel edge follow mode.
    CancelFollow,

    // Viewport
    /// Pan the viewport by (dx, dy) in step units.
    Pan {
        /// Horizontal direction (-1.0 left, 1.0 right).
        dx: f64,
        /// Vertical direction (-1.0 up, 1.0 down).
        dy: f64,
    },
    /// Zoom in.
    ZoomIn,
    /// Zoom out.
    ZoomOut,
    /// Fit entire graph in viewport.
    FitToView,

    // Display
    /// Toggle minimap visibility.
    ToggleMinimap,
    /// Toggle expanded node view (show metadata).
    ToggleNodeExpand,
    /// Toggle cluster expand/collapse for the selected node's cluster.
    ToggleCluster,
    /// Set layout mode.
    SetLayoutMode(LayoutMode),
    /// Set layout orientation.
    SetOrientation(Orientation),
    /// Set render mode.
    SetRenderMode(RenderMode),

    // Search
    /// Enter search mode.
    StartSearch,
    /// Type a character in search mode.
    SearchInput(char),
    /// Delete last character in search mode.
    SearchBackspace,
    /// Jump to next match.
    SearchNext,
    /// Jump to previous match.
    SearchPrev,
    /// Confirm search and select the matched node.
    ConfirmSearch,
    /// Cancel search mode.
    CancelSearch,
}

/// Outputs emitted by the Diagram component.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::DiagramOutput;
///
/// let output = DiagramOutput::NodeSelected("api".to_string());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagramOutput {
    /// A node was selected.
    NodeSelected(String),
    /// Selection was cleared.
    NodeDeselected,
    /// An edge was followed from one node to another.
    EdgeFollowed {
        /// Source node ID.
        from: String,
        /// Target node ID.
        to: String,
    },
    /// Multiple outgoing edges exist — caller should show a picker.
    EdgeChoiceRequired {
        /// Source node ID.
        from: String,
        /// Target node IDs to choose from.
        targets: Vec<String>,
    },
    /// A node's status changed.
    StatusChanged {
        /// Node ID.
        id: String,
        /// Previous status.
        old: NodeStatus,
        /// New status.
        new_status: NodeStatus,
    },
    /// A cluster was toggled (expanded/collapsed).
    ClusterToggled {
        /// Cluster ID.
        id: String,
        /// Whether the cluster is now collapsed.
        collapsed: bool,
    },
    /// A search match was found and selected.
    SearchMatched {
        /// Matched node ID.
        id: String,
        /// Total number of matches.
        total_matches: usize,
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
        state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        use crate::input::Key;

        match event {
            Event::Key(key) if state.search.active => {
                // Search mode: capture typing
                match key.code {
                    Key::Esc => Some(DiagramMessage::CancelSearch),
                    Key::Enter => Some(DiagramMessage::ConfirmSearch),
                    Key::Backspace => Some(DiagramMessage::SearchBackspace),
                    Key::Char('n') if key.modifiers.shift() => Some(DiagramMessage::SearchPrev),
                    Key::Char('n') => Some(DiagramMessage::SearchNext),
                    Key::Char(c) => Some(DiagramMessage::SearchInput(c)),
                    _ => None,
                }
            }
            Event::Key(key) => match key.code {
                // Spatial navigation
                Key::Down | Key::Char('j') => Some(DiagramMessage::SelectDown),
                Key::Up | Key::Char('k') => Some(DiagramMessage::SelectUp),
                Key::Left | Key::Char('h') => Some(DiagramMessage::SelectLeft),
                Key::Right | Key::Char('l') => Some(DiagramMessage::SelectRight),
                // Insertion-order cycling
                Key::Tab if key.modifiers.shift() => Some(DiagramMessage::SelectPrev),
                Key::Tab => Some(DiagramMessage::SelectNext),
                // Edge following
                Key::Enter => Some(DiagramMessage::FollowEdge),
                Key::Backspace => Some(DiagramMessage::GoBack),
                // Viewport
                Key::Char('H') => Some(DiagramMessage::Pan { dx: -1.0, dy: 0.0 }),
                Key::Char('J') => Some(DiagramMessage::Pan { dx: 0.0, dy: 1.0 }),
                Key::Char('K') => Some(DiagramMessage::Pan { dx: 0.0, dy: -1.0 }),
                Key::Char('L') => Some(DiagramMessage::Pan { dx: 1.0, dy: 0.0 }),
                Key::Char('+') | Key::Char('=') => Some(DiagramMessage::ZoomIn),
                Key::Char('-') => Some(DiagramMessage::ZoomOut),
                Key::Char('0') => Some(DiagramMessage::FitToView),
                // Display toggles
                Key::Char('m') => Some(DiagramMessage::ToggleMinimap),
                Key::Char(' ') => Some(DiagramMessage::ToggleNodeExpand),
                Key::Char('c') => Some(DiagramMessage::ToggleCluster),
                Key::Char('/') => Some(DiagramMessage::StartSearch),
                // Edge follow choice (1-9)
                Key::Char(c @ '1'..='9') if state.follow_targets.is_some() => {
                    let idx = (c as usize) - ('1' as usize);
                    Some(DiagramMessage::FollowEdgeChoice(idx))
                }
                Key::Esc if state.follow_targets.is_some() => Some(DiagramMessage::CancelFollow),
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
            DiagramMessage::SelectUp => {
                if state.select_direction(navigation::Direction::Up) {
                    state
                        .selected_node()
                        .map(|n| DiagramOutput::NodeSelected(n.id().to_string()))
                } else {
                    None
                }
            }
            DiagramMessage::SelectDown => {
                if state.select_direction(navigation::Direction::Down) {
                    state
                        .selected_node()
                        .map(|n| DiagramOutput::NodeSelected(n.id().to_string()))
                } else {
                    None
                }
            }
            DiagramMessage::SelectLeft => {
                if state.select_direction(navigation::Direction::Left) {
                    state
                        .selected_node()
                        .map(|n| DiagramOutput::NodeSelected(n.id().to_string()))
                } else {
                    None
                }
            }
            DiagramMessage::SelectRight => {
                if state.select_direction(navigation::Direction::Right) {
                    state
                        .selected_node()
                        .map(|n| DiagramOutput::NodeSelected(n.id().to_string()))
                } else {
                    None
                }
            }
            DiagramMessage::FollowEdge => state.follow_edge(),
            DiagramMessage::FollowEdgeChoice(idx) => state.follow_edge_choice(idx),
            DiagramMessage::GoBack => {
                if state.go_back() {
                    state
                        .selected_node()
                        .map(|n| DiagramOutput::NodeSelected(n.id().to_string()))
                } else {
                    None
                }
            }
            DiagramMessage::CancelFollow => {
                state.follow_targets = None;
                None
            }
            DiagramMessage::Pan { dx, dy } => {
                state.viewport.pan_step(dx, dy);
                None
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
            DiagramMessage::ToggleNodeExpand => {
                if let Some(node) = state.selected_node() {
                    let id = node.id().to_string();
                    if state.expanded_nodes.contains(&id) {
                        state.expanded_nodes.remove(&id);
                    } else {
                        state.expanded_nodes.insert(id);
                    }
                }
                None
            }
            DiagramMessage::ToggleCluster => {
                if let Some(node) = state.selected_node() {
                    if let Some(cluster_id) = node.cluster_id() {
                        let cluster_id = cluster_id.to_string();
                        let collapsed = if state.collapsed_clusters.contains(&cluster_id) {
                            state.collapsed_clusters.remove(&cluster_id);
                            false
                        } else {
                            state.collapsed_clusters.insert(cluster_id.clone());
                            true
                        };
                        return Some(DiagramOutput::ClusterToggled {
                            id: cluster_id,
                            collapsed,
                        });
                    }
                }
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
            DiagramMessage::StartSearch => {
                state.search.start();
                None
            }
            DiagramMessage::SearchInput(ch) => {
                state.search.input(ch, &state.nodes);
                None
            }
            DiagramMessage::SearchBackspace => {
                state.search.backspace(&state.nodes);
                None
            }
            DiagramMessage::SearchNext => {
                state.search.next_match();
                if let Some(idx) = state.search.current_node_index() {
                    state.selected = Some(idx);
                    state.selected_node().map(|n| DiagramOutput::SearchMatched {
                        id: n.id().to_string(),
                        total_matches: state.search.matches.len(),
                    })
                } else {
                    None
                }
            }
            DiagramMessage::SearchPrev => {
                state.search.prev_match();
                if let Some(idx) = state.search.current_node_index() {
                    state.selected = Some(idx);
                    state.selected_node().map(|n| DiagramOutput::SearchMatched {
                        id: n.id().to_string(),
                        total_matches: state.search.matches.len(),
                    })
                } else {
                    None
                }
            }
            DiagramMessage::ConfirmSearch => {
                if let Some(idx) = state.search.current_node_index() {
                    state.selected = Some(idx);
                    let id = state.nodes[idx].id().to_string();
                    let total = state.search.matches.len();
                    state.search.cancel();
                    Some(DiagramOutput::SearchMatched {
                        id,
                        total_matches: total,
                    })
                } else {
                    state.search.cancel();
                    None
                }
            }
            DiagramMessage::CancelSearch => {
                state.search.cancel();
                None
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        // Compute layout (uses cached result if available)
        let mut state_clone = state.clone();
        let layout = state_clone.ensure_layout().clone();

        render::render_diagram(
            state,
            &layout,
            ctx.frame,
            ctx.area,
            ctx.theme,
            ctx.focused,
            ctx.disabled,
            ctx.chrome_owned,
        );
    }
}

#[cfg(test)]
mod snapshot_tests;
