//! A tooltip component for displaying contextual information.
//!
//! `Tooltip` provides a positioned overlay that displays helpful information
//! relative to a target area. Supports configurable positioning with automatic
//! fallback, optional auto-hide, and basic styling.
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Tooltip, TooltipMessage, TooltipState, TooltipPosition, Component, Toggleable};
//!
//! // Create a tooltip with auto-hide
//! let mut state = TooltipState::new("Click to submit the form")
//!     .with_position(TooltipPosition::Below)
//!     .with_duration(3000);
//!
//! // Show the tooltip
//! Tooltip::show(&mut state);
//!
//! // Later, in your tick handler
//! Tooltip::update(&mut state, TooltipMessage::Tick(100));
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::{Component, Toggleable};
use crate::theme::Theme;

/// Position of the tooltip relative to its target.
///
/// When there isn't enough space for the preferred position,
/// the tooltip will automatically fall back to the opposite side.
///
/// # Example
///
/// ```rust
/// use envision::component::TooltipPosition;
///
/// let position = TooltipPosition::default();
/// assert_eq!(position, TooltipPosition::Below);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPosition {
    /// Display above the target.
    Above,
    /// Display below the target.
    #[default]
    Below,
    /// Display to the left of the target.
    Left,
    /// Display to the right of the target.
    Right,
}

/// Messages that can be sent to a Tooltip component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TooltipMessage {
    /// Show the tooltip.
    Show,
    /// Hide the tooltip.
    Hide,
    /// Toggle visibility.
    Toggle,
    /// Set the tooltip content.
    SetContent(String),
    /// Set the position.
    SetPosition(TooltipPosition),
    /// Advance time for auto-hide (milliseconds).
    Tick(u64),
}

/// Output messages from a Tooltip component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TooltipOutput {
    /// Tooltip was shown.
    Shown,
    /// Tooltip was hidden (manually).
    Hidden,
    /// Tooltip expired (auto-hide).
    Expired,
}

/// State for a Tooltip component.
///
/// Contains all the state needed to render and manage a tooltip,
/// including content, position, visibility, and auto-hide settings.
///
/// # Example
///
/// ```rust
/// use envision::component::{TooltipState, TooltipPosition};
/// use ratatui::style::Color;
///
/// let state = TooltipState::new("Helpful tooltip text")
///     .with_title("Info")
///     .with_position(TooltipPosition::Above)
///     .with_duration(5000)
///     .with_fg_color(Color::White)
///     .with_bg_color(Color::DarkGray);
/// ```
#[derive(Clone, Debug)]
pub struct TooltipState {
    /// The tooltip content/text.
    content: String,
    /// Optional title.
    title: Option<String>,
    /// Preferred position relative to target.
    position: TooltipPosition,
    /// Whether the tooltip is visible.
    visible: bool,
    /// Auto-hide duration in milliseconds (None = persistent).
    duration_ms: Option<u64>,
    /// Remaining time before auto-hide.
    remaining_ms: Option<u64>,
    /// Foreground color.
    fg_color: Color,
    /// Background color.
    bg_color: Color,
    /// Border color.
    border_color: Color,
}

impl Default for TooltipState {
    fn default() -> Self {
        Self {
            content: String::new(),
            title: None,
            position: TooltipPosition::Below,
            visible: false,
            duration_ms: None,
            remaining_ms: None,
            fg_color: Color::White,
            bg_color: Color::Black,
            border_color: Color::Gray,
        }
    }
}

impl TooltipState {
    /// Creates a new tooltip with the given content.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TooltipState;
    ///
    /// let state = TooltipState::new("Click to submit");
    /// assert_eq!(state.content(), "Click to submit");
    /// ```
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Self::default()
        }
    }

    /// Sets the tooltip title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TooltipState;
    ///
    /// let state = TooltipState::new("Content").with_title("Info");
    /// assert_eq!(state.title(), Some("Info"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the preferred position.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{TooltipState, TooltipPosition};
    ///
    /// let state = TooltipState::new("Content").with_position(TooltipPosition::Above);
    /// assert_eq!(state.position(), TooltipPosition::Above);
    /// ```
    pub fn with_position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    /// Sets the auto-hide duration in milliseconds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TooltipState;
    ///
    /// let state = TooltipState::new("Content").with_duration(3000);
    /// assert_eq!(state.duration_ms(), Some(3000));
    /// ```
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Sets the foreground color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TooltipState;
    /// use ratatui::style::Color;
    ///
    /// let state = TooltipState::new("Content").with_fg_color(Color::Yellow);
    /// assert_eq!(state.fg_color(), Color::Yellow);
    /// ```
    pub fn with_fg_color(mut self, color: Color) -> Self {
        self.fg_color = color;
        self
    }

    /// Sets the background color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TooltipState;
    /// use ratatui::style::Color;
    ///
    /// let state = TooltipState::new("Content").with_bg_color(Color::DarkGray);
    /// assert_eq!(state.bg_color(), Color::DarkGray);
    /// ```
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Sets the border color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::TooltipState;
    /// use ratatui::style::Color;
    ///
    /// let state = TooltipState::new("Content").with_border_color(Color::Yellow);
    /// assert_eq!(state.border_color(), Color::Yellow);
    /// ```
    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Returns the tooltip content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the tooltip title, if any.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns the preferred position.
    pub fn position(&self) -> TooltipPosition {
        self.position
    }

    /// Returns whether the tooltip is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the auto-hide duration in milliseconds.
    pub fn duration_ms(&self) -> Option<u64> {
        self.duration_ms
    }

    /// Returns the remaining time before auto-hide.
    pub fn remaining_ms(&self) -> Option<u64> {
        self.remaining_ms
    }

    /// Returns the foreground color.
    pub fn fg_color(&self) -> Color {
        self.fg_color
    }

    /// Returns the background color.
    pub fn bg_color(&self) -> Color {
        self.bg_color
    }

    /// Returns the border color.
    pub fn border_color(&self) -> Color {
        self.border_color
    }

    /// Sets the tooltip content.
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    /// Sets the tooltip title.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Sets the preferred position.
    pub fn set_position(&mut self, position: TooltipPosition) {
        self.position = position;
    }

    /// Sets the auto-hide duration.
    pub fn set_duration(&mut self, duration_ms: Option<u64>) {
        self.duration_ms = duration_ms;
    }

    /// Sets the foreground color.
    pub fn set_fg_color(&mut self, color: Color) {
        self.fg_color = color;
    }

    /// Sets the background color.
    pub fn set_bg_color(&mut self, color: Color) {
        self.bg_color = color;
    }

    /// Sets the border color.
    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
    }
}

/// A tooltip component for displaying contextual information.
///
/// `Tooltip` displays a positioned overlay with helpful text relative to
/// a target area. It implements:
/// - [`Component`] for update/view logic
/// - [`Toggleable`] for visibility control
///
/// # Positioning
///
/// The tooltip can be positioned Above, Below, Left, or Right of its target.
/// When there isn't enough space for the preferred position, it automatically
/// falls back to the opposite side.
///
/// # Auto-hide
///
/// When configured with a duration, the tooltip automatically hides after
/// the specified time. Send periodic `Tick(elapsed_ms)` messages to drive
/// this functionality.
///
/// # Visual Format
///
/// ```text
/// Target area [████████]
///
///          ┌─────────────────┐
///          │ Helpful tooltip │  ← Tooltip (Below)
///          │ text here       │
///          └─────────────────┘
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{Tooltip, TooltipMessage, TooltipOutput, TooltipState, Component, Toggleable};
///
/// let mut state = TooltipState::new("Click to submit")
///     .with_duration(3000);
///
/// // Show the tooltip
/// let output = Tooltip::update(&mut state, TooltipMessage::Show);
/// assert_eq!(output, Some(TooltipOutput::Shown));
///
/// // Tick until it expires
/// let output = Tooltip::update(&mut state, TooltipMessage::Tick(3000));
/// assert_eq!(output, Some(TooltipOutput::Expired));
/// assert!(!state.is_visible());
/// ```
pub struct Tooltip;

impl Component for Tooltip {
    type State = TooltipState;
    type Message = TooltipMessage;
    type Output = TooltipOutput;

    fn init() -> Self::State {
        TooltipState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            TooltipMessage::Show => {
                if !state.visible {
                    state.visible = true;
                    state.remaining_ms = state.duration_ms;
                    Some(TooltipOutput::Shown)
                } else {
                    None
                }
            }
            TooltipMessage::Hide => {
                if state.visible {
                    state.visible = false;
                    state.remaining_ms = None;
                    Some(TooltipOutput::Hidden)
                } else {
                    None
                }
            }
            TooltipMessage::Toggle => {
                if state.visible {
                    state.visible = false;
                    state.remaining_ms = None;
                    Some(TooltipOutput::Hidden)
                } else {
                    state.visible = true;
                    state.remaining_ms = state.duration_ms;
                    Some(TooltipOutput::Shown)
                }
            }
            TooltipMessage::SetContent(content) => {
                state.content = content;
                None
            }
            TooltipMessage::SetPosition(position) => {
                state.position = position;
                None
            }
            TooltipMessage::Tick(elapsed) => {
                if !state.visible {
                    return None;
                }
                if let Some(remaining) = state.remaining_ms {
                    if elapsed >= remaining {
                        state.visible = false;
                        state.remaining_ms = None;
                        Some(TooltipOutput::Expired)
                    } else {
                        state.remaining_ms = Some(remaining - elapsed);
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, _theme: &Theme) {
        // Use the area as both target and bounds for basic view
        // Note: Tooltip uses its own colors from state rather than theme
        Self::view_at(state, frame, area, area);
    }
}

impl Toggleable for Tooltip {
    fn is_visible(state: &Self::State) -> bool {
        state.visible
    }

    fn set_visible(state: &mut Self::State, visible: bool) {
        state.visible = visible;
        if visible {
            state.remaining_ms = state.duration_ms;
        } else {
            state.remaining_ms = None;
        }
    }
}

impl Tooltip {
    /// Renders the tooltip relative to a target area within bounds.
    ///
    /// This is the primary rendering method. Call this instead of `view()`
    /// when you need to position the tooltip relative to a specific target.
    ///
    /// # Arguments
    ///
    /// * `state` - The tooltip state
    /// * `frame` - The frame to render to
    /// * `target` - The target area to position relative to
    /// * `bounds` - The bounding area (typically frame.area())
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use envision::component::{Tooltip, TooltipState, Toggleable};
    /// use ratatui::prelude::*;
    ///
    /// let mut state = TooltipState::new("Button help text");
    /// Tooltip::show(&mut state);
    ///
    /// // In your view function:
    /// // let button_area = Rect::new(10, 5, 20, 3);
    /// // Tooltip::view_at(&state, frame, button_area, frame.area());
    /// ```
    pub fn view_at(state: &TooltipState, frame: &mut Frame, target: Rect, bounds: Rect) {
        if !state.visible || state.content.is_empty() {
            return;
        }

        let area = calculate_tooltip_area(state, target, bounds);

        // Clear the area (overlay)
        frame.render_widget(Clear, area);

        // Create block with optional title
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(state.border_color))
            .style(Style::default().bg(state.bg_color));

        if let Some(title) = &state.title {
            block = block.title(format!(" {} ", title));
        }

        let paragraph = Paragraph::new(state.content.as_str())
            .style(Style::default().fg(state.fg_color).bg(state.bg_color))
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

/// Calculates the tooltip area based on position and available space.
fn calculate_tooltip_area(state: &TooltipState, target: Rect, bounds: Rect) -> Rect {
    // Calculate content dimensions
    let content_width = state.content.lines().map(|l| l.len()).max().unwrap_or(0) as u16 + 2; // +2 for borders

    let title_width = state
        .title
        .as_ref()
        .map(|t| t.len() as u16 + 4) // +4 for " Title " padding
        .unwrap_or(0);

    let width = content_width
        .max(title_width)
        .min(bounds.width.saturating_sub(2))
        .max(3); // Minimum width of 3 for borders

    let content_height = state.content.lines().count().max(1) as u16 + 2; // +2 for borders
    let height = content_height.min(bounds.height.saturating_sub(2)).max(3);

    match state.position {
        TooltipPosition::Below => {
            let y = target.bottom();
            if y + height <= bounds.bottom() {
                // Fits below
                let x = clamp_x(target.x, width, bounds);
                Rect::new(x, y, width, height)
            } else {
                // Fallback to above
                let y = target.y.saturating_sub(height);
                let x = clamp_x(target.x, width, bounds);
                Rect::new(x, y, width, height)
            }
        }
        TooltipPosition::Above => {
            if target.y >= height {
                // Fits above
                let y = target.y.saturating_sub(height);
                let x = clamp_x(target.x, width, bounds);
                Rect::new(x, y, width, height)
            } else {
                // Fallback to below
                let y = target.bottom();
                let x = clamp_x(target.x, width, bounds);
                Rect::new(x, y, width, height)
            }
        }
        TooltipPosition::Right => {
            let x = target.right();
            if x + width <= bounds.right() {
                // Fits to the right
                let y = clamp_y(target.y, height, bounds);
                Rect::new(x, y, width, height)
            } else {
                // Fallback to left
                let x = target.x.saturating_sub(width);
                let y = clamp_y(target.y, height, bounds);
                Rect::new(x, y, width, height)
            }
        }
        TooltipPosition::Left => {
            if target.x >= width {
                // Fits to the left
                let x = target.x.saturating_sub(width);
                let y = clamp_y(target.y, height, bounds);
                Rect::new(x, y, width, height)
            } else {
                // Fallback to right
                let x = target.right();
                let y = clamp_y(target.y, height, bounds);
                Rect::new(x, y, width, height)
            }
        }
    }
}

/// Clamps the x coordinate to keep the tooltip within bounds.
fn clamp_x(x: u16, width: u16, bounds: Rect) -> u16 {
    let max_x = bounds.right().saturating_sub(width);
    x.clamp(bounds.x, max_x)
}

/// Clamps the y coordinate to keep the tooltip within bounds.
fn clamp_y(y: u16, height: u16, bounds: Rect) -> u16 {
    let max_y = bounds.bottom().saturating_sub(height);
    y.clamp(bounds.y, max_y)
}

#[cfg(test)]
mod tests;
