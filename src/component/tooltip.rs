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

    fn view(state: &Self::State, frame: &mut Frame, area: Rect) {
        // Use the area as both target and bounds for basic view
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
mod tests {
    use super::*;
    use crate::backend::CaptureBackend;
    use ratatui::Terminal;

    // ========================================
    // TooltipPosition Tests
    // ========================================

    #[test]
    fn test_position_default() {
        let position = TooltipPosition::default();
        assert_eq!(position, TooltipPosition::Below);
    }

    #[test]
    fn test_position_clone() {
        let position = TooltipPosition::Above;
        let cloned = position;
        assert_eq!(cloned, TooltipPosition::Above);
    }

    #[test]
    fn test_position_eq() {
        assert_eq!(TooltipPosition::Above, TooltipPosition::Above);
        assert_ne!(TooltipPosition::Above, TooltipPosition::Below);
        assert_eq!(TooltipPosition::Left, TooltipPosition::Left);
        assert_ne!(TooltipPosition::Left, TooltipPosition::Right);
    }

    // ========================================
    // State Creation Tests
    // ========================================

    #[test]
    fn test_new() {
        let state = TooltipState::new("Test content");
        assert_eq!(state.content(), "Test content");
        assert_eq!(state.title(), None);
        assert_eq!(state.position(), TooltipPosition::Below);
        assert!(!state.is_visible());
        assert_eq!(state.duration_ms(), None);
    }

    #[test]
    fn test_with_title() {
        let state = TooltipState::new("Content").with_title("My Title");
        assert_eq!(state.title(), Some("My Title"));
    }

    #[test]
    fn test_with_position() {
        let state = TooltipState::new("Content").with_position(TooltipPosition::Above);
        assert_eq!(state.position(), TooltipPosition::Above);
    }

    #[test]
    fn test_with_duration() {
        let state = TooltipState::new("Content").with_duration(5000);
        assert_eq!(state.duration_ms(), Some(5000));
    }

    #[test]
    fn test_with_fg_color() {
        let state = TooltipState::new("Content").with_fg_color(Color::Yellow);
        assert_eq!(state.fg_color(), Color::Yellow);
    }

    #[test]
    fn test_with_bg_color() {
        let state = TooltipState::new("Content").with_bg_color(Color::DarkGray);
        assert_eq!(state.bg_color(), Color::DarkGray);
    }

    #[test]
    fn test_with_border_color() {
        let state = TooltipState::new("Content").with_border_color(Color::Cyan);
        assert_eq!(state.border_color(), Color::Cyan);
    }

    #[test]
    fn test_default() {
        let state = TooltipState::default();
        assert_eq!(state.content(), "");
        assert_eq!(state.title(), None);
        assert_eq!(state.position(), TooltipPosition::Below);
        assert!(!state.is_visible());
        assert_eq!(state.duration_ms(), None);
        assert_eq!(state.remaining_ms(), None);
        assert_eq!(state.fg_color(), Color::White);
        assert_eq!(state.bg_color(), Color::Black);
        assert_eq!(state.border_color(), Color::Gray);
    }

    #[test]
    fn test_builder_chain() {
        let state = TooltipState::new("Content")
            .with_title("Title")
            .with_position(TooltipPosition::Left)
            .with_duration(3000)
            .with_fg_color(Color::Red)
            .with_bg_color(Color::Blue)
            .with_border_color(Color::Green);

        assert_eq!(state.content(), "Content");
        assert_eq!(state.title(), Some("Title"));
        assert_eq!(state.position(), TooltipPosition::Left);
        assert_eq!(state.duration_ms(), Some(3000));
        assert_eq!(state.fg_color(), Color::Red);
        assert_eq!(state.bg_color(), Color::Blue);
        assert_eq!(state.border_color(), Color::Green);
    }

    // ========================================
    // Accessor Tests
    // ========================================

    #[test]
    fn test_content() {
        let state = TooltipState::new("My content");
        assert_eq!(state.content(), "My content");
    }

    #[test]
    fn test_title() {
        let state = TooltipState::new("Content").with_title("Header");
        assert_eq!(state.title(), Some("Header"));
    }

    #[test]
    fn test_position() {
        let state = TooltipState::new("Content").with_position(TooltipPosition::Right);
        assert_eq!(state.position(), TooltipPosition::Right);
    }

    #[test]
    fn test_is_visible() {
        let state = TooltipState::new("Content");
        assert!(!state.is_visible());
    }

    #[test]
    fn test_duration_ms() {
        let state = TooltipState::new("Content").with_duration(2000);
        assert_eq!(state.duration_ms(), Some(2000));
    }

    #[test]
    fn test_remaining_ms() {
        let state = TooltipState::new("Content");
        assert_eq!(state.remaining_ms(), None);
    }

    #[test]
    fn test_fg_color() {
        let state = TooltipState::new("Content").with_fg_color(Color::Magenta);
        assert_eq!(state.fg_color(), Color::Magenta);
    }

    #[test]
    fn test_bg_color() {
        let state = TooltipState::new("Content").with_bg_color(Color::LightBlue);
        assert_eq!(state.bg_color(), Color::LightBlue);
    }

    #[test]
    fn test_border_color() {
        let state = TooltipState::new("Content").with_border_color(Color::LightGreen);
        assert_eq!(state.border_color(), Color::LightGreen);
    }

    // ========================================
    // Mutator Tests
    // ========================================

    #[test]
    fn test_set_content() {
        let mut state = TooltipState::new("Old");
        state.set_content("New");
        assert_eq!(state.content(), "New");
    }

    #[test]
    fn test_set_title() {
        let mut state = TooltipState::new("Content");
        state.set_title(Some("New Title".to_string()));
        assert_eq!(state.title(), Some("New Title"));

        state.set_title(None);
        assert_eq!(state.title(), None);
    }

    #[test]
    fn test_set_position() {
        let mut state = TooltipState::new("Content");
        state.set_position(TooltipPosition::Above);
        assert_eq!(state.position(), TooltipPosition::Above);
    }

    #[test]
    fn test_set_duration() {
        let mut state = TooltipState::new("Content");
        state.set_duration(Some(4000));
        assert_eq!(state.duration_ms(), Some(4000));

        state.set_duration(None);
        assert_eq!(state.duration_ms(), None);
    }

    #[test]
    fn test_set_fg_color() {
        let mut state = TooltipState::new("Content");
        state.set_fg_color(Color::Rgb(100, 150, 200));
        assert_eq!(state.fg_color(), Color::Rgb(100, 150, 200));
    }

    #[test]
    fn test_set_bg_color() {
        let mut state = TooltipState::new("Content");
        state.set_bg_color(Color::Indexed(42));
        assert_eq!(state.bg_color(), Color::Indexed(42));
    }

    #[test]
    fn test_set_border_color() {
        let mut state = TooltipState::new("Content");
        state.set_border_color(Color::LightRed);
        assert_eq!(state.border_color(), Color::LightRed);
    }

    // ========================================
    // Show/Hide Tests
    // ========================================

    #[test]
    fn test_show() {
        let mut state = TooltipState::new("Content");
        Tooltip::update(&mut state, TooltipMessage::Show);
        assert!(state.is_visible());
    }

    #[test]
    fn test_show_returns_shown() {
        let mut state = TooltipState::new("Content");
        let output = Tooltip::update(&mut state, TooltipMessage::Show);
        assert_eq!(output, Some(TooltipOutput::Shown));
    }

    #[test]
    fn test_show_already_visible() {
        let mut state = TooltipState::new("Content");
        Tooltip::update(&mut state, TooltipMessage::Show);

        let output = Tooltip::update(&mut state, TooltipMessage::Show);
        assert_eq!(output, None);
    }

    #[test]
    fn test_hide() {
        let mut state = TooltipState::new("Content");
        Tooltip::update(&mut state, TooltipMessage::Show);
        Tooltip::update(&mut state, TooltipMessage::Hide);
        assert!(!state.is_visible());
    }

    #[test]
    fn test_hide_returns_hidden() {
        let mut state = TooltipState::new("Content");
        Tooltip::update(&mut state, TooltipMessage::Show);

        let output = Tooltip::update(&mut state, TooltipMessage::Hide);
        assert_eq!(output, Some(TooltipOutput::Hidden));
    }

    #[test]
    fn test_hide_already_hidden() {
        let mut state = TooltipState::new("Content");
        let output = Tooltip::update(&mut state, TooltipMessage::Hide);
        assert_eq!(output, None);
    }

    #[test]
    fn test_toggle_show() {
        let mut state = TooltipState::new("Content");
        let output = Tooltip::update(&mut state, TooltipMessage::Toggle);
        assert!(state.is_visible());
        assert_eq!(output, Some(TooltipOutput::Shown));
    }

    #[test]
    fn test_toggle_hide() {
        let mut state = TooltipState::new("Content");
        Tooltip::update(&mut state, TooltipMessage::Show);

        let output = Tooltip::update(&mut state, TooltipMessage::Toggle);
        assert!(!state.is_visible());
        assert_eq!(output, Some(TooltipOutput::Hidden));
    }

    // ========================================
    // Auto-hide Tests
    // ========================================

    #[test]
    fn test_show_sets_remaining() {
        let mut state = TooltipState::new("Content").with_duration(3000);
        Tooltip::update(&mut state, TooltipMessage::Show);
        assert_eq!(state.remaining_ms(), Some(3000));
    }

    #[test]
    fn test_tick_decrements() {
        let mut state = TooltipState::new("Content").with_duration(3000);
        Tooltip::update(&mut state, TooltipMessage::Show);

        Tooltip::update(&mut state, TooltipMessage::Tick(1000));
        assert_eq!(state.remaining_ms(), Some(2000));
    }

    #[test]
    fn test_tick_expires() {
        let mut state = TooltipState::new("Content").with_duration(1000);
        Tooltip::update(&mut state, TooltipMessage::Show);

        Tooltip::update(&mut state, TooltipMessage::Tick(1000));
        assert!(!state.is_visible());
    }

    #[test]
    fn test_tick_returns_expired() {
        let mut state = TooltipState::new("Content").with_duration(1000);
        Tooltip::update(&mut state, TooltipMessage::Show);

        let output = Tooltip::update(&mut state, TooltipMessage::Tick(1000));
        assert_eq!(output, Some(TooltipOutput::Expired));
    }

    #[test]
    fn test_tick_no_duration() {
        let mut state = TooltipState::new("Content");
        Tooltip::update(&mut state, TooltipMessage::Show);

        let output = Tooltip::update(&mut state, TooltipMessage::Tick(10000));
        assert_eq!(output, None);
        assert!(state.is_visible());
    }

    #[test]
    fn test_tick_not_visible() {
        let mut state = TooltipState::new("Content").with_duration(1000);
        // Don't show - state is not visible

        let output = Tooltip::update(&mut state, TooltipMessage::Tick(100));
        assert_eq!(output, None);
    }

    #[test]
    fn test_hide_clears_remaining() {
        let mut state = TooltipState::new("Content").with_duration(3000);
        Tooltip::update(&mut state, TooltipMessage::Show);
        assert_eq!(state.remaining_ms(), Some(3000));

        Tooltip::update(&mut state, TooltipMessage::Hide);
        assert_eq!(state.remaining_ms(), None);
    }

    // ========================================
    // SetContent/SetPosition Message Tests
    // ========================================

    #[test]
    fn test_set_content_message() {
        let mut state = TooltipState::new("Old");
        Tooltip::update(&mut state, TooltipMessage::SetContent("New".into()));
        assert_eq!(state.content(), "New");
    }

    #[test]
    fn test_set_position_message() {
        let mut state = TooltipState::new("Content");
        Tooltip::update(
            &mut state,
            TooltipMessage::SetPosition(TooltipPosition::Left),
        );
        assert_eq!(state.position(), TooltipPosition::Left);
    }

    // ========================================
    // Toggleable Trait Tests
    // ========================================

    #[test]
    fn test_toggleable_is_visible() {
        let state = TooltipState::new("Content");
        assert!(!Tooltip::is_visible(&state));
    }

    #[test]
    fn test_toggleable_set_visible() {
        let mut state = TooltipState::new("Content").with_duration(3000);
        Tooltip::set_visible(&mut state, true);
        assert!(Tooltip::is_visible(&state));
        assert_eq!(state.remaining_ms(), Some(3000));

        Tooltip::set_visible(&mut state, false);
        assert!(!Tooltip::is_visible(&state));
        assert_eq!(state.remaining_ms(), None);
    }

    #[test]
    fn test_toggleable_show() {
        let mut state = TooltipState::new("Content");
        Tooltip::show(&mut state);
        assert!(Tooltip::is_visible(&state));
    }

    #[test]
    fn test_toggleable_hide() {
        let mut state = TooltipState::new("Content");
        Tooltip::show(&mut state);
        Tooltip::hide(&mut state);
        assert!(!Tooltip::is_visible(&state));
    }

    // ========================================
    // Position Calculation Tests
    // ========================================

    #[test]
    fn test_position_below() {
        let state = TooltipState::new("Test").with_position(TooltipPosition::Below);
        let target = Rect::new(10, 5, 20, 3);
        let bounds = Rect::new(0, 0, 80, 24);

        let area = calculate_tooltip_area(&state, target, bounds);
        assert_eq!(area.y, target.bottom());
    }

    #[test]
    fn test_position_above() {
        let state = TooltipState::new("Test").with_position(TooltipPosition::Above);
        let target = Rect::new(10, 10, 20, 3);
        let bounds = Rect::new(0, 0, 80, 24);

        let area = calculate_tooltip_area(&state, target, bounds);
        assert!(area.bottom() <= target.y);
    }

    #[test]
    fn test_position_left() {
        let state = TooltipState::new("Test").with_position(TooltipPosition::Left);
        let target = Rect::new(20, 5, 20, 3);
        let bounds = Rect::new(0, 0, 80, 24);

        let area = calculate_tooltip_area(&state, target, bounds);
        assert!(area.right() <= target.x);
    }

    #[test]
    fn test_position_right() {
        let state = TooltipState::new("Test").with_position(TooltipPosition::Right);
        let target = Rect::new(10, 5, 20, 3);
        let bounds = Rect::new(0, 0, 80, 24);

        let area = calculate_tooltip_area(&state, target, bounds);
        assert_eq!(area.x, target.right());
    }

    #[test]
    fn test_position_below_fallback() {
        let state = TooltipState::new("Test").with_position(TooltipPosition::Below);
        // Target at the bottom - no room below
        let target = Rect::new(10, 20, 20, 3);
        let bounds = Rect::new(0, 0, 80, 24);

        let area = calculate_tooltip_area(&state, target, bounds);
        // Should fall back to above
        assert!(area.bottom() <= target.y);
    }

    #[test]
    fn test_position_above_fallback() {
        let state = TooltipState::new("Test").with_position(TooltipPosition::Above);
        // Target at the top - no room above
        let target = Rect::new(10, 0, 20, 3);
        let bounds = Rect::new(0, 0, 80, 24);

        let area = calculate_tooltip_area(&state, target, bounds);
        // Should fall back to below
        assert!(area.y >= target.bottom());
    }

    #[test]
    fn test_position_left_fallback() {
        let state = TooltipState::new("Test").with_position(TooltipPosition::Left);
        // Target at the left edge - no room left
        let target = Rect::new(0, 5, 20, 3);
        let bounds = Rect::new(0, 0, 80, 24);

        let area = calculate_tooltip_area(&state, target, bounds);
        // Should fall back to right
        assert!(area.x >= target.right());
    }

    #[test]
    fn test_position_right_fallback() {
        let state = TooltipState::new("Test").with_position(TooltipPosition::Right);
        // Target at the right edge - no room right
        let target = Rect::new(70, 5, 10, 3);
        let bounds = Rect::new(0, 0, 80, 24);

        let area = calculate_tooltip_area(&state, target, bounds);
        // Should fall back to left
        assert!(area.right() <= target.x);
    }

    // ========================================
    // View Tests
    // ========================================

    #[test]
    fn test_view_hidden() {
        let state = TooltipState::new("Content");

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let target = Rect::new(10, 5, 20, 3);
                Tooltip::view_at(&state, frame, target, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(!output.contains("Content"));
    }

    #[test]
    fn test_view_empty_content() {
        let mut state = TooltipState::new("");
        Tooltip::show(&mut state);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let target = Rect::new(10, 5, 20, 3);
                Tooltip::view_at(&state, frame, target, frame.area());
            })
            .unwrap();

        // Should render nothing for empty content
        let output = terminal.backend().to_string();
        assert!(output.trim().is_empty());
    }

    #[test]
    fn test_view_visible() {
        let mut state = TooltipState::new("Helpful tooltip");
        Tooltip::show(&mut state);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let target = Rect::new(10, 5, 20, 3);
                Tooltip::view_at(&state, frame, target, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Helpful tooltip"));
    }

    #[test]
    fn test_view_with_title() {
        let mut state = TooltipState::new("Content").with_title("Info");
        Tooltip::show(&mut state);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let target = Rect::new(10, 5, 20, 3);
                Tooltip::view_at(&state, frame, target, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Info"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn test_view_multiline() {
        let mut state = TooltipState::new("Line 1\nLine 2\nLine 3");
        Tooltip::show(&mut state);

        let backend = CaptureBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let target = Rect::new(10, 5, 20, 3);
                Tooltip::view_at(&state, frame, target, frame.area());
            })
            .unwrap();

        let output = terminal.backend().to_string();
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2"));
        assert!(output.contains("Line 3"));
    }

    // ========================================
    // Integration Tests
    // ========================================

    #[test]
    fn test_clone() {
        let state = TooltipState::new("Content")
            .with_title("Title")
            .with_position(TooltipPosition::Above)
            .with_duration(5000)
            .with_fg_color(Color::Yellow);

        let cloned = state.clone();
        assert_eq!(cloned.content(), "Content");
        assert_eq!(cloned.title(), Some("Title"));
        assert_eq!(cloned.position(), TooltipPosition::Above);
        assert_eq!(cloned.duration_ms(), Some(5000));
        assert_eq!(cloned.fg_color(), Color::Yellow);
    }

    #[test]
    fn test_init() {
        let state = Tooltip::init();
        assert_eq!(state.content(), "");
        assert!(!state.is_visible());
    }

    #[test]
    fn test_full_workflow() {
        let mut state = TooltipState::new("Click to submit").with_duration(3000);

        // Show
        let output = Tooltip::update(&mut state, TooltipMessage::Show);
        assert_eq!(output, Some(TooltipOutput::Shown));
        assert!(state.is_visible());
        assert_eq!(state.remaining_ms(), Some(3000));

        // Tick partial
        Tooltip::update(&mut state, TooltipMessage::Tick(1000));
        assert_eq!(state.remaining_ms(), Some(2000));
        assert!(state.is_visible());

        // Tick more
        Tooltip::update(&mut state, TooltipMessage::Tick(1000));
        assert_eq!(state.remaining_ms(), Some(1000));

        // Tick to expire
        let output = Tooltip::update(&mut state, TooltipMessage::Tick(1000));
        assert_eq!(output, Some(TooltipOutput::Expired));
        assert!(!state.is_visible());
        assert_eq!(state.remaining_ms(), None);
    }
}
