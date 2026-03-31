//! A resizable split panel layout component.
//!
//! [`SplitPanel`] divides an area into two panes (horizontal or vertical)
//! with a draggable split ratio. The parent controls what to render in
//! each pane — this component only manages the layout and focus. State is
//! stored in [`SplitPanelState`], updated via [`SplitPanelMessage`], and
//! produces [`SplitPanelOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! See also [`PaneLayout`](super::PaneLayout) for N-pane layouts.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{
//!     Component, Focusable, SplitPanel, SplitPanelState,
//!     SplitPanelMessage, SplitOrientation,
//! };
//!
//! let mut state = SplitPanelState::new(SplitOrientation::Vertical);
//! SplitPanel::set_focused(&mut state, true);
//!
//! assert_eq!(state.ratio(), 0.5);
//! assert!(state.is_first_pane_focused());
//!
//! // Resize: shift split 10% to the right
//! SplitPanel::update(&mut state, SplitPanelMessage::GrowFirst);
//! assert!((state.ratio() - 0.6).abs() < f32::EPSILON);
//!
//! // Toggle focus to the second pane
//! SplitPanel::update(&mut state, SplitPanelMessage::FocusOther);
//! assert!(state.is_second_pane_focused());
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use super::{Component, Disableable, Focusable};
use crate::input::{Event, KeyCode, KeyModifiers};
use crate::theme::Theme;

/// The orientation of a split panel.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum SplitOrientation {
    /// Panes are arranged left-right (vertical divider).
    Vertical,
    /// Panes are arranged top-bottom (horizontal divider).
    Horizontal,
}

/// Messages that can be sent to a SplitPanel.
#[derive(Clone, Debug, PartialEq)]
pub enum SplitPanelMessage {
    /// Toggle focus between the two panes.
    FocusOther,
    /// Focus the first pane.
    FocusFirst,
    /// Focus the second pane.
    FocusSecond,
    /// Increase the first pane's share by the resize step.
    GrowFirst,
    /// Decrease the first pane's share by the resize step.
    ShrinkFirst,
    /// Set the split ratio directly (0.0 to 1.0).
    SetRatio(f32),
    /// Reset the split to 50/50.
    ResetRatio,
}

/// Output messages from a SplitPanel.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SplitPanelOutput {
    /// Focus changed to the first pane.
    FocusedFirst,
    /// Focus changed to the second pane.
    FocusedSecond,
    /// The split ratio changed.
    RatioChanged(f32),
}

/// Identifies which pane has focus.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
enum Pane {
    First,
    Second,
}

/// State for a SplitPanel component.
///
/// Manages the split ratio, orientation, and which pane has focus.
/// The parent is responsible for rendering content into each pane.
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SplitPanelState {
    /// The orientation of the split.
    orientation: SplitOrientation,
    /// Split ratio: 0.0 = all second pane, 1.0 = all first pane.
    ratio: f32,
    /// Which pane currently has focus.
    focused_pane: Pane,
    /// Whether the overall component is focused.
    focused: bool,
    /// Whether the component is disabled.
    disabled: bool,
    /// How much the ratio changes per resize step.
    resize_step: f32,
    /// Minimum ratio (prevents collapsing first pane).
    min_ratio: f32,
    /// Maximum ratio (prevents collapsing second pane).
    max_ratio: f32,
}

impl PartialEq for SplitPanelState {
    fn eq(&self, other: &Self) -> bool {
        self.orientation == other.orientation
            && (self.ratio - other.ratio).abs() < f32::EPSILON
            && self.focused_pane == other.focused_pane
            && self.focused == other.focused
            && self.disabled == other.disabled
            && (self.resize_step - other.resize_step).abs() < f32::EPSILON
            && (self.min_ratio - other.min_ratio).abs() < f32::EPSILON
            && (self.max_ratio - other.max_ratio).abs() < f32::EPSILON
    }
}

impl Default for SplitPanelState {
    fn default() -> Self {
        Self {
            orientation: SplitOrientation::Vertical,
            ratio: 0.5,
            focused_pane: Pane::First,
            focused: false,
            disabled: false,
            resize_step: 0.1,
            min_ratio: 0.1,
            max_ratio: 0.9,
        }
    }
}

impl SplitPanelState {
    /// Creates a new split panel with the given orientation and a 50/50 split.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SplitPanelState, SplitOrientation};
    ///
    /// let state = SplitPanelState::new(SplitOrientation::Horizontal);
    /// assert_eq!(state.ratio(), 0.5);
    /// assert_eq!(state.orientation(), &SplitOrientation::Horizontal);
    /// ```
    pub fn new(orientation: SplitOrientation) -> Self {
        Self {
            orientation,
            ..Default::default()
        }
    }

    /// Creates a split panel with a custom ratio.
    ///
    /// The ratio is clamped to `[min_ratio, max_ratio]`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SplitPanelState, SplitOrientation};
    ///
    /// let state = SplitPanelState::with_ratio(SplitOrientation::Vertical, 0.3);
    /// assert!((state.ratio() - 0.3).abs() < f32::EPSILON);
    /// ```
    pub fn with_ratio(orientation: SplitOrientation, ratio: f32) -> Self {
        let mut state = Self::new(orientation);
        state.ratio = ratio.clamp(state.min_ratio, state.max_ratio);
        state
    }

    /// Returns the current orientation.
    pub fn orientation(&self) -> &SplitOrientation {
        &self.orientation
    }

    /// Sets the orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SplitPanelState, SplitOrientation};
    ///
    /// let mut state = SplitPanelState::new(SplitOrientation::Vertical);
    /// state.set_orientation(SplitOrientation::Horizontal);
    /// assert_eq!(state.orientation(), &SplitOrientation::Horizontal);
    /// ```
    pub fn set_orientation(&mut self, orientation: SplitOrientation) {
        self.orientation = orientation;
    }

    /// Returns the current split ratio.
    ///
    /// 0.5 means equal split. Values closer to 1.0 give more space
    /// to the first pane.
    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    /// Sets the split ratio, clamped to `[min_ratio, max_ratio]`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SplitPanelState, SplitOrientation};
    ///
    /// let mut state = SplitPanelState::new(SplitOrientation::Vertical);
    /// state.set_ratio(0.7);
    /// assert!((state.ratio() - 0.7).abs() < f32::EPSILON);
    /// ```
    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(self.min_ratio, self.max_ratio);
    }

    /// Returns true if the first pane has focus.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SplitPanelState, SplitOrientation};
    ///
    /// let state = SplitPanelState::new(SplitOrientation::Vertical);
    /// assert!(state.is_first_pane_focused());
    /// assert!(!state.is_second_pane_focused());
    /// ```
    pub fn is_first_pane_focused(&self) -> bool {
        self.focused_pane == Pane::First
    }

    /// Returns true if the second pane has focus.
    pub fn is_second_pane_focused(&self) -> bool {
        self.focused_pane == Pane::Second
    }

    /// Returns the resize step size (default 0.1 = 10%).
    pub fn resize_step(&self) -> f32 {
        self.resize_step
    }

    /// Sets the resize step size.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SplitPanelState, SplitOrientation};
    ///
    /// let state = SplitPanelState::new(SplitOrientation::Vertical)
    ///     .with_resize_step(0.05);
    /// assert!((state.resize_step() - 0.05).abs() < f32::EPSILON);
    /// ```
    pub fn with_resize_step(mut self, step: f32) -> Self {
        self.resize_step = step;
        self
    }

    /// Sets the minimum and maximum ratio bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SplitPanelState, SplitOrientation};
    ///
    /// let state = SplitPanelState::new(SplitOrientation::Vertical)
    ///     .with_bounds(0.2, 0.8);
    /// ```
    pub fn with_bounds(mut self, min: f32, max: f32) -> Self {
        self.min_ratio = min;
        self.max_ratio = max;
        self.ratio = self.ratio.clamp(min, max);
        self
    }

    /// Returns true if the component is focused.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns true if the component is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the disabled state (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SplitPanelState, SplitOrientation};
    ///
    /// let state = SplitPanelState::new(SplitOrientation::Vertical).with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Maps an input event to a split panel message.
    pub fn handle_event(&self, event: &Event) -> Option<SplitPanelMessage> {
        SplitPanel::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    pub fn dispatch_event(&mut self, event: &Event) -> Option<SplitPanelOutput> {
        SplitPanel::dispatch_event(self, event)
    }

    /// Updates the state with a message, returning any output.
    pub fn update(&mut self, msg: SplitPanelMessage) -> Option<SplitPanelOutput> {
        SplitPanel::update(self, msg)
    }

    /// Computes the layout areas for the two panes.
    ///
    /// Returns `(first_pane_area, second_pane_area)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SplitPanelState, SplitOrientation};
    /// use ratatui::prelude::Rect;
    ///
    /// let state = SplitPanelState::new(SplitOrientation::Vertical);
    /// let area = Rect::new(0, 0, 80, 24);
    /// let (first, second) = state.layout(area);
    /// assert_eq!(first.width + second.width, 80);
    /// ```
    pub fn layout(&self, area: Rect) -> (Rect, Rect) {
        let direction = match self.orientation {
            SplitOrientation::Vertical => Direction::Horizontal,
            SplitOrientation::Horizontal => Direction::Vertical,
        };

        let total = match self.orientation {
            SplitOrientation::Vertical => area.width,
            SplitOrientation::Horizontal => area.height,
        };

        let first_size = ((total as f32) * self.ratio).round() as u16;
        let first_size = first_size.min(total);

        let chunks = Layout::default()
            .direction(direction)
            .constraints([Constraint::Length(first_size), Constraint::Min(0)])
            .split(area);

        (chunks[0], chunks[1])
    }
}

/// A resizable split panel layout component.
///
/// `SplitPanel` manages the split ratio and pane focus for a two-pane
/// layout. The parent uses [`SplitPanelState::layout()`] to get the
/// pane areas and renders content into them.
///
/// # Navigation
///
/// - `Tab` — Toggle focus between panes
/// - `Ctrl+Left/Up` — Grow first pane (shrink second)
/// - `Ctrl+Right/Down` — Shrink first pane (grow second)
/// - `Ctrl+0` — Reset to 50/50 split
///
/// # Rendering
///
/// The `view()` method renders placeholder borders for each pane.
/// For real use, call `state.layout(area)` to get pane areas and
/// render your own content.
///
/// # Example
///
/// ```rust
/// use envision::component::{
///     Component, Focusable, SplitPanel, SplitPanelState,
///     SplitPanelMessage, SplitOrientation,
/// };
///
/// let mut state = SplitPanelState::new(SplitOrientation::Vertical);
/// SplitPanel::set_focused(&mut state, true);
///
/// // Get layout areas for rendering
/// let area = ratatui::layout::Rect::new(0, 0, 80, 24);
/// let (left, right) = state.layout(area);
/// assert!(left.width > 0);
/// assert!(right.width > 0);
/// ```
pub struct SplitPanel;

impl Component for SplitPanel {
    type State = SplitPanelState;
    type Message = SplitPanelMessage;
    type Output = SplitPanelOutput;

    fn init() -> Self::State {
        SplitPanelState::default()
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        if let Some(key) = event.as_key() {
            // Tab toggles pane focus
            if key.code == KeyCode::Tab || key.code == KeyCode::BackTab {
                return Some(SplitPanelMessage::FocusOther);
            }

            // Ctrl+arrow resizes
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Left | KeyCode::Up => return Some(SplitPanelMessage::ShrinkFirst),
                    KeyCode::Right | KeyCode::Down => return Some(SplitPanelMessage::GrowFirst),
                    KeyCode::Char('0') => return Some(SplitPanelMessage::ResetRatio),
                    _ => {}
                }
            }

            None
        } else {
            None
        }
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            SplitPanelMessage::FocusOther => {
                state.focused_pane = match state.focused_pane {
                    Pane::First => Pane::Second,
                    Pane::Second => Pane::First,
                };
                match state.focused_pane {
                    Pane::First => Some(SplitPanelOutput::FocusedFirst),
                    Pane::Second => Some(SplitPanelOutput::FocusedSecond),
                }
            }
            SplitPanelMessage::FocusFirst => {
                if state.focused_pane != Pane::First {
                    state.focused_pane = Pane::First;
                    Some(SplitPanelOutput::FocusedFirst)
                } else {
                    None
                }
            }
            SplitPanelMessage::FocusSecond => {
                if state.focused_pane != Pane::Second {
                    state.focused_pane = Pane::Second;
                    Some(SplitPanelOutput::FocusedSecond)
                } else {
                    None
                }
            }
            SplitPanelMessage::GrowFirst => {
                let new_ratio = (state.ratio + state.resize_step).min(state.max_ratio);
                if (new_ratio - state.ratio).abs() > f32::EPSILON {
                    state.ratio = new_ratio;
                    Some(SplitPanelOutput::RatioChanged(new_ratio))
                } else {
                    None
                }
            }
            SplitPanelMessage::ShrinkFirst => {
                let new_ratio = (state.ratio - state.resize_step).max(state.min_ratio);
                if (new_ratio - state.ratio).abs() > f32::EPSILON {
                    state.ratio = new_ratio;
                    Some(SplitPanelOutput::RatioChanged(new_ratio))
                } else {
                    None
                }
            }
            SplitPanelMessage::SetRatio(ratio) => {
                let clamped = ratio.clamp(state.min_ratio, state.max_ratio);
                if (clamped - state.ratio).abs() > f32::EPSILON {
                    state.ratio = clamped;
                    Some(SplitPanelOutput::RatioChanged(clamped))
                } else {
                    None
                }
            }
            SplitPanelMessage::ResetRatio => {
                let target = 0.5_f32.clamp(state.min_ratio, state.max_ratio);
                if (target - state.ratio).abs() > f32::EPSILON {
                    state.ratio = target;
                    Some(SplitPanelOutput::RatioChanged(target))
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme) {
        crate::annotation::with_registry(|reg| {
            reg.open(
                area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::SplitPanel)
                    .with_id("split_panel")
                    .with_focus(state.focused)
                    .with_disabled(state.disabled),
            );
        });

        let (first_area, second_area) = state.layout(area);

        let first_focused = state.focused && state.focused_pane == Pane::First;
        let second_focused = state.focused && state.focused_pane == Pane::Second;

        let first_border = if state.disabled {
            theme.disabled_style()
        } else if first_focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let second_border = if state.disabled {
            theme.disabled_style()
        } else if second_focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let first_block = Block::default()
            .borders(Borders::ALL)
            .border_style(first_border)
            .title(" Pane 1 ");

        let second_block = Block::default()
            .borders(Borders::ALL)
            .border_style(second_border)
            .title(" Pane 2 ");

        frame.render_widget(first_block, first_area);
        frame.render_widget(second_block, second_area);

        crate::annotation::with_registry(|reg| {
            reg.close();
        });
    }
}

impl Focusable for SplitPanel {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for SplitPanel {
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
