//! StepIndicatorState constructors, builders, accessors, setters, and instance methods.
//!
//! Extracted from the main module to keep file sizes manageable.

use ratatui::style::Style;

use super::{
    Step, StepIndicator, StepIndicatorMessage, StepIndicatorOutput, StepIndicatorState,
    StepOrientation, StepStatus,
};
use crate::component::Component;

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
