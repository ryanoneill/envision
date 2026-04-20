//! Public types for the Diagram component.
//!
//! This module defines the data model for nodes, edges, clusters,
//! and configuration enums used by [`DiagramState`](super::DiagramState).

use ratatui::style::Color;

/// Status of a node in the diagram.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::NodeStatus;
///
/// let status = NodeStatus::default();
/// assert_eq!(status, NodeStatus::Healthy);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum NodeStatus {
    /// The node is operating normally.
    #[default]
    Healthy,
    /// The node is experiencing issues but still functional.
    Degraded,
    /// The node is not operational.
    Down,
    /// The node status is not known.
    Unknown,
}

/// Shape of a diagram node's border.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::NodeShape;
///
/// let shape = NodeShape::default();
/// assert_eq!(shape, NodeShape::Rectangle);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum NodeShape {
    /// Standard rectangular border using box-drawing characters.
    #[default]
    Rectangle,
    /// Rounded corners using `╭╮╯╰` characters.
    RoundedRectangle,
    /// Diamond shape for decision/conditional nodes.
    Diamond,
}

/// Visual style of an edge line.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::EdgeStyle;
///
/// let style = EdgeStyle::default();
/// assert_eq!(style, EdgeStyle::Solid);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum EdgeStyle {
    /// Continuous line: `───────`
    #[default]
    Solid,
    /// Alternating segments: `── ── ──`
    Dashed,
    /// Dotted line: `·······`
    Dotted,
}

/// Which layout algorithm to use for positioning nodes.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::LayoutMode;
///
/// let mode = LayoutMode::default();
/// assert_eq!(mode, LayoutMode::Hierarchical);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum LayoutMode {
    /// Sugiyama-style layered layout. Best for DAGs, dependency graphs,
    /// and any graph with a natural flow direction.
    #[default]
    Hierarchical,
    /// Fruchterman-Reingold force-directed layout. Best for network
    /// diagrams and graphs without a clear hierarchy.
    ForceDirected,
}

/// Rendering fidelity mode for edges.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::RenderMode;
///
/// let mode = RenderMode::default();
/// assert_eq!(mode, RenderMode::BoxDrawing);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum RenderMode {
    /// Unicode box-drawing characters for edges. Standard fidelity.
    #[default]
    BoxDrawing,
    /// Braille patterns for edges, giving 8 sub-pixels per cell.
    /// Higher fidelity for dense graphs with many edge crossings.
    Braille,
}

/// Orientation of the hierarchical layout.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::Orientation;
///
/// let o = Orientation::default();
/// assert_eq!(o, Orientation::LeftToRight);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum Orientation {
    /// Root nodes on the left, flow goes right.
    #[default]
    LeftToRight,
    /// Root nodes on top, flow goes down.
    TopToBottom,
}

// ---------------------------------------------------------------------------
// DiagramNode
// ---------------------------------------------------------------------------

/// A node in the diagram.
///
/// Nodes are the primary visual elements. Each has a unique string ID,
/// a display label, and optional status, color, shape, metadata, and
/// cluster membership.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::{DiagramNode, NodeStatus, NodeShape};
///
/// let node = DiagramNode::new("api", "API Gateway")
///     .with_status(NodeStatus::Healthy)
///     .with_shape(NodeShape::RoundedRectangle)
///     .with_metadata("version", "2.1.0");
///
/// assert_eq!(node.id(), "api");
/// assert_eq!(node.label(), "API Gateway");
/// assert_eq!(node.status(), &NodeStatus::Healthy);
/// assert_eq!(node.shape(), &NodeShape::RoundedRectangle);
/// assert_eq!(node.metadata().len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DiagramNode {
    id: String,
    label: String,
    status: NodeStatus,
    color: Option<Color>,
    shape: NodeShape,
    metadata: Vec<(String, String)>,
    cluster_id: Option<String>,
}

impl DiagramNode {
    /// Creates a new node with the given ID and label.
    ///
    /// The node starts with [`NodeStatus::Healthy`], [`NodeShape::Rectangle`],
    /// no color override, no metadata, and no cluster.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    ///
    /// let node = DiagramNode::new("db", "Database");
    /// assert_eq!(node.id(), "db");
    /// assert_eq!(node.label(), "Database");
    /// ```
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            status: NodeStatus::default(),
            color: None,
            shape: NodeShape::default(),
            metadata: Vec::new(),
            cluster_id: None,
        }
    }

    /// Sets the node's status.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{DiagramNode, NodeStatus};
    ///
    /// let node = DiagramNode::new("db", "DB").with_status(NodeStatus::Degraded);
    /// assert_eq!(node.status(), &NodeStatus::Degraded);
    /// ```
    pub fn with_status(mut self, status: NodeStatus) -> Self {
        self.status = status;
        self
    }

    /// Sets a color override for the node border.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    /// use ratatui::style::Color;
    ///
    /// let node = DiagramNode::new("api", "API").with_color(Color::Cyan);
    /// assert_eq!(node.color(), Some(Color::Cyan));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the node's visual shape.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{DiagramNode, NodeShape};
    ///
    /// let node = DiagramNode::new("decision", "Approve?")
    ///     .with_shape(NodeShape::Diamond);
    /// assert_eq!(node.shape(), &NodeShape::Diamond);
    /// ```
    pub fn with_shape(mut self, shape: NodeShape) -> Self {
        self.shape = shape;
        self
    }

    /// Adds a metadata key-value pair to the node.
    ///
    /// Metadata is shown when the node is expanded (via Space key).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    ///
    /// let node = DiagramNode::new("pod", "nginx-abc123")
    ///     .with_metadata("namespace", "default")
    ///     .with_metadata("image", "nginx:1.25");
    /// assert_eq!(node.metadata().len(), 2);
    /// assert_eq!(node.metadata()[0], ("namespace".to_string(), "default".to_string()));
    /// ```
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// Assigns this node to a cluster.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    ///
    /// let node = DiagramNode::new("api", "API").with_cluster("us-east");
    /// assert_eq!(node.cluster_id(), Some("us-east"));
    /// ```
    pub fn with_cluster(mut self, cluster_id: impl Into<String>) -> Self {
        self.cluster_id = Some(cluster_id.into());
        self
    }

    /// Returns the node's unique identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    ///
    /// let node = DiagramNode::new("svc", "Service");
    /// assert_eq!(node.id(), "svc");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the node's display label.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    ///
    /// let node = DiagramNode::new("svc", "My Service");
    /// assert_eq!(node.label(), "My Service");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the node's current status.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{DiagramNode, NodeStatus};
    ///
    /// let node = DiagramNode::new("db", "DB");
    /// assert_eq!(node.status(), &NodeStatus::Healthy);
    /// ```
    pub fn status(&self) -> &NodeStatus {
        &self.status
    }

    /// Sets the node's status.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{DiagramNode, NodeStatus};
    ///
    /// let mut node = DiagramNode::new("db", "DB");
    /// node.set_status(NodeStatus::Down);
    /// assert_eq!(node.status(), &NodeStatus::Down);
    /// ```
    pub fn set_status(&mut self, status: NodeStatus) {
        self.status = status;
    }

    /// Returns the optional color override.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    ///
    /// let node = DiagramNode::new("api", "API");
    /// assert_eq!(node.color(), None);
    /// ```
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Sets a color override.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    /// use ratatui::style::Color;
    ///
    /// let mut node = DiagramNode::new("api", "API");
    /// node.set_color(Some(Color::Red));
    /// assert_eq!(node.color(), Some(Color::Red));
    /// ```
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    /// Returns the node's visual shape.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{DiagramNode, NodeShape};
    ///
    /// let node = DiagramNode::new("x", "X");
    /// assert_eq!(node.shape(), &NodeShape::Rectangle);
    /// ```
    pub fn shape(&self) -> &NodeShape {
        &self.shape
    }

    /// Returns the node's metadata key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    ///
    /// let node = DiagramNode::new("x", "X");
    /// assert!(node.metadata().is_empty());
    /// ```
    pub fn metadata(&self) -> &[(String, String)] {
        &self.metadata
    }

    /// Returns the cluster ID this node belongs to, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    ///
    /// let node = DiagramNode::new("api", "API").with_cluster("prod");
    /// assert_eq!(node.cluster_id(), Some("prod"));
    /// ```
    pub fn cluster_id(&self) -> Option<&str> {
        self.cluster_id.as_deref()
    }

    /// Sets the node's label.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramNode;
    ///
    /// let mut node = DiagramNode::new("api", "API");
    /// node.set_label("API v2");
    /// assert_eq!(node.label(), "API v2");
    /// ```
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }
}

// ---------------------------------------------------------------------------
// DiagramEdge
// ---------------------------------------------------------------------------

/// A directed edge connecting two nodes in the diagram.
///
/// Edges are identified by their (from, to) node ID pair and carry
/// optional label, color, style, and directionality.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::{DiagramEdge, EdgeStyle};
///
/// let edge = DiagramEdge::new("api", "db")
///     .with_label("SQL")
///     .with_style(EdgeStyle::Dashed);
///
/// assert_eq!(edge.from(), "api");
/// assert_eq!(edge.to(), "db");
/// assert_eq!(edge.label(), Some("SQL"));
/// assert_eq!(edge.style(), &EdgeStyle::Dashed);
/// assert!(!edge.bidirectional());
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DiagramEdge {
    from: String,
    to: String,
    label: Option<String>,
    color: Option<Color>,
    style: EdgeStyle,
    bidirectional: bool,
}

impl DiagramEdge {
    /// Creates a new directed edge from one node to another.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    ///
    /// let edge = DiagramEdge::new("a", "b");
    /// assert_eq!(edge.from(), "a");
    /// assert_eq!(edge.to(), "b");
    /// ```
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: None,
            color: None,
            style: EdgeStyle::default(),
            bidirectional: false,
        }
    }

    /// Sets the edge label (displayed at the midpoint).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    ///
    /// let edge = DiagramEdge::new("a", "b").with_label("HTTP");
    /// assert_eq!(edge.label(), Some("HTTP"));
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets a color override for the edge.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    /// use ratatui::style::Color;
    ///
    /// let edge = DiagramEdge::new("a", "b").with_color(Color::Yellow);
    /// assert_eq!(edge.color(), Some(Color::Yellow));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the edge line style.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{DiagramEdge, EdgeStyle};
    ///
    /// let edge = DiagramEdge::new("a", "b").with_style(EdgeStyle::Dotted);
    /// assert_eq!(edge.style(), &EdgeStyle::Dotted);
    /// ```
    pub fn with_style(mut self, style: EdgeStyle) -> Self {
        self.style = style;
        self
    }

    /// Makes the edge bidirectional (arrowheads on both ends).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    ///
    /// let edge = DiagramEdge::new("a", "b").with_bidirectional(true);
    /// assert!(edge.bidirectional());
    /// ```
    pub fn with_bidirectional(mut self, bidirectional: bool) -> Self {
        self.bidirectional = bidirectional;
        self
    }

    /// Returns the source node ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    ///
    /// let edge = DiagramEdge::new("src", "dst");
    /// assert_eq!(edge.from(), "src");
    /// ```
    pub fn from(&self) -> &str {
        &self.from
    }

    /// Returns the target node ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    ///
    /// let edge = DiagramEdge::new("src", "dst");
    /// assert_eq!(edge.to(), "dst");
    /// ```
    pub fn to(&self) -> &str {
        &self.to
    }

    /// Returns the edge label, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    ///
    /// let edge = DiagramEdge::new("a", "b");
    /// assert_eq!(edge.label(), None);
    /// ```
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the optional color override.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    ///
    /// let edge = DiagramEdge::new("a", "b");
    /// assert_eq!(edge.color(), None);
    /// ```
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Sets the edge color.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    /// use ratatui::style::Color;
    ///
    /// let mut edge = DiagramEdge::new("a", "b");
    /// edge.set_color(Some(Color::Green));
    /// assert_eq!(edge.color(), Some(Color::Green));
    /// ```
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    /// Returns the edge line style.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{DiagramEdge, EdgeStyle};
    ///
    /// let edge = DiagramEdge::new("a", "b");
    /// assert_eq!(edge.style(), &EdgeStyle::Solid);
    /// ```
    pub fn style(&self) -> &EdgeStyle {
        &self.style
    }

    /// Returns whether the edge is bidirectional.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramEdge;
    ///
    /// let edge = DiagramEdge::new("a", "b");
    /// assert!(!edge.bidirectional());
    /// ```
    pub fn bidirectional(&self) -> bool {
        self.bidirectional
    }
}

// ---------------------------------------------------------------------------
// DiagramCluster
// ---------------------------------------------------------------------------

/// A named group of nodes displayed with a shared border.
///
/// Clusters visually group related nodes together. Nodes are assigned
/// to clusters via [`DiagramNode::with_cluster`].
///
/// # Examples
///
/// ```
/// use envision::component::diagram::DiagramCluster;
/// use ratatui::style::Color;
///
/// let cluster = DiagramCluster::new("us-east", "US East")
///     .with_color(Color::Blue);
///
/// assert_eq!(cluster.id(), "us-east");
/// assert_eq!(cluster.label(), "US East");
/// assert_eq!(cluster.color(), Some(Color::Blue));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DiagramCluster {
    id: String,
    label: String,
    color: Option<Color>,
}

impl DiagramCluster {
    /// Creates a new cluster with the given ID and label.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramCluster;
    ///
    /// let cluster = DiagramCluster::new("prod", "Production");
    /// assert_eq!(cluster.id(), "prod");
    /// assert_eq!(cluster.label(), "Production");
    /// ```
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            color: None,
        }
    }

    /// Sets a color for the cluster border.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramCluster;
    /// use ratatui::style::Color;
    ///
    /// let cluster = DiagramCluster::new("dev", "Dev").with_color(Color::Gray);
    /// assert_eq!(cluster.color(), Some(Color::Gray));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Returns the cluster's unique identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramCluster;
    ///
    /// let cluster = DiagramCluster::new("c1", "Cluster");
    /// assert_eq!(cluster.id(), "c1");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the cluster's display label.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramCluster;
    ///
    /// let cluster = DiagramCluster::new("c1", "My Cluster");
    /// assert_eq!(cluster.label(), "My Cluster");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the optional color override.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramCluster;
    ///
    /// let cluster = DiagramCluster::new("c1", "C");
    /// assert_eq!(cluster.color(), None);
    /// ```
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Sets the cluster color.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramCluster;
    /// use ratatui::style::Color;
    ///
    /// let mut cluster = DiagramCluster::new("c1", "C");
    /// cluster.set_color(Some(Color::Magenta));
    /// assert_eq!(cluster.color(), Some(Color::Magenta));
    /// ```
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    /// Sets the cluster label.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramCluster;
    ///
    /// let mut cluster = DiagramCluster::new("c1", "Old");
    /// cluster.set_label("New Label");
    /// assert_eq!(cluster.label(), "New Label");
    /// ```
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
    }
}
