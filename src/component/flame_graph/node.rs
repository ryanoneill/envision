//! The FlameNode type representing a single stack frame in a flame graph.

use ratatui::style::Color;

/// A node in the flame graph (a stack frame).
///
/// Each node has a label (typically a function name), a value
/// (time, samples, bytes, etc.), an optional color, and child frames.
///
/// # Example
///
/// ```rust
/// use envision::component::FlameNode;
/// use ratatui::style::Color;
///
/// let node = FlameNode::new("main()", 500)
///     .with_color(Color::Red)
///     .with_child(FlameNode::new("compute()", 300))
///     .with_child(FlameNode::new("io()", 100));
///
/// assert_eq!(node.label(), "main()");
/// assert_eq!(node.value(), 500);
/// assert_eq!(node.total_value(), 500);
/// assert_eq!(node.self_value(), 100);
/// assert_eq!(node.children().len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct FlameNode {
    /// Display label (function name, service, etc.).
    pub(crate) label: String,
    /// Value (time, samples, bytes, etc.).
    pub(crate) value: u64,
    /// Color for this frame.
    pub(crate) color: Color,
    /// Child frames.
    pub(crate) children: Vec<FlameNode>,
}

impl FlameNode {
    /// Creates a new flame node with the given label and value.
    ///
    /// The default color is `Color::Cyan`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    /// use ratatui::style::Color;
    ///
    /// let node = FlameNode::new("main()", 500);
    /// assert_eq!(node.label(), "main()");
    /// assert_eq!(node.value(), 500);
    /// assert_eq!(node.color(), Color::Cyan);
    /// ```
    pub fn new(label: impl Into<String>, value: u64) -> Self {
        Self {
            label: label.into(),
            value,
            color: Color::Cyan,
            children: Vec::new(),
        }
    }

    /// Sets the color for this frame (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    /// use ratatui::style::Color;
    ///
    /// let node = FlameNode::new("main()", 500).with_color(Color::Red);
    /// assert_eq!(node.color(), Color::Red);
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Adds a child frame (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    ///
    /// let node = FlameNode::new("main()", 500)
    ///     .with_child(FlameNode::new("compute()", 300));
    /// assert_eq!(node.children().len(), 1);
    /// ```
    pub fn with_child(mut self, child: FlameNode) -> Self {
        self.children.push(child);
        self
    }

    /// Sets all children at once (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    ///
    /// let children = vec![
    ///     FlameNode::new("a()", 100),
    ///     FlameNode::new("b()", 200),
    /// ];
    /// let node = FlameNode::new("main()", 500).with_children(children);
    /// assert_eq!(node.children().len(), 2);
    /// ```
    pub fn with_children(mut self, children: Vec<FlameNode>) -> Self {
        self.children = children;
        self
    }

    /// Returns the label of this frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    ///
    /// let node = FlameNode::new("main()", 500);
    /// assert_eq!(node.label(), "main()");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the value of this frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    ///
    /// let node = FlameNode::new("main()", 500);
    /// assert_eq!(node.value(), 500);
    /// ```
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Returns the color of this frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    /// use ratatui::style::Color;
    ///
    /// let node = FlameNode::new("main()", 500);
    /// assert_eq!(node.color(), Color::Cyan);
    /// ```
    pub fn color(&self) -> Color {
        self.color
    }

    /// Returns the children of this frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    ///
    /// let node = FlameNode::new("main()", 500)
    ///     .with_child(FlameNode::new("a()", 100));
    /// assert_eq!(node.children().len(), 1);
    /// ```
    pub fn children(&self) -> &[FlameNode] {
        &self.children
    }

    /// Returns the total value of this frame including all descendants.
    ///
    /// This is the node's own value, which represents the total time
    /// spent in this frame and all its descendants.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    ///
    /// let node = FlameNode::new("main()", 500)
    ///     .with_child(FlameNode::new("compute()", 300));
    /// assert_eq!(node.total_value(), 500);
    /// ```
    pub fn total_value(&self) -> u64 {
        self.value
    }

    /// Returns the self value (value minus children's total values).
    ///
    /// This represents time spent exclusively in this frame, not in
    /// any of its children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    ///
    /// let node = FlameNode::new("main()", 500)
    ///     .with_child(FlameNode::new("compute()", 300))
    ///     .with_child(FlameNode::new("io()", 100));
    /// assert_eq!(node.self_value(), 100); // 500 - 300 - 100
    /// ```
    pub fn self_value(&self) -> u64 {
        let children_total: u64 = self.children.iter().map(|c| c.total_value()).sum();
        self.value.saturating_sub(children_total)
    }

    /// Returns true if this node has children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameNode;
    ///
    /// let leaf = FlameNode::new("leaf()", 100);
    /// assert!(!leaf.has_children());
    ///
    /// let parent = FlameNode::new("parent()", 200)
    ///     .with_child(FlameNode::new("child()", 100));
    /// assert!(parent.has_children());
    /// ```
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}
