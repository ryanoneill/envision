//! Layout engine for the Diagram component.
//!
//! Provides pluggable layout algorithms that position nodes in graph space.
//! Each algorithm produces a [`LayoutResult`] with positioned nodes and
//! edge paths.

mod sugiyama;

pub(crate) use sugiyama::SugiyamaLayout;

use super::graph::IndexedGraph;
use super::types::{DiagramCluster, DiagramEdge, DiagramNode, Orientation};
use super::viewport::BoundingBox;

/// A positioned node in graph coordinates.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::NodePosition;
///
/// let pos = NodePosition::new("api".to_string(), 10.0, 5.0, 20.0, 3.0);
/// assert_eq!(pos.id(), "api");
/// assert_eq!(pos.x(), 10.0);
/// assert_eq!(pos.y(), 5.0);
/// assert_eq!(pos.width(), 20.0);
/// assert_eq!(pos.height(), 3.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct NodePosition {
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl NodePosition {
    /// Creates a new node position.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::NodePosition;
    ///
    /// let pos = NodePosition::new("n1".to_string(), 0.0, 0.0, 15.0, 3.0);
    /// assert_eq!(pos.id(), "n1");
    /// ```
    pub fn new(id: String, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the node ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::NodePosition;
    ///
    /// let pos = NodePosition::new("abc".to_string(), 0.0, 0.0, 10.0, 3.0);
    /// assert_eq!(pos.id(), "abc");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the x coordinate (left edge).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::NodePosition;
    ///
    /// let pos = NodePosition::new("n".to_string(), 25.0, 10.0, 10.0, 3.0);
    /// assert_eq!(pos.x(), 25.0);
    /// ```
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Returns the y coordinate (top edge).
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::NodePosition;
    ///
    /// let pos = NodePosition::new("n".to_string(), 25.0, 10.0, 10.0, 3.0);
    /// assert_eq!(pos.y(), 10.0);
    /// ```
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Returns the node width in graph units.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::NodePosition;
    ///
    /// let pos = NodePosition::new("n".to_string(), 0.0, 0.0, 18.0, 3.0);
    /// assert_eq!(pos.width(), 18.0);
    /// ```
    pub fn width(&self) -> f64 {
        self.width
    }

    /// Returns the node height in graph units.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::NodePosition;
    ///
    /// let pos = NodePosition::new("n".to_string(), 0.0, 0.0, 18.0, 5.0);
    /// assert_eq!(pos.height(), 5.0);
    /// ```
    pub fn height(&self) -> f64 {
        self.height
    }

    /// Returns the center x coordinate.
    pub(crate) fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    /// Returns the center y coordinate.
    pub(crate) fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }
}

/// A single segment of an edge path.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::PathSegment;
///
/// let seg = PathSegment::LineTo(50.0, 25.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum PathSegment {
    /// Move to a position without drawing.
    MoveTo(f64, f64),
    /// Draw a line to this position.
    LineTo(f64, f64),
}

/// A routed edge as a sequence of path segments.
///
/// # Examples
///
/// ```
/// use envision::component::diagram::{EdgePath, PathSegment};
///
/// let path = EdgePath::new(
///     "api".to_string(),
///     "db".to_string(),
///     vec![
///         PathSegment::MoveTo(30.0, 5.0),
///         PathSegment::LineTo(50.0, 5.0),
///         PathSegment::LineTo(50.0, 15.0),
///     ],
/// );
/// assert_eq!(path.from_id(), "api");
/// assert_eq!(path.to_id(), "db");
/// assert_eq!(path.segments().len(), 3);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct EdgePath {
    from_id: String,
    to_id: String,
    segments: Vec<PathSegment>,
}

impl EdgePath {
    /// Creates a new edge path.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{EdgePath, PathSegment};
    ///
    /// let path = EdgePath::new("a".to_string(), "b".to_string(), vec![]);
    /// assert_eq!(path.from_id(), "a");
    /// ```
    pub fn new(from_id: String, to_id: String, segments: Vec<PathSegment>) -> Self {
        Self {
            from_id,
            to_id,
            segments,
        }
    }

    /// Returns the source node ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::EdgePath;
    ///
    /// let path = EdgePath::new("src".to_string(), "dst".to_string(), vec![]);
    /// assert_eq!(path.from_id(), "src");
    /// ```
    pub fn from_id(&self) -> &str {
        &self.from_id
    }

    /// Returns the target node ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::EdgePath;
    ///
    /// let path = EdgePath::new("src".to_string(), "dst".to_string(), vec![]);
    /// assert_eq!(path.to_id(), "dst");
    /// ```
    pub fn to_id(&self) -> &str {
        &self.to_id
    }

    /// Returns the path segments.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::EdgePath;
    ///
    /// let path = EdgePath::new("a".to_string(), "b".to_string(), vec![]);
    /// assert!(path.segments().is_empty());
    /// ```
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }
}

/// The complete output of a layout algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutResult {
    /// Positioned nodes.
    pub(crate) node_positions: Vec<NodePosition>,
    /// Routed edge paths.
    pub(crate) edge_paths: Vec<EdgePath>,
    /// Bounding box of the entire layout.
    pub(crate) bounding_box: BoundingBox,
}

impl LayoutResult {
    /// Returns the positioned nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::LayoutResult;
    ///
    /// let result = LayoutResult::empty();
    /// assert!(result.node_positions().is_empty());
    /// ```
    pub fn node_positions(&self) -> &[NodePosition] {
        &self.node_positions
    }

    /// Returns the routed edge paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::LayoutResult;
    ///
    /// let result = LayoutResult::empty();
    /// assert!(result.edge_paths().is_empty());
    /// ```
    pub fn edge_paths(&self) -> &[EdgePath] {
        &self.edge_paths
    }

    /// Returns the layout bounding box.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::LayoutResult;
    ///
    /// let result = LayoutResult::empty();
    /// assert_eq!(result.bounding_box().width(), 0.0);
    /// ```
    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }

    /// Creates an empty layout result.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::LayoutResult;
    ///
    /// let result = LayoutResult::empty();
    /// assert!(result.node_positions().is_empty());
    /// assert!(result.edge_paths().is_empty());
    /// ```
    pub fn empty() -> Self {
        Self {
            node_positions: Vec::new(),
            edge_paths: Vec::new(),
            bounding_box: BoundingBox::default(),
        }
    }
}

/// Hints that influence layout without strictly controlling it.
pub(crate) struct LayoutHints<'a> {
    pub(crate) orientation: Orientation,
    pub(crate) node_spacing: f64,
    pub(crate) layer_spacing: f64,
    #[allow(dead_code)] // Used in Phase 8 for incremental stability
    pub(crate) previous_layout: Option<&'a LayoutResult>,
}

impl Default for LayoutHints<'_> {
    fn default() -> Self {
        Self {
            orientation: Orientation::default(),
            node_spacing: 4.0,
            layer_spacing: 8.0,
            previous_layout: None,
        }
    }
}

/// Trait for layout algorithms.
///
/// Each algorithm takes the graph structure, node/edge data, cluster info,
/// and hints, and produces positioned nodes with routed edge paths.
pub(crate) trait LayoutAlgorithm {
    fn compute(
        &self,
        graph: &IndexedGraph,
        nodes: &[DiagramNode],
        edges: &[DiagramEdge],
        clusters: &[DiagramCluster],
        hints: &LayoutHints<'_>,
    ) -> LayoutResult;
}
