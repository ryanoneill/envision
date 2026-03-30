//! Supporting types for the DependencyGraph component.

use ratatui::prelude::*;

/// Status of a node in the dependency graph.
///
/// Determines the visual indicator and default color of the node.
///
/// # Example
///
/// ```rust
/// use envision::component::NodeStatus;
///
/// let status = NodeStatus::default();
/// assert_eq!(status, NodeStatus::Healthy);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

/// Orientation of the graph layout.
///
/// # Example
///
/// ```rust
/// use envision::component::GraphOrientation;
///
/// let orientation = GraphOrientation::default();
/// assert_eq!(orientation, GraphOrientation::LeftToRight);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum GraphOrientation {
    /// Layers flow from left to right.
    #[default]
    LeftToRight,
    /// Layers flow from top to bottom.
    TopToBottom,
}

/// A node in the dependency graph.
///
/// Each node has an id, display label, status, and optional metadata.
///
/// # Example
///
/// ```rust
/// use envision::component::{GraphNode, NodeStatus};
/// use ratatui::style::Color;
///
/// let node = GraphNode::new("api", "API Gateway")
///     .with_status(NodeStatus::Healthy)
///     .with_color(Color::Green)
///     .with_metadata("version", "1.2.3");
/// assert_eq!(node.id, "api");
/// assert_eq!(node.label, "API Gateway");
/// assert_eq!(node.status, NodeStatus::Healthy);
/// assert_eq!(node.metadata.len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct GraphNode {
    /// Unique identifier for the node.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Current status.
    pub status: NodeStatus,
    /// Override color (otherwise derived from status).
    pub color: Option<Color>,
    /// Key-value metadata pairs.
    pub metadata: Vec<(String, String)>,
}

impl GraphNode {
    /// Creates a new graph node with the given id and label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GraphNode;
    ///
    /// let node = GraphNode::new("svc", "My Service");
    /// assert_eq!(node.id, "svc");
    /// assert_eq!(node.label, "My Service");
    /// ```
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            status: NodeStatus::default(),
            color: None,
            metadata: Vec::new(),
        }
    }

    /// Sets the node status (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{GraphNode, NodeStatus};
    ///
    /// let node = GraphNode::new("svc", "Service").with_status(NodeStatus::Down);
    /// assert_eq!(node.status, NodeStatus::Down);
    /// ```
    pub fn with_status(mut self, status: NodeStatus) -> Self {
        self.status = status;
        self
    }

    /// Sets the override color (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GraphNode;
    /// use ratatui::style::Color;
    ///
    /// let node = GraphNode::new("svc", "Service").with_color(Color::Cyan);
    /// assert_eq!(node.color, Some(Color::Cyan));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Adds a metadata key-value pair (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GraphNode;
    ///
    /// let node = GraphNode::new("svc", "Service")
    ///     .with_metadata("version", "2.0")
    ///     .with_metadata("port", "8080");
    /// assert_eq!(node.metadata.len(), 2);
    /// ```
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }
}

/// A directed edge between two nodes.
///
/// # Example
///
/// ```rust
/// use envision::component::GraphEdge;
/// use ratatui::style::Color;
///
/// let edge = GraphEdge::new("api", "db")
///     .with_label("HTTP")
///     .with_color(Color::Yellow);
/// assert_eq!(edge.from, "api");
/// assert_eq!(edge.to, "db");
/// assert_eq!(edge.label, Some("HTTP".to_string()));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct GraphEdge {
    /// Source node id.
    pub from: String,
    /// Target node id.
    pub to: String,
    /// Optional edge label (e.g., "HTTP", "gRPC").
    pub label: Option<String>,
    /// Optional override color.
    pub color: Option<Color>,
}

impl GraphEdge {
    /// Creates a new directed edge between two nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GraphEdge;
    ///
    /// let edge = GraphEdge::new("a", "b");
    /// assert_eq!(edge.from, "a");
    /// assert_eq!(edge.to, "b");
    /// assert!(edge.label.is_none());
    /// assert!(edge.color.is_none());
    /// ```
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: None,
            color: None,
        }
    }

    /// Sets the edge label (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GraphEdge;
    ///
    /// let edge = GraphEdge::new("a", "b").with_label("gRPC");
    /// assert_eq!(edge.label, Some("gRPC".to_string()));
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the edge color (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::GraphEdge;
    /// use ratatui::style::Color;
    ///
    /// let edge = GraphEdge::new("a", "b").with_color(Color::Red);
    /// assert_eq!(edge.color, Some(Color::Red));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}
