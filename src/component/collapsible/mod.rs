//! A collapsible component with an expandable content section.
//!
//! [`Collapsible`] provides a single section with a clickable header that
//! expands or collapses a content area. Unlike [`Accordion`](super::Accordion)
//! which manages multiple panels, `Collapsible` is a standalone building block
//! that can be composed into larger layouts.
//!
//! The component renders a header with an expand/collapse indicator and an
//! optional bordered content area. The parent application is responsible for
//! rendering actual content into the area returned by
//! [`CollapsibleState::content_area()`].
//!
//! State is stored in [`CollapsibleState`], updated via [`CollapsibleMessage`],
//! and produces [`CollapsibleOutput`].
//!
//! Implements [`Toggleable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Collapsible, CollapsibleMessage, CollapsibleOutput, CollapsibleState, Component};
//!
//! let mut state = CollapsibleState::new("Details");
//!
//! // Toggle the section (collapses it, since default is expanded)
//! let output = Collapsible::update(&mut state, CollapsibleMessage::Toggle);
//! assert_eq!(output, Some(CollapsibleOutput::Toggled(false)));
//! assert!(!state.expanded());
//!
//! // Expand the section
//! let output = Collapsible::update(&mut state, CollapsibleMessage::Expand);
//! assert_eq!(output, Some(CollapsibleOutput::Expanded));
//! assert!(state.expanded());
//! ```

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, EventContext, RenderContext, Toggleable};
use crate::input::{Event, Key};

/// Messages that can be sent to a Collapsible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollapsibleMessage {
    /// Toggle the expanded state.
    Toggle,
    /// Set expanded to true.
    Expand,
    /// Set expanded to false.
    Collapse,
    /// Change the header text.
    SetHeader(String),
    /// Change the content height.
    SetContentHeight(u16),
}

/// Output messages from a Collapsible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollapsibleOutput {
    /// The section was expanded.
    Expanded,
    /// The section was collapsed.
    Collapsed,
    /// The section was toggled to the given expanded state.
    Toggled(bool),
}

/// State for a Collapsible component.
///
/// # Example
///
/// ```rust
/// use envision::component::CollapsibleState;
///
/// let state = CollapsibleState::new("Settings")
///     .with_expanded(false)
///     .with_content_height(10);
/// assert_eq!(state.header(), "Settings");
/// assert!(!state.expanded());
/// assert_eq!(state.content_height(), 10);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct CollapsibleState {
    /// The header text.
    header: String,
    /// Whether the content is visible.
    expanded: bool,
    /// Height to allocate for content when expanded.
    content_height: u16,
}

impl Default for CollapsibleState {
    fn default() -> Self {
        Self {
            header: String::new(),
            expanded: true,
            content_height: 5,
        }
    }
}

impl CollapsibleState {
    /// Creates a new collapsible with the given header text.
    ///
    /// The section starts expanded with a content height of 5.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let state = CollapsibleState::new("Details");
    /// assert_eq!(state.header(), "Details");
    /// assert!(state.expanded());
    /// assert_eq!(state.content_height(), 5);
    /// ```
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            ..Default::default()
        }
    }

    /// Sets the initial expanded state (builder method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let state = CollapsibleState::new("Details").with_expanded(false);
    /// assert!(!state.expanded());
    /// ```
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Sets the content area height (builder method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let state = CollapsibleState::new("Details").with_content_height(10);
    /// assert_eq!(state.content_height(), 10);
    /// ```
    pub fn with_content_height(mut self, height: u16) -> Self {
        self.content_height = height;
        self
    }

    /// Returns the header text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let state = CollapsibleState::new("My Section");
    /// assert_eq!(state.header(), "My Section");
    /// ```
    pub fn header(&self) -> &str {
        &self.header
    }

    /// Sets the header text.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let mut state = CollapsibleState::new("Old Header");
    /// state.set_header("New Header");
    /// assert_eq!(state.header(), "New Header");
    /// ```
    pub fn set_header(&mut self, header: impl Into<String>) {
        self.header = header.into();
    }

    /// Returns whether the section is expanded.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let state = CollapsibleState::new("Details");
    /// assert!(state.expanded());
    /// ```
    pub fn expanded(&self) -> bool {
        self.expanded
    }

    /// Returns whether the section is expanded.
    ///
    /// This is an alias for [`expanded()`](Self::expanded).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let state = CollapsibleState::new("Details");
    /// assert!(state.is_expanded());
    /// ```
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// Sets the expanded state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let mut state = CollapsibleState::new("Details");
    /// state.set_expanded(false);
    /// assert!(!state.expanded());
    /// ```
    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }

    /// Toggles the expanded state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let mut state = CollapsibleState::new("Details");
    /// assert!(state.expanded());
    /// state.toggle();
    /// assert!(!state.expanded());
    /// state.toggle();
    /// assert!(state.expanded());
    /// ```
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Returns the content area height.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let state = CollapsibleState::new("Details").with_content_height(8);
    /// assert_eq!(state.content_height(), 8);
    /// ```
    pub fn content_height(&self) -> u16 {
        self.content_height
    }

    /// Sets the content area height.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    ///
    /// let mut state = CollapsibleState::new("Details");
    /// state.set_content_height(12);
    /// assert_eq!(state.content_height(), 12);
    /// ```
    pub fn set_content_height(&mut self, height: u16) {
        self.content_height = height;
    }

    /// Returns the content area `Rect` for rendering content below the header.
    ///
    /// When expanded, this returns the area below the header row, bounded by
    /// the available space and the configured `content_height`. When collapsed,
    /// returns a zero-height `Rect`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CollapsibleState;
    /// use ratatui::prelude::Rect;
    ///
    /// let state = CollapsibleState::new("Details").with_content_height(5);
    /// let area = Rect::new(0, 0, 40, 10);
    ///
    /// let content = state.content_area(area);
    /// assert_eq!(content.y, 1); // Below header
    /// assert_eq!(content.height, 5);
    ///
    /// let collapsed = CollapsibleState::new("Details").with_expanded(false);
    /// let content = collapsed.content_area(area);
    /// assert_eq!(content.height, 0);
    /// ```
    pub fn content_area(&self, area: Rect) -> Rect {
        if !self.expanded || area.height <= 1 {
            return Rect::new(
                area.x,
                area.y.saturating_add(1).min(area.bottom()),
                area.width,
                0,
            );
        }

        let available = area.height.saturating_sub(1);
        let height = self.content_height.min(available);

        Rect::new(area.x, area.y + 1, area.width, height)
    }

    /// Updates the collapsible state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CollapsibleMessage, CollapsibleOutput, CollapsibleState};
    ///
    /// let mut state = CollapsibleState::new("Details");
    /// let output = state.update(CollapsibleMessage::Collapse);
    /// assert_eq!(output, Some(CollapsibleOutput::Collapsed));
    /// assert!(!state.expanded());
    /// ```
    pub fn update(&mut self, msg: CollapsibleMessage) -> Option<CollapsibleOutput> {
        Collapsible::update(self, msg)
    }
}

/// A collapsible component with an expandable content section.
///
/// The collapsible displays a header with an expand/collapse indicator. When
/// expanded, a bordered content area is shown below the header. The parent
/// application renders content into the area returned by
/// [`CollapsibleState::content_area()`].
///
/// # Keyboard Navigation
///
/// When focused:
/// - Space or Enter: toggle expanded state
/// - Right arrow: expand
/// - Left arrow: collapse
///
/// # Visual Layout
///
/// ```text
/// ▾ Section Header          (expanded)
/// │ [content rendered by parent]
/// └─────────────────────
///
/// ▸ Section Header          (collapsed)
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{Collapsible, CollapsibleMessage, CollapsibleState, Component};
///
/// let mut state = CollapsibleState::new("Advanced Settings");
///
/// // Collapse
/// Collapsible::update(&mut state, CollapsibleMessage::Collapse);
/// assert!(!state.expanded());
///
/// // Expand
/// Collapsible::update(&mut state, CollapsibleMessage::Expand);
/// assert!(state.expanded());
/// ```
pub struct Collapsible;

impl Component for Collapsible {
    type State = CollapsibleState;
    type Message = CollapsibleMessage;
    type Output = CollapsibleOutput;

    fn init() -> Self::State {
        CollapsibleState::default()
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            CollapsibleMessage::Toggle => {
                state.expanded = !state.expanded;
                Some(CollapsibleOutput::Toggled(state.expanded))
            }
            CollapsibleMessage::Expand => {
                if !state.expanded {
                    state.expanded = true;
                    Some(CollapsibleOutput::Expanded)
                } else {
                    None
                }
            }
            CollapsibleMessage::Collapse => {
                if state.expanded {
                    state.expanded = false;
                    Some(CollapsibleOutput::Collapsed)
                } else {
                    None
                }
            }
            CollapsibleMessage::SetHeader(header) => {
                state.header = header;
                None
            }
            CollapsibleMessage::SetContentHeight(height) => {
                state.content_height = height;
                None
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
                Key::Char(' ') | Key::Enter => Some(CollapsibleMessage::Toggle),
                Key::Right => Some(CollapsibleMessage::Expand),
                Key::Left => Some(CollapsibleMessage::Collapse),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        if ctx.area.height == 0 || ctx.area.width == 0 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                ctx.area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
                    "Collapsible".to_string(),
                ))
                .with_id("collapsible")
                .with_focus(ctx.focused)
                .with_disabled(ctx.disabled)
                .with_expanded(state.expanded),
            );
        });

        let indicator = if state.expanded {
            "\u{25be}"
        } else {
            "\u{25b8}"
        };
        let header_text = format!("{} {}", indicator, state.header);

        let header_style = if ctx.disabled {
            ctx.theme.disabled_style()
        } else if ctx.focused {
            ctx.theme.focused_style()
        } else {
            ctx.theme.normal_style()
        };

        let header_line = Line::from(Span::styled(header_text, header_style));
        let header_area = Rect::new(ctx.area.x, ctx.area.y, ctx.area.width, 1);
        ctx.frame
            .render_widget(Paragraph::new(header_line), header_area);

        // Content ctx.area (below header) -- only render border when expanded
        if state.expanded && ctx.area.height > 1 {
            let available = ctx.area.height.saturating_sub(1);
            let content_h = state.content_height.min(available);

            if content_h > 0 {
                let content_area = Rect::new(ctx.area.x, ctx.area.y + 1, ctx.area.width, content_h);
                let border_style = if ctx.disabled {
                    ctx.theme.disabled_style()
                } else if ctx.focused {
                    ctx.theme.focused_border_style()
                } else {
                    ctx.theme.border_style()
                };
                let content_block = Block::default()
                    .borders(Borders::LEFT | Borders::BOTTOM)
                    .border_style(border_style);
                ctx.frame.render_widget(content_block, content_area);
            }
        }
    }
}

impl Toggleable for Collapsible {
    fn is_visible(state: &Self::State) -> bool {
        state.expanded
    }

    fn set_visible(state: &mut Self::State, visible: bool) {
        state.expanded = visible;
    }
}

#[cfg(test)]
mod tests;
