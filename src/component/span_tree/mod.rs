//! A hierarchical span tree component for trace visualization.
//!
//! [`SpanTree`] displays hierarchical spans with horizontal timing bars
//! aligned to a shared time axis, similar to the trace view in Jaeger or
//! Zipkin. Each row shows a label on the left and a proportional duration
//! bar on the right.
//!
//! State is stored in [`SpanTreeState`], updated via [`SpanTreeMessage`],
//! and produces [`SpanTreeOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, SpanTree, SpanTreeState, SpanTreeMessage, SpanNode,
//! };
//! use ratatui::style::Color;
//!
//! let root = SpanNode::new("root", "frontend/request", 0.0, 1000.0)
//!     .with_color(Color::Cyan)
//!     .with_child(
//!         SpanNode::new("db", "db/query", 100.0, 400.0)
//!             .with_color(Color::Green),
//!     );
//! let mut state = SpanTreeState::new(vec![root]);
//! state.set_focused(true);
//!
//! // Navigate down
//! SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
//! assert_eq!(state.selected_index(), Some(1));
//! ```

use std::collections::HashSet;

use ratatui::prelude::*;

use super::{Component, Disableable, Focusable, ViewContext};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::scroll::ScrollState;
use crate::theme::Theme;

mod render;
mod types;

pub use types::{FlatSpan, SpanNode};

/// Messages that can be sent to a SpanTree component.
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, SpanTree, SpanTreeState, SpanTreeMessage, SpanNode};
///
/// let root = SpanNode::new("r", "root", 0.0, 100.0);
/// let mut state = SpanTreeState::new(vec![root]);
/// state.set_focused(true);
/// SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum SpanTreeMessage {
    /// Replace all root spans.
    SetRoots(Vec<SpanNode>),
    /// Move selection up.
    SelectUp,
    /// Move selection down.
    SelectDown,
    /// Expand the selected node.
    Expand,
    /// Collapse the selected node.
    Collapse,
    /// Toggle expand/collapse of the selected node.
    Toggle,
    /// Expand all nodes.
    ExpandAll,
    /// Collapse all nodes.
    CollapseAll,
    /// Set the label column width.
    SetLabelWidth(u16),
}

/// Output messages from a SpanTree component.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, SpanTree, SpanTreeState, SpanTreeMessage, SpanTreeOutput, SpanNode,
/// };
///
/// let root = SpanNode::new("r", "root", 0.0, 100.0)
///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
/// let mut state = SpanTreeState::new(vec![root]);
/// state.set_focused(true);
///
/// let output = SpanTree::update(&mut state, SpanTreeMessage::Collapse);
/// assert_eq!(output, Some(SpanTreeOutput::Collapsed("r".into())));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
#[non_exhaustive]
pub enum SpanTreeOutput {
    /// A span was selected. Contains the span id.
    Selected(String),
    /// A span was expanded. Contains the span id.
    Expanded(String),
    /// A span was collapsed. Contains the span id.
    Collapsed(String),
}

/// State for a SpanTree component.
///
/// Contains the root spans, selection state, expanded set, and layout
/// configuration.
///
/// # Example
///
/// ```rust
/// use envision::component::{SpanTreeState, SpanNode};
///
/// let root = SpanNode::new("r", "root", 0.0, 1000.0)
///     .with_child(SpanNode::new("c", "child", 100.0, 500.0));
/// let state = SpanTreeState::new(vec![root]);
///
/// assert_eq!(state.roots().len(), 1);
/// assert_eq!(state.global_start(), 0.0);
/// assert_eq!(state.global_end(), 1000.0);
/// assert_eq!(state.selected_index(), Some(0));
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SpanTreeState {
    /// Root spans.
    roots: Vec<SpanNode>,
    /// Selected index in flattened view.
    selected_index: Option<usize>,
    /// Set of expanded node IDs. All nodes are expanded by default.
    expanded: HashSet<String>,
    /// Scroll state for vertical scrolling.
    scroll: ScrollState,
    /// Earliest start time across all spans.
    global_start: f64,
    /// Latest end time across all spans.
    global_end: f64,
    /// Width allocated for labels.
    label_width: u16,
    /// Optional title.
    title: Option<String>,
    /// Whether the component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
}

impl Default for SpanTreeState {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            selected_index: None,
            expanded: HashSet::new(),
            scroll: ScrollState::default(),
            global_start: 0.0,
            global_end: 0.0,
            label_width: 30,
            title: None,
            focused: false,
            disabled: false,
        }
    }
}

impl SpanTreeState {
    /// Creates a new span tree state with the given root spans.
    ///
    /// All nodes with children start expanded. The first node is selected
    /// if the tree is non-empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
    /// let state = SpanTreeState::new(vec![root]);
    ///
    /// assert_eq!(state.roots().len(), 1);
    /// assert_eq!(state.selected_index(), Some(0));
    /// assert_eq!(state.global_start(), 0.0);
    /// assert_eq!(state.global_end(), 100.0);
    /// ```
    pub fn new(roots: Vec<SpanNode>) -> Self {
        let mut expanded = HashSet::new();
        for root in &roots {
            Self::collect_expanded_ids(root, &mut expanded);
        }

        let (global_start, global_end) = Self::compute_global_range(&roots);
        let selected_index = if roots.is_empty() { None } else { Some(0) };

        let mut state = Self {
            roots,
            selected_index,
            expanded,
            scroll: ScrollState::default(),
            global_start,
            global_end,
            label_width: 30,
            title: None,
            focused: false,
            disabled: false,
        };
        state.scroll.set_content_length(state.flatten().len());
        state
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)])
    ///     .with_title("Trace");
    /// assert_eq!(state.title(), Some("Trace"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the label column width (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)])
    ///     .with_label_width(40);
    /// assert_eq!(state.label_width(), 40);
    /// ```
    pub fn with_label_width(mut self, width: u16) -> Self {
        self.label_width = width;
        self
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)])
    ///     .with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // ---- Accessors ----

    /// Returns the root spans.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![
    ///     SpanNode::new("a", "svc-a", 0.0, 10.0),
    ///     SpanNode::new("b", "svc-b", 5.0, 15.0),
    /// ]);
    /// assert_eq!(state.roots().len(), 2);
    /// ```
    pub fn roots(&self) -> &[SpanNode] {
        &self.roots
    }

    /// Returns a mutable reference to the root spans.
    ///
    /// This is safe because span nodes are simple data containers.
    /// The expanded set tracks nodes by string id, so mutating node
    /// data does not corrupt expand/collapse state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    /// use ratatui::style::Color;
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0);
    /// let mut state = SpanTreeState::new(vec![root]);
    /// state.roots_mut()[0].set_color(Color::Red);
    /// assert_eq!(state.roots()[0].color(), Color::Red);
    /// ```
    /// **Note**: After modifying the collection, the scrollbar may be inaccurate
    /// until the next render. Prefer dedicated methods (e.g., `push_event()`) when available.
    pub fn roots_mut(&mut self) -> &mut Vec<SpanNode> {
        &mut self.roots
    }

    /// Replaces all root spans and recomputes global times.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let mut state = SpanTreeState::new(vec![SpanNode::new("old", "old", 0.0, 10.0)]);
    /// state.set_roots(vec![SpanNode::new("new", "new", 5.0, 50.0)]);
    /// assert_eq!(state.roots().len(), 1);
    /// assert_eq!(state.global_start(), 5.0);
    /// assert_eq!(state.global_end(), 50.0);
    /// ```
    pub fn set_roots(&mut self, roots: Vec<SpanNode>) {
        self.expanded.clear();
        for root in &roots {
            Self::collect_expanded_ids(root, &mut self.expanded);
        }

        let (global_start, global_end) = Self::compute_global_range(&roots);
        self.global_start = global_start;
        self.global_end = global_end;
        self.roots = roots;
        self.selected_index = if self.roots.is_empty() { None } else { Some(0) };
        self.scroll.set_content_length(self.flatten().len());
    }

    /// Returns the currently selected flat index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    /// assert_eq!(state.selected_index(), Some(0));
    ///
    /// let empty = SpanTreeState::default();
    /// assert_eq!(empty.selected_index(), None);
    /// ```
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Returns the currently selected span in flattened view.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0);
    /// let state = SpanTreeState::new(vec![root]);
    /// let selected = state.selected_span().unwrap();
    /// assert_eq!(selected.id(), "r");
    /// assert_eq!(selected.label(), "root");
    /// ```
    pub fn selected_span(&self) -> Option<FlatSpan> {
        let flat = self.flatten();
        let idx = self.selected_index?;
        flat.into_iter().nth(idx)
    }

    /// Returns the earliest start time across all spans.
    pub fn global_start(&self) -> f64 {
        self.global_start
    }

    /// Returns the latest end time across all spans.
    pub fn global_end(&self) -> f64 {
        self.global_end
    }

    /// Returns the label column width.
    pub fn label_width(&self) -> u16 {
        self.label_width
    }

    /// Returns the title, if set.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanTreeState;
    ///
    /// let mut state = SpanTreeState::default();
    /// state.set_title("Trace View");
    /// assert_eq!(state.title(), Some("Trace View"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns true if the component is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanTreeState;
    ///
    /// let state = SpanTreeState::default();
    /// assert!(!state.is_focused());
    /// ```
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    /// state.set_focused(true);
    /// assert!(state.is_focused());
    /// ```
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanTreeState;
    ///
    /// let state = SpanTreeState::default();
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Returns true if the tree is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SpanTreeState;
    ///
    /// assert!(SpanTreeState::default().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    /// Returns the set of expanded node IDs.
    pub fn expanded_ids(&self) -> &HashSet<String> {
        &self.expanded
    }

    // ---- Expand/Collapse ----

    /// Expands a node by its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
    /// let mut state = SpanTreeState::new(vec![root]);
    /// state.collapse("r");
    /// assert!(!state.expanded_ids().contains("r"));
    /// state.expand("r");
    /// assert!(state.expanded_ids().contains("r"));
    /// ```
    pub fn expand(&mut self, id: &str) {
        self.expanded.insert(id.to_string());
        self.scroll.set_content_length(self.flatten().len());
    }

    /// Collapses a node by its ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
    /// let mut state = SpanTreeState::new(vec![root]);
    /// state.collapse("r");
    /// assert!(!state.expanded_ids().contains("r"));
    /// ```
    pub fn collapse(&mut self, id: &str) {
        self.expanded.remove(id);
        // Clamp selection if it went out of bounds
        let visible = self.flatten().len();
        if let Some(idx) = self.selected_index {
            if idx >= visible {
                self.selected_index = Some(visible.saturating_sub(1));
            }
        }
        self.scroll.set_content_length(visible);
    }

    /// Expands all nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
    /// let mut state = SpanTreeState::new(vec![root]);
    /// state.collapse_all();
    /// state.expand_all();
    /// assert!(state.expanded_ids().contains("r"));
    /// ```
    pub fn expand_all(&mut self) {
        self.expanded.clear();
        for root in &self.roots {
            Self::collect_expanded_ids(root, &mut self.expanded);
        }
        self.scroll.set_content_length(self.flatten().len());
    }

    /// Collapses all nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c", "child", 10.0, 50.0));
    /// let mut state = SpanTreeState::new(vec![root]);
    /// state.collapse_all();
    /// assert!(state.expanded_ids().is_empty());
    /// ```
    pub fn collapse_all(&mut self) {
        self.expanded.clear();
        self.selected_index = if self.roots.is_empty() { None } else { Some(0) };
        self.scroll.set_content_length(self.flatten().len());
    }

    // ---- Flatten ----

    /// Flattens the visible hierarchy into a list of [`FlatSpan`] items.
    ///
    /// Only expanded nodes have their children included. The order is
    /// depth-first, matching the visual tree order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode};
    ///
    /// let root = SpanNode::new("r", "root", 0.0, 100.0)
    ///     .with_child(SpanNode::new("c1", "child-1", 10.0, 50.0))
    ///     .with_child(SpanNode::new("c2", "child-2", 50.0, 90.0));
    /// let state = SpanTreeState::new(vec![root]);
    /// let flat = state.flatten();
    /// assert_eq!(flat.len(), 3);
    /// assert_eq!(flat[0].id(), "r");
    /// assert_eq!(flat[0].depth(), 0);
    /// assert_eq!(flat[1].id(), "c1");
    /// assert_eq!(flat[1].depth(), 1);
    /// ```
    pub fn flatten(&self) -> Vec<FlatSpan> {
        let mut result = Vec::new();
        for root in &self.roots {
            self.flatten_node(root, 0, &mut result);
        }
        result
    }

    // ---- Instance methods ----

    /// Maps an input event to a span tree message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SpanTreeState, SpanNode, SpanTreeMessage};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    /// state.set_focused(true);
    /// let msg = state.handle_event(&Event::key(KeyCode::Down));
    /// assert_eq!(msg, Some(SpanTreeMessage::SelectDown));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<SpanTreeMessage> {
        SpanTree::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<SpanTreeOutput> {
        SpanTree::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: SpanTreeMessage) -> Option<SpanTreeOutput> {
        SpanTree::update(self, msg)
    }

    // ---- Private helpers ----

    /// Recursively collects all node IDs with children into the expanded set.
    fn collect_expanded_ids(node: &SpanNode, expanded: &mut HashSet<String>) {
        if node.has_children() {
            expanded.insert(node.id.clone());
            for child in &node.children {
                Self::collect_expanded_ids(child, expanded);
            }
        }
    }

    /// Computes the global start and end times across all spans.
    fn compute_global_range(roots: &[SpanNode]) -> (f64, f64) {
        let mut min_start = f64::INFINITY;
        let mut max_end = f64::NEG_INFINITY;
        for root in roots {
            Self::compute_range_recursive(root, &mut min_start, &mut max_end);
        }
        if min_start.is_infinite() {
            (0.0, 0.0)
        } else {
            (min_start, max_end)
        }
    }

    /// Recursively computes min start and max end across all nodes.
    fn compute_range_recursive(node: &SpanNode, min_start: &mut f64, max_end: &mut f64) {
        if node.start < *min_start {
            *min_start = node.start;
        }
        if node.end > *max_end {
            *max_end = node.end;
        }
        for child in &node.children {
            Self::compute_range_recursive(child, min_start, max_end);
        }
    }

    /// Recursively flattens a node and its visible children.
    fn flatten_node(&self, node: &SpanNode, depth: usize, result: &mut Vec<FlatSpan>) {
        let is_expanded = self.expanded.contains(&node.id);
        result.push(FlatSpan {
            id: node.id.clone(),
            label: node.label.clone(),
            start: node.start,
            end: node.end,
            color: node.color,
            status: node.status.clone(),
            depth,
            has_children: node.has_children(),
            is_expanded,
        });

        if is_expanded {
            for child in &node.children {
                self.flatten_node(child, depth + 1, result);
            }
        }
    }
}

/// A hierarchical span tree component for trace visualization.
///
/// Displays hierarchical spans with horizontal timing bars aligned to
/// a shared time axis. Each row shows a label with tree
/// expand/collapse indicators on the left, and a proportional
/// duration bar on the right.
///
/// # Visual Format
///
/// ```text
/// ┌─ Trace ─────────────────────────────────────────┐
/// │ Label                     │ 0ms    500ms   1000ms│
/// │───────────────────────────┼──────────────────────│
/// │ ▾ frontend/request        │ ████████████████████ │
/// │   ▾ api/handler           │   ██████████████     │
/// │     db/query              │     ████             │
/// │     cache/lookup          │          ███         │
/// │   auth/validate           │ ███                  │
/// └─────────────────────────────────────────────────┘
/// ```
///
/// # Keyboard Navigation
///
/// - `Up/k`: Move selection up
/// - `Down/j`: Move selection down
/// - `Right/l`: Expand selected node
/// - `Left/h`: Collapse selected node
/// - `Space/Enter`: Toggle expand/collapse
/// - `Shift+Right`: Increase label width
/// - `Shift+Left`: Decrease label width
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, SpanTree, SpanTreeState, SpanTreeMessage, SpanNode,
/// };
/// use ratatui::style::Color;
///
/// let root = SpanNode::new("req", "frontend/request", 0.0, 1000.0)
///     .with_color(Color::Cyan)
///     .with_child(
///         SpanNode::new("api", "api/handler", 50.0, 800.0)
///             .with_color(Color::Yellow)
///             .with_child(SpanNode::new("db", "db/query", 100.0, 400.0).with_color(Color::Green))
///     );
///
/// let mut state = SpanTreeState::new(vec![root]);
/// state.set_focused(true);
///
/// // Navigate through spans
/// SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
/// SpanTree::update(&mut state, SpanTreeMessage::Collapse);
/// ```
pub struct SpanTree;

impl Component for SpanTree {
    type State = SpanTreeState;
    type Message = SpanTreeMessage;
    type Output = SpanTreeOutput;

    fn init() -> Self::State {
        SpanTreeState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            SpanTreeMessage::SetRoots(roots) => {
                state.set_roots(roots);
                None
            }
            SpanTreeMessage::SetLabelWidth(width) => {
                state.label_width = width.clamp(10, 100);
                None
            }
            SpanTreeMessage::ExpandAll => {
                state.expand_all();
                None
            }
            SpanTreeMessage::CollapseAll => {
                state.collapse_all();
                None
            }
            _ => {
                if state.disabled {
                    return None;
                }

                let flat = state.flatten();
                if flat.is_empty() {
                    return None;
                }

                let selected = state.selected_index?;

                match msg {
                    SpanTreeMessage::SelectUp => {
                        if selected > 0 {
                            state.selected_index = Some(selected - 1);
                            let span = &flat[selected - 1];
                            return Some(SpanTreeOutput::Selected(span.id.clone()));
                        }
                        None
                    }
                    SpanTreeMessage::SelectDown => {
                        if selected < flat.len() - 1 {
                            state.selected_index = Some(selected + 1);
                            let span = &flat[selected + 1];
                            return Some(SpanTreeOutput::Selected(span.id.clone()));
                        }
                        None
                    }
                    SpanTreeMessage::Expand => {
                        if let Some(span) = flat.get(selected) {
                            if span.has_children && !span.is_expanded {
                                let id = span.id.clone();
                                state.expanded.insert(id.clone());
                                state.scroll.set_content_length(state.flatten().len());
                                return Some(SpanTreeOutput::Expanded(id));
                            }
                        }
                        None
                    }
                    SpanTreeMessage::Collapse => {
                        if let Some(span) = flat.get(selected) {
                            if span.has_children && span.is_expanded {
                                let id = span.id.clone();
                                state.expanded.remove(&id);
                                let new_flat = state.flatten();
                                if selected >= new_flat.len() {
                                    state.selected_index = Some(new_flat.len().saturating_sub(1));
                                }
                                state.scroll.set_content_length(new_flat.len());
                                return Some(SpanTreeOutput::Collapsed(id));
                            }
                        }
                        None
                    }
                    SpanTreeMessage::Toggle => {
                        if let Some(span) = flat.get(selected) {
                            if span.has_children {
                                let id = span.id.clone();
                                if span.is_expanded {
                                    state.expanded.remove(&id);
                                    let new_flat = state.flatten();
                                    if selected >= new_flat.len() {
                                        state.selected_index =
                                            Some(new_flat.len().saturating_sub(1));
                                    }
                                    state.scroll.set_content_length(new_flat.len());
                                    return Some(SpanTreeOutput::Collapsed(id));
                                } else {
                                    state.expanded.insert(id.clone());
                                    state.scroll.set_content_length(state.flatten().len());
                                    return Some(SpanTreeOutput::Expanded(id));
                                }
                            }
                        }
                        None
                    }
                    // Already handled above
                    SpanTreeMessage::SetRoots(_)
                    | SpanTreeMessage::SetLabelWidth(_)
                    | SpanTreeMessage::ExpandAll
                    | SpanTreeMessage::CollapseAll => unreachable!(),
                }
            }
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
            match key.code {
                KeyCode::Up | KeyCode::Char('k') if !has_shift => Some(SpanTreeMessage::SelectUp),
                KeyCode::Down | KeyCode::Char('j') if !has_shift => {
                    Some(SpanTreeMessage::SelectDown)
                }
                KeyCode::Right | KeyCode::Char('l') if has_shift => Some(
                    SpanTreeMessage::SetLabelWidth(state.label_width.saturating_add(2)),
                ),
                KeyCode::Left | KeyCode::Char('h') if has_shift => Some(
                    SpanTreeMessage::SetLabelWidth(state.label_width.saturating_sub(2)),
                ),
                KeyCode::Right | KeyCode::Char('l') => Some(SpanTreeMessage::Expand),
                KeyCode::Left | KeyCode::Char('h') => Some(SpanTreeMessage::Collapse),
                KeyCode::Char(' ') | KeyCode::Enter => Some(SpanTreeMessage::Toggle),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, _ctx: &ViewContext) {
        render::render_span_tree(state, frame, area, theme);
    }
}

impl Focusable for SpanTree {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for SpanTree {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
