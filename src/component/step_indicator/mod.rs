//! A pipeline/workflow visualization component showing step-by-step progress.
//!
//! [`StepIndicator`] displays a series of steps with their current status,
//! connected by configurable connectors. Supports both horizontal and vertical
//! orientations. State is stored in [`StepIndicatorState`], updated via
//! [`StepIndicatorMessage`], and produces [`StepIndicatorOutput`]. Steps are
//! defined with [`Step`] and have a [`StepStatus`].
//!
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     StepIndicator, StepIndicatorMessage, StepIndicatorOutput,
//!     StepIndicatorState, Component,
//!     step_indicator::{Step, StepStatus},
//! };
//!
//! let steps = vec![
//!     Step::new("Build").with_status(StepStatus::Completed),
//!     Step::new("Test").with_status(StepStatus::Active),
//!     Step::new("Deploy").with_status(StepStatus::Pending),
//! ];
//! let mut state = StepIndicatorState::new(steps);
//!
//! // Complete the active step and activate the next
//! StepIndicator::update(&mut state, StepIndicatorMessage::CompleteActive);
//! StepIndicator::update(&mut state, StepIndicatorMessage::ActivateNext);
//! ```

use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, EventContext, RenderContext};
use crate::input::{Event, Key};
use crate::theme::Theme;

/// The status of a single step in a workflow.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum StepStatus {
    /// Step has not started yet.
    Pending,
    /// Step is currently in progress.
    Active,
    /// Step completed successfully.
    Completed,
    /// Step failed.
    Failed,
    /// Step was skipped.
    Skipped,
}

impl StepStatus {
    fn icon(&self) -> &'static str {
        match self {
            StepStatus::Pending => "○",
            StepStatus::Active => "●",
            StepStatus::Completed => "✓",
            StepStatus::Failed => "✗",
            StepStatus::Skipped => "⊘",
        }
    }
}

/// The orientation of the step indicator layout.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum StepOrientation {
    /// Steps are displayed left to right.
    Horizontal,
    /// Steps are displayed top to bottom.
    Vertical,
}

/// A single step in a workflow.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Step {
    label: String,
    status: StepStatus,
    description: Option<String>,
}

impl Step {
    /// Creates a new pending step.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    ///
    /// let step = Step::new("Build");
    /// assert_eq!(step.label(), "Build");
    /// ```
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            status: StepStatus::Pending,
            description: None,
        }
    }

    /// Sets the step status (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepStatus};
    ///
    /// let step = Step::new("Build").with_status(StepStatus::Completed);
    /// assert_eq!(step.status(), &StepStatus::Completed);
    /// ```
    pub fn with_status(mut self, status: StepStatus) -> Self {
        self.status = status;
        self
    }

    /// Sets the step description (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    ///
    /// let step = Step::new("Test").with_description("Run unit tests");
    /// assert_eq!(step.description(), Some("Run unit tests"));
    /// ```
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Returns the step label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    ///
    /// let step = Step::new("Build");
    /// assert_eq!(step.label(), "Build");
    /// ```
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the step status.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepStatus};
    ///
    /// let step = Step::new("Test");
    /// assert_eq!(step.status(), &StepStatus::Pending);
    /// ```
    pub fn status(&self) -> &StepStatus {
        &self.status
    }

    /// Returns the step description, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    ///
    /// let step = Step::new("Deploy");
    /// assert_eq!(step.description(), None);
    /// ```
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// Messages that can be sent to a StepIndicator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepIndicatorMessage {
    /// Set the status of a specific step.
    SetStatus {
        /// The step index.
        index: usize,
        /// The new status.
        status: StepStatus,
    },
    /// Activate the next pending step after the current active step.
    ActivateNext,
    /// Complete the currently active step.
    CompleteActive,
    /// Fail the currently active step.
    FailActive,
    /// Skip a specific step.
    Skip(usize),
    /// Reset all steps to pending.
    Reset,
    /// Move keyboard focus to the next step.
    FocusNext,
    /// Move keyboard focus to the previous step.
    FocusPrev,
    /// Select the currently focused step.
    Select,
    /// Jump focus to the first step.
    First,
    /// Jump focus to the last step.
    Last,
}

/// Output messages from a StepIndicator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepIndicatorOutput {
    /// A step's status changed.
    StatusChanged {
        /// The step index.
        index: usize,
        /// The new status.
        status: StepStatus,
    },
    /// All steps are completed.
    AllCompleted,
    /// A step was selected via keyboard.
    Selected(usize),
    /// Focus moved to a different step.
    FocusChanged(usize),
    /// All steps were reset.
    Reset,
}

/// State for a StepIndicator component.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct StepIndicatorState {
    steps: Vec<Step>,
    orientation: StepOrientation,
    focused_index: usize,
    show_descriptions: bool,
    title: Option<String>,
    connector: String,
    show_border: bool,
    /// Per-status style overrides. When set, these take precedence over
    /// the default theme-based styles.
    #[cfg_attr(feature = "serialization", serde(skip, default))]
    status_style_overrides: HashMap<StepStatus, Style>,
    /// Per-step-index style overrides. When set, these take precedence
    /// over both status-based overrides and the default theme styles.
    /// Use this to give specific steps (e.g., "intake", "review") a
    /// fixed color regardless of their current status.
    #[cfg_attr(feature = "serialization", serde(skip, default))]
    step_style_overrides: HashMap<usize, Style>,
}

impl Default for StepIndicatorState {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            orientation: StepOrientation::Horizontal,
            focused_index: 0,
            show_descriptions: false,
            title: None,
            connector: "───".to_string(),
            show_border: true,
            status_style_overrides: HashMap::new(),
            step_style_overrides: HashMap::new(),
        }
    }
}

impl StepIndicatorState {
    /// Creates a new step indicator with the given steps.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepStatus};
    /// use envision::component::StepIndicatorState;
    ///
    /// let steps = vec![
    ///     Step::new("Step 1").with_status(StepStatus::Completed),
    ///     Step::new("Step 2").with_status(StepStatus::Active),
    ///     Step::new("Step 3"),
    /// ];
    /// let state = StepIndicatorState::new(steps);
    /// assert_eq!(state.steps().len(), 3);
    /// ```
    pub fn new(steps: Vec<Step>) -> Self {
        Self {
            steps,
            ..Self::default()
        }
    }

    /// Sets the orientation (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepOrientation};
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")])
    ///     .with_orientation(StepOrientation::Vertical);
    /// assert_eq!(state.orientation(), &StepOrientation::Vertical);
    /// ```
    pub fn with_orientation(mut self, orientation: StepOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")])
    ///     .with_title("Pipeline");
    /// assert_eq!(state.title(), Some("Pipeline"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the connector string (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")])
    ///     .with_connector("-->");
    /// assert_eq!(state.connector(), "-->");
    /// ```
    pub fn with_connector(mut self, connector: impl Into<String>) -> Self {
        self.connector = connector.into();
        self
    }

    /// Sets whether descriptions are shown (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")])
    ///     .with_show_descriptions(true);
    /// assert!(state.show_descriptions());
    /// ```
    pub fn with_show_descriptions(mut self, show: bool) -> Self {
        self.show_descriptions = show;
        self
    }

    /// Sets whether the border is shown (builder pattern).
    ///
    /// Defaults to `true`. When set to `false`, the `StepIndicator` renders
    /// its steps directly into the full widget area with no surrounding
    /// box — useful for inline breadcrumbs and single-row layouts.
    ///
    /// # Title interaction
    ///
    /// When the border is hidden, the state's [`title`](Self::title) is
    /// **not rendered**. The title is drawn as part of the border block,
    /// so disabling the border silently suppresses it. If you want this
    /// to be explicit, set the title to `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")])
    ///     .with_show_border(false);
    /// assert!(!state.show_border());
    /// ```
    pub fn with_show_border(mut self, show: bool) -> Self {
        self.show_border = show;
        self
    }

    /// Sets a style override for a specific step status (builder pattern).
    ///
    /// When set, this style is used instead of the default theme-based
    /// style for steps with the given status.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepStatus};
    /// use envision::component::StepIndicatorState;
    /// use ratatui::style::{Color, Style};
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("Build")])
    ///     .with_status_style(StepStatus::Completed, Style::default().fg(Color::Cyan))
    ///     .with_status_style(StepStatus::Failed, Style::default().fg(Color::Red));
    /// assert!(state.status_style_override(&StepStatus::Completed).is_some());
    /// ```
    pub fn with_status_style(mut self, status: StepStatus, style: Style) -> Self {
        self.status_style_overrides.insert(status, style);
        self
    }

    /// Sets a style override for a specific step by index (builder pattern).
    ///
    /// When set, this style is used for the step at the given index
    /// regardless of its current status. Per-index overrides take
    /// precedence over per-status overrides.
    ///
    /// Use this to give specific steps a fixed color (e.g., "intake"
    /// is always Cyan, "review" is always Yellow) regardless of whether
    /// they are pending, active, or completed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    /// use ratatui::style::{Color, Style};
    ///
    /// let state = StepIndicatorState::new(vec![
    ///     Step::new("Intake"),
    ///     Step::new("Review"),
    ///     Step::new("Approve"),
    /// ])
    /// .with_step_style(0, Style::default().fg(Color::Cyan))
    /// .with_step_style(1, Style::default().fg(Color::Yellow));
    /// assert!(state.step_style_override(0).is_some());
    /// ```
    pub fn with_step_style(mut self, index: usize, style: Style) -> Self {
        self.step_style_overrides.insert(index, style);
        self
    }

    /// Returns the steps.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("Build"), Step::new("Test")]);
    /// assert_eq!(state.steps().len(), 2);
    /// ```
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Returns a specific step, if it exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("Build"), Step::new("Test")]);
    /// assert_eq!(state.step(0).unwrap().label(), "Build");
    /// assert!(state.step(99).is_none());
    /// ```
    pub fn step(&self, index: usize) -> Option<&Step> {
        self.steps.get(index)
    }

    /// Returns the orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepOrientation};
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")]);
    /// assert_eq!(state.orientation(), &StepOrientation::Horizontal);
    /// ```
    pub fn orientation(&self) -> &StepOrientation {
        &self.orientation
    }

    /// Returns the focused step index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    /// assert_eq!(state.focused_index(), 0);
    /// ```
    pub fn focused_index(&self) -> usize {
        self.focused_index
    }

    /// Returns the index of the currently active step, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepStatus};
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![
    ///     Step::new("Build").with_status(StepStatus::Completed),
    ///     Step::new("Test").with_status(StepStatus::Active),
    ///     Step::new("Deploy"),
    /// ]);
    /// assert_eq!(state.active_step_index(), Some(1));
    /// ```
    pub fn active_step_index(&self) -> Option<usize> {
        self.steps
            .iter()
            .position(|s| s.status == StepStatus::Active)
    }

    /// Returns true if all steps are completed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepStatus};
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![
    ///     Step::new("Build").with_status(StepStatus::Completed),
    ///     Step::new("Test").with_status(StepStatus::Completed),
    /// ]);
    /// assert!(state.is_all_completed());
    /// ```
    pub fn is_all_completed(&self) -> bool {
        !self.steps.is_empty()
            && self
                .steps
                .iter()
                .all(|s| s.status == StepStatus::Completed || s.status == StepStatus::Skipped)
    }

    /// Returns the connector string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")]).with_connector("→");
    /// assert_eq!(state.connector(), "→");
    /// ```
    pub fn connector(&self) -> &str {
        &self.connector
    }

    /// Returns the title, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")]);
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
    /// use envision::component::StepIndicatorState;
    /// use envision::component::step_indicator::Step;
    ///
    /// let mut state = StepIndicatorState::new(vec![Step::new("Step 1")]);
    /// state.set_title("Progress");
    /// assert_eq!(state.title(), Some("Progress"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Returns whether descriptions are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::default();
    /// assert!(!state.show_descriptions());
    /// ```
    pub fn show_descriptions(&self) -> bool {
        self.show_descriptions
    }

    /// Returns whether the border is shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StepIndicatorState;
    ///
    /// let state = StepIndicatorState::default();
    /// assert!(state.show_border());
    /// ```
    pub fn show_border(&self) -> bool {
        self.show_border
    }

    /// Sets whether descriptions are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StepIndicatorState;
    /// use envision::component::step_indicator::Step;
    ///
    /// let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    /// state.set_show_descriptions(true);
    /// assert!(state.show_descriptions());
    /// ```
    pub fn set_show_descriptions(&mut self, show: bool) {
        self.show_descriptions = show;
    }

    /// Sets the orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StepIndicatorState;
    /// use envision::component::step_indicator::{Step, StepOrientation};
    ///
    /// let mut state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")]);
    /// state.set_orientation(StepOrientation::Vertical);
    /// assert_eq!(state.orientation(), &StepOrientation::Vertical);
    /// ```
    pub fn set_orientation(&mut self, orientation: StepOrientation) {
        self.orientation = orientation;
    }

    /// Sets whether the border is shown.
    ///
    /// See [`with_show_border`](Self::with_show_border) for the title
    /// interaction when `show` is `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::StepIndicatorState;
    /// use envision::component::step_indicator::Step;
    ///
    /// let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    /// state.set_show_border(false);
    /// assert!(!state.show_border());
    /// ```
    pub fn set_show_border(&mut self, show: bool) {
        self.show_border = show;
    }

    /// Returns the per-status style override, if one is set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepStatus};
    /// use envision::component::StepIndicatorState;
    /// use ratatui::style::{Color, Style};
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A")])
    ///     .with_status_style(StepStatus::Active, Style::default().fg(Color::Yellow));
    /// assert!(state.status_style_override(&StepStatus::Active).is_some());
    /// assert!(state.status_style_override(&StepStatus::Pending).is_none());
    /// ```
    pub fn status_style_override(&self, status: &StepStatus) -> Option<&Style> {
        self.status_style_overrides.get(status)
    }

    /// Sets a per-status style override.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepStatus};
    /// use envision::component::StepIndicatorState;
    /// use ratatui::style::{Color, Style};
    ///
    /// let mut state = StepIndicatorState::new(vec![Step::new("A")]);
    /// state.set_status_style(StepStatus::Active, Style::default().fg(Color::Yellow));
    /// assert!(state.status_style_override(&StepStatus::Active).is_some());
    /// ```
    pub fn set_status_style(&mut self, status: StepStatus, style: Style) {
        self.status_style_overrides.insert(status, style);
    }

    /// Removes a per-status style override.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::{Step, StepStatus};
    /// use envision::component::StepIndicatorState;
    /// use ratatui::style::{Color, Style};
    ///
    /// let mut state = StepIndicatorState::new(vec![Step::new("A")])
    ///     .with_status_style(StepStatus::Active, Style::default().fg(Color::Yellow));
    /// state.clear_status_style(&StepStatus::Active);
    /// assert!(state.status_style_override(&StepStatus::Active).is_none());
    /// ```
    pub fn clear_status_style(&mut self, status: &StepStatus) {
        self.status_style_overrides.remove(status);
    }

    /// Returns the per-step-index style override, if one is set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    /// use ratatui::style::{Color, Style};
    ///
    /// let state = StepIndicatorState::new(vec![Step::new("A"), Step::new("B")])
    ///     .with_step_style(0, Style::default().fg(Color::Cyan));
    /// assert!(state.step_style_override(0).is_some());
    /// assert!(state.step_style_override(1).is_none());
    /// ```
    pub fn step_style_override(&self, index: usize) -> Option<&Style> {
        self.step_style_overrides.get(&index)
    }

    /// Sets a per-step-index style override.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    /// use ratatui::style::{Color, Style};
    ///
    /// let mut state = StepIndicatorState::new(vec![Step::new("Intake"), Step::new("Review")]);
    /// state.set_step_style(0, Style::default().fg(Color::Cyan));
    /// assert!(state.step_style_override(0).is_some());
    /// ```
    pub fn set_step_style(&mut self, index: usize, style: Style) {
        self.step_style_overrides.insert(index, style);
    }

    /// Removes a per-step-index style override.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::step_indicator::Step;
    /// use envision::component::StepIndicatorState;
    /// use ratatui::style::{Color, Style};
    ///
    /// let mut state = StepIndicatorState::new(vec![Step::new("A")])
    ///     .with_step_style(0, Style::default().fg(Color::Cyan));
    /// state.clear_step_style(0);
    /// assert!(state.step_style_override(0).is_none());
    /// ```
    pub fn clear_step_style(&mut self, index: usize) {
        self.step_style_overrides.remove(&index);
    }

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{StepIndicatorState, StepIndicatorMessage, StepIndicatorOutput};
    /// use envision::component::step_indicator::{Step, StepStatus};
    ///
    /// let steps = vec![
    ///     Step::new("Build").with_status(StepStatus::Active),
    ///     Step::new("Test"),
    /// ];
    /// let mut state = StepIndicatorState::new(steps);
    /// let output = state.update(StepIndicatorMessage::CompleteActive);
    /// assert!(matches!(output, Some(StepIndicatorOutput::StatusChanged { .. })));
    /// ```
    pub fn update(&mut self, msg: StepIndicatorMessage) -> Option<StepIndicatorOutput> {
        StepIndicator::update(self, msg)
    }
}

/// A pipeline/workflow visualization component.
///
/// `StepIndicator` displays a series of steps connected by configurable
/// connectors, with status icons and color-coded styling.
///
/// # Visual Format (Horizontal)
///
/// ```text
/// ✓ Build ─── ● Test ─── ○ Deploy
/// ```
///
/// # Visual Format (Vertical)
///
/// ```text
/// ✓ Build
/// │
/// ● Test
/// │
/// ○ Deploy
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     StepIndicator, StepIndicatorMessage, StepIndicatorState, Component,
///     step_indicator::{Step, StepStatus},
/// };
///
/// let steps = vec![
///     Step::new("Build").with_status(StepStatus::Completed),
///     Step::new("Test").with_status(StepStatus::Active),
///     Step::new("Deploy"),
/// ];
/// let mut state = StepIndicatorState::new(steps);
/// StepIndicator::update(&mut state, StepIndicatorMessage::CompleteActive);
/// ```
pub struct StepIndicator;

impl Component for StepIndicator {
    type State = StepIndicatorState;
    type Message = StepIndicatorMessage;
    type Output = StepIndicatorOutput;

    fn init() -> Self::State {
        StepIndicatorState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            StepIndicatorMessage::SetStatus { index, status } => {
                if let Some(step) = state.steps.get_mut(index) {
                    step.status = status.clone();
                    let output = StepIndicatorOutput::StatusChanged { index, status };
                    if state.is_all_completed() {
                        return Some(StepIndicatorOutput::AllCompleted);
                    }
                    return Some(output);
                }
                None
            }
            StepIndicatorMessage::ActivateNext => {
                // Find the first pending step after the last completed/active
                let next = state
                    .steps
                    .iter()
                    .position(|s| s.status == StepStatus::Pending);
                if let Some(index) = next {
                    state.steps[index].status = StepStatus::Active;
                    Some(StepIndicatorOutput::StatusChanged {
                        index,
                        status: StepStatus::Active,
                    })
                } else {
                    None
                }
            }
            StepIndicatorMessage::CompleteActive => {
                if let Some(index) = state.active_step_index() {
                    state.steps[index].status = StepStatus::Completed;
                    if state.is_all_completed() {
                        return Some(StepIndicatorOutput::AllCompleted);
                    }
                    Some(StepIndicatorOutput::StatusChanged {
                        index,
                        status: StepStatus::Completed,
                    })
                } else {
                    None
                }
            }
            StepIndicatorMessage::FailActive => {
                if let Some(index) = state.active_step_index() {
                    state.steps[index].status = StepStatus::Failed;
                    Some(StepIndicatorOutput::StatusChanged {
                        index,
                        status: StepStatus::Failed,
                    })
                } else {
                    None
                }
            }
            StepIndicatorMessage::Skip(index) => {
                if let Some(step) = state.steps.get_mut(index) {
                    step.status = StepStatus::Skipped;
                    if state.is_all_completed() {
                        return Some(StepIndicatorOutput::AllCompleted);
                    }
                    Some(StepIndicatorOutput::StatusChanged {
                        index,
                        status: StepStatus::Skipped,
                    })
                } else {
                    None
                }
            }
            StepIndicatorMessage::Reset => {
                for step in &mut state.steps {
                    step.status = StepStatus::Pending;
                }
                state.focused_index = 0;
                Some(StepIndicatorOutput::Reset)
            }
            StepIndicatorMessage::FocusNext => {
                if state.steps.is_empty() {
                    return None;
                }
                state.focused_index = (state.focused_index + 1) % state.steps.len();
                Some(StepIndicatorOutput::FocusChanged(state.focused_index))
            }
            StepIndicatorMessage::FocusPrev => {
                if state.steps.is_empty() {
                    return None;
                }
                state.focused_index = state
                    .focused_index
                    .checked_sub(1)
                    .unwrap_or(state.steps.len() - 1);
                Some(StepIndicatorOutput::FocusChanged(state.focused_index))
            }
            StepIndicatorMessage::Select => {
                if state.steps.is_empty() {
                    return None;
                }
                Some(StepIndicatorOutput::Selected(state.focused_index))
            }
            StepIndicatorMessage::First => {
                if state.steps.is_empty() {
                    return None;
                }
                state.focused_index = 0;
                Some(StepIndicatorOutput::FocusChanged(0))
            }
            StepIndicatorMessage::Last => {
                if state.steps.is_empty() {
                    return None;
                }
                state.focused_index = state.steps.len() - 1;
                Some(StepIndicatorOutput::FocusChanged(state.focused_index))
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
            match key.code {
                Key::Left | Key::Char('h') => Some(StepIndicatorMessage::FocusPrev),
                Key::Right | Key::Char('l') => Some(StepIndicatorMessage::FocusNext),
                Key::Home => Some(StepIndicatorMessage::First),
                Key::End => Some(StepIndicatorMessage::Last),
                Key::Enter => Some(StepIndicatorMessage::Select),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::StepIndicator)
                    .with_id("step_indicator")
                    .with_focus(ctx.focused)
                    .with_disabled(ctx.disabled),
            );
        });

        let inner = if state.show_border {
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_style(if ctx.focused {
                    ctx.theme.focused_border_style()
                } else {
                    ctx.theme.border_style()
                });
            if let Some(title) = &state.title {
                block = block.title(format!(" {} ", title));
            }
            let inner = block.inner(ctx.area);
            ctx.frame.render_widget(block, ctx.area);
            inner
        } else {
            ctx.area
        };

        if state.steps.is_empty() {
            return;
        }

        match state.orientation {
            StepOrientation::Horizontal => {
                render_horizontal(state, ctx.frame, inner, ctx.theme, ctx.focused);
            }
            StepOrientation::Vertical => {
                render_vertical(state, ctx.frame, inner, ctx.theme, ctx.focused);
            }
        }
    }
}

fn step_style(
    index: usize,
    status: &StepStatus,
    is_focused_step: bool,
    theme: &Theme,
    step_overrides: &HashMap<usize, Style>,
    status_overrides: &HashMap<StepStatus, Style>,
) -> Style {
    // Priority: per-step-index > per-status > theme default
    let base = step_overrides
        .get(&index)
        .or_else(|| status_overrides.get(status))
        .copied()
        .unwrap_or_else(|| match status {
            StepStatus::Completed => theme.success_style(),
            StepStatus::Active => theme.info_style(),
            StepStatus::Failed => theme.error_style(),
            StepStatus::Skipped => theme.disabled_style(),
            StepStatus::Pending => theme.normal_style(),
        });
    if is_focused_step {
        base.add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        base
    }
}

fn render_horizontal(
    state: &StepIndicatorState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
) {
    let mut spans = Vec::new();

    for (i, step) in state.steps.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(
                format!(" {} ", state.connector),
                theme.normal_style(),
            ));
        }

        let is_focused_step = focused && i == state.focused_index;
        let style = step_style(
            i,
            &step.status,
            is_focused_step,
            theme,
            &state.step_style_overrides,
            &state.status_style_overrides,
        );

        spans.push(Span::styled(
            format!("{} {}", step.status.icon(), step.label),
            style,
        ));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn render_vertical(
    state: &StepIndicatorState,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
) {
    let mut lines = Vec::new();

    for (i, step) in state.steps.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(Span::styled("│", theme.normal_style())));
        }

        let is_focused_step = focused && i == state.focused_index;
        let style = step_style(
            i,
            &step.status,
            is_focused_step,
            theme,
            &state.step_style_overrides,
            &state.status_style_overrides,
        );

        lines.push(Line::from(Span::styled(
            format!("{} {}", step.status.icon(), step.label),
            style,
        )));

        if state.show_descriptions {
            if let Some(desc) = &step.description {
                lines.push(Line::from(Span::styled(
                    format!("  {}", desc),
                    theme.normal_style(),
                )));
            }
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests;
