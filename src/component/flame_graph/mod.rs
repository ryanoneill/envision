//! An interactive flame graph component for profiling data visualization.
//!
//! [`FlameGraph`] renders stack frames as horizontal bars where width is
//! proportional to time or samples. Supports zoom into subtrees and
//! search/highlight. Root is rendered at the top (icicle graph style).
//!
//! State is stored in [`FlameGraphState`], updated via [`FlameGraphMessage`],
//! and produces [`FlameGraphOutput`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, FlameGraph, FlameGraphState, FlameGraphMessage, FlameNode,
//! };
//! use ratatui::style::Color;
//!
//! let root = FlameNode::new("main()", 500)
//!     .with_color(Color::Red)
//!     .with_child(
//!         FlameNode::new("compute()", 300)
//!             .with_child(FlameNode::new("sort()", 200))
//!     )
//!     .with_child(FlameNode::new("io()", 100));
//! let mut state = FlameGraphState::with_root(root);
//!
//! // Navigate down into children
//! FlameGraph::update(&mut state, FlameGraphMessage::SelectDown);
//! assert_eq!(state.selected_depth(), 1);
//! ```

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};

mod node;
mod render;

pub use node::FlameNode;

/// Messages that can be sent to a FlameGraph component.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, FlameGraph, FlameGraphState, FlameGraphMessage, FlameNode,
/// };
///
/// let root = FlameNode::new("main()", 500)
///     .with_child(FlameNode::new("compute()", 300));
/// let mut state = FlameGraphState::with_root(root);
/// FlameGraph::update(&mut state, FlameGraphMessage::SelectDown);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum FlameGraphMessage {
    /// Set the root frame.
    SetRoot(FlameNode),
    /// Clear the graph.
    Clear,
    /// Zoom into the selected frame.
    ZoomIn,
    /// Zoom back to parent.
    ZoomOut,
    /// Reset zoom to original root.
    ResetZoom,
    /// Move selection to parent depth.
    SelectUp,
    /// Move selection to child depth.
    SelectDown,
    /// Move selection to previous sibling at current depth.
    SelectLeft,
    /// Move selection to next sibling at current depth.
    SelectRight,
    /// Set search query for highlighting.
    SetSearch(String),
    /// Clear search query.
    ClearSearch,
}

/// Output messages from a FlameGraph component.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, FlameGraph, FlameGraphState, FlameGraphMessage, FlameGraphOutput, FlameNode,
/// };
///
/// let root = FlameNode::new("main()", 500)
///     .with_child(FlameNode::new("compute()", 300));
/// let mut state = FlameGraphState::with_root(root);
///
/// let output = FlameGraph::update(&mut state, FlameGraphMessage::SelectDown);
/// assert!(matches!(output, Some(FlameGraphOutput::FrameSelected { .. })));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum FlameGraphOutput {
    /// A frame was selected.
    FrameSelected {
        /// The label of the selected frame.
        label: String,
        /// The total value of the selected frame.
        value: u64,
        /// The self value of the selected frame (excluding children).
        self_value: u64,
    },
    /// Zoomed into a frame.
    ZoomedIn(String),
    /// Zoomed out from a frame.
    ZoomedOut,
}

/// State for a FlameGraph component.
///
/// Contains the root flame node, zoom stack, selection state, and search
/// configuration.
///
/// # Example
///
/// ```rust
/// use envision::component::{FlameGraphState, FlameNode};
///
/// let root = FlameNode::new("main()", 500)
///     .with_child(FlameNode::new("compute()", 300));
/// let state = FlameGraphState::with_root(root);
///
/// assert!(state.root().is_some());
/// assert_eq!(state.selected_depth(), 0);
/// assert_eq!(state.selected_index(), 0);
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct FlameGraphState {
    /// The root frame.
    root: Option<FlameNode>,
    /// Stack of zoomed-into node labels (for zoom navigation).
    zoom_stack: Vec<String>,
    /// Selected depth level.
    selected_depth: usize,
    /// Selected frame index at current depth.
    selected_index: usize,
    /// Search query for highlighting.
    search_query: String,
    /// Optional title.
    title: Option<String>,
}

impl FlameGraphState {
    /// Creates an empty flame graph state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameGraphState;
    ///
    /// let state = FlameGraphState::new();
    /// assert!(state.root().is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a flame graph state with the given root frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let root = FlameNode::new("main()", 500);
    /// let state = FlameGraphState::with_root(root);
    /// assert!(state.root().is_some());
    /// assert_eq!(state.root().unwrap().label(), "main()");
    /// ```
    pub fn with_root(root: FlameNode) -> Self {
        Self {
            root: Some(root),
            ..Self::default()
        }
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let state = FlameGraphState::with_root(FlameNode::new("main()", 500))
    ///     .with_title("CPU Profile");
    /// assert_eq!(state.title(), Some("CPU Profile"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    // ---- Accessors ----

    /// Returns the root frame, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    /// assert_eq!(state.root().unwrap().label(), "main()");
    /// ```
    pub fn root(&self) -> Option<&FlameNode> {
        self.root.as_ref()
    }

    /// Returns a mutable reference to the root frame, if any.
    ///
    /// This is safe because the flame node tree is simple data.
    /// The zoom stack references nodes by label, so mutating node
    /// values or children does not corrupt navigation state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    /// use ratatui::style::Color;
    ///
    /// let root = FlameNode::new("main()", 500);
    /// let mut state = FlameGraphState::with_root(root);
    /// if let Some(r) = state.root_mut() {
    ///     *r = r.clone().with_color(Color::Red);
    /// }
    /// assert!(state.root().is_some());
    /// ```
    pub fn root_mut(&mut self) -> Option<&mut FlameNode> {
        self.root.as_mut()
    }

    /// Sets the root frame.
    ///
    /// Resets zoom and selection state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let mut state = FlameGraphState::new();
    /// state.set_root(FlameNode::new("main()", 500));
    /// assert!(state.root().is_some());
    /// ```
    pub fn set_root(&mut self, root: FlameNode) {
        self.root = Some(root);
        self.zoom_stack.clear();
        self.selected_depth = 0;
        self.selected_index = 0;
    }

    /// Clears the flame graph.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    /// state.clear();
    /// assert!(state.root().is_none());
    /// ```
    pub fn clear(&mut self) {
        self.root = None;
        self.zoom_stack.clear();
        self.selected_depth = 0;
        self.selected_index = 0;
        self.search_query.clear();
    }

    /// Returns the currently visible root (after zoom).
    ///
    /// Follows the zoom stack to find the currently displayed subtree root.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let root = FlameNode::new("main()", 500)
    ///     .with_child(FlameNode::new("compute()", 300));
    /// let state = FlameGraphState::with_root(root);
    /// assert_eq!(state.current_view_root().unwrap().label(), "main()");
    /// ```
    pub fn current_view_root(&self) -> Option<&FlameNode> {
        let mut current = self.root.as_ref()?;
        for label in &self.zoom_stack {
            let found = current.children.iter().find(|c| &c.label == label);
            match found {
                Some(child) => current = child,
                None => return Some(current),
            }
        }
        Some(current)
    }

    /// Returns the currently selected frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let root = FlameNode::new("main()", 500)
    ///     .with_child(FlameNode::new("compute()", 300));
    /// let state = FlameGraphState::with_root(root);
    /// assert_eq!(state.selected_frame().unwrap().label(), "main()");
    /// ```
    pub fn selected_frame(&self) -> Option<&FlameNode> {
        let view_root = self.current_view_root()?;
        Self::find_at_depth(view_root, self.selected_depth, self.selected_index)
    }

    /// Returns the zoom stack.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameGraphState;
    ///
    /// let state = FlameGraphState::new();
    /// assert!(state.zoom_stack().is_empty());
    /// ```
    pub fn zoom_stack(&self) -> &[String] {
        &self.zoom_stack
    }

    /// Returns the selected depth level.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameGraphState;
    ///
    /// let state = FlameGraphState::new();
    /// assert_eq!(state.selected_depth(), 0);
    /// ```
    pub fn selected_depth(&self) -> usize {
        self.selected_depth
    }

    /// Returns the selected frame index at the current depth.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameGraphState;
    ///
    /// let state = FlameGraphState::new();
    /// assert_eq!(state.selected_index(), 0);
    /// ```
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns the search query.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameGraphState;
    ///
    /// let state = FlameGraphState::new();
    /// assert_eq!(state.search_query(), "");
    /// ```
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Returns the search query as an `Option<&str>`, or `None` if empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let state = FlameGraphState::new();
    /// assert_eq!(state.search(), None);
    ///
    /// let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    /// state.set_search("compute".to_string());
    /// assert_eq!(state.search(), Some("compute"));
    /// ```
    pub fn search(&self) -> Option<&str> {
        if self.search_query.is_empty() {
            None
        } else {
            Some(&self.search_query)
        }
    }

    /// Sets the search query.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    /// state.set_search("compute".to_string());
    /// assert_eq!(state.search_query(), "compute");
    /// ```
    pub fn set_search(&mut self, query: String) {
        self.search_query = query;
    }

    /// Returns the title, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameGraphState;
    ///
    /// let state = FlameGraphState::new();
    /// assert_eq!(state.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::FlameGraphState;
    ///
    /// let mut state = FlameGraphState::new();
    /// state.set_title("CPU Profile");
    /// assert_eq!(state.title(), Some("CPU Profile"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    // ---- Zoom ----

    /// Zooms into the currently selected frame, making it the new view root.
    ///
    /// Only zooms if the selected frame has children.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let root = FlameNode::new("main()", 500)
    ///     .with_child(FlameNode::new("compute()", 300)
    ///         .with_child(FlameNode::new("sort()", 200)));
    /// let mut state = FlameGraphState::with_root(root);
    ///
    /// // Select compute() (depth 1, index 0)
    /// state.select_down();
    /// // Zoom into compute()
    /// let zoomed = state.zoom_in();
    /// assert!(zoomed);
    /// assert_eq!(state.current_view_root().unwrap().label(), "compute()");
    /// ```
    pub fn zoom_in(&mut self) -> bool {
        if let Some(frame) = self.selected_frame() {
            if !frame.children.is_empty() {
                let label = frame.label.clone();
                self.zoom_stack.push(label);
                self.selected_depth = 0;
                self.selected_index = 0;
                return true;
            }
        }
        false
    }

    /// Zooms out one level.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let root = FlameNode::new("main()", 500)
    ///     .with_child(FlameNode::new("compute()", 300)
    ///         .with_child(FlameNode::new("sort()", 200)));
    /// let mut state = FlameGraphState::with_root(root);
    /// state.select_down();
    /// state.zoom_in();
    /// assert!(state.zoom_out());
    /// assert_eq!(state.current_view_root().unwrap().label(), "main()");
    /// ```
    pub fn zoom_out(&mut self) -> bool {
        if self.zoom_stack.pop().is_some() {
            self.selected_depth = 0;
            self.selected_index = 0;
            true
        } else {
            false
        }
    }

    /// Resets zoom to the original root.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{FlameGraphState, FlameNode};
    ///
    /// let root = FlameNode::new("main()", 500)
    ///     .with_child(FlameNode::new("compute()", 300)
    ///         .with_child(FlameNode::new("sort()", 200)));
    /// let mut state = FlameGraphState::with_root(root);
    /// state.select_down();
    /// state.zoom_in();
    /// state.reset_zoom();
    /// assert!(state.zoom_stack().is_empty());
    /// assert_eq!(state.current_view_root().unwrap().label(), "main()");
    /// ```
    pub fn reset_zoom(&mut self) {
        self.zoom_stack.clear();
        self.selected_depth = 0;
        self.selected_index = 0;
    }

    // ---- Navigation ----

    /// Moves selection up to parent depth.
    ///
    /// Returns true if the selection changed.
    pub fn select_up(&mut self) -> bool {
        if self.root.is_none() {
            return false;
        }
        if self.selected_depth > 0 {
            // Find which parent contains the currently selected node
            let parent_index = self.find_parent_index();
            self.selected_depth -= 1;
            self.selected_index = parent_index;
            true
        } else {
            false
        }
    }

    /// Moves selection down to child depth.
    ///
    /// Selects the first child of the nearest ancestor that has children
    /// at the next depth level.
    ///
    /// Returns true if the selection changed.
    pub fn select_down(&mut self) -> bool {
        let view_root = match self.current_view_root() {
            Some(r) => r,
            None => return false,
        };
        let frames_at_next = Self::frames_at_depth(view_root, self.selected_depth + 1);
        if frames_at_next.is_empty() {
            return false;
        }

        // Find the first child of the currently selected frame
        if let Some(selected) = self.selected_frame() {
            if !selected.children.is_empty() {
                // Find the index of the first child in the next depth level
                let first_child_label = &selected.children[0].label;
                let child_idx = frames_at_next
                    .iter()
                    .position(|f| &f.label == first_child_label)
                    .unwrap_or(0);
                self.selected_depth += 1;
                self.selected_index = child_idx;
                return true;
            }
        }

        // If selected frame has no children, go to first frame at next depth
        self.selected_depth += 1;
        self.selected_index = 0;
        true
    }

    /// Moves selection to the previous sibling at the current depth.
    ///
    /// Returns true if the selection changed.
    pub fn select_left(&mut self) -> bool {
        if self.root.is_none() {
            return false;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
            true
        } else {
            false
        }
    }

    /// Moves selection to the next sibling at the current depth.
    ///
    /// Returns true if the selection changed.
    pub fn select_right(&mut self) -> bool {
        let view_root = match self.current_view_root() {
            Some(r) => r,
            None => return false,
        };
        let frames_at_depth = Self::frames_at_depth(view_root, self.selected_depth);
        if self.selected_index + 1 < frames_at_depth.len() {
            self.selected_index += 1;
            true
        } else {
            false
        }
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: FlameGraphMessage) -> Option<FlameGraphOutput> {
        FlameGraph::update(self, msg)
    }

    // ---- Private helpers ----

    /// Finds the parent index of the currently selected node.
    fn find_parent_index(&self) -> usize {
        if self.selected_depth == 0 {
            return 0;
        }
        let view_root = match self.current_view_root() {
            Some(r) => r,
            None => return 0,
        };
        let parent_frames = Self::frames_at_depth(view_root, self.selected_depth - 1);
        let child_frames = Self::frames_at_depth(view_root, self.selected_depth);

        // Find which parent owns the currently selected child
        if let Some(selected_child) = child_frames.get(self.selected_index) {
            for (pi, parent) in parent_frames.iter().enumerate() {
                if parent
                    .children
                    .iter()
                    .any(|c| c.label == selected_child.label && c.value == selected_child.value)
                {
                    return pi;
                }
            }
        }
        0
    }

    /// Returns all frames at a given depth level under a root.
    fn frames_at_depth(root: &FlameNode, depth: usize) -> Vec<&FlameNode> {
        let mut result = Vec::new();
        Self::collect_at_depth(root, 0, depth, &mut result);
        result
    }

    /// Recursively collects frames at a specific depth.
    fn collect_at_depth<'a>(
        node: &'a FlameNode,
        current_depth: usize,
        target_depth: usize,
        result: &mut Vec<&'a FlameNode>,
    ) {
        if current_depth == target_depth {
            result.push(node);
            return;
        }
        for child in &node.children {
            Self::collect_at_depth(child, current_depth + 1, target_depth, result);
        }
    }

    /// Finds the node at a given depth and index.
    fn find_at_depth(root: &FlameNode, depth: usize, index: usize) -> Option<&FlameNode> {
        let frames = Self::frames_at_depth(root, depth);
        frames.into_iter().nth(index)
    }

    /// Returns the maximum depth of the tree rooted at the current view root.
    pub(crate) fn max_depth(&self) -> usize {
        match self.current_view_root() {
            Some(root) => Self::compute_max_depth(root, 0),
            None => 0,
        }
    }

    /// Recursively computes the maximum depth.
    fn compute_max_depth(node: &FlameNode, current: usize) -> usize {
        if node.children.is_empty() {
            return current;
        }
        node.children
            .iter()
            .map(|c| Self::compute_max_depth(c, current + 1))
            .max()
            .unwrap_or(current)
    }
}

/// An interactive flame graph component for profiling data visualization.
///
/// Renders stack frames as horizontal bars where width is proportional
/// to time or samples. Root is displayed at the top (icicle graph style).
///
/// # Visual Format
///
/// ```text
/// +- Flame Graph -----------------------------------------+
/// | ████████████████████████████████████████ main()       |  depth 0
/// | ████████████████████████████ compute()  ██████ io()   |  depth 1
/// | ██████████████ sort()  ████████ hash()                |  depth 2
/// | ████ merge()                                          |  depth 3
/// |-------------------------------------------------------|
/// | Selected: sort()  150 samples (30%)  self: 50         |
/// +-------------------------------------------------------+
/// ```
///
/// # Keyboard Navigation
///
/// - `Up/k`: Select parent depth
/// - `Down/j`: Select child depth
/// - `Left/h`: Select previous sibling
/// - `Right/l`: Select next sibling
/// - `Enter`: Zoom into selected frame
/// - `Escape/Backspace`: Zoom out
/// - `Home`: Reset zoom to root
/// - `/`: Set search query
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, FlameGraph, FlameGraphState, FlameGraphMessage, FlameNode,
/// };
/// use ratatui::style::Color;
///
/// let root = FlameNode::new("main()", 500)
///     .with_color(Color::Red)
///     .with_child(
///         FlameNode::new("compute()", 300)
///             .with_child(FlameNode::new("sort()", 200))
///     )
///     .with_child(FlameNode::new("io()", 100));
///
/// let mut state = FlameGraphState::with_root(root);
///
/// // Navigate and zoom
/// FlameGraph::update(&mut state, FlameGraphMessage::SelectDown);
/// FlameGraph::update(&mut state, FlameGraphMessage::ZoomIn);
/// ```
pub struct FlameGraph;

impl Component for FlameGraph {
    type State = FlameGraphState;
    type Message = FlameGraphMessage;
    type Output = FlameGraphOutput;

    fn init() -> Self::State {
        FlameGraphState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            FlameGraphMessage::SetRoot(root) => {
                state.set_root(root);
                None
            }
            FlameGraphMessage::Clear => {
                state.clear();
                None
            }
            FlameGraphMessage::SetSearch(query) => {
                state.search_query = query;
                None
            }
            FlameGraphMessage::ClearSearch => {
                state.search_query.clear();
                None
            }
            FlameGraphMessage::ResetZoom => {
                state.reset_zoom();
                None
            }
            _ => {
                state.root.as_ref()?;
                match msg {
                    FlameGraphMessage::SelectUp => {
                        if state.select_up() {
                            make_selected_output(state)
                        } else {
                            None
                        }
                    }
                    FlameGraphMessage::SelectDown => {
                        if state.select_down() {
                            make_selected_output(state)
                        } else {
                            None
                        }
                    }
                    FlameGraphMessage::SelectLeft => {
                        if state.select_left() {
                            make_selected_output(state)
                        } else {
                            None
                        }
                    }
                    FlameGraphMessage::SelectRight => {
                        if state.select_right() {
                            make_selected_output(state)
                        } else {
                            None
                        }
                    }
                    FlameGraphMessage::ZoomIn => {
                        if let Some(frame) = state.selected_frame() {
                            let label = frame.label.clone();
                            if state.zoom_in() {
                                Some(FlameGraphOutput::ZoomedIn(label))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    FlameGraphMessage::ZoomOut => {
                        if state.zoom_out() {
                            Some(FlameGraphOutput::ZoomedOut)
                        } else {
                            None
                        }
                    }
                    // Already handled above
                    FlameGraphMessage::SetRoot(_)
                    | FlameGraphMessage::Clear
                    | FlameGraphMessage::SetSearch(_)
                    | FlameGraphMessage::ClearSearch
                    | FlameGraphMessage::ResetZoom => unreachable!(),
                }
            }
        }
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }
        if let Some(key) = event.as_key() {
            match key.key {
                Key::Up | Key::Char('k') => Some(FlameGraphMessage::SelectUp),
                Key::Down | Key::Char('j') => Some(FlameGraphMessage::SelectDown),
                Key::Left | Key::Char('h') => Some(FlameGraphMessage::SelectLeft),
                Key::Right | Key::Char('l') => Some(FlameGraphMessage::SelectRight),
                Key::Enter => Some(FlameGraphMessage::ZoomIn),
                Key::Esc | Key::Backspace => Some(FlameGraphMessage::ZoomOut),
                Key::Home => Some(FlameGraphMessage::ResetZoom),
                Key::Char('/') => Some(FlameGraphMessage::SetSearch(String::new())),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        render::render_flame_graph(
            state,
            ctx.frame,
            ctx.area,
            ctx.theme,
            ctx.focused,
            ctx.disabled,
        );
    }
}

/// Creates a `FrameSelected` output from the current state.
fn make_selected_output(state: &FlameGraphState) -> Option<FlameGraphOutput> {
    state
        .selected_frame()
        .map(|frame| FlameGraphOutput::FrameSelected {
            label: frame.label.clone(),
            value: frame.total_value(),
            self_value: frame.self_value(),
        })
}

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;
