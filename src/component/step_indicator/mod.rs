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

mod state;

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

        let show_border = !ctx.chrome_owned && state.show_border;
        let inner = if show_border {
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
