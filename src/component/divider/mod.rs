//! A horizontal or vertical separator line with optional label.
//!
//! [`Divider`] provides a simple visual separator for dividing content areas.
//! It can be oriented horizontally or vertically, and optionally displays a
//! centered label. This is a **display-only** component that does not receive
//! keyboard focus. State is stored in [`DividerState`] and updated via
//! [`DividerMessage`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Divider, DividerMessage, DividerState, DividerOrientation, Component};
//!
//! // Create a horizontal divider with a label
//! let state = DividerState::new()
//!     .with_label("Section");
//! assert_eq!(state.label(), Some("Section"));
//! assert_eq!(state.orientation(), &DividerOrientation::Horizontal);
//!
//! // Create a vertical divider
//! let state = DividerState::vertical();
//! assert_eq!(state.orientation(), &DividerOrientation::Vertical);
//! ```

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::{Component, EventContext, RenderContext};
use crate::input::Event;

/// Orientation of the divider.
///
/// # Example
///
/// ```rust
/// use envision::component::DividerOrientation;
///
/// let orientation = DividerOrientation::default();
/// assert_eq!(orientation, DividerOrientation::Horizontal);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum DividerOrientation {
    /// Horizontal separator line spanning the full width.
    #[default]
    Horizontal,
    /// Vertical separator line spanning the full height.
    Vertical,
}

/// Messages that can be sent to a Divider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DividerMessage {
    /// Change the label text.
    SetLabel(Option<String>),
    /// Change the orientation.
    SetOrientation(DividerOrientation),
}

/// State for a Divider component.
///
/// Contains the orientation, optional label, optional color override,
/// and disabled state.
///
/// # Example
///
/// ```rust
/// use envision::component::DividerState;
///
/// let state = DividerState::new();
/// assert!(state.label().is_none());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct DividerState {
    /// The orientation of the divider.
    orientation: DividerOrientation,
    /// Optional centered label text.
    label: Option<String>,
    /// Optional color override for the divider line.
    color: Option<Color>,
}

impl Default for DividerState {
    fn default() -> Self {
        Self {
            orientation: DividerOrientation::Horizontal,
            label: None,
            color: None,
        }
    }
}

impl DividerState {
    /// Creates a new horizontal divider with no label.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DividerState, DividerOrientation};
    ///
    /// let state = DividerState::new();
    /// assert_eq!(state.orientation(), &DividerOrientation::Horizontal);
    /// assert!(state.label().is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new horizontal divider.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DividerState, DividerOrientation};
    ///
    /// let state = DividerState::horizontal();
    /// assert_eq!(state.orientation(), &DividerOrientation::Horizontal);
    /// ```
    pub fn horizontal() -> Self {
        Self {
            orientation: DividerOrientation::Horizontal,
            ..Self::default()
        }
    }

    /// Creates a new vertical divider.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DividerState, DividerOrientation};
    ///
    /// let state = DividerState::vertical();
    /// assert_eq!(state.orientation(), &DividerOrientation::Vertical);
    /// ```
    pub fn vertical() -> Self {
        Self {
            orientation: DividerOrientation::Vertical,
            ..Self::default()
        }
    }

    // ---- Builders ----

    /// Sets the label (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DividerState;
    ///
    /// let state = DividerState::new().with_label("Section");
    /// assert_eq!(state.label(), Some("Section"));
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the color override (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DividerState;
    /// use ratatui::style::Color;
    ///
    /// let state = DividerState::new().with_color(Color::Red);
    /// assert_eq!(state.color(), Some(Color::Red));
    /// ```
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the orientation (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DividerState, DividerOrientation};
    ///
    /// let state = DividerState::new()
    ///     .with_orientation(DividerOrientation::Vertical);
    /// assert_eq!(state.orientation(), &DividerOrientation::Vertical);
    /// ```
    pub fn with_orientation(mut self, orientation: DividerOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    // ---- Getters ----

    /// Returns the label text if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DividerState;
    ///
    /// let state = DividerState::new();
    /// assert_eq!(state.label(), None);
    ///
    /// let state = DividerState::new().with_label("Title");
    /// assert_eq!(state.label(), Some("Title"));
    /// ```
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DividerState, DividerOrientation};
    ///
    /// let state = DividerState::new();
    /// assert_eq!(state.orientation(), &DividerOrientation::Horizontal);
    /// ```
    pub fn orientation(&self) -> &DividerOrientation {
        &self.orientation
    }

    /// Returns the color override if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DividerState;
    ///
    /// let state = DividerState::new();
    /// assert_eq!(state.color(), None);
    /// ```
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    // ---- Setters ----

    /// Sets the label text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DividerState;
    ///
    /// let mut state = DividerState::new();
    /// state.set_label(Some("New Label".to_string()));
    /// assert_eq!(state.label(), Some("New Label"));
    ///
    /// state.set_label(None);
    /// assert_eq!(state.label(), None);
    /// ```
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    /// Sets the color override.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DividerState;
    /// use ratatui::style::Color;
    ///
    /// let mut state = DividerState::new();
    /// state.set_color(Some(Color::Red));
    /// assert_eq!(state.color(), Some(Color::Red));
    /// ```
    pub fn set_color(&mut self, color: Option<Color>) {
        self.color = color;
    }

    /// Sets the orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DividerState, DividerOrientation};
    ///
    /// let mut state = DividerState::new();
    /// state.set_orientation(DividerOrientation::Vertical);
    /// assert_eq!(state.orientation(), &DividerOrientation::Vertical);
    /// ```
    pub fn set_orientation(&mut self, orientation: DividerOrientation) {
        self.orientation = orientation;
    }

    // ---- Instance methods ----

    /// Updates the divider state with a message.
    ///
    /// This is an instance method that delegates to [`Divider::update`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{DividerState, DividerMessage};
    ///
    /// let mut state = DividerState::new();
    /// state.update(DividerMessage::SetLabel(Some("Updated".to_string())));
    /// assert_eq!(state.label(), Some("Updated"));
    /// ```
    pub fn update(&mut self, msg: DividerMessage) -> Option<()> {
        Divider::update(self, msg)
    }

    /// Maps an input event to a divider message.
    ///
    /// This is an instance method that delegates to [`Divider::handle_event`].
    /// Since the divider is display-only, this always returns `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DividerState;
    /// use envision::input::Event;
    /// use ratatui::crossterm::event::KeyCode;
    ///
    /// let state = DividerState::new();
    /// assert!(state.handle_event(&Event::key(KeyCode::Enter)).is_none());
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<DividerMessage> {
        Divider::handle_event(self, event, &EventContext::default())
    }

    /// Dispatches an event by mapping it to a message and updating state.
    ///
    /// This is an instance method that delegates to [`Divider::dispatch_event`].
    /// Since the divider is display-only, this always returns `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::DividerState;
    /// use envision::input::Event;
    /// use ratatui::crossterm::event::KeyCode;
    ///
    /// let mut state = DividerState::new();
    /// assert!(state.dispatch_event(&Event::key(KeyCode::Enter)).is_none());
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<()> {
        Divider::dispatch_event(self, event, &EventContext::default())
    }
}

/// A horizontal or vertical separator line with optional label.
///
/// `Divider` renders a line of box-drawing characters (`─` for horizontal,
/// `│` for vertical) to visually separate content areas. An optional label
/// can be centered on the divider line.
///
/// This is a display-only component that does not receive keyboard focus.
///
/// # Example
///
/// ```rust
/// use envision::component::{Component, Divider, DividerState, DividerMessage};
///
/// let mut state = DividerState::new().with_label("Settings");
/// assert_eq!(state.label(), Some("Settings"));
///
/// Divider::update(&mut state, DividerMessage::SetLabel(None));
/// assert_eq!(state.label(), None);
/// ```
pub struct Divider;

impl Component for Divider {
    type State = DividerState;
    type Message = DividerMessage;
    type Output = ();

    fn init() -> Self::State {
        DividerState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            DividerMessage::SetLabel(label) => state.label = label,
            DividerMessage::SetOrientation(orientation) => state.orientation = orientation,
        }
        None
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::divider("divider")
                    .with_label(state.label.as_deref().unwrap_or(""))
                    .with_disabled(ctx.disabled),
            );
        });

        if ctx.area.width == 0 || ctx.area.height == 0 {
            return;
        }

        let style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else if let Some(color) = state.color {
            Style::default().fg(color)
        } else {
            ctx.theme.normal_style()
        };

        match state.orientation {
            DividerOrientation::Horizontal => {
                render_horizontal(state, ctx.frame, ctx.area, style);
            }
            DividerOrientation::Vertical => {
                render_vertical(state, ctx.frame, ctx.area, style);
            }
        }
    }
}

/// Renders a horizontal divider line with optional centered label.
fn render_horizontal(state: &DividerState, frame: &mut Frame, area: Rect, style: Style) {
    let line_char = "\u{2500}"; // ─
    let width = area.width as usize;

    let text = match &state.label {
        Some(label) => {
            let label_with_padding = format!(" {} ", label);
            let label_len = label_with_padding.len();
            if label_len >= width {
                // Label too long, just show the label truncated
                let truncated: String = label_with_padding.chars().take(width).collect();
                Line::from(Span::styled(truncated, style))
            } else {
                let remaining = width - label_len;
                let left = remaining / 2;
                let right = remaining - left;
                let line = format!(
                    "{}{}{}",
                    line_char.repeat(left),
                    label_with_padding,
                    line_char.repeat(right),
                );
                Line::from(Span::styled(line, style))
            }
        }
        None => Line::from(Span::styled(line_char.repeat(width), style)),
    };

    let render_area = Rect::new(area.x, area.y, area.width, 1.min(area.height));
    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, render_area);
}

/// Renders a vertical divider line with optional centered label.
fn render_vertical(state: &DividerState, frame: &mut Frame, area: Rect, style: Style) {
    let line_char = "\u{2502}"; // │
    let height = area.height as usize;

    let middle_row = height / 2;

    let mut lines = Vec::with_capacity(height);
    for row in 0..height {
        let content = match &state.label {
            Some(label) if row == middle_row => {
                // Show first character of label in the middle
                label
                    .chars()
                    .next()
                    .map_or_else(|| line_char.to_string(), |c| c.to_string())
            }
            _ => line_char.to_string(),
        };
        lines.push(Line::from(Span::styled(content, style)));
    }

    let render_area = Rect::new(area.x, area.y, 1.min(area.width), area.height);
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, render_area);
}

#[cfg(test)]
mod tests;
