//! An N-pane layout manager with proportional sizing and focus cycling.
//!
//! [`PaneLayout`] divides a screen area into multiple panes with configurable
//! proportions, min/max size constraints, and keyboard-driven focus management.
//! The parent controls what to render in each pane — this component only manages
//! the layout and focus. State is stored in [`PaneLayoutState`], updated via
//! [`PaneLayoutMessage`], and produces [`PaneLayoutOutput`]. Panes are configured
//! with [`PaneConfig`].
//!
//!
//! See also [`SplitPanel`](super::SplitPanel) for a simpler two-pane layout.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     PaneLayout, PaneLayoutState, PaneLayoutMessage, Component,
//!     pane_layout::{PaneConfig, PaneDirection},
//! };
//! use ratatui::prelude::Rect;
//!
//! let panes = vec![
//!     PaneConfig::new("left").with_proportion(0.3),
//!     PaneConfig::new("center").with_proportion(0.5),
//!     PaneConfig::new("right").with_proportion(0.2),
//! ];
//! let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
//!
//! // Get layout rects for a given area
//! let area = Rect::new(0, 0, 100, 40);
//! let rects = state.layout(area);
//! assert_eq!(rects.len(), 3);
//!
//! // Cycle focus
//! PaneLayout::update(&mut state, PaneLayoutMessage::FocusNext);
//! assert_eq!(state.focused_pane_id(), Some("center"));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};

/// The direction in which panes are arranged.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum PaneDirection {
    /// Panes are arranged left to right.
    Horizontal,
    /// Panes are arranged top to bottom.
    Vertical,
}

/// Configuration for a single pane.
///
/// # Example
///
/// ```rust
/// use envision::component::pane_layout::PaneConfig;
///
/// let pane = PaneConfig::new("sidebar")
///     .with_title("Files")
///     .with_proportion(0.25)
///     .with_min_size(10)
///     .with_max_size(50);
///
/// assert_eq!(pane.id(), "sidebar");
/// assert_eq!(pane.title(), Some("Files"));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct PaneConfig {
    id: String,
    title: Option<String>,
    proportion: f32,
    min_size: u16,
    max_size: u16,
}

impl PaneConfig {
    /// Creates a new pane with default proportion (equal share).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    ///
    /// let pane = PaneConfig::new("sidebar");
    /// assert_eq!(pane.id(), "sidebar");
    /// assert_eq!(pane.title(), None);
    /// ```
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: None,
            proportion: 1.0,
            min_size: 1,
            max_size: 0,
        }
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    ///
    /// let pane = PaneConfig::new("sidebar").with_title("Files");
    /// assert_eq!(pane.title(), Some("Files"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the proportion (builder pattern).
    ///
    /// Proportions are normalized relative to other panes' proportions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    ///
    /// let pane = PaneConfig::new("main").with_proportion(0.7);
    /// assert!((pane.proportion() - 0.7).abs() < f32::EPSILON);
    /// ```
    pub fn with_proportion(mut self, proportion: f32) -> Self {
        self.proportion = proportion.max(0.0);
        self
    }

    /// Sets the minimum size in cells (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    ///
    /// let pane = PaneConfig::new("sidebar").with_min_size(20);
    /// assert_eq!(pane.min_size(), 20);
    /// ```
    pub fn with_min_size(mut self, min_size: u16) -> Self {
        self.min_size = min_size.max(1);
        self
    }

    /// Sets the maximum size in cells (builder pattern).
    ///
    /// A value of 0 means no maximum.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    ///
    /// let pane = PaneConfig::new("sidebar").with_max_size(60);
    /// assert_eq!(pane.max_size(), 60);
    /// ```
    pub fn with_max_size(mut self, max_size: u16) -> Self {
        self.max_size = max_size;
        self
    }

    /// Returns the pane ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the pane title.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the pane title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    ///
    /// let mut pane = PaneConfig::new("sidebar");
    /// pane.set_title("Files");
    /// assert_eq!(pane.title(), Some("Files"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns the pane proportion.
    pub fn proportion(&self) -> f32 {
        self.proportion
    }

    /// Returns the minimum size.
    pub fn min_size(&self) -> u16 {
        self.min_size
    }

    /// Sets the minimum size.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    ///
    /// let mut pane = PaneConfig::new("sidebar");
    /// pane.set_min_size(20);
    /// assert_eq!(pane.min_size(), 20);
    /// ```
    pub fn set_min_size(&mut self, min_size: u16) {
        self.min_size = min_size;
    }

    /// Returns the maximum size.
    pub fn max_size(&self) -> u16 {
        self.max_size
    }

    /// Sets the maximum size.
    ///
    /// A value of 0 means no maximum.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    ///
    /// let mut pane = PaneConfig::new("sidebar");
    /// pane.set_max_size(60);
    /// assert_eq!(pane.max_size(), 60);
    /// ```
    pub fn set_max_size(&mut self, max_size: u16) {
        self.max_size = max_size;
    }

    /// Sets the pane proportion.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::PaneConfig;
    ///
    /// let mut pane = PaneConfig::new("sidebar");
    /// pane.set_proportion(0.3);
    /// assert!((pane.proportion() - 0.3).abs() < f32::EPSILON);
    /// ```
    pub fn set_proportion(&mut self, proportion: f32) {
        self.proportion = proportion;
    }
}

/// Messages that can be sent to a PaneLayout.
#[derive(Clone, Debug, PartialEq)]
pub enum PaneLayoutMessage {
    /// Focus the next pane (wrapping).
    FocusNext,
    /// Focus the previous pane (wrapping).
    FocusPrev,
    /// Focus a pane by ID.
    FocusPane(String),
    /// Focus a pane by index.
    FocusPaneIndex(usize),
    /// Grow the focused pane.
    GrowFocused,
    /// Shrink the focused pane.
    ShrinkFocused,
    /// Grow a specific pane by ID.
    GrowPane(String),
    /// Shrink a specific pane by ID.
    ShrinkPane(String),
    /// Set a specific pane's proportion.
    SetProportion {
        /// The pane ID.
        id: String,
        /// The new proportion.
        proportion: f32,
    },
    /// Reset all panes to equal proportions.
    ResetProportions,
}

/// Output messages from a PaneLayout.
#[derive(Clone, Debug, PartialEq)]
pub enum PaneLayoutOutput {
    /// Focus changed to a different pane.
    FocusChanged {
        /// The pane ID.
        pane_id: String,
        /// The pane index.
        index: usize,
    },
    /// A pane's proportion changed.
    ProportionChanged {
        /// The pane ID.
        pane_id: String,
        /// The new proportion.
        proportion: f32,
    },
    /// All proportions were reset.
    ProportionsReset,
}

/// State for a PaneLayout component.
///
/// Manages the direction, pane configurations, focus, and proportions.
/// The parent is responsible for rendering content into each pane's area.
///
/// # Example
///
/// ```rust
/// use envision::component::pane_layout::{PaneConfig, PaneDirection};
/// use envision::component::PaneLayoutState;
/// use ratatui::prelude::Rect;
///
/// let panes = vec![
///     PaneConfig::new("a").with_proportion(0.5),
///     PaneConfig::new("b").with_proportion(0.5),
/// ];
/// let state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
///
/// let area = Rect::new(0, 0, 80, 24);
/// let rects = state.layout(area);
/// assert_eq!(rects.len(), 2);
/// assert_eq!(rects[0].width, 40);
/// assert_eq!(rects[1].width, 40);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct PaneLayoutState {
    direction: PaneDirection,
    panes: Vec<PaneConfig>,
    focused_pane: usize,
    resize_step: f32,
}

impl Default for PaneLayoutState {
    fn default() -> Self {
        Self {
            direction: PaneDirection::Horizontal,
            panes: Vec::new(),
            focused_pane: 0,
            resize_step: 0.05,
        }
    }
}

impl PaneLayoutState {
    /// Creates a new pane layout with the given direction and panes.
    ///
    /// Pane proportions are automatically normalized to sum to 1.0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("left"),
    ///     PaneConfig::new("right"),
    /// ]);
    /// assert_eq!(state.pane_count(), 2);
    /// assert_eq!(state.focused_pane_id(), Some("left"));
    /// ```
    pub fn new(direction: PaneDirection, panes: Vec<PaneConfig>) -> Self {
        let mut state = Self {
            direction,
            panes,
            ..Self::default()
        };
        state.normalize_proportions();
        state
    }

    /// Sets the resize step (builder pattern).
    ///
    /// The resize step controls how much each grow/shrink operation changes
    /// proportions. Defaults to 0.05 (5%).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("a"),
    ///     PaneConfig::new("b"),
    /// ]).with_resize_step(0.1);
    /// assert!((state.resize_step() - 0.1).abs() < f32::EPSILON);
    /// ```
    pub fn with_resize_step(mut self, step: f32) -> Self {
        self.resize_step = step.clamp(0.01, 0.5);
        self
    }

    // ---- Layout computation ----

    /// Computes the layout rectangles for each pane within the given area.
    ///
    /// Respects min/max size constraints. Returns one `Rect` per pane.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    /// use ratatui::prelude::Rect;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Vertical, vec![
    ///     PaneConfig::new("top").with_proportion(0.5),
    ///     PaneConfig::new("bottom").with_proportion(0.5),
    /// ]);
    /// let rects = state.layout(Rect::new(0, 0, 80, 40));
    /// assert_eq!(rects.len(), 2);
    /// assert_eq!(rects[0].height, 20);
    /// assert_eq!(rects[1].height, 20);
    /// ```
    pub fn layout(&self, area: Rect) -> Vec<Rect> {
        if self.panes.is_empty() {
            return vec![];
        }

        let total = match self.direction {
            PaneDirection::Horizontal => area.width,
            PaneDirection::Vertical => area.height,
        };

        let sizes = self.compute_sizes(total);

        let mut rects = Vec::with_capacity(self.panes.len());
        let mut offset = 0u16;

        for (i, &size) in sizes.iter().enumerate() {
            let rect = match self.direction {
                PaneDirection::Horizontal => Rect::new(
                    area.x + offset,
                    area.y,
                    if i == sizes.len() - 1 {
                        total.saturating_sub(offset)
                    } else {
                        size
                    },
                    area.height,
                ),
                PaneDirection::Vertical => Rect::new(
                    area.x,
                    area.y + offset,
                    area.width,
                    if i == sizes.len() - 1 {
                        total.saturating_sub(offset)
                    } else {
                        size
                    },
                ),
            };
            rects.push(rect);
            offset += size;
        }

        rects
    }

    /// Returns the area for a specific pane by ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    /// use ratatui::prelude::Rect;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("left").with_proportion(0.5),
    ///     PaneConfig::new("right").with_proportion(0.5),
    /// ]);
    /// let area = Rect::new(0, 0, 80, 24);
    /// let left_area = state.pane_area(area, "left").unwrap();
    /// assert_eq!(left_area.width, 40);
    /// ```
    pub fn pane_area(&self, area: Rect, pane_id: &str) -> Option<Rect> {
        let index = self.panes.iter().position(|p| p.id == pane_id)?;
        let rects = self.layout(area);
        rects.into_iter().nth(index)
    }

    // ---- Accessors ----

    /// Returns the direction.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Vertical, vec![
    ///     PaneConfig::new("top"),
    ///     PaneConfig::new("bottom"),
    /// ]);
    /// assert_eq!(state.direction(), &PaneDirection::Vertical);
    /// ```
    pub fn direction(&self) -> &PaneDirection {
        &self.direction
    }

    /// Returns the pane configurations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("left").with_title("Left"),
    ///     PaneConfig::new("right").with_title("Right"),
    /// ]);
    /// assert_eq!(state.panes().len(), 2);
    /// assert_eq!(state.panes()[0].title(), Some("Left"));
    /// ```
    pub fn panes(&self) -> &[PaneConfig] {
        &self.panes
    }

    /// Returns the number of panes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("left"),
    ///     PaneConfig::new("right"),
    /// ]);
    /// assert_eq!(state.pane_count(), 2);
    /// ```
    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }

    /// Returns the focused pane index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("left"),
    ///     PaneConfig::new("right"),
    /// ]);
    /// assert_eq!(state.focused_pane_index(), 0);
    /// ```
    pub fn focused_pane_index(&self) -> usize {
        self.focused_pane
    }

    /// Returns the focused pane ID, if panes exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("left"),
    ///     PaneConfig::new("right"),
    /// ]);
    /// assert_eq!(state.focused_pane_id(), Some("left"));
    /// ```
    pub fn focused_pane_id(&self) -> Option<&str> {
        self.panes.get(self.focused_pane).map(|p| p.id.as_str())
    }

    /// Returns a pane configuration by ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("sidebar").with_title("Files"),
    /// ]);
    /// let pane = state.pane("sidebar").unwrap();
    /// assert_eq!(pane.title(), Some("Files"));
    /// ```
    pub fn pane(&self, id: &str) -> Option<&PaneConfig> {
        self.panes.iter().find(|p| p.id == id)
    }

    /// Returns the resize step.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::PaneLayoutState;
    ///
    /// let state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("a"),
    ///     PaneConfig::new("b"),
    /// ]);
    /// assert!((state.resize_step() - 0.05).abs() < f32::EPSILON); // default
    /// ```
    pub fn resize_step(&self) -> f32 {
        self.resize_step
    }

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::pane_layout::{PaneConfig, PaneDirection};
    /// use envision::component::{PaneLayoutState, PaneLayoutMessage, PaneLayoutOutput};
    ///
    /// let mut state = PaneLayoutState::new(PaneDirection::Horizontal, vec![
    ///     PaneConfig::new("left"),
    ///     PaneConfig::new("right"),
    /// ]);
    /// let output = state.update(PaneLayoutMessage::FocusNext);
    /// assert_eq!(state.focused_pane_id(), Some("right"));
    /// ```
    pub fn update(&mut self, msg: PaneLayoutMessage) -> Option<PaneLayoutOutput> {
        PaneLayout::update(self, msg)
    }

    // ---- Internal ----

    fn normalize_proportions(&mut self) {
        let total: f32 = self.panes.iter().map(|p| p.proportion).sum();
        if total > 0.0 {
            for pane in &mut self.panes {
                pane.proportion /= total;
            }
        }
    }

    fn compute_sizes(&self, total: u16) -> Vec<u16> {
        let n = self.panes.len();
        if n == 0 {
            return vec![];
        }

        let total_f = total as f32;
        let mut sizes: Vec<u16> = self
            .panes
            .iter()
            .map(|p| {
                let raw = (p.proportion * total_f).round() as u16;
                let clamped_min = raw.max(p.min_size);
                if p.max_size > 0 {
                    clamped_min.min(p.max_size)
                } else {
                    clamped_min
                }
            })
            .collect();

        // Adjust to exactly fill the total space
        let computed_total: u16 = sizes.iter().sum();
        if computed_total != total && !sizes.is_empty() {
            let diff = total as i32 - computed_total as i32;
            // Distribute the difference to the last pane
            let last = sizes.len() - 1;
            sizes[last] = (sizes[last] as i32 + diff).max(1) as u16;
        }

        sizes
    }

    fn grow_pane(&mut self, index: usize) -> Option<PaneLayoutOutput> {
        if self.panes.len() < 2 || index >= self.panes.len() {
            return None;
        }

        let step = self.resize_step;
        let min_proportion = 0.05;

        // Find a neighbor to take from
        let neighbor = if index + 1 < self.panes.len() {
            index + 1
        } else {
            index - 1
        };

        if self.panes[neighbor].proportion - step < min_proportion {
            return None;
        }

        self.panes[index].proportion += step;
        self.panes[neighbor].proportion -= step;
        self.normalize_proportions();

        Some(PaneLayoutOutput::ProportionChanged {
            pane_id: self.panes[index].id.clone(),
            proportion: self.panes[index].proportion,
        })
    }

    fn shrink_pane(&mut self, index: usize) -> Option<PaneLayoutOutput> {
        if self.panes.len() < 2 || index >= self.panes.len() {
            return None;
        }

        let step = self.resize_step;
        let min_proportion = 0.05;

        if self.panes[index].proportion - step < min_proportion {
            return None;
        }

        // Find a neighbor to give to
        let neighbor = if index + 1 < self.panes.len() {
            index + 1
        } else {
            index - 1
        };

        self.panes[index].proportion -= step;
        self.panes[neighbor].proportion += step;
        self.normalize_proportions();

        Some(PaneLayoutOutput::ProportionChanged {
            pane_id: self.panes[index].id.clone(),
            proportion: self.panes[index].proportion,
        })
    }
}

/// An N-pane layout manager component.
///
/// `PaneLayout` manages the layout of multiple panes with proportional
/// sizing and focus cycling. The parent renders content into each pane's
/// computed area using [`PaneLayoutState::layout`].
///
/// # Key Bindings
///
/// - `Tab` — Focus next pane
/// - `BackTab` — Focus previous pane
/// - `Ctrl+Right` / `Ctrl+Down` — Grow focused pane
/// - `Ctrl+Left` / `Ctrl+Up` — Shrink focused pane
/// - `Ctrl+0` — Reset proportions
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     PaneLayout, PaneLayoutState, PaneLayoutMessage, Component,
///     pane_layout::{PaneConfig, PaneDirection},
/// };
/// use ratatui::prelude::Rect;
///
/// let panes = vec![
///     PaneConfig::new("left"),
///     PaneConfig::new("right"),
/// ];
/// let mut state = PaneLayoutState::new(PaneDirection::Horizontal, panes);
///
/// let area = Rect::new(0, 0, 80, 24);
/// let rects = state.layout(area);
/// assert_eq!(rects.len(), 2);
/// ```
pub struct PaneLayout;

impl Component for PaneLayout {
    type State = PaneLayoutState;
    type Message = PaneLayoutMessage;
    type Output = PaneLayoutOutput;

    fn init() -> Self::State {
        PaneLayoutState::default()
    }

    fn handle_event(
        _state: &Self::State,
        event: &Event,
        ctx: &EventContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        let key = event.as_key()?;
        let ctrl = key.modifiers.ctrl();

        match key.key {
            Key::Tab if key.modifiers.shift() => Some(PaneLayoutMessage::FocusPrev),
            Key::Tab if !ctrl => Some(PaneLayoutMessage::FocusNext),
            Key::Right | Key::Down if ctrl => Some(PaneLayoutMessage::GrowFocused),
            Key::Left | Key::Up if ctrl => Some(PaneLayoutMessage::ShrinkFocused),
            Key::Char('0') if ctrl => Some(PaneLayoutMessage::ResetProportions),
            _ => None,
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            PaneLayoutMessage::FocusNext => {
                if state.panes.is_empty() {
                    return None;
                }
                state.focused_pane = (state.focused_pane + 1) % state.panes.len();
                Some(PaneLayoutOutput::FocusChanged {
                    pane_id: state.panes[state.focused_pane].id.clone(),
                    index: state.focused_pane,
                })
            }
            PaneLayoutMessage::FocusPrev => {
                if state.panes.is_empty() {
                    return None;
                }
                state.focused_pane = state
                    .focused_pane
                    .checked_sub(1)
                    .unwrap_or(state.panes.len() - 1);
                Some(PaneLayoutOutput::FocusChanged {
                    pane_id: state.panes[state.focused_pane].id.clone(),
                    index: state.focused_pane,
                })
            }
            PaneLayoutMessage::FocusPane(id) => {
                if let Some(index) = state.panes.iter().position(|p| p.id == id) {
                    state.focused_pane = index;
                    Some(PaneLayoutOutput::FocusChanged { pane_id: id, index })
                } else {
                    None
                }
            }
            PaneLayoutMessage::FocusPaneIndex(index) => {
                if index >= state.panes.len() {
                    return None;
                }
                state.focused_pane = index;
                Some(PaneLayoutOutput::FocusChanged {
                    pane_id: state.panes[index].id.clone(),
                    index,
                })
            }
            PaneLayoutMessage::GrowFocused => {
                let index = state.focused_pane;
                state.grow_pane(index)
            }
            PaneLayoutMessage::ShrinkFocused => {
                let index = state.focused_pane;
                state.shrink_pane(index)
            }
            PaneLayoutMessage::GrowPane(id) => {
                if let Some(index) = state.panes.iter().position(|p| p.id == id) {
                    state.grow_pane(index)
                } else {
                    None
                }
            }
            PaneLayoutMessage::ShrinkPane(id) => {
                if let Some(index) = state.panes.iter().position(|p| p.id == id) {
                    state.shrink_pane(index)
                } else {
                    None
                }
            }
            PaneLayoutMessage::SetProportion { id, proportion } => {
                if let Some(index) = state.panes.iter().position(|p| p.id == id) {
                    state.panes[index].proportion = proportion.max(0.0);
                    state.normalize_proportions();
                    Some(PaneLayoutOutput::ProportionChanged {
                        pane_id: id,
                        proportion: state.panes[index].proportion,
                    })
                } else {
                    None
                }
            }
            PaneLayoutMessage::ResetProportions => {
                if state.panes.is_empty() {
                    return None;
                }
                let equal = 1.0 / state.panes.len() as f32;
                for pane in &mut state.panes {
                    pane.proportion = equal;
                }
                Some(PaneLayoutOutput::ProportionsReset)
            }
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::PaneLayout)
                    .with_id("pane_layout")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let rects = state.layout(ctx.area);

        for (i, (pane, rect)) in state.panes.iter().zip(rects.iter()).enumerate() {
            let is_focused_pane = ctx.focused && i == state.focused_pane;
            let border_style = if ctx.disabled {
                ctx.theme.disabled_style()
            } else if is_focused_pane {
                ctx.theme.focused_border_style()
            } else {
                ctx.theme.border_style()
            };

            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style);

            if let Some(title) = &pane.title {
                block = block.title(format!(" {} ", title));
            }

            ctx.frame.render_widget(block, *rect);
        }
    }
}

#[cfg(test)]
mod tests;
