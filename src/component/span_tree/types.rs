/// Node and flattened-span types for the SpanTree component.
///
/// These are the primary data types used to define span hierarchies
/// and their flattened representation for rendering.
use ratatui::style::Color;

/// A node in the span tree.
///
/// Each node represents a single span with a label, start/end times,
/// color, optional status text, and optional children.
///
/// # Example
///
/// ```rust
/// use envision::component::SpanNode;
/// use ratatui::style::Color;
///
/// let node = SpanNode::new("svc-1", "api/handler", 10.0, 250.0)
///     .with_color(Color::Yellow)
///     .with_status("200 OK")
///     .with_child(SpanNode::new("db-1", "db/query", 20.0, 100.0));
///
/// assert_eq!(node.label(), "api/handler");
/// assert_eq!(node.duration(), 240.0);
/// assert_eq!(node.children().len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SpanNode {
    /// Unique identifier.
    pub(super) id: String,
    /// Display label (e.g., service name or operation).
    pub(super) label: String,
    /// Start time (milliseconds or any unit).
    pub(super) start: f64,
    /// End time.
    pub(super) end: f64,
    /// Color for the duration bar.
    pub(super) color: Color,
    /// Optional status text (e.g., "200 OK", "ERROR").
    pub(super) status: Option<String>,
    /// Child spans.
    pub(super) children: Vec<SpanNode>,
}

impl SpanNode {
    /// Creates a new span node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("id-1", "my-service", 0.0, 100.0);
    /// assert_eq!(node.id(), "id-1");
    /// assert_eq!(node.label(), "my-service");
    /// assert_eq!(node.start(), 0.0);
    /// assert_eq!(node.end(), 100.0);
    /// assert_eq!(node.duration(), 100.0);
    /// ```
    pub fn new(id: impl Into<String>, label: impl Into<String>, start: f64, end: f64) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            start,
            end,
            color: Color::Cyan,
            status: None,
            children: Vec::new(),
        }
    }

    /// Sets the bar color (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    /// use ratatui::style::Color;
    ///
    /// let node = SpanNode::new("id", "svc", 0.0, 10.0).with_color(Color::Red);
    /// assert_eq!(node.color(), Color::Red);
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the status text (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("id", "svc", 0.0, 10.0).with_status("200 OK");
    /// assert_eq!(node.status(), Some("200 OK"));
    /// ```
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Adds a child span (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("parent", "svc", 0.0, 100.0)
    ///     .with_child(SpanNode::new("child", "db", 10.0, 50.0));
    /// assert_eq!(node.children().len(), 1);
    /// ```
    pub fn with_child(mut self, child: SpanNode) -> Self {
        self.children.push(child);
        self
    }

    /// Sets the children (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let children = vec![
    ///     SpanNode::new("c1", "a", 0.0, 10.0),
    ///     SpanNode::new("c2", "b", 10.0, 20.0),
    /// ];
    /// let node = SpanNode::new("p", "parent", 0.0, 20.0)
    ///     .with_children(children);
    /// assert_eq!(node.children().len(), 2);
    /// ```
    pub fn with_children(mut self, children: Vec<SpanNode>) -> Self {
        self.children = children;
        self
    }

    /// Returns the node's unique identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the node's display label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the start time.
    pub fn start(&self) -> f64 {
        self.start
    }

    /// Returns the end time.
    pub fn end(&self) -> f64 {
        self.end
    }

    /// Returns the bar color.
    pub fn color(&self) -> Color {
        self.color
    }

    /// Sets the bar color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    /// use ratatui::style::Color;
    ///
    /// let mut node = SpanNode::new("id", "svc", 0.0, 10.0);
    /// node.set_color(Color::Red);
    /// assert_eq!(node.color(), Color::Red);
    /// ```
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Returns the status text, if set.
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Returns the children.
    pub fn children(&self) -> &[SpanNode] {
        &self.children
    }

    /// Returns the duration (end - start).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("id", "svc", 50.0, 200.0);
    /// assert_eq!(node.duration(), 150.0);
    /// ```
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }

    /// Returns true if this node has children.
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// A flattened view of a span node for rendering.
///
/// Created by [`super::SpanTreeState::flatten`], this provides all the
/// information needed to render a single row in the span tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct FlatSpan {
    /// The span's unique identifier.
    pub(super) id: String,
    /// Display label.
    pub(super) label: String,
    /// Start time.
    pub(super) start: f64,
    /// End time.
    pub(super) end: f64,
    /// Bar color.
    pub(super) color: Color,
    /// Optional status text.
    pub(super) status: Option<String>,
    /// Depth in the hierarchy (0 = root).
    pub(super) depth: usize,
    /// Whether this node has children.
    pub(super) has_children: bool,
    /// Whether this node is currently expanded.
    pub(super) is_expanded: bool,
}

impl FlatSpan {
    /// Returns the span's unique identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the display label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the start time.
    pub fn start(&self) -> f64 {
        self.start
    }

    /// Returns the end time.
    pub fn end(&self) -> f64 {
        self.end
    }

    /// Returns the bar color.
    pub fn color(&self) -> Color {
        self.color
    }

    /// Returns the status text.
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Returns the depth in the hierarchy.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns true if this node has children.
    pub fn has_children(&self) -> bool {
        self.has_children
    }

    /// Returns true if this node is expanded.
    pub fn is_expanded(&self) -> bool {
        self.is_expanded
    }

    /// Returns the duration (end - start).
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}
