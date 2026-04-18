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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("svc-1", "api/handler", 0.0, 100.0);
    /// assert_eq!(node.id(), "svc-1");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the node's display label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("id", "api/handler", 0.0, 100.0);
    /// assert_eq!(node.label(), "api/handler");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the start time.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("id", "svc", 42.0, 100.0);
    /// assert_eq!(node.start(), 42.0);
    /// ```
    pub fn start(&self) -> f64 {
        self.start
    }

    /// Returns the end time.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("id", "svc", 0.0, 250.0);
    /// assert_eq!(node.end(), 250.0);
    /// ```
    pub fn end(&self) -> f64 {
        self.end
    }

    /// Returns the bar color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    /// use ratatui::style::Color;
    ///
    /// let node = SpanNode::new("id", "svc", 0.0, 10.0);
    /// assert_eq!(node.color(), Color::Cyan); // default color
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("id", "svc", 0.0, 10.0).with_status("200 OK");
    /// assert_eq!(node.status(), Some("200 OK"));
    ///
    /// let plain = SpanNode::new("id", "svc", 0.0, 10.0);
    /// assert_eq!(plain.status(), None);
    /// ```
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Returns the children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let node = SpanNode::new("p", "parent", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
    /// assert_eq!(node.children().len(), 1);
    /// assert_eq!(node.children()[0].id(), "c");
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanNode;
    ///
    /// let leaf = SpanNode::new("l", "leaf", 0.0, 10.0);
    /// assert!(!leaf.has_children());
    ///
    /// let parent = SpanNode::new("p", "parent", 0.0, 10.0)
    ///     .with_child(SpanNode::new("c", "child", 1.0, 5.0));
    /// assert!(parent.has_children());
    /// ```
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
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    /// let flat = state.flatten();
    /// assert_eq!(flat[0].id(), "r");
    /// ```
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the display label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![SpanNode::new("r", "api/handler", 0.0, 10.0)]);
    /// let flat = state.flatten();
    /// assert_eq!(flat[0].label(), "api/handler");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the start time.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 50.0, 200.0)]);
    /// let flat = state.flatten();
    /// assert_eq!(flat[0].start(), 50.0);
    /// ```
    pub fn start(&self) -> f64 {
        self.start
    }

    /// Returns the end time.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 200.0)]);
    /// let flat = state.flatten();
    /// assert_eq!(flat[0].end(), 200.0);
    /// ```
    pub fn end(&self) -> f64 {
        self.end
    }

    /// Returns the bar color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    /// use ratatui::style::Color;
    ///
    /// let node = SpanNode::new("r", "root", 0.0, 10.0).with_color(Color::Red);
    /// let state = SpanTreeState::new(vec![node]);
    /// let flat = state.flatten();
    /// assert_eq!(flat[0].color(), Color::Red);
    /// ```
    pub fn color(&self) -> Color {
        self.color
    }

    /// Returns the status text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let node = SpanNode::new("r", "root", 0.0, 10.0).with_status("200 OK");
    /// let state = SpanTreeState::new(vec![node]);
    /// let flat = state.flatten();
    /// assert_eq!(flat[0].status(), Some("200 OK"));
    /// ```
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// Returns the depth in the hierarchy.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
    /// let state = SpanTreeState::new(vec![root]);
    /// let flat = state.flatten();
    /// assert_eq!(flat[0].depth(), 0);
    /// assert_eq!(flat[1].depth(), 1);
    /// ```
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns true if this node has children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
    /// let state = SpanTreeState::new(vec![root]);
    /// let flat = state.flatten();
    /// assert!(flat[0].has_children());
    /// assert!(!flat[1].has_children());
    /// ```
    pub fn has_children(&self) -> bool {
        self.has_children
    }

    /// Returns true if this node is expanded.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
    /// let state = SpanTreeState::new(vec![root]);
    /// let flat = state.flatten();
    /// assert!(flat[0].is_expanded()); // expanded by default
    /// ```
    pub fn is_expanded(&self) -> bool {
        self.is_expanded
    }

    /// Returns the duration (end - start).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 50.0, 200.0)]);
    /// let flat = state.flatten();
    /// assert_eq!(flat[0].duration(), 150.0);
    /// ```
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}
