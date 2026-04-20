//! State methods for the Diagram component.

use super::graph::IndexedGraph;
use super::layout::{
    ForceDirectedLayout, LayoutAlgorithm, LayoutHints, LayoutResult, SugiyamaLayout,
};
use super::types::{
    DiagramCluster, DiagramEdge, DiagramNode, LayoutMode, NodeStatus, Orientation, RenderMode,
};
use super::{DiagramOutput, DiagramState, navigation};

impl DiagramState {
    /// Creates a new empty diagram state.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::{DiagramState, DiagramNode};
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
    /// use envision::component::diagram::{DiagramState, DiagramNode, DiagramEdge};
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
    /// use envision::component::diagram::{DiagramState, DiagramCluster};
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::{DiagramState, LayoutMode};
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
    /// use envision::component::diagram::{DiagramState, Orientation};
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
    /// use envision::component::diagram::{DiagramState, RenderMode};
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::{DiagramState, DiagramNode};
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::{DiagramState, DiagramEdge};
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::{DiagramState, DiagramNode};
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::{DiagramState, LayoutMode};
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
    /// use envision::component::diagram::{DiagramState, Orientation};
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
    /// use envision::component::diagram::{DiagramState, RenderMode};
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::DiagramState;
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
    /// use envision::component::diagram::{DiagramState, LayoutMode};
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
    /// use envision::component::diagram::{DiagramState, Orientation};
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
    /// use envision::component::diagram::{DiagramState, DiagramNode};
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
    /// use envision::component::diagram::{DiagramState, DiagramEdge};
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
    /// use envision::component::diagram::{DiagramState, DiagramNode};
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
    /// use envision::component::diagram::{DiagramState, DiagramEdge};
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
    /// use envision::component::diagram::{DiagramState, DiagramNode, NodeStatus};
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
    /// use envision::component::diagram::{DiagramState, DiagramNode};
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
    /// use envision::component::diagram::{DiagramState, DiagramNode};
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
    /// use envision::component::diagram::{DiagramState, DiagramNode};
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

    /// Selects the nearest node in the given direction using spatial navigation.
    ///
    /// Returns `true` if the selection changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{DiagramState, DiagramNode};
    ///
    /// let mut state = DiagramState::new()
    ///     .with_node(DiagramNode::new("a", "A"))
    ///     .with_node(DiagramNode::new("b", "B"));
    /// // Spatial nav requires layout to be computed first
    /// state.select_next(); // select something first
    /// ```
    pub(crate) fn select_direction(&mut self, direction: navigation::Direction) -> bool {
        let current = match self.selected {
            Some(idx) => idx,
            None => {
                // No selection — select first node
                if !self.nodes.is_empty() {
                    self.selected = Some(0);
                    return true;
                }
                return false;
            }
        };

        let layout = self.ensure_layout().clone();
        if let Some(target) =
            navigation::find_nearest_in_direction(current, layout.node_positions(), direction)
        {
            self.selected = Some(target);
            true
        } else {
            false
        }
    }

    /// Follows an outgoing edge from the selected node.
    ///
    /// If the node has exactly one outgoing edge, jumps directly.
    /// If it has multiple, sets `follow_targets` for the caller to
    /// show a picker.
    ///
    /// Returns the appropriate output, or `None` if no selection or no edges.
    pub(crate) fn follow_edge(&mut self) -> Option<DiagramOutput> {
        let sel_idx = self.selected?;
        let graph = IndexedGraph::build(&self.nodes, &self.edges);
        let targets = navigation::outgoing_targets(sel_idx, &graph);

        match targets.len() {
            0 => None,
            1 => {
                let target_idx = targets[0];
                let from_id = self.nodes[sel_idx].id().to_string();
                let to_id = self.nodes[target_idx].id().to_string();
                self.selection_history.push(sel_idx);
                self.selected = Some(target_idx);
                Some(DiagramOutput::EdgeFollowed {
                    from: from_id,
                    to: to_id,
                })
            }
            _ => {
                let from_id = self.nodes[sel_idx].id().to_string();
                let target_ids: Vec<String> = targets
                    .iter()
                    .filter_map(|&idx| self.nodes.get(idx).map(|n| n.id().to_string()))
                    .collect();
                self.follow_targets = Some(target_ids.clone());
                Some(DiagramOutput::EdgeChoiceRequired {
                    from: from_id,
                    targets: target_ids,
                })
            }
        }
    }

    /// Follows a specific edge choice from the follow targets list.
    pub(crate) fn follow_edge_choice(&mut self, choice_idx: usize) -> Option<DiagramOutput> {
        let targets = self.follow_targets.take()?;
        let target_id = targets.get(choice_idx)?;
        let sel_idx = self.selected?;
        let from_id = self.nodes[sel_idx].id().to_string();

        if let Some(target_idx) = self.nodes.iter().position(|n| n.id() == target_id) {
            self.selection_history.push(sel_idx);
            self.selected = Some(target_idx);
            Some(DiagramOutput::EdgeFollowed {
                from: from_id,
                to: target_id.clone(),
            })
        } else {
            None
        }
    }

    /// Goes back to the previous node in selection history.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::diagram::{DiagramState, DiagramNode};
    ///
    /// let mut state = DiagramState::new()
    ///     .with_node(DiagramNode::new("a", "A"))
    ///     .with_node(DiagramNode::new("b", "B"));
    /// state.select_next(); // select a (index 0)
    /// state.select_next(); // select b (index 1)
    /// // go_back not available here since we used select_next, not follow_edge
    /// assert!(!state.go_back());
    /// ```
    pub fn go_back(&mut self) -> bool {
        if let Some(prev) = self.selection_history.pop() {
            self.selected = Some(prev);
            self.follow_targets = None;
            true
        } else {
            false
        }
    }

    /// Computes the layout if dirty, returning a reference to the cached result.
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
                LayoutMode::ForceDirected => ForceDirectedLayout::default().compute(
                    &graph,
                    &self.nodes,
                    &self.edges,
                    &self.clusters,
                    &hints,
                ),
            };

            self.viewport
                .set_content_bounds(result.bounding_box.clone());
            self.cached_layout = Some(result);
            self.layout_dirty = false;
        }
        self.cached_layout.as_ref().expect("layout just computed")
    }
}
