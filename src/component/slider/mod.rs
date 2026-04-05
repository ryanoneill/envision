//! A numeric range selection component with keyboard control.
//!
//! [`Slider`] provides an interactive slider for selecting a numeric value
//! within a configurable range. State is stored in [`SliderState`], updated
//! via [`SliderMessage`], and produces [`SliderOutput`].
//!
//!
//! See also [`ProgressBar`](super::ProgressBar) for a display-only progress indicator.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Slider, SliderMessage, SliderOutput, SliderState, Component};
//!
//! // Create a slider with range 0..=100
//! let mut state = SliderState::new(0.0, 100.0);
//! assert_eq!(state.value(), 0.0);
//!
//! // Increment the value
//! let output = Slider::update(&mut state, SliderMessage::Increment);
//! assert_eq!(output, Some(SliderOutput::ValueChanged(1.0)));
//! assert_eq!(state.value(), 1.0);
//!
//! // Set value directly (clamped to range)
//! let output = Slider::update(&mut state, SliderMessage::SetValue(50.0));
//! assert_eq!(output, Some(SliderOutput::ValueChanged(50.0)));
//! assert_eq!(state.value(), 50.0);
//!
//! // Check percentage
//! assert!((state.percentage() - 0.5).abs() < f64::EPSILON);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, ViewContext};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

/// Orientation of the slider.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum SliderOrientation {
    /// Horizontal slider (left to right).
    #[default]
    Horizontal,
    /// Vertical slider (bottom to top).
    Vertical,
}

/// Messages that can be sent to a Slider.
#[derive(Clone, Debug, PartialEq)]
pub enum SliderMessage {
    /// Increase value by one step.
    Increment,
    /// Decrease value by one step.
    Decrement,
    /// Increase value by step * 10.
    IncrementPage,
    /// Decrease value by step * 10.
    DecrementPage,
    /// Set value directly (clamped to range).
    SetValue(f64),
    /// Set value to the minimum.
    SetMin,
    /// Set value to the maximum.
    SetMax,
}

/// Output messages from a Slider.
#[derive(Clone, Debug, PartialEq)]
pub enum SliderOutput {
    /// The slider value changed. Contains the new value.
    ValueChanged(f64),
}

/// State for a Slider component.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct SliderState {
    /// The current value.
    value: f64,
    /// The minimum value.
    min: f64,
    /// The maximum value.
    max: f64,
    /// The step increment.
    step: f64,
    /// The slider orientation.
    orientation: SliderOrientation,
    /// Optional label.
    label: Option<String>,
    /// Whether to display the current value.
    show_value: bool,
}

impl Default for SliderState {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            orientation: SliderOrientation::default(),
            label: None,
            show_value: true,
        }
    }
}

impl SliderState {
    /// Creates a new slider with the given range.
    ///
    /// The initial value is set to `min`. The step size defaults to 1.0.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SliderState;
    ///
    /// let state = SliderState::new(0.0, 100.0);
    /// assert_eq!(state.value(), 0.0);
    /// assert_eq!(state.min(), 0.0);
    /// assert_eq!(state.max(), 100.0);
    /// assert_eq!(state.step(), 1.0);
    /// ```
    pub fn new(min: f64, max: f64) -> Self {
        Self {
            value: min,
            min,
            max,
            ..Self::default()
        }
    }

    /// Sets the initial value (builder pattern).
    ///
    /// The value is clamped to the slider's range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SliderState;
    ///
    /// let state = SliderState::new(0.0, 100.0).with_value(50.0);
    /// assert_eq!(state.value(), 50.0);
    /// ```
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    /// Sets the step size (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SliderState;
    ///
    /// let state = SliderState::new(0.0, 100.0).with_step(5.0);
    /// assert_eq!(state.step(), 5.0);
    /// ```
    pub fn with_step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Sets the orientation (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SliderState, SliderOrientation};
    ///
    /// let state = SliderState::new(0.0, 100.0)
    ///     .with_orientation(SliderOrientation::Vertical);
    /// ```
    pub fn with_orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the label (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SliderState;
    ///
    /// let state = SliderState::new(0.0, 100.0).with_label("Volume");
    /// assert_eq!(state.label(), Some("Volume"));
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets whether to show the current value (builder pattern).
    ///
    /// Default is `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SliderState;
    ///
    /// let state = SliderState::new(0.0, 100.0).with_show_value(false);
    /// ```
    pub fn with_show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }

    /// Returns the current value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SliderState;
    ///
    /// let state = SliderState::new(0.0, 100.0).with_value(42.0);
    /// assert_eq!(state.value(), 42.0);
    /// ```
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Sets the current value, clamping it to the range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SliderState;
    ///
    /// let mut state = SliderState::new(0.0, 100.0);
    /// state.set_value(50.0);
    /// assert_eq!(state.value(), 50.0);
    ///
    /// // Values are clamped to the range
    /// state.set_value(200.0);
    /// assert_eq!(state.value(), 100.0);
    /// ```
    pub fn set_value(&mut self, value: f64) {
        self.value = value.clamp(self.min, self.max);
    }

    /// Returns the minimum value.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Returns the maximum value.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Returns the step size.
    pub fn step(&self) -> f64 {
        self.step
    }

    /// Returns the current value as a percentage (0.0..=1.0).
    ///
    /// Returns 0.0 if min equals max (degenerate range).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SliderState;
    ///
    /// let state = SliderState::new(0.0, 100.0).with_value(25.0);
    /// assert!((state.percentage() - 0.25).abs() < f64::EPSILON);
    ///
    /// let state = SliderState::new(10.0, 10.0);
    /// assert!((state.percentage() - 0.0).abs() < f64::EPSILON);
    /// ```
    pub fn percentage(&self) -> f64 {
        let range = self.max - self.min;
        if range == 0.0 {
            0.0
        } else {
            (self.value - self.min) / range
        }
    }

    /// Returns the label, if set.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns whether the value display is enabled.
    pub fn show_value(&self) -> bool {
        self.show_value
    }

    /// Sets whether the value display is enabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::SliderState;
    ///
    /// let mut state = SliderState::new(0.0, 100.0);
    /// state.set_show_value(true);
    /// assert!(state.show_value());
    /// ```
    pub fn set_show_value(&mut self, show: bool) {
        self.show_value = show;
    }

    /// Returns the orientation.
    pub fn orientation(&self) -> &SliderOrientation {
        &self.orientation
    }

    /// Sets the orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SliderState, SliderOrientation};
    ///
    /// let mut state = SliderState::new(0.0, 100.0);
    /// state.set_orientation(SliderOrientation::Vertical);
    /// assert_eq!(state.orientation(), &SliderOrientation::Vertical);
    /// ```
    pub fn set_orientation(&mut self, orientation: SliderOrientation) {
        self.orientation = orientation;
    }

    /// Updates the slider state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{SliderMessage, SliderOutput, SliderState};
    ///
    /// let mut state = SliderState::new(0.0, 100.0);
    /// let output = state.update(SliderMessage::Increment);
    /// assert_eq!(output, Some(SliderOutput::ValueChanged(1.0)));
    /// assert_eq!(state.value(), 1.0);
    /// ```
    pub fn update(&mut self, msg: SliderMessage) -> Option<SliderOutput> {
        Slider::update(self, msg)
    }
}

/// A numeric range selection component with keyboard control.
///
/// `Slider` provides an interactive control for selecting a numeric value
/// within a configurable range. The slider supports both horizontal and
/// vertical orientations, configurable step sizes, and optional labels.
///
/// # Keyboard Controls
///
/// Horizontal mode:
/// - Right / l: increment by step
/// - Left / h: decrement by step
///
/// Vertical mode:
/// - Up / k: increment by step
/// - Down / j: decrement by step
///
/// Both modes:
/// - PageUp: increment by step * 10
/// - PageDown: decrement by step * 10
/// - Home: set to minimum
/// - End: set to maximum
///
/// # Visual Format
///
/// Horizontal:
/// ```text
/// Label: 42.0
/// ████████████░░░░░░░░░░░░░░░░░░░
/// ```
///
/// The filled portion uses block characters (`\u{2588}`) and the empty
/// portion uses light shade characters (`\u{2591}`).
///
/// # Example
///
/// ```rust
/// use envision::component::{Slider, SliderMessage, SliderOutput, SliderState, Component};
///
/// let mut state = SliderState::new(0.0, 100.0)
///     .with_value(50.0)
///     .with_step(5.0)
///     .with_label("Volume");
///
/// // Increment the value
/// let output = Slider::update(&mut state, SliderMessage::Increment);
/// assert_eq!(output, Some(SliderOutput::ValueChanged(55.0)));
/// assert_eq!(state.value(), 55.0);
/// ```
pub struct Slider;

impl Component for Slider {
    type State = SliderState;
    type Message = SliderMessage;
    type Output = SliderOutput;

    fn init() -> Self::State {
        SliderState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        let old_value = state.value;

        match msg {
            SliderMessage::Increment => {
                state.value = (state.value + state.step).min(state.max);
            }
            SliderMessage::Decrement => {
                state.value = (state.value - state.step).max(state.min);
            }
            SliderMessage::IncrementPage => {
                state.value = (state.value + state.step * 10.0).min(state.max);
            }
            SliderMessage::DecrementPage => {
                state.value = (state.value - state.step * 10.0).max(state.min);
            }
            SliderMessage::SetValue(v) => {
                state.value = v.clamp(state.min, state.max);
            }
            SliderMessage::SetMin => {
                state.value = state.min;
            }
            SliderMessage::SetMax => {
                state.value = state.max;
            }
        }

        if (state.value - old_value).abs() > f64::EPSILON {
            Some(SliderOutput::ValueChanged(state.value))
        } else {
            None
        }
    }

    fn handle_event(
        state: &Self::State,
        event: &Event,
        ctx: &ViewContext,
    ) -> Option<Self::Message> {
        if !ctx.focused || ctx.disabled {
            return None;
        }

        if let Some(key) = event.as_key() {
            match state.orientation {
                SliderOrientation::Horizontal => match key.code {
                    KeyCode::Right | KeyCode::Char('l') => Some(SliderMessage::Increment),
                    KeyCode::Left | KeyCode::Char('h') => Some(SliderMessage::Decrement),
                    KeyCode::PageUp => Some(SliderMessage::IncrementPage),
                    KeyCode::PageDown => Some(SliderMessage::DecrementPage),
                    KeyCode::Home => Some(SliderMessage::SetMin),
                    KeyCode::End => Some(SliderMessage::SetMax),
                    _ => None,
                },
                SliderOrientation::Vertical => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => Some(SliderMessage::Increment),
                    KeyCode::Down | KeyCode::Char('j') => Some(SliderMessage::Decrement),
                    KeyCode::PageUp => Some(SliderMessage::IncrementPage),
                    KeyCode::PageDown => Some(SliderMessage::DecrementPage),
                    KeyCode::Home => Some(SliderMessage::SetMin),
                    KeyCode::End => Some(SliderMessage::SetMax),
                    _ => None,
                },
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, _ctx: &ViewContext) {
        match state.orientation {
            SliderOrientation::Horizontal => view_horizontal(state, frame, area, theme),
            SliderOrientation::Vertical => view_vertical(state, frame, area, theme),
        }
    }
}

/// Renders the slider in horizontal orientation.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn view_horizontal(state: &SliderState, frame: &mut Frame, area: Rect, theme: &Theme) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let (label_style, filled_style, empty_style) = compute_styles(state, theme);

    let mut lines = Vec::new();

    // Build label line if needed
    if state.label.is_some() || state.show_value {
        let label_text = build_label_text(state);
        lines.push(Line::from(Span::styled(label_text, label_style)));
    }

    // Build track line
    let track_width = area.width as usize;
    let pct = state.percentage();
    let filled = (pct * track_width as f64).round() as usize;
    let empty = track_width.saturating_sub(filled);

    let mut spans = Vec::new();
    if filled > 0 {
        spans.push(Span::styled("\u{2588}".repeat(filled), filled_style));
    }
    if empty > 0 {
        spans.push(Span::styled("\u{2591}".repeat(empty), empty_style));
    }
    lines.push(Line::from(spans));

    let paragraph = Paragraph::new(Text::from(lines));

    let value_str = format_value(state.value);
    let annotation = crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
        "Slider".to_string(),
    ))
    .with_id("slider")
    .with_value(value_str);

    let annotation = if let Some(label) = &state.label {
        annotation.with_label(label.as_str())
    } else {
        annotation
    };

    let annotated = crate::annotation::Annotate::new(paragraph, annotation)
        .focused(state.focused)
        .disabled(state.disabled);
    frame.render_widget(annotated, area);
}

/// Renders the slider in vertical orientation.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn view_vertical(state: &SliderState, frame: &mut Frame, area: Rect, theme: &Theme) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let (label_style, filled_style, empty_style) = compute_styles(state, theme);

    let mut lines = Vec::new();

    // Reserve space for label at top
    let label_lines = if state.label.is_some() || state.show_value {
        1
    } else {
        0
    };
    let track_height = (area.height as usize).saturating_sub(label_lines);

    // Label at top
    if state.label.is_some() || state.show_value {
        let label_text = build_label_text(state);
        lines.push(Line::from(Span::styled(label_text, label_style)));
    }

    // Track from top (max) to bottom (min)
    let pct = state.percentage();
    let filled = (pct * track_height as f64).round() as usize;
    let empty = track_height.saturating_sub(filled);

    // Empty portion at top (higher values not yet reached)
    for _ in 0..empty {
        lines.push(Line::from(Span::styled("\u{2591}", empty_style)));
    }
    // Filled portion at bottom (lower values, already reached)
    for _ in 0..filled {
        lines.push(Line::from(Span::styled("\u{2588}", filled_style)));
    }

    let paragraph = Paragraph::new(Text::from(lines));

    let value_str = format_value(state.value);
    let annotation = crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
        "Slider".to_string(),
    ))
    .with_id("slider")
    .with_value(value_str);

    let annotation = if let Some(label) = &state.label {
        annotation.with_label(label.as_str())
    } else {
        annotation
    };

    let annotated = crate::annotation::Annotate::new(paragraph, annotation)
        .focused(state.focused)
        .disabled(state.disabled);
    frame.render_widget(annotated, area);
}

/// Computes the styles for label, filled, and empty portions.
fn compute_styles(state: &SliderState, theme: &Theme) -> (Style, Style, Style) {
    if state.disabled {
        let disabled = theme.disabled_style();
        (disabled, disabled, disabled)
    } else if state.focused {
        let label_style = theme.focused_style();
        let filled_style = theme.focused_style();
        let empty_style = theme.normal_style();
        (label_style, filled_style, empty_style)
    } else {
        let label_style = theme.normal_style();
        let filled_style = theme.normal_style();
        let empty_style = theme.normal_style();
        (label_style, filled_style, empty_style)
    }
}

/// Builds the label text combining label and value display.
fn build_label_text(state: &SliderState) -> String {
    let mut parts = Vec::new();

    if let Some(label) = &state.label {
        parts.push(label.clone());
    }

    if state.show_value {
        parts.push(format_value(state.value));
    }

    parts.join(": ")
}

/// Formats a value for display, omitting decimal places when the value is a whole number.
fn format_value(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

#[cfg(test)]
mod tests;
